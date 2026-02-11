use crate::audio_engine::mixer::{Mixer, MixerController, MixerState, CrossfadeConfig};
use crate::audio_engine::output::CpalOutput;
use std::path::Path;
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// 播放状态
/// 
/// 表示音频播放器的当前状态
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlaybackState {
    /// 停止状态
    Stopped,
    /// 正在播放
    Playing,
    /// 已暂停
    Paused,
    /// 正在预加载下一首
    Preloading,
    /// 正在进行交叉淡化
    Crossfading,
}

/// 播放器命令
/// 
/// 用于在音频线程中执行各种播放控制操作
#[derive(Debug)]
pub enum PlayerCommand {
    /// 播放指定路径的音频文件
    Play(String),
    /// 暂停播放
    Pause,
    /// 恢复播放
    Resume,
    /// 停止播放
    Stop,
    /// 跳转到指定位置（秒）
    Seek(f64),
    /// 设置音量 (0.0 - 1.0)
    SetVolume(f32),
    /// 预加载下一首音频
    PreloadNext(String),
    /// 设置交叉淡化配置
    SetCrossfadeConfig(CrossfadeConfig),
}

/// 播放器内部状态
/// 
/// 使用原子类型和互斥锁实现线程安全的状态共享
pub struct PlayerState {
    /// 播放状态（使用原子类型实现线程安全）
    state: AtomicU8,
    /// 当前音量 (0.0 - 1.0)，使用 u32 存储 f32 的位模式
    volume: AtomicU32,
    /// 播放位置（秒），使用 u32 存储 f32 的位模式
    position: AtomicU32,
    /// 总时长（秒），使用 u32 存储 f32 的位模式
    duration: AtomicU32,
    /// 当前文件路径（线程安全）
    current_path: Mutex<Option<String>>,
    /// 下一首文件路径（线程安全）
    next_path: Mutex<Option<String>>,
    /// 是否启用交叉淡化
    crossfade_enabled: AtomicBool,
}

// 状态常量
const STATE_STOPPED: u8 = 0;
const STATE_PLAYING: u8 = 1;
const STATE_PAUSED: u8 = 2;
const STATE_PRELOADING: u8 = 3;
const STATE_CROSSFADING: u8 = 4;

impl PlayerState {
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(STATE_STOPPED),
            volume: AtomicU32::new(1.0f32.to_bits()),
            position: AtomicU32::new(0.0f32.to_bits()),
            duration: AtomicU32::new(0.0f32.to_bits()),
            current_path: Mutex::new(None),
            next_path: Mutex::new(None),
            crossfade_enabled: AtomicBool::new(true),
        }
    }

    pub fn set_state(&self, state: PlaybackState) {
        let value = match state {
            PlaybackState::Stopped => STATE_STOPPED,
            PlaybackState::Playing => STATE_PLAYING,
            PlaybackState::Paused => STATE_PAUSED,
            PlaybackState::Preloading => STATE_PRELOADING,
            PlaybackState::Crossfading => STATE_CROSSFADING,
        };
        self.state.store(value, Ordering::Relaxed);
    }

    pub fn get_state(&self) -> PlaybackState {
        match self.state.load(Ordering::Relaxed) {
            STATE_PLAYING => PlaybackState::Playing,
            STATE_PAUSED => PlaybackState::Paused,
            STATE_PRELOADING => PlaybackState::Preloading,
            STATE_CROSSFADING => PlaybackState::Crossfading,
            _ => PlaybackState::Stopped,
        }
    }

    pub fn set_volume(&self, volume: f32) {
        self.volume.store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn get_volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }

    pub fn set_position(&self, position: f32) {
        self.position.store(position.to_bits(), Ordering::Relaxed);
    }

    pub fn get_position(&self) -> f32 {
        f32::from_bits(self.position.load(Ordering::Relaxed))
    }

    pub fn set_duration(&self, duration: f32) {
        self.duration.store(duration.to_bits(), Ordering::Relaxed);
    }

    pub fn get_duration(&self) -> f32 {
        f32::from_bits(self.duration.load(Ordering::Relaxed))
    }

    pub fn set_path(&self, path: Option<String>) {
        if let Ok(mut guard) = self.current_path.lock() {
            *guard = path;
        }
    }

    pub fn get_path(&self) -> Option<String> {
        self.current_path.lock().ok().and_then(|g| g.clone())
    }

    pub fn set_next_path(&self, path: Option<String>) {
        if let Ok(mut guard) = self.next_path.lock() {
            *guard = path;
        }
    }

    pub fn get_next_path(&self) -> Option<String> {
        self.next_path.lock().ok().and_then(|g| g.clone())
    }

    pub fn set_crossfade_enabled(&self, enabled: bool) {
        self.crossfade_enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn is_crossfade_enabled(&self) -> bool {
        self.crossfade_enabled.load(Ordering::Relaxed)
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}

/// 音频播放器
/// 
/// 使用混音器和双缓冲实现无缝播放。内部使用独立的音频线程处理解码和播放，
/// 通过命令通道与主线程通信，确保 UI 响应性。
/// 
/// # 特性
/// - 支持交叉淡化（Crossfade）实现无缝切歌
/// - 支持预加载下一首曲目
/// - 线程安全的音量、位置控制
/// 
/// # 示例
/// ```ignore
/// let mut player = AudioPlayer::new()?;
/// player.start_engine()?;
/// player.play("/path/to/music.mp3")?;
/// ```
pub struct AudioPlayer {
    /// 共享状态（线程间共享）
    state: Arc<PlayerState>,
    /// 命令发送器（用于向音频线程发送命令）
    command_sender: Option<std::sync::mpsc::Sender<PlayerCommand>>,
    /// 音频线程句柄
    audio_thread: Option<JoinHandle<()>>,
}

impl AudioPlayer {
    /// 创建新的音频播放器
    pub fn new() -> anyhow::Result<Self> {
        let state = Arc::new(PlayerState::new());
        
        Ok(Self {
            state,
            command_sender: None,
            audio_thread: None,
        })
    }

    /// 启动音频引擎
    pub fn start_engine(&mut self) -> anyhow::Result<()> {
        if self.audio_thread.is_some() {
            return Ok(()); // 已经在运行
        }

        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<PlayerCommand>();

        let state = Arc::clone(&self.state);

        // 启动音频线程
        let handle = thread::spawn(move || {
            if let Err(e) = Self::audio_thread_main(state, cmd_rx) {
                eprintln!("Audio thread error: {}", e);
                panic!("Audio thread crashed: {}", e);
            }
        });

        self.command_sender = Some(cmd_tx);
        self.audio_thread = Some(handle);
        Ok(())
    }

    /// 初始化音频输出设备
    fn init_audio_output(mixer_controller: MixerController) -> anyhow::Result<CpalOutput> {
        let mut output = CpalOutput::new()
            .map_err(|e| anyhow::anyhow!("Failed to create audio output: {}. Please check your audio device.", e))?;
        
        // 设置音频回调
        output.set_callback(Box::new(move |data: &mut [f32]| {
            let samples = mixer_controller.get_samples(data.len());
            for (i, sample) in data.iter_mut().enumerate() {
                *sample = samples.get(i).copied().unwrap_or(0.0);
            }
        }));

        output.start()
            .map_err(|e| anyhow::anyhow!("Failed to start audio output: {}. Please check your audio device.", e))?;
        
        Ok(output)
    }

    /// 处理播放器命令
    fn process_command(
        cmd: PlayerCommand,
        mixer: &Arc<Mixer>,
        state: &Arc<PlayerState>,
    ) {
        match cmd {
            PlayerCommand::Play(path) => {
                if let Err(e) = mixer.load_track(&path) {
                    eprintln!("Failed to load track: {}", e);
                } else {
                    state.set_path(Some(path));
                    state.set_state(PlaybackState::Playing);
                    state.set_duration(mixer.current_duration());
                }
            }
            PlayerCommand::Pause => {
                mixer.pause();
                state.set_state(PlaybackState::Paused);
            }
            PlayerCommand::Resume => {
                mixer.resume();
                state.set_state(PlaybackState::Playing);
            }
            PlayerCommand::Stop => {
                mixer.stop();
                state.set_state(PlaybackState::Stopped);
                state.set_position(0.0);
                state.set_path(None);
                state.set_next_path(None);
            }
            PlayerCommand::Seek(position) => {
                // TODO: 实现 seek
                state.set_position(position as f32);
            }
            PlayerCommand::SetVolume(volume) => {
                mixer.set_volume(volume);
                state.set_volume(volume);
            }
            PlayerCommand::PreloadNext(path) => {
                if let Err(e) = mixer.preload_next(&path) {
                    eprintln!("Failed to preload track: {}", e);
                } else {
                    state.set_next_path(Some(path));
                    state.set_state(PlaybackState::Preloading);
                }
            }
            PlayerCommand::SetCrossfadeConfig(config) => {
                mixer.set_crossfade_config(config);
            }
        }
    }

    /// 处理播放/预加载状态的音频解码
    fn handle_playing_state(
        mixer: &Arc<Mixer>,
        state: &Arc<PlayerState>,
    ) {
        // 解码当前轨道
        match mixer.decode_current_frame() {
            Ok(true) => {
                state.set_position(mixer.current_position());

                // 检查是否需要预加载
                if state.is_crossfade_enabled() && mixer.should_preload() {
                    // TODO: 通过 Tauri Event 发送信号通知前端
                }

                // 检查是否应该开始交叉淡化
                if state.is_crossfade_enabled() && mixer.should_start_crossfade() {
                    mixer.start_crossfade();
                    state.set_state(PlaybackState::Crossfading);
                }
            }
            Ok(false) => {
                // 当前轨道结束
                if mixer.get_state() != MixerState::Crossfading && !mixer.is_playing() {
                    state.set_state(PlaybackState::Stopped);
                }
            }
            Err(e) => {
                eprintln!("Decode error: {}", e);
            }
        }

        // 如果在预加载状态，解码下一首
        if mixer.get_state() == MixerState::Preloading {
            if let Err(e) = mixer.decode_next_frame() {
                eprintln!("Preload decode error: {}", e);
            }
        }
    }

    /// 处理交叉淡化状态的音频解码
    fn handle_crossfading_state(
        mixer: &Arc<Mixer>,
        state: &Arc<PlayerState>,
    ) {
        // 解码当前轨道和下一首
        let _ = mixer.decode_current_frame();
        let _ = mixer.decode_next_frame();

        // 更新位置
        state.set_position(mixer.current_position());

        // 检查交叉淡化是否完成
        if !mixer.double_buffer().is_crossfading() {
            mixer.complete_crossfade();
            state.set_path(state.get_next_path());
            state.set_next_path(None);
            state.set_state(PlaybackState::Playing);
            state.set_duration(mixer.current_duration());
        }
    }

    /// 音频线程主函数
    fn audio_thread_main(
        state: Arc<PlayerState>,
        cmd_rx: std::sync::mpsc::Receiver<PlayerCommand>,
    ) -> anyhow::Result<()> {
        // 创建混音器和音频输出
        let mixer = Arc::new(Mixer::new(48000, 2));
        let mixer_controller = MixerController::new(Arc::clone(&mixer));
        let mut output = Self::init_audio_output(mixer_controller)?;

        // 主循环
        loop {
            // 处理命令
            match cmd_rx.try_recv() {
                Ok(cmd) => Self::process_command(cmd, &mixer, &state),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }

            // 根据当前状态处理音频
            match mixer.get_state() {
                MixerState::Playing | MixerState::Preloading => {
                    Self::handle_playing_state(&mixer, &state);
                }
                MixerState::Crossfading => {
                    Self::handle_crossfading_state(&mixer, &state);
                }
                _ => {
                    thread::sleep(std::time::Duration::from_millis(1));
                }
            }

            // 短暂休眠以避免CPU占用过高
            thread::sleep(std::time::Duration::from_micros(100));
        }

        output.stop();
        Ok(())
    }

    /// 加载并播放文件
    pub fn play<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        if let Some(ref sender) = self.command_sender {
            sender.send(PlayerCommand::Play(path_str))?;
        }
        
        Ok(())
    }

    /// 预加载下一首
    pub fn preload_next<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        if let Some(ref sender) = self.command_sender {
            sender.send(PlayerCommand::PreloadNext(path_str))?;
        }
        
        Ok(())
    }

    /// 暂停
    pub fn pause(&self) {
        if let Some(ref sender) = self.command_sender {
            let _ = sender.send(PlayerCommand::Pause);
        }
    }

    /// 恢复播放
    pub fn resume(&self) {
        if let Some(ref sender) = self.command_sender {
            let _ = sender.send(PlayerCommand::Resume);
        }
    }

    /// 停止
    pub fn stop(&self) {
        if let Some(ref sender) = self.command_sender {
            let _ = sender.send(PlayerCommand::Stop);
        }
    }

    /// 跳转到指定位置（秒）
    pub fn seek(&self, position: f64) -> anyhow::Result<()> {
        if let Some(ref sender) = self.command_sender {
            sender.send(PlayerCommand::Seek(position))?;
        }
        Ok(())
    }

    /// 设置音量 (0.0 - 1.0)
    pub fn set_volume(&self, volume: f32) {
        if let Some(ref sender) = self.command_sender {
            let _ = sender.send(PlayerCommand::SetVolume(volume));
        }
    }

    /// 设置交叉淡化配置
    pub fn set_crossfade_config(&self, config: CrossfadeConfig) {
        if let Some(ref sender) = self.command_sender {
            let _ = sender.send(PlayerCommand::SetCrossfadeConfig(config));
        }
    }

    /// 启用/禁用交叉淡化
    pub fn set_crossfade_enabled(&self, enabled: bool) {
        self.state.set_crossfade_enabled(enabled);
    }

    /// 获取音量
    pub fn get_volume(&self) -> f32 {
        self.state.get_volume()
    }

    /// 获取当前播放位置（秒）
    pub fn get_position(&self) -> f32 {
        self.state.get_position()
    }

    /// 获取总时长（秒）
    pub fn get_duration(&self) -> f32 {
        self.state.get_duration()
    }

    /// 获取当前播放状态
    pub fn get_state(&self) -> PlaybackState {
        self.state.get_state()
    }

    /// 检查是否正在播放
    pub fn is_playing(&self) -> bool {
        matches!(self.get_state(), PlaybackState::Playing | PlaybackState::Preloading | PlaybackState::Crossfading)
    }

    /// 获取当前播放的文件路径
    pub fn get_current_path(&self) -> Option<String> {
        self.state.get_path()
    }

    /// 获取下一首文件路径
    pub fn get_next_path(&self) -> Option<String> {
        self.state.get_next_path()
    }

    /// 检查是否启用了交叉淡化
    pub fn is_crossfade_enabled(&self) -> bool {
        self.state.is_crossfade_enabled()
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        match Self::new() {
            Ok(player) => player,
            Err(e) => {
                eprintln!("Failed to create AudioPlayer: {}", e);
                panic!("AudioPlayer initialization failed: {}", e);
            }
        }
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        // 停止音频线程
        self.stop();
        
        // 关闭命令通道
        self.command_sender = None;
        
        // 等待线程结束
        if let Some(handle) = self.audio_thread.take() {
            let _ = handle.join();
        }
    }
}

/// 线程安全的播放器包装
/// 
/// 提供对 AudioPlayer 的线程安全访问，可在多线程环境中共享使用。
/// 内部使用 Arc<Mutex<>> 实现共享所有权和互斥访问。
/// 
/// # 使用场景
/// - 需要在多个线程/任务中控制播放
/// - 作为 Tauri 应用状态共享
/// 
/// # 示例
/// ```ignore
/// let player = SharedAudioPlayer::new()?;
/// player.play("/path/to/music.mp3")?;
/// player.set_volume(0.8);
/// ```
#[derive(Clone)]
pub struct SharedAudioPlayer {
    /// 内部播放器实例（线程安全包装）
    inner: Arc<Mutex<AudioPlayer>>,
}

impl SharedAudioPlayer {
    pub fn new() -> anyhow::Result<Self> {
        let mut player = AudioPlayer::new()?;
        player.start_engine()?;
        
        Ok(Self {
            inner: Arc::new(Mutex::new(player)),
        })
    }

    fn with_player<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&AudioPlayer) -> R,
    {
        match self.inner.lock() {
            Ok(player) => f(&player),
            Err(e) => {
                eprintln!("Failed to lock player: {}", e);
                panic!("Player lock poisoned: {}", e);
            }
        }
    }

    pub fn play<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        self.with_player(|player| player.play(path))
    }

    pub fn preload_next<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        self.with_player(|player| player.preload_next(path))
    }

    pub fn pause(&self) {
        self.with_player(|player| player.pause());
    }

    pub fn resume(&self) {
        self.with_player(|player| player.resume());
    }

    pub fn stop(&self) {
        self.with_player(|player| player.stop());
    }

    pub fn seek(&self, position: f64) -> anyhow::Result<()> {
        self.with_player(|player| player.seek(position))
    }

    pub fn set_volume(&self, volume: f32) {
        self.with_player(|player| player.set_volume(volume));
    }

    pub fn set_crossfade_enabled(&self, enabled: bool) {
        self.with_player(|player| player.set_crossfade_enabled(enabled));
    }

    pub fn set_crossfade_config(&self, config: CrossfadeConfig) {
        self.with_player(|player| player.set_crossfade_config(config));
    }

    pub fn get_volume(&self) -> f32 {
        self.with_player(|player| player.get_volume())
    }

    pub fn get_position(&self) -> f32 {
        self.with_player(|player| player.get_position())
    }

    pub fn get_duration(&self) -> f32 {
        self.with_player(|player| player.get_duration())
    }

    pub fn get_state(&self) -> PlaybackState {
        self.with_player(|player| player.get_state())
    }

    pub fn is_playing(&self) -> bool {
        self.with_player(|player| player.is_playing())
    }

    pub fn get_current_path(&self) -> Option<String> {
        self.with_player(|player| player.get_current_path())
    }

    pub fn get_next_path(&self) -> Option<String> {
        self.with_player(|player| player.get_next_path())
    }

    pub fn is_crossfade_enabled(&self) -> bool {
        self.with_player(|player| player.is_crossfade_enabled())
    }

    // ========== Phase 4: BPM同步方法 ==========
    
    /// 设置当前轨道BPM信息
    pub fn set_current_bpm(&self, bpm: f64, beat_positions: Vec<f64>) {
        // 通过命令发送到音频线程
        // 这里简化处理，直接存储在状态中
        // 实际实现需要通过命令通道发送到音频线程
    }

    /// 设置下一首轨道BPM信息
    pub fn set_next_bpm(&self, bpm: f64, beat_positions: Vec<f64>) {
        // 同上
    }

    /// 启用/禁用BPM同步
    pub fn set_bpm_sync(&self, enabled: bool) {
        // 同上
    }

    /// 检查BPM同步是否启用
    pub fn is_bpm_sync(&self) -> bool {
        false // 占位实现
    }

    /// 获取当前速度比率
    pub fn speed_ratio(&self) -> f64 {
        1.0 // 占位实现
    }

    /// 设置播放速度
    pub fn set_speed(&self, _speed_ratio: f64) -> anyhow::Result<()> {
        Ok(()) // 占位实现
    }
}

## 项目概述
基于 Chordial 现有架构（Tauri + Vue 3 + Rust），实现智能 DJ 混音（Automix）功能。

## 当前状态
- ✅ 音乐库管理（本地 + WebDAV + API）
- ✅ 多格式音频元数据解析
- ✅ 播放器核心功能
- ✅ AMLL 歌词系统
- 🔴 **缺失：音频播放引擎（需要新建）**
- 🔴 **缺失：Beat Detection**
- 🔴 **缺失：Automix 逻辑**

## 执行路线图

### Phase 1: 音频引擎基础（Week 1-2）
**目标**：建立 Rust 音频播放引擎，实现基础播放能力

**任务清单**：
1. **依赖引入** - 添加 symphonia、cpal、rubato、rustfft、crossbeam
2. **解码器封装** - 实现 SymphoniaDecoder（MP3/FLAC/WAV）
3. **输出层封装** - 实现 CpalOutput（统一 48kHz）
4. **基础播放控制** - Play/Pause/Stop/Seek
5. **Tauri 命令绑定** - 前端控制接口

**验证标准**：
- 能播放本地音频文件
- 音量控制正常
- 前端能控制播放/暂停

---

### Phase 2: 双缓冲与无缝切换（Week 3-4）
**目标**：实现 Gapless 播放和 Crossfade

**任务清单**：
1. **RingBuffer 实现** - 使用 crossbeam 实现循环缓冲区
2. **预加载机制** - EOF 前 10 秒触发下一首解码
3. **Crossfade 状态机** - PlayingA → Crossfading → PlayingB
4. **音量曲线** - 对数 dB 曲线淡入淡出
5. **播放队列管理** - 与现有前端播放列表集成

**验证标准**：
- 两首歌之间无空白间隙
- Crossfade 过渡平滑（3-10 秒可配置）

---

### Phase 3: Beat Detection 引擎（Week 5-6）
**目标**：实现 BPM 检测和 Beat 位置标记

**任务清单**：
1. **Spectral Flux 实现** - 2048 FFT + 512 Hop
2. **多频带加权** - 低频 Kick 权重 1.5x
3. **Onset Curve 归一化** - Z-score + 对数压缩
4. **Comb Filter BPM 检测** - 60-200 BPM 扫描
5. **数据库缓存** - sqlx + SQLite 存储分析结果

**验证标准**：
- 电子/流行音乐 BPM 检测准确率 > 85%
- 分析结果持久化，二次加载 < 100ms

---

### Phase 4: 时间拉伸与变速（Week 7-8）
**目标**：实现 BPM 匹配的时间拉伸

**任务清单**：
1. **Rubato 封装** - StreamingTimeStretcher 实现
2. **块处理逻辑** - 输入队列 → Rubato → 输出队列
3. **延迟补偿** - 算法延迟计算
4. **双时间轴映射** - 原曲时间 ↔ 拉伸后时间
5. **速度渐变** - Speed Ramping 避免突变

**验证标准**：
- ±6% 变速无明显失真
- 大范围变速保持音高

---

### Phase 5: 实时同步系统（Week 9-10）
**目标**：实现 Beat 对齐的无锁播放

**任务清单**：
1. **TripleBuffer 实现** - 状态共享（当前轨 + 预加载轨）
2. **原子时钟** - AtomicU64 Sample 级精度
3. **命令队列** - crossbeam::ArrayQueue 控制指令
4. **MixScheduler** - 计算 Mix-out/Mix-in 点
5. **小节对齐** - 4 拍边界切换

**验证标准**：
- 播放回调零分配
- 切歌点精确到 Beat（误差 < 10ms）

---

### Phase 6: 前端 UI 集成（Week 11-12）
**目标**：完成用户界面和交互

**任务清单**：
1. **Automix 开关** - 全局/歌单级控制
2. **混音参数设置** - Crossfade 时长、BPM 匹配开关
3. **Beat 标记可视化** - 波形上显示 Beat 竖线
4. **下一首预览** - 显示预加载状态和匹配度
5. **Tauri 事件通道** - 分析进度、播放状态实时同步

**验证标准**：
- UI 响应流畅
- 实时显示分析进度
- 可配置参数生效

---

## 风险与应对

| 风险 | 等级 | 应对策略 |
|------|------|----------|
| 音频回调延迟 | 🔴 高 | 使用无锁队列，Callback 内禁止分配 |
| 时间拉伸音质 | 🟡 中 | 参数调优，提供质量/性能切换 |
| Beat 检测精度 | 🟡 中 | 多频带加权，支持手动校正 |
| 跨平台兼容性 | 🟡 中 | 优先 Windows，逐步测试 macOS/Linux |

## 建议启动方式

**MVP 验证（Week 1）**：
1. Day 1-2: 搭建基础音频链路（Symphonia → CPAL）
2. Day 3-4: 实现基础 Crossfade
3. Day 5-7: 集成简单 BPM 检测（aubio-rs）

确认可行后再继续后续阶段。

---

**请确认此计划后，我将开始 Phase 1 的执行。**
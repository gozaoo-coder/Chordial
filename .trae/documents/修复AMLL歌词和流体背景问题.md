## 修复方案

### 1. 修复 Rust 后端多编码歌词读取 (高优先级)

**文件**: `src-tauri/src/lyric_enhancer.rs`

问题：`std::fs::read_to_string` 默认使用 UTF-8，无法正确处理 GBK/GB2312 编码的中文歌词文件。

修复：使用字节读取 + 编码检测 + 手动解码：
```rust
// 替换 std::fs::read_to_string
let bytes = std::fs::read(&lyric_path)?;
let content = safe_bytes_to_string(&bytes); // 使用已有的编码检测函数
```

### 2. 添加 GBK 编码支持 (高优先级)

**文件**: `src-tauri/src/audio_metadata/utils/encoding.rs`

问题：检测到 GBK 特征但没有实际解码。

修复方案选项：
- 选项A：添加 `encoding_rs` 的 GBK 支持（需要额外依赖）
- 选项B：使用 `gbk` crate 专门处理 GBK 编码
- 选项C：改进现有检测逻辑，优先尝试 GB18030 解码

### 3. 修复流体背景显示 (中优先级)

**文件**: `src/components/player/PlayerBackground.vue`

检查点：
- 确认 `coverUrl` 是否正确传递到组件
- 确认 `BackgroundRender` 组件是否正确渲染
- 检查是否需要添加 `v-show` 或调整 z-index

### 4. 验证 AMLL 歌词数据格式 (中优先级)

**文件**: `src/components/player/AMLLyrics.vue`

检查点：
- 确认 `lyricsData` 格式正确
- 确认时间单位统一为毫秒
- 添加更多调试日志以便排查问题

## 执行顺序
1. 先修复 Rust 后端编码问题（影响歌词正确性）
2. 再修复流体背景显示问题
3. 最后验证 AMLL 数据格式

请确认此计划后，我将开始执行修复。
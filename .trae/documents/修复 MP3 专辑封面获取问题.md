## 问题诊断

MP3 格式的歌曲无法正确获取专辑封面图，根本原因是 **ID3v2.2 版本的 PIC 帧格式与 ID3v2.3/2.4 的 APIC 帧格式不同**，但代码中没有正确处理这种差异。

### ID3v2.2 PIC 帧 vs ID3v2.3/2.4 APIC 帧

| 特性 | ID3v2.2 PIC | ID3v2.3/2.4 APIC |
|------|-------------|------------------|
| 帧 ID | PIC (3字符) | APIC (4字符) |
| 图片格式 | 3字节固定长度 (如 "PNG", "JPG") | null结尾的 MIME 类型字符串 |
| 图片类型 | 1字节 | 1字节 |
| 描述 | 编码后文本 | 编码后文本 |

### 需要修改的文件

1. **`src-tauri/src/audio_metadata/readers/mp3.rs`**
   - 修改 `parse_apic_frame` 函数，添加 `version` 参数处理
   - 对 ID3v2.2 的 PIC 帧使用 3 字节固定长度格式解析
   - 将 3 字节格式（如 "PNG"）转换为 MIME 类型（如 "image/png"）

### 修复步骤

1. 修改 `parse_apic_frame` 函数签名，添加 `version: Id3v2Version` 参数
2. 在函数内部根据版本区分处理逻辑：
   - ID3v2.2: 读取 3 字节格式，转换为 MIME 类型
   - ID3v2.3/2.4: 读取 null 结尾的 MIME 类型
3. 添加格式到 MIME 类型的映射函数

### 验证

修复后需要验证：
- ID3v2.2 格式的 MP3 文件能正确显示封面
- ID3v2.3/2.4 格式的 MP3 文件仍能正确显示封面
- 其他音频格式（FLAC、M4A）不受影响
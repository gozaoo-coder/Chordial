## 问题分析

运行 `npm run tauri build` 发现 **26 个 Rust 编译警告**，全部位于 `src-tauri/src/audio_metadata/parsers/` 目录下的 5 个文件中：

### 警告分类：

#### 1. `flac.rs` - 5个警告
- 未使用导入：`SeekFrom`, `read_u32_be`
- 未使用变量：`min_block_size`, `max_block_size`, `bits_per_sample`

#### 2. `mp3.rs` - 3个警告
- 未使用导入：`read_u8`
- 未使用变量：`layer`
- 未使用常量：`MP3_FRAME_SYNC`

#### 3. `m4a.rs` - 5个警告
- 未使用导入：`read_u32_be`, `read_u8`
- 未使用变量：`header_size`, `sample_size`, `data`, `metadata` (函数参数), `data_type`

#### 4. `ogg.rs` - 8个警告
- 未使用导入：`SeekFrom`, `Picture`, `PictureType`
- 未使用变量：`vorbis_version`, `data`, `start_pos`, `metadata` (函数参数)
- 未读字段：`OggPage` 结构体的 `version`, `bitstream_serial`, `crc_checksum`, `num_segments`, `segment_table`

#### 5. `wav.rs` - 5个警告
- 未使用导入：`Picture`, `PictureType`
- 未使用变量：`block_align`, `bits_per_sample`, `valid_bits`
- 未读字段：`WavChunkId::Unknown(String)` 的字段

## 修复方案

### 方案一：删除未使用的导入（推荐）
直接删除未使用的 `use` 语句。

### 方案二：变量名前加下划线
对于未使用的变量，在名称前加 `_` 前缀，如 `_min_block_size`。

### 方案三：结构体字段添加 `#[allow(dead_code)]`
对于故意保留但未使用的结构体字段，添加属性宏抑制警告。

### 方案四：将未使用字段改为单元类型
对于 `WavChunkId::Unknown(String)`，按照编译器建议改为 `WavChunkId::Unknown(())`。

## 具体修改计划

1. **flac.rs**: 删除 `SeekFrom` 和 `read_u32_be` 导入；变量加 `_` 前缀
2. **mp3.rs**: 删除 `read_u8` 导入；变量加 `_` 前缀；删除 `MP3_FRAME_SYNC` 常量
3. **m4a.rs**: 删除 `read_u32_be` 和 `read_u8` 导入；变量加 `_` 前缀
4. **ogg.rs**: 删除 `SeekFrom`, `Picture`, `PictureType` 导入；变量加 `_` 前缀；为 `OggPage` 字段添加 `#[allow(dead_code)]`
5. **wav.rs**: 删除 `Picture`, `PictureType` 导入；变量加 `_` 前缀；将 `Unknown(String)` 改为 `Unknown(())`

修复后将重新运行构建验证所有警告已消除。
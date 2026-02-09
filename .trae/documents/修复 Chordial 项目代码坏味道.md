## 修复计划

### 阶段 1: 高优先级修复（P0）

1. **修复 Mutex 锁持有时间过长**
   - 文件: `src-tauri/src/lib.rs`
   - 修改: `scan_all_sources` 函数，将锁的获取和释放控制在最小范围
   - 策略: 先获取配置信息，释放锁后再执行扫描

2. **修复错误处理不当**
   - 文件: `src-tauri/src/lib.rs`
   - 修改: 将 `Result<T, String>` 改为 `Result<T, AppError>`
   - 策略: 利用已定义的 `AppError` 类型，添加 `Serialize` 支持以便 Tauri 返回

3. **提取重复代码 - 默认 PNG 数据**
   - 文件: `src-tauri/src/lib.rs`
   - 修改: 将 67 字节 PNG 数据提取为常量 `DEFAULT_TRANSPARENT_PNG`
   - 策略: 使用 `lazy_static` 或 `const` 定义

4. **优化不必要的克隆**
   - 文件: `src-tauri/src/lib.rs`
   - 修改: `refresh_source` 函数，使用引用而非克隆
   - 策略: 修改 `build_artist_album_data` 签名接受 `&[Track]`

### 阶段 2: 中优先级修复（P1）

5. **修复魔法数字**
   - 文件: `src-tauri/src/audio_metadata/mod.rs`
   - 修改: 定义常量 `MAX_METADATA_SIZE: u64 = 100 * 1024 * 1024`

6. **拆分过长函数**
   - 文件: `src-tauri/src/scanner/music_scanner.rs`
   - 修改: 将 `build_artist_album_data` 拆分为多个小函数
   - 策略: 提取艺术家构建、专辑构建、曲目增强为独立函数

7. **缓存正则表达式**
   - 文件: `src-tauri/src/lyric_parser/mod.rs`
   - 修改: 使用 `lazy_static` 或 `once_cell` 缓存正则表达式

8. **修复 Option/Result 处理**
   - 文件: `src-tauri/src/lib.rs`
   - 修改: `get_track_info` 函数，正确处理错误

### 阶段 3: 低优先级修复（P2）

9. **修复未使用变量和命名问题**
   - 文件: 多处
   - 修改: 移除未使用变量，统一命名风格

### 阶段 4: 验证

10. **运行测试验证**
    - 执行 `cargo test` 确保所有测试通过
    - 执行 `cargo clippy` 检查 lint 警告
    - 执行 `cargo build` 确保编译成功

## 风险控制

- 每次只修改一个问题，确保可回滚
- 修改后运行相关测试
- 保持代码的外部行为不变
- 使用 `cargo fmt` 保持代码格式一致
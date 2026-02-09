## 新增 webDev 音乐源实现计划

### 需求分析
为前端新增 `webDev` 音乐源，需要实现从 API → 音乐源聚合 → 用户界面 → 音乐来源与获取方法管理的完整链路。

### 现有架构分析
根据代码研究，项目已有音乐源架构：
- **SourceType**: 已定义 `LocalFolder` 和 `WebDisk` 两种类型
- **SourceManager**: 管理音乐源的添加/删除/启用
- **MusicScanner**: 支持本地文件夹和网盘扫描
- **前端界面**: MusicSourceManager.vue 提供用户管理界面
- **API 层**: 前后端通过 Tauri Commands 通信

### 任务分解

#### Phase 1: 后端核心扩展 (强依赖: 无)
1. **扩展 SourceType 枚举** - 在 `source.rs` 中添加 `WebDev` 变体
2. **实现 WebDev 音乐源** - 创建 `webdev_source.rs` 模块
3. **扩展 SourceManager** - 添加 `add_webdev_source` 方法
4. **扩展 MusicScanner** - 添加 `scan_webdev` 扫描逻辑

#### Phase 2: 后端 API 接口 (强依赖: Phase 1)
5. **添加 Tauri Commands** - 在 `lib.rs` 添加 `add_webdev_source` 等命令

#### Phase 3: 前端 API 层 (强依赖: Phase 2)
6. **扩展 sources.js** - 添加 `addWebDev` API 函数
7. **扩展 library.js** - 如有需要，添加 WebDev 相关逻辑

#### Phase 4: 前端界面 (强依赖: Phase 3)
8. **扩展 MusicSourceManager.vue** - 添加 WebDev 源添加界面
9. **添加 WebDev 配置组件** - 创建 `WebDevSourceForm.vue` 组件

#### Phase 5: 音乐来源与获取方法管理 (强依赖: Phase 1-4)
10. **扩展资源获取逻辑** - 在 `musicResource.js` 添加 WebDev 文件获取
11. **扩展 Track 类** - 支持 WebDev 源的音频加载

### 执行批次

```
Batch-1 (并行): [Task-1, Task-2, Task-3, Task-4]  // 后端核心
Batch-2 (串行): [Task-5]                          // 后端API
Batch-3 (并行): [Task-6, Task-7]                  // 前端API
Batch-4 (并行): [Task-8, Task-9]                  // 前端界面
Batch-5 (并行): [Task-10, Task-11]                // 资源管理
```

### 关键文件路径
- 后端: `src-tauri/src/music_source/source.rs`
- 后端: `src-tauri/src/music_source/source_manager.rs`
- 后端: `src-tauri/src/scanner/music_scanner.rs`
- 后端: `src-tauri/src/lib.rs`
- 前端: `src/api/musicSource/sources.js`
- 前端: `src/views/MusicSourceManager.vue`

### 预期产出
1. 后端支持 `WebDev` 音乐源类型
2. 用户可通过界面添加 WebDev 音乐源
3. WebDev 源的音乐可被扫描、播放和管理
4. 完整的类型定义和错误处理

请确认此计划后，我将开始执行具体实现。
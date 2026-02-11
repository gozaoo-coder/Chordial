## 问题诊断

### 1. **上帝文件/上帝对象**（最严重）

* 文件包含1300+行代码，混合了：

  * 状态管理（AppState）

  * 音乐源管理命令（add\_local\_source, remove\_source 等）

  * 音乐库查询命令（get\_track\_info, get\_album\_art 等）

  * 音频播放控制命令（play\_audio, pause\_audio 等）

  * BPM分析命令（analyze\_audio\_beat 等）

  * 窗口控制命令（toggle\_always\_on\_top 等）

  * 常量定义（DEFAULT\_TRANSPARENT\_PNG）

### 2. **重复代码模式**

* 大量命令函数重复相同的锁获取模式

* 多次重复的 `if let Some(library) = cache_manager.load_library().ok()` 模式

### 3. **错误处理不一致**

* 部分函数使用 `.unwrap()`（如 `pause_audio`）

* 部分函数使用 `.map_err(|e| e.to_string())?`

* 错误消息有些是英文，有些是中文

### 4. **函数过长**

* `scan_all_sources` 函数超过100行，职责过多

* `refresh_source` 函数逻辑复杂

### 5. **模块组织混乱**

* 命令函数散落在文件中，没有按功能分组

### 6. **死代码**

* `greet` 函数看起来是示例代码，可能不需要

***

## 修复计划

### Phase 1: 提取模块

将命令按功能分组提取到独立模块：

```
src-tauri/src/
├── lib.rs                 # 仅保留模块声明和run函数
├── commands/              # 新建目录
│   ├── mod.rs            # 命令模块聚合
│   ├── source.rs         # 音乐源管理命令
│   ├── library.rs        # 音乐库查询命令
│   ├── playback.rs       # 音频播放控制命令
│   ├── analysis.rs       # BPM分析命令
│   └── window.rs         # 窗口控制命令
├── state.rs              # AppState 定义
└── constants.rs          # 常量定义（DEFAULT_TRANSPARENT_PNG）
```

### Phase 2: 消除重复代码

* 创建辅助宏或函数来简化锁获取

* 统一错误处理模式

### Phase 3: 代码清理

* 统一错误消息语言

* 移除或保留 `greet` 函数

* 优化过长函数

### Phase 4: 验证

* 确保代码能编译通过

* 保持原有功能不变


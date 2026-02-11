# Chordial 项目性能优化计划

## 1. 时间复杂度优化

### 1.1 `build_artist_album_data` 并行化（高优先级）
**文件：** `scanner/music_scanner.rs:215-321`  
**问题：** O(n²) 算法，嵌套循环处理 artist/album  
**优化方案：**
- 使用 `rayon` 并行迭代器处理 tracks
- 预分配 HashMap 容量：`HashMap::with_capacity(tracks.len())`
- 分离 artist 和 album 构建为独立阶段

### 1.2 `decode_to_mono` 向量预分配（高优先级）
**文件：** `audio_engine/analyzer.rs:59-93`  
**问题：** `mono_samples` 未预分配，频繁扩容  
**优化方案：** 根据文件大小估算样本数并预分配

### 1.3 排除模式查找优化（中优先级）
**文件：** `scanner/music_scanner.rs:717-719`  
**问题：** `exclude_patterns.iter().any()` 线性查找 O(m)  
**优化方案：** 使用 `HashSet` 存储排除模式，O(1) 查找

### 1.4 音频混合优化（高优先级）
**文件：** `audio_engine/buffer/ring_buffer.rs:160-210`  
**问题：** 每次创建新 Vec，迭代器开销  
**优化方案：** 预分配输出缓冲区，使用索引循环替代迭代器

---

## 2. 内存管理优化

### 2.1 分析器缓存锁优化（高优先级）
**文件：** `audio_engine/analyzer.rs:10, 28-42`  
**问题：** `Mutex` 成为并行分析瓶颈  
**优化方案：** 使用 `parking_lot::RwLock` 替代 `Mutex`

### 2.2 封面数据共享（中优先级）
**文件：** `scanner/music_scanner.rs:284`  
**问题：** 大体积二进制数据克隆  
**优化方案：** 使用 `Arc<Vec<u8>>` 共享数据

### 2.3 Mutex 锁持有时间优化（中优先级）
**文件：** `scanner/music_scanner.rs:353-410`  
**问题：** `progress` Mutex 在回调期间一直持有  
**优化方案：** 缩短锁作用域，克隆数据后立即释放锁

### 2.4 使用 `parking_lot` 替代标准库 Mutex（中优先级）
**优势：** 更小内存占用、更快锁获取、不会 poison

---

## 3. 缓存使用优化

### 3.1 添加内存缓存层（高优先级）
**文件：** `cache/cache_manager.rs`  
**问题：** 只有磁盘缓存，没有内存缓存层  
**优化方案：** 使用 `moka` crate 实现 LRU 内存缓存

### 3.2 异步 I/O 操作（中优先级）
**文件：** `cache/cache_manager.rs:128-134`  
**问题：** 同步文件写入阻塞线程  
**优化方案：** 使用 `tokio::fs` 异步写入

### 3.3 分析缓存大小限制（中优先级）
**文件：** `audio_engine/analyzer.rs:10`  
**问题：** AnalysisCache 无大小限制  
**优化方案：** 实现 LRU 淘汰策略或最大条目限制

---

## 4. 推荐依赖添加

```toml
[dependencies]
dashmap = "5.5"      # 高性能并发哈希表
moka = "0.12"        # 高性能 LRU 缓存
parking_lot = "0.12" # 更快的 Mutex/RwLock
rayon = "1.8"        # 并行迭代器
walkdir = "2.4"      # 目录遍历
```

---

## 5. 优化执行顺序

**阶段一（核心性能）：**
1. build_artist_album_data 并行化
2. analyzer 缓存使用 RwLock
3. ring_buffer 预分配优化

**阶段二（内存优化）：**
4. 添加 moka 内存缓存层
5. 封面数据使用 Arc 共享
6. Mutex 锁持有时间优化

**阶段三（完善）：**
7. 异步 I/O 操作
8. 缓存大小限制
9. 其他小优化

---

## 6. 预期性能提升

| 优化项 | 预期提升 |
|--------|----------|
| build_artist_album_data 并行化 | 扫描速度提升 2-4 倍（多核） |
| analyzer RwLock | 并行分析吞吐量提升 3-5 倍 |
| ring_buffer 预分配 | 音频播放 CPU 占用降低 10-20% |
| moka 内存缓存 | 缓存读取速度提升 10-100 倍 |
| parking_lot | 锁操作延迟降低 20-50% |

---

请确认此优化计划后，我将开始执行具体的代码优化。
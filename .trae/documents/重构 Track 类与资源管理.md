## 目标
重构 Track 类以支持多歌手摘要，并创建与 Blob URL 绑定的资源管理类。

## 实施步骤

### 第一阶段：创建 BlobResource 类
创建 `src/class/BlobResource.js`：
- 包装 Blob 和 Object URL
- 与 ResourceManager 集成，实现引用计数
- 提供 `use()` 和 `release()` 方法
- 自动清理资源，防止内存泄漏

### 第二阶段：修改 Track 类
修改 `src/class/Track.js`：
1. 将 `_artistSummary` 改为 `_artistSummaries`（数组）
2. 将 `album` 改为 `_albumSummary`（AlbumSummary 类型）
3. 添加 getter `artists` 返回 ArtistSummary[]
4. 添加 getter `album` 返回 AlbumSummary
5. 更新 `getDisplayArtist()` 方法，拼接多个歌手名
6. 更新 `getPrimaryArtist()` 返回第一个歌手
7. 更新 `toJSON()` 方法

### 第三阶段：修改后端 TrackMetadata
修改 `src-tauri/src/music_source/source_manager.rs`：
1. 将 `artist_summary: Option<ArtistSummary>` 改为 `artist_summaries: Vec<ArtistSummary>`
2. 更新扫描器的 `build_artist_album_data` 方法，为每个歌手创建摘要

### 第四阶段：更新前端类型定义
修改 `src/types/track.ts`：
1. 更新 Track 接口，artist_summaries 改为数组
2. album 改为 AlbumSummary 类型

### 第五阶段：更新 API 层
修改 `src/api/musicSource/library.js` 等：
1. 确保返回的数据正确包装为 Track 实例
2. 处理多歌手摘要的转换

## 关键设计

### BlobResource 类设计
```javascript
class BlobResource {
  constructor(key, blob, resourceManager) {
    this.key = key;
    this.blob = blob;
    this.url = URL.createObjectURL(blob);
    this._resourceManager = resourceManager;
    this._released = false;
  }
  
  use() {
    this._resourceManager?.acquire(this.key);
    return this.url;
  }
  
  release() {
    this._resourceManager?.release(this.key);
  }
  
  destroy() {
    if (!this._released) {
      URL.revokeObjectURL(this.url);
      this._released = true;
    }
  }
}
```

### Track 类新设计
```javascript
class Track {
  constructor(data) {
    // 多歌手摘要数组
    this._artistSummaries = (data.artist_summaries || [])
      .map(a => new ArtistSummary(a));
    
    // 专辑摘要
    this._albumSummary = data.album_summary 
      ? new AlbumSummary(data.album_summary) 
      : null;
  }
  
  get artists() { return this._artistSummaries; }
  get album() { return this._albumSummary; }
  
  getDisplayArtist() {
    return this._artistSummaries.map(a => a.name).join(' / ') || '未知歌手';
  }
  
  getPrimaryArtist() {
    return this._artistSummaries[0] || null;
  }
}
```

## 预期收益
1. 支持多歌手展示
2. Blob URL 生命周期可控，防止内存泄漏
3. 资源可复用，提高性能
4. 类型更清晰，使用更便捷
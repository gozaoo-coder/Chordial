/**
 * 资源管理器 - 带引用计数的资源缓存
 */
class ResourceManager {
    constructor() {
        this.cache = new Map(); // 缓存资源
        this.referenceCount = new Map(); // 引用计数
        this.pendingRequests = new Map(); // 避免重复请求
    }

    /**
     * 获取资源
     * @param {string} key - 资源唯一标识
     * @param {Function} fetchFn - 获取资源的函数（返回Promise）
     * @returns {Promise<{url: string, release: Function}>}
     */
    async getResource(key, fetchFn) {
        // 如果已经有缓存，直接返回
        if (this.cache.has(key)) {
            this._increaseReference(key);
            return this._createResourceObject(key);
        }

        // 如果正在请求中，返回同一个Promise
        if (this.pendingRequests.has(key)) {
            await this.pendingRequests.get(key);
            this._increaseReference(key);
            return this._createResourceObject(key);
        }

        // 开始新请求
        const requestPromise = fetchFn();
        this.pendingRequests.set(key, requestPromise);

        try {
            const data = await requestPromise;
            const blob = new Blob([data]);
            const url = URL.createObjectURL(blob);
            
            this.cache.set(key, {
                url,
                blob,
                data
            });
            this.referenceCount.set(key, 1);
            
            return this._createResourceObject(key);
        } finally {
            this.pendingRequests.delete(key);
        }
    }

    /**
     * 增加引用计数
     */
    _increaseReference(key) {
        const count = this.referenceCount.get(key) || 0;
        this.referenceCount.set(key, count + 1);
    }

    /**
     * 创建资源对象（包含release方法）
     */
    _createResourceObject(key) {
        const resource = this.cache.get(key);
        
        return {
            url: resource.url,
            release: () => this._releaseResource(key)
        };
    }

    /**
     * 释放资源引用
     */
    _releaseResource(key) {
        const count = this.referenceCount.get(key) || 0;
        
        if (count <= 1) {
            // 最后一个引用，清理资源
            const resource = this.cache.get(key);
            if (resource) {
                URL.revokeObjectURL(resource.url);
                this.cache.delete(key);
                this.referenceCount.delete(key);
            }
        } else {
            // 还有引用，只减少计数
            this.referenceCount.set(key, count - 1);
        }
    }

    /**
     * 预加载资源
     */
    preload(key, fetchFn) {
        if (!this.cache.has(key) && !this.pendingRequests.has(key)) {
            this.getResource(key, fetchFn);
        }
    }

    /**
     * 强制清理所有资源（用于内存紧张时）
     */
    clear() {
        for (const [key, resource] of this.cache) {
            URL.revokeObjectURL(resource.url);
        }
        this.cache.clear();
        this.referenceCount.clear();
    }
}

// 使用示例
// class AlbumCoverSource {
//     constructor(albumId, manager) {
//         this.albumId = albumId;
//         this.manager = manager;
//         this.key = `album-cover-${albumId}`;
//     }

//     async get() {
//         return await this.manager.getResource(this.key, async () => {
//             // 模拟从不同来源获取数据
//             return await this._fetchCoverData();
//         });
//     }

//     async _fetchCoverData() {
//         // 实际实现：
//         // 1. 检查本地缓存
//         // 2. 检查在线源
//         // 3. 下载并存储到缓存
//         console.log(`正在获取专辑 ${this.albumId} 封面...`);
        
//         // 模拟网络请求
//         await new Promise(resolve => setTimeout(resolve, 1000));
        
//         // 返回模拟的图片数据
//         const canvas = document.createElement('canvas');
//         canvas.width = 300;
//         canvas.height = 300;
//         const ctx = canvas.getContext('2d');
        
//         // 生成简单图片
//         ctx.fillStyle = `hsl(${this.albumId % 360}, 70%, 50%)`;
//         ctx.fillRect(0, 0, 300, 300);
//         ctx.fillStyle = 'white';
//         ctx.font = '48px Arial';
//         ctx.fillText(`Album ${this.albumId}`, 50, 150);
        
//         return await new Promise(resolve => {
//             canvas.toBlob(resolve, 'image/jpeg', 0.8);
//         });
//     }
// }

// 使用
const resourceManager = new ResourceManager();

// 导出为 ES Module
export default ResourceManager;

// 这里默认导出类定义，便于浏览器或 Node 直接使用
if (typeof module !== 'undefined' && module.exports) {
  module.exports = ResourceManager;
}

/**
 * 资源管理器 - 带引用计数的资源缓存
 * 
 * # 性能优化
 * - 实现 LRU (Least Recently Used) 缓存策略
 * - 设置最大缓存条目数防止内存无限增长
 * - 创建 Blob URL 后释放原始 data 减少内存占用
 */
class ResourceManager {
    constructor(maxSize = 50) {
        this.maxSize = maxSize; // 最大缓存条目数
        this.cache = new Map(); // 使用 Map 保持插入顺序，实现 LRU
        this.referenceCount = new Map(); // 引用计数
        this.pendingRequests = new Map(); // 避免重复请求
        this.accessOrder = new Map(); // 记录访问顺序
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
            this._updateAccessOrder(key);
            return this._createResourceObject(key);
        }

        // 如果正在请求中，返回同一个Promise
        if (this.pendingRequests.has(key)) {
            await this.pendingRequests.get(key);
            this._increaseReference(key);
            this._updateAccessOrder(key);
            return this._createResourceObject(key);
        }

        // 检查是否需要清理旧资源
        this._ensureCapacity();

        // 开始新请求
        const requestPromise = fetchFn();
        this.pendingRequests.set(key, requestPromise);

        try {
            const data = await requestPromise;
            const blob = new Blob([data]);
            const url = URL.createObjectURL(blob);
            
            // 只保留 url 和 blob，释放原始 data 减少内存占用
            this.cache.set(key, {
                url,
                blob
                // data 不缓存，减少内存占用
            });
            this.referenceCount.set(key, 1);
            this._updateAccessOrder(key);
            
            return this._createResourceObject(key);
        } finally {
            this.pendingRequests.delete(key);
        }
    }

    /**
     * 确保缓存容量不超过限制
     */
    _ensureCapacity() {
        if (this.cache.size >= this.maxSize) {
            // 找到最旧的、引用计数为 0 的资源进行清理
            let cleaned = false;
            for (const [oldKey, oldResource] of this.cache) {
                const count = this.referenceCount.get(oldKey) || 0;
                if (count === 0) {
                    this._cleanupResource(oldKey);
                    cleaned = true;
                    break;
                }
            }
            
            // 如果没有找到可清理的，清理最旧的资源（即使引用计数不为 0）
            if (!cleaned && this.cache.size > 0) {
                const oldestKey = this.cache.keys().next().value;
                console.warn(`ResourceManager: 缓存已满，强制清理资源 ${oldestKey}`);
                this._cleanupResource(oldestKey);
            }
        }
    }

    /**
     * 更新访问顺序（LRU）
     */
    _updateAccessOrder(key) {
        // 删除旧位置，重新插入到最新
        if (this.cache.has(key)) {
            const resource = this.cache.get(key);
            this.cache.delete(key);
            this.cache.set(key, resource);
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
            this._cleanupResource(key);
        } else {
            // 还有引用，只减少计数
            this.referenceCount.set(key, count - 1);
        }
    }

    /**
     * 清理资源
     */
    _cleanupResource(key) {
        const resource = this.cache.get(key);
        if (resource) {
            URL.revokeObjectURL(resource.url);
            this.cache.delete(key);
            this.referenceCount.delete(key);
        }
    }

    /**
     * 获取当前引用计数
     * @param {string} key - 资源标识
     * @returns {number}
     */
    getReferenceCount(key) {
        return this.referenceCount.get(key) || 0;
    }

    /**
     * 获取资源（仅增加引用计数，不创建新资源）
     * @param {string} key - 资源标识
     * @returns {{url: string, release: Function}|null}
     */
    acquire(key) {
        if (this.cache.has(key)) {
            this._increaseReference(key);
            this._updateAccessOrder(key);
            return this._createResourceObject(key);
        }
        return null;
    }

    /**
     * 释放资源引用
     * @param {string} key - 资源标识
     */
    release(key) {
        this._releaseResource(key);
    }

    /**
     * 检查资源是否存在
     * @param {string} key - 资源标识
     * @returns {boolean}
     */
    has(key) {
        return this.cache.has(key);
    }

    /**
     * 获取资源信息
     * @param {string} key - 资源标识
     * @returns {Object|null}
     */
    getResourceInfo(key) {
        const resource = this.cache.get(key);
        if (resource) {
            return {
                key,
                url: resource.url,
                size: resource.blob?.size || 0,
                type: resource.blob?.type || 'unknown',
                referenceCount: this.getReferenceCount(key)
            };
        }
        return null;
    }

    /**
     * 获取所有资源信息
     * @returns {Array}
     */
    getAllResourceInfo() {
        const info = [];
        for (const key of this.cache.keys()) {
            info.push(this.getResourceInfo(key));
        }
        return info;
    }

    /**
     * 获取缓存统计信息
     * @returns {Object}
     */
    getStats() {
        let totalSize = 0;
        let totalReferences = 0;
        
        for (const [key, resource] of this.cache) {
            totalSize += resource.blob?.size || 0;
            totalReferences += this.referenceCount.get(key) || 0;
        }
        
        return {
            totalEntries: this.cache.size,
            maxSize: this.maxSize,
            totalSize,
            totalReferences,
            pendingRequests: this.pendingRequests.size
        };
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
        this.pendingRequests.clear();
    }

    /**
     * 释放指定资源的引用（公共接口）
     * @param {string} key - 资源标识
     */
    releaseResource(key) {
        this._releaseResource(key);
    }

    /**
     * 获取缓存中的资源（不增加引用计数）
     * @param {string} key - 资源标识
     * @returns {Object|undefined}
     */
    getCachedResource(key) {
        return this.cache.get(key);
    }
}

// 创建全局单例实例
const resourceManager = new ResourceManager();

// 导出为 ES Module
export { ResourceManager };
export { resourceManager };
export default resourceManager;

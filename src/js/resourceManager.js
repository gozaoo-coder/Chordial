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
     * 获取当前引用计数
     * @param {string} key - 资源标识
     * @returns {number}
     */
    getReferenceCount(key) {
        return this.referenceCount.get(key) || 0;
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

// 创建全局单例实例
const resourceManager = new ResourceManager();

// 导出为 ES Module
export { ResourceManager };
export { resourceManager };
export default resourceManager;

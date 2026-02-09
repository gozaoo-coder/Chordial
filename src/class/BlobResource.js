/**
 * BlobResource 类
 * 与 Blob URL 绑定的资源管理类
 * 
 * 特性：
 * - 自动引用计数
 * - 自动释放资源
 * - 支持资源复用
 * - 防止内存泄漏
 */
import { ResourceManager, resourceManager } from '@/js/resourceManager.js';

// 全局资源管理器实例
const globalResourceManager = resourceManager;

export class BlobResource {
  /**
   * 创建 BlobResource 实例
   * @param {string} key - 资源唯一标识
   * @param {Blob|ArrayBuffer|Uint8Array} data - 资源数据
   * @param {ResourceManager} [resourceManager] - 可选的资源管理器，默认使用全局实例
   */
  constructor(key, data, resourceManager = globalResourceManager) {
    this.key = key;
    this._resourceManager = resourceManager;
    this._released = false;
    this._acquired = false;

    // 创建 Blob 和 Object URL
    if (data instanceof Blob) {
      this.blob = data;
    } else if (data instanceof ArrayBuffer) {
      this.blob = new Blob([data]);
    } else if (data instanceof Uint8Array) {
      this.blob = new Blob([data.buffer]);
    } else if (Array.isArray(data)) {
      this.blob = new Blob([new Uint8Array(data).buffer]);
    } else {
      throw new Error('Invalid data type for BlobResource');
    }

    this.url = URL.createObjectURL(this.blob);
    this.size = this.blob.size;
    this.type = this.blob.type;
  }

  /**
   * 从 fetch 响应创建 BlobResource
   * @param {string} key - 资源唯一标识
   * @param {Response} response - fetch 响应对象
   * @param {ResourceManager} [resourceManager] - 可选的资源管理器
   * @returns {Promise<BlobResource>}
   */
  static async fromResponse(key, response, resourceManager) {
    const blob = await response.blob();
    return new BlobResource(key, blob, resourceManager);
  }

  /**
   * 从 ArrayBuffer 创建 BlobResource
   * @param {string} key - 资源唯一标识
   * @param {ArrayBuffer} buffer - 二进制数据
   * @param {string} [type] - MIME 类型
   * @param {ResourceManager} [resourceManager] - 可选的资源管理器
   * @returns {BlobResource}
   */
  static fromArrayBuffer(key, buffer, type = 'application/octet-stream', resourceManager) {
    const blob = new Blob([buffer], { type });
    return new BlobResource(key, blob, resourceManager);
  }

  /**
   * 使用资源（增加引用计数）
   * @returns {string} Blob URL，可用于 img 标签 src 等
   */
  use() {
    if (this._released) {
      throw new Error('Cannot use released BlobResource');
    }
    
    if (!this._acquired) {
      this._resourceManager?.acquire?.(this.key);
      this._acquired = true;
    }
    
    return this.url;
  }

  /**
   * 获取 URL（不增加引用计数）
   * @returns {string} Blob URL
   */
  getUrl() {
    if (this._released) {
      throw new Error('Cannot get URL of released BlobResource');
    }
    return this.url;
  }

  /**
   * 释放资源引用
   * 当引用计数为 0 时，自动销毁资源
   */
  release() {
    if (this._acquired && !this._released) {
      this._resourceManager?.release?.(this.key);
      this._acquired = false;
    }
  }

  /**
   * 销毁资源（立即释放内存）
   * 无论引用计数如何，立即释放资源
   */
  destroy() {
    if (!this._released) {
      // 如果已被使用，先释放引用
      if (this._acquired) {
        this.release();
      }
      
      // 释放 Object URL
      URL.revokeObjectURL(this.url);
      
      // 清理引用
      this.blob = null;
      this._released = true;
    }
  }

  /**
   * 检查资源是否已释放
   * @returns {boolean}
   */
  isReleased() {
    return this._released;
  }

  /**
   * 检查资源是否正在使用
   * @returns {boolean}
   */
  isInUse() {
    return this._acquired && !this._released;
  }

  /**
   * 创建资源的副本（新的 Blob URL）
   * @returns {BlobResource}
   */
  clone() {
    if (this._released) {
      throw new Error('Cannot clone released BlobResource');
    }
    return new BlobResource(`${this.key}_clone_${Date.now()}`, this.blob, this._resourceManager);
  }

  /**
   * 转换为 Base64 Data URL
   * @returns {Promise<string>}
   */
  async toDataUrl() {
    if (this._released) {
      throw new Error('Cannot convert released BlobResource to Data URL');
    }
    
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onloadend = () => resolve(reader.result);
      reader.onerror = reject;
      reader.readAsDataURL(this.blob);
    });
  }

  /**
   * 转换为 ArrayBuffer
   * @returns {Promise<ArrayBuffer>}
   */
  async toArrayBuffer() {
    if (this._released) {
      throw new Error('Cannot convert released BlobResource to ArrayBuffer');
    }
    
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onloadend = () => resolve(reader.result);
      reader.onerror = reject;
      reader.readAsArrayBuffer(this.blob);
    });
  }

  /**
   * 获取资源信息
   * @returns {Object}
   */
  getInfo() {
    return {
      key: this.key,
      url: this.url,
      size: this.size,
      type: this.type,
      isReleased: this._released,
      isInUse: this.isInUse()
    };
  }

  /**
   * 确保资源在使用后自动释放
   * @param {Function} callback - 使用资源的回调函数，接收 url 作为参数
   * @returns {Promise<any>} 回调函数的返回值
   */
  async withAutoRelease(callback) {
    const url = this.use();
    try {
      return await callback(url);
    } finally {
      this.release();
    }
  }
}

/**
 * 资源池管理器
 * 用于批量管理 BlobResource
 */
export class BlobResourcePool {
  constructor(resourceManager = globalResourceManager) {
    this._resources = new Map();
    this._resourceManager = resourceManager;
  }

  /**
   * 添加资源到池
   * @param {string} key - 资源标识
   * @param {Blob|ArrayBuffer} data - 资源数据
   * @returns {BlobResource}
   */
  add(key, data) {
    const resource = new BlobResource(key, data, this._resourceManager);
    this._resources.set(key, resource);
    return resource;
  }

  /**
   * 获取资源
   * @param {string} key - 资源标识
   * @returns {BlobResource|undefined}
   */
  get(key) {
    return this._resources.get(key);
  }

  /**
   * 使用资源（自动管理生命周期）
   * @param {string} key - 资源标识
   * @returns {string|null} Blob URL 或 null
   */
  use(key) {
    const resource = this._resources.get(key);
    if (resource && !resource.isReleased()) {
      return resource.use();
    }
    return null;
  }

  /**
   * 释放资源
   * @param {string} key - 资源标识
   */
  release(key) {
    const resource = this._resources.get(key);
    if (resource) {
      resource.release();
    }
  }

  /**
   * 移除并销毁资源
   * @param {string} key - 资源标识
   */
  remove(key) {
    const resource = this._resources.get(key);
    if (resource) {
      resource.destroy();
      this._resources.delete(key);
    }
  }

  /**
   * 清空所有资源
   */
  clear() {
    for (const resource of this._resources.values()) {
      resource.destroy();
    }
    this._resources.clear();
  }

  /**
   * 获取池大小
   * @returns {number}
   */
  size() {
    return this._resources.size;
  }

  /**
   * 检查是否包含资源
   * @param {string} key - 资源标识
   * @returns {boolean}
   */
  has(key) {
    const resource = this._resources.get(key);
    return resource && !resource.isReleased();
  }
}

// 导出全局资源管理器
export { globalResourceManager };

export default BlobResource;

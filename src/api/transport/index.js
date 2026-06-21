/**
 * Transport 抽象层统一出口。
 *
 * @example
 * // 在任何 API 模块中：
 * import { transport } from '@/api/transport';
 * const result = await transport.command('config_get', { key: 'theme' });
 * const url = transport.streamUrl('audio', 'local', '/path/to/song.mp3');
 *
 * // 运行时切换传输模式（调试/配置页）：
 * import { setTransportMode } from '@/api/transport';
 * setTransportMode('http');
 */

export { getTransport, setTransportMode, getTransportMode } from './selector.js';

/**
 * 便捷的单例 transport 引用。
 * 大多数情况下可直接 `import { transport } from '@/api/transport'` 使用。
 *
 * @type {import('./base.js').Transport}
 */
export const transport = {
  get command() {
    const t = getTransportModule();
    return t.command;
  },
  get streamUrl() {
    const t = getTransportModule();
    return t.streamUrl;
  },
};

// 延迟加载，避免循环依赖
import { getTransport } from './selector.js';

function getTransportModule() {
  return getTransport();
}

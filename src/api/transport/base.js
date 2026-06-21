/**
 * Transport 接口规范。
 *
 * Transport 是 Chordial 前端与后端之间的通信抽象层，
 * 封装了两种通信方式：
 * - `invoke` — Tauri 库调用形式（进程内，通过 Tauri IPC）
 * - `http`   — Web 服务器形式（网络 HTTP REST API）
 *
 * 所有 API 模块通过 transport 进行后端交互，无需关心底层通信方式。
 *
 * @interface Transport
 */

/**
 * 调用后端命令（对应 Tauri invoke）。
 *
 * @function command
 * @memberof Transport
 * @param {string} name - 命令名称（如 "config_get", "library_get_song"）
 * @param {object} [args={}] - 命令参数
 * @returns {Promise<any>} 命令返回值（与 Tauri invoke 语义一致）
 */

/**
 * 构建媒体流 URL。
 *
 * @function streamUrl
 * @memberof Transport
 * @param {'audio'|'image'|'lyric'} type - 资源类型
 * @param {string} sourceName - 来源名称（如 "local"）
 * @param {string} entityId - 来源内部的实体 ID
 * @returns {string} 可直接用于 <audio>/<img>/fetch 的 URL
 */

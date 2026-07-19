import { createApp } from "vue";
import App from "./App.vue";
import router from "./routers";
import { initWindowState } from '@/api/window.js';
import { AmllSettingsStore } from '@/stores/amllSettings.js';
import { initLibraryEvents } from '@/composables/useLibraryEvents.js';

import './style.css'
import './app.css'
import 'bootstrap-icons/font/bootstrap-icons.css';
import '@applemusic-like-lyrics/core/style.css';

// 初始化 AMLL 配置（从 localStorage 读取持久化设置）
AmllSettingsStore.init();

const app = createApp(App);

app.use(router);

app.mount("#app");

// 启动窗口状态持久化（自动恢复位置/尺寸/最大化）
initWindowState();

// 启动全局 music-library-changed 事件监听
// 后端在 local_add_folder / local_remove_folder / local_rescan 后 emit 该事件，
// 前端通过 useLibraryEvents() 订阅以触发专辑/艺人列表自动刷新
initLibraryEvents();

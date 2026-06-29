import { createApp } from "vue";
import App from "./App.vue";
import router from "./routers";
import { initWindowState } from '@/api/window.js';

import './style.css'
import './app.css'
import 'bootstrap-icons/font/bootstrap-icons.css';
import '@applemusic-like-lyrics/core/style.css';

const app = createApp(App);

app.use(router);

app.mount("#app");

// 启动窗口状态持久化（自动恢复位置/尺寸/最大化）
initWindowState();

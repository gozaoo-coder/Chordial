import { createApp } from "vue";
import App from "./App.vue";
import router from "./routers";

import './style.css'
import 'bootstrap-icons/font/bootstrap-icons.css';

const app = createApp(App);

app.use(router);

app.mount("#app");

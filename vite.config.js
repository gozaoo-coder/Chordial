import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    react({ include: /\.(jsx|tsx)$/ }),
  ],
  resolve: {
    alias: {
      "@": resolve(__dirname, "./src"),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // 构建优化：Tauri WebView (Chromium) 支持现代 ES 语法，跳过转译降级
  // 拆分 vendor chunk 降低首屏体积
  // 注意：AMLL/pixi 含 WASM，由 rollup 默认 chunking 处理，避免 manualChunks 干扰
  build: {
    target: "esnext",
    minify: "esbuild",
    sourcemap: false,
    chunkSizeWarningLimit: 1500,
    rollupOptions: {
      output: {
        manualChunks: {
          "vue-vendor": ["vue", "vue-router"],
          "tauri": ["@tauri-apps/api", "@tauri-apps/plugin-dialog", "@tauri-apps/plugin-notification", "@tauri-apps/plugin-opener", "@tauri-apps/plugin-window-state"],
        },
      },
    },
  },
}));

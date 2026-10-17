/// <reference types="vitest/config" />
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

const rootDir = path.dirname(fileURLToPath(import.meta.url));
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
    }),
    react(),
  ],

  resolve: {
    alias: {
      "@": path.resolve(rootDir, "src"),
    },
  },

  assetsInclude: ["**/*.lottie"],

  // Playwright レポート等の HTML を dep scan のエントリに含めない
  optimizeDeps: {
    entries: ["index.html"],
  },

  test: {
    environment: "node",
    include: ["tests/**/*.{test,spec}.{ts,tsx}"],
  },

  build: {
    // Vite 8 は Rolldown の codeSplitting を使う（manualChunks は非推奨）
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            {
              name: "vendor",
              test: /node_modules\/(react|react-dom)\//,
            },
            {
              name: "router",
              test: /node_modules\/@tanstack\/react-router\//,
            },
            {
              name: "animations",
              test: /node_modules\/@lottiefiles\/dotlottie-react\//,
            },
            {
              name: "i18n",
              test: /node_modules\/(i18next|react-i18next)\//,
            },
            {
              name: "icons",
              test: /node_modules\/react-icons\//,
            },
            {
              name: "tauri",
              test: /node_modules\/@tauri-apps\//,
            },
          ],
        },
        chunkFileNames: "js/[name]-[hash].js",
        entryFileNames: "js/[name]-[hash].js",
        assetFileNames: (assetInfo) => {
          const info = assetInfo.names?.[0]?.split(".") ?? [];
          const ext = info[info.length - 1];
          const assetName = assetInfo.names?.[0] ?? "";
          if (/\.(css)$/.test(assetName)) {
            return `css/[name]-[hash].${ext}`;
          }
          if (/\.(png|jpe?g|gif|svg|ico|webp)$/.test(assetName)) {
            return `images/[name]-[hash].${ext}`;
          }
          if (/\.(woff2?|ttf|eot)$/.test(assetName)) {
            return `fonts/[name]-[hash].${ext}`;
          }
          return `assets/[name]-[hash].${ext}`;
        },
      },
    },
    chunkSizeWarningLimit: 1000,
    sourcemap: false,
    target: "esnext",
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
      // 3. tell Vite to ignore watching the Tauri Rust project / e2e artifacts
      ignored: ["**/rust-native/**", "**/playwright-report/**", "**/test-results/**"],
    },
  },
}));

import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// 依据:tauri2-svelte5-shadcn/vite.config.ts —— 显式启用 runes,固定端口,相对基路径
export default defineConfig({
  plugins: [svelte({ compilerOptions: { runes: true } })],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/crates/tauri-app/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'es2021',
    outDir: 'dist',
    emptyOutDir: true,
  },
});

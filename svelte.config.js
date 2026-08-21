import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

// svelte-check 经 tsconfig.json 的 include 限定到 src/
export default {
  preprocess: vitePreprocess(),
  compilerOptions: {
    runes: true,
  },
};

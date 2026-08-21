import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

// svelte-check 配置:排除 reference/(本地参考仓库,非项目源码)
export default {
  preprocess: vitePreprocess(),
  compilerOptions: {
    runes: true,
  },
};

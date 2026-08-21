// Tauri invoke 封装:统一调用 nextool-core 经 Tauri command 暴露的函数
// 依据:tauri2-svelte5-shadcn/src/lib/commands.svelte.ts + OpenCovibe/src/lib/api.ts
// 用官方 @tauri-apps/api/core 的 invoke(非手写 __TAURI_INTERNALS__ 访问)

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

/** 调用 Tauri command,返回字符串结果或抛出错误 */
export async function invoke(cmd: string, args?: Record<string, unknown>): Promise<string> {
  return tauriInvoke<string>(cmd, args);
}

// Tauri invoke 封装:统一调用 nextool-core 经 Tauri command 暴露的函数
// 用官方 @tauri-apps/api/core 的 invoke

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

/** 调用 Tauri command,返回字符串结果或抛出错误 */
export async function invoke(cmd: string, args?: Record<string, unknown>): Promise<string> {
  return tauriInvoke<string>(cmd, args);
}

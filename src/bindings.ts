// Tauri invoke 封装:统一调用 nextool-core/fileconv 经 Tauri command 暴露的函数
// 用官方 @tauri-apps/api/core 的 invoke

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

/** 调用 Tauri command,返回结果(字符串或路径数组)或抛出错误 */
export async function invoke<T = string>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return tauriInvoke<T>(cmd, args);
}

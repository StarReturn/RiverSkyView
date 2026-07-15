import { invoke } from "@tauri-apps/api/core";
import type { AppError } from "@/types";

export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (err) {
    const appError = err as AppError;
    if (appError && appError.code && appError.message) {
      throw appError;
    }
    throw {
      code: "INTERNAL",
      message: typeof err === "string" ? err : "未知错误",
      details: err,
    } as AppError;
  }
}

import { invoke } from "@tauri-apps/api/core";
import type { TauriCommandMap, TauriCommandResultMap } from "./bindings";

export async function invokeTyped<K extends keyof TauriCommandMap>(
  command: K,
  args?: TauriCommandMap[K],
): Promise<TauriCommandResultMap[K]> {
  return invoke<TauriCommandResultMap[K]>(command, args as Record<string, unknown> | undefined);
}

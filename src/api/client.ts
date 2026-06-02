type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

let invokePromise: Promise<TauriInvoke | null> | null = null;

export function hasTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function resolveInvoke(): Promise<TauriInvoke | null> {
  if (!invokePromise) {
    if (!hasTauriRuntime()) {
      invokePromise = Promise.resolve(null);
    } else {
      invokePromise = import("@tauri-apps/api/core")
        .then((module) => (typeof module.invoke === "function" ? module.invoke : null))
        .catch(() => null);
    }
  }
  return invokePromise;
}

import type { AppSnapshot } from "../types";

export async function autoSync(snapshot: AppSnapshot): Promise<AppSnapshot> {
  return snapshot;
}

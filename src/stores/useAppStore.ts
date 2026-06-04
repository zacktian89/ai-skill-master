import { ref, provide, inject, type InjectionKey, type Ref } from "vue";
import type { AppSnapshot, ThemeMode } from "../types";
import * as api from "../api";

export interface AppStore {
  snapshot: Ref<AppSnapshot | null>;
  themeMode: Ref<ThemeMode>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  toasts: Ref<{ id: string; message: string; type: "error" | "success" | "warning" | "info" }[]>;
  setThemeMode: (next: ThemeMode) => void;
  refresh: () => Promise<void>;
  applySnapshot: (next: AppSnapshot) => void;
  setError: (err: string | null) => void;
  addToast: (message: string, type?: "error" | "success" | "warning" | "info", duration?: number) => void;
  removeToast: (id: string) => void;
}

export const AppStoreKey: InjectionKey<AppStore> = Symbol("AppStore");

const themeStorageKey = "skillmaster-theme";

function readThemeMode(): ThemeMode {
  if (typeof localStorage === "undefined") return "dark";
  return localStorage.getItem(themeStorageKey) === "light" ? "light" : "dark";
}

export function createAppStore() {
  const snapshot = ref<AppSnapshot | null>(null);
  const themeMode = ref<ThemeMode>(readThemeMode());
  const loading = ref(true);
  const error = ref<string | null>(null);
  const toasts = ref<{ id: string; message: string; type: "error" | "success" | "warning" | "info" }[]>([]);

  function setThemeMode(next: ThemeMode) {
    themeMode.value = next;
    localStorage.setItem(themeStorageKey, next);
  }

  function addToast(
    message: string,
    type: "error" | "success" | "warning" | "info" = "info",
    duration = 4000
  ) {
    const id = Math.random().toString(36).substring(2, 9);
    toasts.value.push({ id, message, type });
    if (duration > 0) {
      setTimeout(() => {
        removeToast(id);
      }, duration);
    }
  }

  function removeToast(id: string) {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }

  function setError(err: string | null) {
    error.value = err;
    if (err) {
      addToast(err, "error", 5000);
    }
  }

  function applySnapshot(next: AppSnapshot) {
    snapshot.value = next;
  }

  async function refresh() {
    loading.value = true;
    error.value = null;
    try {
      const next = await api.getSnapshot();
      snapshot.value = next;
    } catch (cause) {
      setError(String(cause));
    } finally {
      loading.value = false;
    }
  }

  const store: AppStore = {
    snapshot,
    themeMode,
    loading,
    error,
    toasts,
    setThemeMode,
    refresh,
    applySnapshot,
    setError,
    addToast,
    removeToast,
  };

  provide(AppStoreKey, store);
  return store;
}

export function useAppStore() {
  const store = inject(AppStoreKey);
  if (!store) {
    throw new Error("useAppStore must be used after createAppStore is called");
  }
  return store;
}

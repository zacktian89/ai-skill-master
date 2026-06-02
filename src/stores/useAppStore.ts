import { ref, provide, inject, type InjectionKey, type Ref } from "vue";
import type { AppSnapshot, ThemeMode } from "../types";
import * as api from "../api";

export interface AppStore {
  snapshot: Ref<AppSnapshot | null>;
  themeMode: Ref<ThemeMode>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  setThemeMode: (next: ThemeMode) => void;
  refresh: () => Promise<void>;
  applySnapshot: (next: AppSnapshot) => void;
  setError: (err: string | null) => void;
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

  function setThemeMode(next: ThemeMode) {
    themeMode.value = next;
    localStorage.setItem(themeStorageKey, next);
  }

  function setError(err: string | null) {
    error.value = err;
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
      error.value = String(cause);
    } finally {
      loading.value = false;
    }
  }

  const store: AppStore = {
    snapshot,
    themeMode,
    loading,
    error,
    setThemeMode,
    refresh,
    applySnapshot,
    setError,
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

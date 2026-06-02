import { ref, computed } from "vue";
import type { ScannedCategory } from "../types";

const scanCaches = new Map<(path: string) => Promise<ScannedCategory[]>, Map<string, ScannedCategory[]>>();

export function clearSkillScannerCaches() {
  scanCaches.clear();
}

function getScanCache(scanApi: (path: string) => Promise<ScannedCategory[]>) {
  let cache = scanCaches.get(scanApi);
  if (!cache) {
    cache = new Map();
    scanCaches.set(scanApi, cache);
  }
  return cache;
}

export function useSkillScanner(
  getPath: () => string | null | undefined,
  scanApi: (path: string) => Promise<ScannedCategory[]>,
  options?: { onError?: (err: unknown) => void }
) {
  const scannedCategories = ref<ScannedCategory[]>([]);
  const scanning = ref(false);
  const cache = getScanCache(scanApi);

  const scannedSkillsCount = computed(() => {
    return scannedCategories.value.reduce((acc, cat) => acc + cat.skills.length, 0);
  });

  async function loadScan() {
    const path = getPath();
    if (!path) {
      scannedCategories.value = [];
      return;
    }
    const cached = cache.get(path);
    if (cached) {
      scannedCategories.value = cached;
      return;
    }
    await refreshScan();
  }

  async function refreshScan() {
    const path = getPath();
    if (!path) {
      scannedCategories.value = [];
      return;
    }
    scanning.value = true;
    try {
      const nextCategories = await scanApi(path);
      cache.set(path, nextCategories);
      scannedCategories.value = nextCategories;
    } catch (err) {
      if (options?.onError) {
        options.onError(err);
      }
      scannedCategories.value = [];
    } finally {
      scanning.value = false;
    }
  }

  function clearScan() {
    scannedCategories.value = [];
  }

  return {
    scannedCategories,
    scanning,
    scannedSkillsCount,
    loadScan,
    refreshScan,
    clearScan,
  };
}

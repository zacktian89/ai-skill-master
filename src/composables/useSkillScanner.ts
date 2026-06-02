import { ref, computed } from "vue";
import type { ScannedCategory } from "../types";

export function useSkillScanner(
  getPath: () => string | null | undefined,
  scanApi: (path: string) => Promise<ScannedCategory[]>,
  options?: { onError?: (err: unknown) => void }
) {
  const scannedCategories = ref<ScannedCategory[]>([]);
  const scanning = ref(false);

  const scannedSkillsCount = computed(() => {
    return scannedCategories.value.reduce((acc, cat) => acc + cat.skills.length, 0);
  });

  async function refreshScan() {
    const path = getPath();
    if (!path) {
      scannedCategories.value = [];
      return;
    }
    scanning.value = true;
    try {
      scannedCategories.value = await scanApi(path);
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
    refreshScan,
    clearScan,
  };
}

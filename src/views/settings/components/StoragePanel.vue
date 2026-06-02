<script setup lang="ts">
import { inject } from "vue";
import { FolderOpen, RotateCcw } from "lucide-vue-next";
import * as api from "../../../api";
import { openDirectory } from "../../../utils/dialog";
import { AppStoreKey } from "../../../stores/useAppStore";
import { useAsyncAction } from "../../../composables/useAsyncAction";
import type { AppSnapshot } from "../../../types";

defineProps<{
  snapshot: AppSnapshot;
  canRebuild: boolean;
}>();

const emit = defineEmits<{
  success: [next: AppSnapshot];
}>();

const appStore = inject(AppStoreKey, null);

const { busy, run: executeAsync } = useAsyncAction({
  onError: (err) => {
    if (appStore) appStore.setError(String(err));
  }
});

async function chooseLibraryTarget() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await executeAsync(
        () => api.migrateLibrary(selected),
        (next) => emit("success", next)
      );
    }
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
  }
}

async function rebuildState() {
  await executeAsync(
    () => api.rebuildState(),
    (next) => emit("success", next)
  );
}
</script>

<template>
  <div class="detail-header">
    <div>
      <h2>存储位置</h2>
    </div>
  </div>

  <section class="detail-section">
    <dl class="detail-kv detail-kv--wide">
      <div>
        <dt>技能库路径</dt>
        <dd>{{ snapshot.state.skillLibraryPath }}</dd>
      </div>
      <div>
        <dt>状态文件</dt>
        <dd>{{ snapshot.paths.stateFile }}</dd>
      </div>
      <div>
        <dt>备份文件</dt>
        <dd>{{ snapshot.paths.backupFile }}</dd>
      </div>
    </dl>
  </section>

  <section class="detail-section">
    <div class="section-heading">
      <h3>迁移与维护</h3>
    </div>
    <div class="button-row">
      <button class="primary-button" :disabled="busy" @click="chooseLibraryTarget">
        <FolderOpen :size="16" />
        迁移技能库
      </button>
      <button v-if="canRebuild" class="secondary-button" :disabled="busy" @click="rebuildState">
        <RotateCcw :size="16" />
        重建状态
      </button>
    </div>
    <div v-if="snapshot.state.migrationNotice" class="inline-panel">
      <strong>迁移结果</strong>
      <span>{{ snapshot.state.migrationNotice.message }}</span>
      <span>新目录：{{ snapshot.state.migrationNotice.newLibraryPath }}</span>
      <span>旧目录：{{ snapshot.state.migrationNotice.oldLibraryPath }}</span>
    </div>
  </section>
</template>

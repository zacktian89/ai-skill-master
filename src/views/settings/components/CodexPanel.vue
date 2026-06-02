<script setup lang="ts">
import { inject } from "vue";
import { FolderOpen } from "lucide-vue-next";
import * as api from "../../../api";
import { openDirectory } from "../../../utils/dialog";
import StatusTag from "../../../components/StatusTag.vue";
import { AppStoreKey } from "../../../stores/useAppStore";
import { useAsyncAction } from "../../../composables/useAsyncAction";
import type { AppSnapshot } from "../../../types";

defineProps<{
  snapshot: AppSnapshot;
  applyState: {
    tone: string;
    label: string;
    message: string;
  };
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

async function chooseCodexPath() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await executeAsync(
        () => api.setCodexPath(selected),
        (next) => emit("success", next)
      );
    }
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
  }
}
</script>

<template>
  <div class="detail-header">
    <div>
      <h2>路径与链接</h2>
    </div>
    <div class="tag-row">
      <StatusTag :type="snapshot.state.codexSkillsPath ? 'success' : 'muted'">
        {{ snapshot.state.codexSkillsPath ? "已设置" : "未设置" }}
      </StatusTag>
    </div>
  </div>

  <section class="detail-section">
    <dl class="detail-kv detail-kv--wide">
      <div>
        <dt>Codex 目录</dt>
        <dd>{{ snapshot.state.codexSkillsPath || "未设置" }}</dd>
      </div>
      <div>
        <dt>当前状态</dt>
        <dd>{{ applyState.message }}</dd>
      </div>
    </dl>
  </section>

  <section class="detail-section">
    <div class="section-heading">
      <h3>Codex 目录</h3>
    </div>
    <div class="button-row">
      <button class="primary-button" :disabled="busy" @click="chooseCodexPath">
        <FolderOpen :size="16" />
        选择目录
      </button>
    </div>
    <div class="inline-panel" :class="`inline-panel--${applyState.tone}`">
      <strong>{{ applyState.label }}</strong>
      <span>{{ snapshot.state.syncStatus.message || applyState.message }}</span>
    </div>
    <div v-if="snapshot.state.syncStatus.pendingActions.length" class="issue-list">
      <div
        v-for="item in snapshot.state.syncStatus.pendingActions"
        :key="`${item.skillId}-${item.target}-${item.kind}`"
        class="issue-card issue-card--warning"
      >
        <strong>{{ item.skillId }}</strong>
        <p>{{ item.kind === 'inspect' ? '需要处理后再应用' : '链接操作未完成' }}</p>
        <small>{{ item.message }} · {{ item.target }}</small>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { inject } from "vue";
import { RotateCcw } from "lucide-vue-next";
import * as api from "../../../api";
import { AppStoreKey } from "../../../stores/useAppStore";
import { useAsyncAction } from "../../../composables/useAsyncAction";
import type { AppSnapshot, DiagnosticItem } from "../../../types";

defineProps<{
  snapshot: AppSnapshot;
  issueItems: DiagnosticItem[];
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

function issueCategory(item: DiagnosticItem): string {
  if (item.code.startsWith("sync-")) return "路径与链接";
  if (item.code === "managed-link-mismatch") return "内容冲突";
  if (item.code.startsWith("state-")) return "恢复";
  if (item.code.includes("project") || item.code === "effective-state-error") return "规则";
  return "存储";
}

function issueTone(item: DiagnosticItem): "danger" | "warning" | "neutral" {
  if (item.level === "error") return "danger";
  if (item.level === "warning") return "warning";
  return "neutral";
}

function diagnosticGuide(item: DiagnosticItem) {
  if (item.code === "skill-library-missing") {
    return {
      action: "检查技能库目录；必要时迁移到新的位置。",
    };
  }
  if (item.code === "state-rebuild-required") {
    return {
      action: "执行状态重建。",
    };
  }
  if (item.code === "state-restored-from-backup") {
    return {
      action: "检查诊断项，必要时重建状态。",
    };
  }
  if (item.code === "effective-state-error") {
    return {
      action: "检查项目规则与当前上下文记录，必要时重建状态。",
    };
  }
  return {
    action: "检查对应路径、项目或目标目录，然后重新诊断。",
  };
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
      <h2>问题与修复</h2>
    </div>
  </div>

  <section class="detail-section">
    <div class="section-heading">
      <h3>当前问题</h3>
    </div>
    <div v-if="issueItems.length" class="issue-list">
      <div
        v-for="item in issueItems"
        :key="`${item.code}-${item.detail}`"
        class="issue-card"
        :class="`issue-card--${issueTone(item)}`"
      >
        <strong>{{ item.title }}</strong>
        <p>{{ item.detail }}</p>
        <small>{{ issueCategory(item) }} · 下一步：{{ diagnosticGuide(item).action }}</small>
      </div>
    </div>
    <div v-else class="content-empty content-empty--compact">当前没有需要处理的问题。</div>
  </section>

  <section class="detail-section">
    <dl class="detail-kv">
      <div>
        <dt>状态恢复</dt>
        <dd>{{ snapshot.stateLoad.message || "当前状态文件正常加载。" }}</dd>
      </div>
      <div>
        <dt>恢复入口</dt>
        <dd>{{ canRebuild ? "可重建状态文件" : "当前不需要重建" }}</dd>
      </div>
    </dl>
  </section>

  <section class="detail-section detail-section--danger">
    <div class="section-heading">
      <h3>恢复操作</h3>
    </div>
    <div class="button-row">
      <button v-if="canRebuild" class="danger-button" :disabled="busy" @click="rebuildState">
        <RotateCcw :size="16" />
        重建状态文件
      </button>
    </div>
  </section>
</template>

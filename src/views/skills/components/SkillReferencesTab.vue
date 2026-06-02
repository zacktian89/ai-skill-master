<script setup lang="ts">
import { Trash2 } from "lucide-vue-next";
import AgentIcon from "../../../components/icons/AgentIcon.vue";
import StatusTag from "../../../components/StatusTag.vue";
import type { SkillReferenceDetail } from "../../../types";

defineProps<{
  selectedReferences: SkillReferenceDetail[];
  busy: boolean;
}>();

defineEmits<{
  "open-delete-reference": [reference: SkillReferenceDetail];
}>();

const scopeLabels: Record<string, string> = {
  user: "个人目录",
  project: "项目目录",
  custom: "自定义目录",
};

const referenceStatusLabels: Record<string, string> = {
  healthy: "正常",
  missing: "缺失",
  conflict: "冲突",
  stale: "失效",
};
</script>

<template>
  <section class="reference-pane">
    <div v-if="selectedReferences.length" class="reference-list">
      <article
        v-for="reference in selectedReferences"
        :key="reference.id"
        class="reference-row"
      >
        <div class="reference-row-main">
          <div class="reference-row-top">
            <div class="reference-title">
              <span class="reference-app-icon" :title="reference.targetName" aria-hidden="true">
                <AgentIcon :name="reference.targetName" :size="15" />
              </span>
              <strong>{{ reference.targetName }}</strong>
            </div>
            <div class="reference-actions">
              <StatusTag :type="reference.status">
                {{ referenceStatusLabels[reference.status] || reference.status }}
              </StatusTag>
              <StatusTag>{{ scopeLabels[reference.scope] || reference.scope }}</StatusTag>
              <button
                v-if="reference.removable"
                class="ghost-icon-button ghost-icon-button--danger"
                type="button"
                :disabled="busy"
                aria-label="删除引用"
                title="删除引用"
                @click="$emit('open-delete-reference', reference)"
              >
                <Trash2 :size="15" />
              </button>
            </div>
          </div>
          <code class="reference-path">{{ reference.symlinkPath }}</code>
        </div>
      </article>
    </div>

    <!-- Empty View -->
    <div v-else class="content-empty content-empty--compact">
      暂无引用。
    </div>
  </section>
</template>

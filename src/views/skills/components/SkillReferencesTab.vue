<script setup lang="ts">
import { List, Network, Plus, Trash2 } from "lucide-vue-next";
import AgentIcon from "../../../components/icons/AgentIcon.vue";
import StatusTag from "../../../components/StatusTag.vue";
import type { Skill, SkillReferenceDetail } from "../../../types";

type ReferenceViewMode = "list" | "graph";

defineProps<{
  selectedSkill: Skill;
  selectedReferences: SkillReferenceDetail[];
  referenceViewMode: ReferenceViewMode;
  busy: boolean;
}>();

defineEmits<{
  "update:referenceViewMode": [mode: ReferenceViewMode];
  "open-add-reference": [];
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
    <div class="reference-pane-header">
      <button
        class="icon-button icon-button--compact"
        type="button"
        :disabled="busy"
        aria-label="新增引用"
        title="新增引用"
        @click="$emit('open-add-reference')"
      >
        <Plus :size="16" />
      </button>
      <div class="segmented-control segmented-control--compact" aria-label="引用视图切换">
        <button
          type="button"
          :class="{ active: referenceViewMode === 'list' }"
          aria-label="列表视图"
          title="列表视图"
          @click="$emit('update:referenceViewMode', 'list')"
        >
          <List :size="15" />
        </button>
        <button
          type="button"
          :class="{ active: referenceViewMode === 'graph' }"
          aria-label="连线图"
          title="连线图"
          @click="$emit('update:referenceViewMode', 'graph')"
        >
          <Network :size="15" />
        </button>
      </div>
    </div>

    <!-- List View -->
    <div v-if="selectedReferences.length && referenceViewMode === 'list'" class="reference-list">
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

    <!-- Graph View -->
    <div v-else-if="selectedReferences.length" class="reference-graph" aria-label="Skill 引用连线图">
      <div class="reference-center-node">
        <span>当前 Skill</span>
        <strong>{{ selectedSkill.name }}</strong>
        <code>{{ selectedSkill.id }}</code>
      </div>
      <div class="reference-graph-stack">
        <article
          v-for="reference in selectedReferences"
          :key="reference.id"
          class="reference-graph-item"
        >
          <div class="reference-line" aria-hidden="true"></div>
          <div class="reference-node">
            <div>
              <strong>{{ reference.targetName }}</strong>
              <span>{{ referenceStatusLabels[reference.status] || reference.status }}</span>
            </div>
            <small>{{ reference.symlinkPath }}</small>
          </div>
        </article>
      </div>
    </div>

    <!-- Empty View -->
    <div v-else class="content-empty content-empty--compact">
      暂无引用。
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { CircleHelp, Folder, Github, Plus, ShoppingBag, Trash2 } from "lucide-vue-next";
import SkillDescriptionTab from "./SkillDescriptionTab.vue";
import SkillReferencesTab from "./SkillReferencesTab.vue";
import type { Skill, SkillSourceKind, SkillReferenceDetail } from "../../../types";

type DetailTab = "references" | "description";

const props = defineProps<{
  selectedSkill: Skill;
  selectedReferences: SkillReferenceDetail[];
  selectedIssues: any[];
  isMarkdownLoading: boolean;
  skillMarkdown: string | null;
  parsedMarkdown: {
    metadata: Record<string, string>;
    body: string;
  };
  renderedMarkdown: string;
  activeDetailTab: DetailTab;
  busy: boolean;
}>();

defineEmits<{
  "update:activeDetailTab": [tab: DetailTab];
  "delete-click": [];
  "open-add-reference": [];
  "open-delete-reference": [reference: SkillReferenceDetail];
}>();

const sourceIcons = {
  local: Folder,
  github: Github,
  openclawMarket: ShoppingBag,
  unknown: CircleHelp,
} as const;

const sourceLabels = {
  local: "本地",
  github: "GitHub",
  openclawMarket: "OpenClaw Market",
  unknown: "未知来源",
} as const;

const sourceKind = computed<SkillSourceKind>(() => props.selectedSkill.source?.kind ?? "local");

const label = computed(() => props.selectedSkill.source?.label || sourceLabels[sourceKind.value]);
const icon = computed(() => sourceIcons[sourceKind.value]);
</script>

<template>
  <div class="extension-detail">
    <header class="extension-header">
      <div class="extension-identity">
        <div class="extension-icon" :title="label">
          <component :is="icon" :size="28" />
        </div>
        <div class="extension-title-group">
          <h2>{{ selectedSkill.name }}</h2>
          <div class="extension-meta">
            <code>{{ selectedSkill.id }}</code>
            <span>{{ label }}</span>
          </div>
        </div>
      </div>

      <div class="extension-command-panel">
        <div class="extension-actions">
          <button
            class="icon-button"
            type="button"
            :disabled="busy"
            aria-label="新增引用"
            title="新增引用"
            @click="$emit('open-add-reference')"
          >
            <Plus :size="16" />
          </button>
          <button
            class="danger-button danger-button--icon"
            :disabled="busy"
            aria-label="删除 Skill"
            @click="$emit('delete-click')"
          >
            <Trash2 :size="16" />
          </button>
        </div>
      </div>
    </header>

    <!-- Issue Strip -->
    <div v-if="selectedIssues.length" class="issue-strip">
      <div v-for="issue in selectedIssues" :key="issue.key">
        <strong>{{ issue.title }}</strong>
        <span>{{ issue.detail }}</span>
      </div>
    </div>

    <!-- Tabs Nav -->
    <nav class="detail-tabs" aria-label="Skill detail tabs">
      <button
        class="detail-tab"
        :class="{ active: activeDetailTab === 'description' }"
        type="button"
        @click="$emit('update:activeDetailTab', 'description')"
      >
        详情
      </button>
      <button
        class="detail-tab"
        :class="{ active: activeDetailTab === 'references' }"
        type="button"
        @click="$emit('update:activeDetailTab', 'references')"
      >
        引用
      </button>
    </nav>

    <!-- Tab Panels -->
    <SkillReferencesTab
      v-if="activeDetailTab === 'references'"
      :selected-references="selectedReferences"
      :busy="busy"
      @open-delete-reference="$emit('open-delete-reference', $event)"
    />

    <SkillDescriptionTab
      v-else
      :is-markdown-loading="isMarkdownLoading"
      :skill-markdown="skillMarkdown"
      :parsed-markdown="parsedMarkdown"
      :rendered-markdown="renderedMarkdown"
    />
  </div>
</template>

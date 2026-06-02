<script setup lang="ts">
import { computed } from "vue";
import { ArrowLeft, CircleHelp, Folder, Github, ShoppingBag } from "lucide-vue-next";
import SkillDescriptionTab from "../views/skills/components/SkillDescriptionTab.vue";
import SkillActionMenu from "./SkillActionMenu.vue";
import type { ProjectRule, ScannedSkill, Skill, SkillSourceKind } from "../types";

const props = defineProps<{
  skill: ScannedSkill;
  librarySkill?: Skill | null;
  rule?: ProjectRule;
  busy?: boolean;
  showCategoryTitle?: boolean;
  isMarkdownLoading: boolean;
  skillMarkdown: string | null;
  parsedMarkdown: {
    metadata: Record<string, string>;
    body: string;
  };
  renderedMarkdown: string;
}>();

const emit = defineEmits<{
  back: [];
  "toggle-rule": [skillId: string];
  "remove-reference": [skillId: string, skillPath: string];
  "import-skill": [skillPath: string];
  "delete-unmanaged-skill": [skillId: string, skillName: string, skillPath: string];
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

const sourceKind = computed<SkillSourceKind>(() => props.librarySkill?.source?.kind ?? "local");
const sourceLabel = computed(() => props.librarySkill?.source?.label || sourceLabels[sourceKind.value]);
const sourceIcon = computed(() => sourceIcons[sourceKind.value]);
const previewName = computed(() => props.librarySkill?.name ?? props.skill.name);
const previewId = computed(() => props.librarySkill?.id ?? props.skill.id);

function forwardToggleRule(skillId: string) {
  emit("toggle-rule", skillId);
}

function forwardRemoveReference(skillId: string, skillPath: string) {
  emit("remove-reference", skillId, skillPath);
}

function forwardImportSkill(skillPath: string) {
  emit("import-skill", skillPath);
}

function forwardDeleteUnmanagedSkill(skillId: string, skillName: string, skillPath: string) {
  emit("delete-unmanaged-skill", skillId, skillName, skillPath);
}
</script>

<template>
  <div class="extension-detail">
    <header class="extension-header">
      <div class="extension-identity">
        <button
          class="icon-button skill-preview-back"
          type="button"
          aria-label="返回技能列表"
          @click="$emit('back')"
        >
          <ArrowLeft :size="16" />
        </button>
        <div class="extension-icon" :title="sourceLabel">
          <component :is="sourceIcon" :size="28" />
        </div>
        <div class="extension-title-group">
          <h2>{{ previewName }}</h2>
          <div class="extension-meta">
            <code>{{ previewId }}</code>
            <span>{{ sourceLabel }}</span>
          </div>
        </div>
      </div>

      <div class="extension-command-panel">
        <div class="extension-actions">
          <SkillActionMenu
            :skill="skill"
            :rule="rule"
            :busy="busy"
            :show-category-title="showCategoryTitle"
            @toggle-rule="forwardToggleRule"
            @remove-reference="forwardRemoveReference"
            @import-skill="forwardImportSkill"
            @delete-unmanaged-skill="forwardDeleteUnmanagedSkill"
          />
        </div>
      </div>
    </header>

    <nav class="detail-tabs" aria-label="Skill detail tabs">
      <button class="detail-tab active" type="button">详情</button>
    </nav>

    <SkillDescriptionTab
      :is-markdown-loading="isMarkdownLoading"
      :skill-markdown="skillMarkdown"
      :parsed-markdown="parsedMarkdown"
      :rendered-markdown="renderedMarkdown"
    />
  </div>
</template>

<style scoped>
.extension-identity {
  grid-template-columns: auto 42px minmax(0, 1fr);
}

.skill-preview-back {
  align-self: start;
}

.detail-tab {
  cursor: default;
}
</style>

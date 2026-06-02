<script setup lang="ts">
import { Folder, Link as LinkIcon, Plus } from "lucide-vue-next";
import type { ScannedCategory, ProjectRule } from "../types";
import SkillActionMenu from "./SkillActionMenu.vue";
import { useI18n } from "../composables/useI18n";

defineProps<{
  categories: ScannedCategory[];
  rules: Record<string, ProjectRule>;
  busy?: boolean;
  showCategoryTitle?: boolean;
  showAddButton?: boolean;
  showDisabledBadge?: boolean;
}>();

const { t } = useI18n();

const emit = defineEmits<{
  "add-skill-click": [category: ScannedCategory];
  "toggle-rule": [skillId: string];
  "remove-reference": [skillId: string, skillPath: string];
  "import-skill": [skillPath: string];
  "delete-unmanaged-skill": [skillId: string, skillName: string, skillPath: string];
  "preview-skill": [skill: ScannedCategory["skills"][number]];
}>();

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
  <div class="scanned-categories-list">
    <div
      v-for="category in categories"
      :key="category.path"
      class="scanned-category-item"
      :class="{ 'scanned-category-item--with-title': showCategoryTitle }"
    >
      <!-- Category Header -->
      <div v-if="showCategoryTitle" class="scanned-category-title">
        <div class="scanned-category-left-group">
          <span>{{ t('agents.scannedModule') }}</span>
          <code>{{ category.name }}</code>
        </div>
        <button
          v-if="showAddButton"
          class="ghost-icon-button scanned-category-add-btn"
          type="button"
          :disabled="busy"
          :aria-label="t('agents.addSkillToModule')"
          :title="t('agents.addSkillToModule')"
          @click="$emit('add-skill-click', category)"
        >
          <Plus :size="12" />
        </button>
      </div>

      <!-- Skills List -->
      <div class="scanned-skills-list" :class="{ 'scanned-skills-list--no-title': !showCategoryTitle }">
        <article
          v-for="skill in category.skills"
          :key="skill.path"
          class="project-skill-row"
          :class="{ 'project-skill-row--disabled': rules[skill.id] === 'disable' }"
          role="button"
          tabindex="0"
          @click="$emit('preview-skill', skill)"
          @keydown.enter.prevent="$emit('preview-skill', skill)"
          @keydown.space.prevent="$emit('preview-skill', skill)"
        >
          <div class="project-skill-main">
            <div class="project-skill-header">
              <div class="project-skill-title-row">
                <LinkIcon
                  v-if="skill.isManaged"
                  class="project-skill-title-icon"
                  :size="15"
                  :aria-label="t('general.managed')"
                />
                <Folder
                  v-else
                  class="project-skill-title-icon"
                  :size="15"
                  :aria-label="t('general.unmanaged')"
                />
                <strong>{{ skill.name }}</strong>
                <span
                  v-if="showDisabledBadge && rules[skill.id] === 'disable'"
                  class="badge badge--error disabled-badge"
                >
                  {{ t('agents.disabledBadge') }}
                </span>
              </div>
              <div class="project-skill-actions">
                <SkillActionMenu
                  :skill="skill"
                  :rule="rules[skill.id]"
                  :busy="busy"
                  :show-category-title="showCategoryTitle"
                  @toggle-rule="forwardToggleRule"
                  @remove-reference="forwardRemoveReference"
                  @import-skill="forwardImportSkill"
                  @delete-unmanaged-skill="forwardDeleteUnmanagedSkill"
                />
              </div>
            </div>
            <div class="project-skill-meta">
              <code>{{ skill.id }}</code>
            </div>
            <div v-if="skill.description" class="project-skill-description">
              {{ skill.description }}
            </div>
          </div>
        </article>
      </div>
    </div>
  </div>
</template>

<style scoped>
.scanned-category-item--with-title {
  margin-bottom: var(--spacing-3xl);
}

.scanned-category-title {
  margin-bottom: var(--spacing-xs);
  font-weight: var(--font-weight-semibold);
  font-size: var(--font-size-md);
  color: var(--text-muted);
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.scanned-category-left-group {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.scanned-category-add-btn {
  width: var(--spacing-5xl);
  height: var(--spacing-5xl);
  border-radius: var(--radius-sm);
}

.scanned-skills-list--no-title {
  border-left: none;
  padding-left: 0;
}

.disabled-badge {
  font-size: var(--font-size-xs);
  padding: var(--spacing-2xs) var(--spacing-sm);
}

.project-skill-row {
  display: block; /* Override default grid */
  padding: var(--spacing-md);
}

.project-skill-main {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  width: 100%;
}

.project-skill-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  gap: var(--spacing-md);
}

.project-skill-title-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--spacing-xs);
  min-width: 0;
  flex: 1;
}

.project-skill-meta {
  color: var(--text-tertiary);
  font-family: ui-monospace, "SFMono-Regular", "SF Mono", "JetBrains Mono", "Menlo", monospace;
  font-size: var(--font-size-md);
  line-height: 1;
}

.project-skill-meta code {
  color: var(--text-tertiary);
  font-family: inherit;
  font-size: inherit;
}

.project-skill-description {
  color: var(--text-secondary);
  font-size: var(--font-size-md);
  line-height: 1.45;
  overflow-wrap: anywhere;
  display: block;
}

.project-skill-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-shrink: 0;
}
</style>

<script setup lang="ts">
import { Folder, Link as LinkIcon, Plus } from "lucide-vue-next";
import type { ScannedCategory, ProjectRule } from "../types";
import SkillActionMenu from "./SkillActionMenu.vue";

const props = defineProps<{
  categories: ScannedCategory[];
  rules: Record<string, ProjectRule>;
  busy?: boolean;
  showCategoryTitle?: boolean;
  showAddButton?: boolean;
  showDisabledBadge?: boolean;
}>();

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
      :style="showCategoryTitle ? { marginBottom: '20px' } : undefined"
    >
      <!-- Category Header -->
      <div
        v-if="showCategoryTitle"
        class="scanned-category-title"
        style="margin-bottom: 8px; font-weight: 600; font-size: 13px; color: var(--text-muted); display: flex; align-items: center; justify-content: space-between; width: 100%;"
      >
        <div style="display: flex; align-items: center; gap: 6px;">
          <span>📁 模块:</span>
          <code>{{ category.name }}</code>
        </div>
        <button
          v-if="showAddButton"
          class="ghost-icon-button"
          type="button"
          :disabled="busy"
          aria-label="向此模块添加skill"
          title="向此模块添加skill"
          style="width: 24px; height: 24px; border-radius: 6px;"
          @click="$emit('add-skill-click', category)"
        >
          <Plus :size="12" />
        </button>
      </div>

      <!-- Skills List -->
      <div class="scanned-skills-list" :style="!showCategoryTitle ? { borderLeft: 'none', paddingLeft: 0 } : undefined">
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
          <div class="project-skill-copy">
            <div class="project-skill-title-row">
              <LinkIcon
                v-if="skill.isManaged"
                class="project-skill-title-icon"
                :size="15"
                aria-label="已托管"
              />
              <Folder
                v-else
                class="project-skill-title-icon"
                :size="15"
                aria-label="未托管"
              />
              <strong>{{ skill.name }}</strong>
              <span
                v-if="showDisabledBadge && rules[skill.id] === 'disable'"
                class="badge badge--error"
                style="font-size: 10px; padding: 2px 6px;"
              >
                已停用
              </span>
            </div>
            <small class="project-skill-meta">
              <code>{{ skill.id }}</code>
            </small>
            <small v-if="skill.description" class="project-skill-description">
              {{ skill.description }}
            </small>
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
        </article>
      </div>
    </div>
  </div>
</template>

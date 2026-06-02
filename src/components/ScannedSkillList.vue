<script setup lang="ts">
import { computed } from "vue";
import { Folder, Link, Plus, Trash2 } from "lucide-vue-next";
import SwitchToggle from "./SwitchToggle.vue";
import type { ScannedCategory, ProjectRule } from "../types";

const props = defineProps<{
  categories: ScannedCategory[];
  rules: Record<string, ProjectRule>;
  busy?: boolean;
  showCategoryTitle?: boolean;
  showAddButton?: boolean;
  showDisabledBadge?: boolean;
}>();

defineEmits<{
  "add-skill-click": [category: ScannedCategory];
  "toggle-rule": [skillId: string];
  "remove-reference": [skillId: string, skillPath: string];
  "import-skill": [skillPath: string];
  "delete-unmanaged-skill": [skillId: string, skillName: string, skillPath: string];
}>();

const removeAriaLabel = computed(() =>
  props.showCategoryTitle ? "从项目移除技能引用" : "从 Agent 移除技能引用"
);
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
        >
          <div class="project-skill-copy">
            <div class="project-skill-title-row">
              <Link
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
            <small>
              <code>{{ skill.id }}</code>
              <span v-if="skill.description"> · {{ skill.description }}</span>
            </small>
          </div>

          <div class="project-skill-actions">
            <!-- If managed, show Enable/Disable switch + Delete reference button -->
            <template v-if="skill.isManaged">
              <SwitchToggle
                :checked="rules[skill.id] !== 'disable'"
                :disabled="busy"
                title="启用/停用技能"
                @change="$emit('toggle-rule', skill.id)"
              />

              <button
                class="ghost-icon-button ghost-icon-button--danger"
                type="button"
                :disabled="busy"
                :aria-label="removeAriaLabel"
                :title="removeAriaLabel"
                @click="$emit('remove-reference', skill.id, skill.path)"
              >
                <Trash2 :size="15" />
              </button>
            </template>

            <!-- If unmanaged, show 托管 + Delete folder buttons -->
            <template v-else>
              <button
                class="primary-button"
                type="button"
                :disabled="busy"
                style="font-size: 12px; height: 28px; padding: 0 10px;"
                @click="$emit('import-skill', skill.path)"
              >
                托管
              </button>
              <button
                class="ghost-icon-button ghost-icon-button--danger"
                type="button"
                :disabled="busy"
                aria-label="删除未托管 skill 文件夹"
                title="删除未托管 skill 文件夹"
                @click="$emit('delete-unmanaged-skill', skill.id, skill.name, skill.path)"
              >
                <Trash2 :size="15" />
              </button>
            </template>
          </div>
        </article>
      </div>
    </div>
  </div>
</template>

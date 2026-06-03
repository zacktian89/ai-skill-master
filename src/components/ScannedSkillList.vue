<script setup lang="ts">
import { ref, nextTick, onBeforeUnmount } from "vue";
import { Folder, Link as LinkIcon, Plus, ChevronRight, Power, PowerOff, Trash2 } from "lucide-vue-next";
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

const collapsedPaths = ref<Record<string, boolean>>({});

function toggleCollapse(path: string) {
  collapsedPaths.value[path] = !collapsedPaths.value[path];
}

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

const contextMenuOpen = ref<{ x: number; y: number; skill: ScannedCategory["skills"][number] } | null>(null);
const contextMenuRef = ref<HTMLElement | null>(null);
let contextMenuCloseTimer: number | null = null;
const menuMargin = 8;
const fallbackMenuWidth = 148;

function closeContextMenu() {
  contextMenuOpen.value = null;
  if (contextMenuCloseTimer !== null) {
    window.clearTimeout(contextMenuCloseTimer);
    contextMenuCloseTimer = null;
  }
  document.removeEventListener("click", closeContextMenu);
  document.removeEventListener("keydown", contextMenuOnEscape);
}

function contextMenuOnEscape(event: KeyboardEvent) {
  if (event.key === "Escape") closeContextMenu();
}

function clampMenuPosition(x: number, y: number, width: number, height: number) {
  const maxX = Math.max(menuMargin, window.innerWidth - width - menuMargin);
  const maxY = Math.max(menuMargin, window.innerHeight - height - menuMargin);
  return {
    x: Math.min(Math.max(menuMargin, x), maxX),
    y: Math.min(Math.max(menuMargin, y), maxY),
  };
}

async function handleContextMenu(event: MouseEvent, skill: ScannedCategory["skills"][number]) {
  event.preventDefault();
  closeContextMenu();
  
  const initialPosition = clampMenuPosition(event.clientX, event.clientY, fallbackMenuWidth, 0);
  contextMenuOpen.value = { ...initialPosition, skill };
  
  await nextTick();
  const menuRect = contextMenuRef.value?.getBoundingClientRect();
  if (contextMenuOpen.value && menuRect) {
    const menuWidth = menuRect.width || fallbackMenuWidth;
    const position = clampMenuPosition(event.clientX, event.clientY, menuWidth, menuRect.height);
    contextMenuOpen.value = { ...position, skill };
  }
  
  contextMenuCloseTimer = window.setTimeout(() => {
    contextMenuCloseTimer = null;
    document.addEventListener("click", closeContextMenu);
    document.addEventListener("keydown", contextMenuOnEscape);
  });
}

function runContextMenuAction(action: () => void) {
  action();
  closeContextMenu();
}

onBeforeUnmount(closeContextMenu);
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
      <div
        v-if="showCategoryTitle"
        class="scanned-category-title"
        style="cursor: pointer; user-select: none;"
        @click="toggleCollapse(category.path)"
      >
        <div class="scanned-category-left-group">
          <ChevronRight
            :size="15"
            :style="{
              transform: collapsedPaths[category.path] ? 'rotate(0deg)' : 'rotate(90deg)',
              transition: 'transform 0.2s ease',
              marginRight: '2px'
            }"
          />
          <Folder :size="15" style="color: var(--brand-600); margin-right: 2px;" />
          <code>{{ category.name }}</code>
          <span class="category-skills-count" style="margin-left: 4px; opacity: 0.6; font-size: 0.9em;">({{ category.skills.length }})</span>
        </div>
        <button
          v-if="showAddButton"
          class="ghost-icon-button scanned-category-add-btn"
          type="button"
          :disabled="busy"
          :aria-label="t('agents.addSkillToModule')"
          :title="t('agents.addSkillToModule')"
          @click.stop="$emit('add-skill-click', category)"
        >
          <Plus :size="12" />
        </button>
      </div>

      <!-- Skills List -->
      <div
        v-show="!collapsedPaths[category.path]"
        class="scanned-skills-list"
        :class="{ 'scanned-skills-list--no-title': !showCategoryTitle }"
      >
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
          @contextmenu.prevent="handleContextMenu($event, skill)"
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

  <Teleport to="body">
    <div
      v-if="contextMenuOpen"
      ref="contextMenuRef"
      class="global-context-menu"
      :style="{ left: `${contextMenuOpen.x}px`, top: `${contextMenuOpen.y}px` }"
      role="menu"
      @click.stop
    >
      <button
        v-if="contextMenuOpen?.skill?.isManaged"
        type="button"
        role="menuitem"
        class="global-context-menu-item"
        :disabled="busy"
        @click="runContextMenuAction(() => forwardToggleRule(contextMenuOpen?.skill?.id || ''))"
      >
        <Power v-if="rules[contextMenuOpen?.skill?.id || ''] === 'disable'" :size="15" />
        <PowerOff v-else :size="15" />
        <span>{{ rules[contextMenuOpen?.skill?.id || ''] === "disable" ? t("skills.open") : t("dialog.close") }}</span>
      </button>

      <button
        v-if="contextMenuOpen?.skill?.isManaged"
        type="button"
        role="menuitem"
        class="global-context-menu-item global-context-menu-item--danger"
        :disabled="busy"
        @click="runContextMenuAction(() => forwardRemoveReference(contextMenuOpen?.skill?.id || '', contextMenuOpen?.skill?.path || ''))"
      >
        <Trash2 :size="15" />
        <span>{{ t("skills.deleteReference") }}</span>
      </button>

      <button
        v-if="!contextMenuOpen?.skill?.isManaged"
        type="button"
        role="menuitem"
        class="global-context-menu-item"
        :disabled="busy"
        @click="runContextMenuAction(() => forwardImportSkill(contextMenuOpen?.skill?.path || ''))"
      >
        <LinkIcon :size="15" />
        <span>{{ t("skills.manage") }}</span>
      </button>

      <button
        v-if="!contextMenuOpen?.skill?.isManaged"
        type="button"
        role="menuitem"
        class="global-context-menu-item global-context-menu-item--danger"
        :disabled="busy"
        @click="runContextMenuAction(() => forwardDeleteUnmanagedSkill(contextMenuOpen?.skill?.id || '', contextMenuOpen?.skill?.name || '', contextMenuOpen?.skill?.path || ''))"
      >
        <Trash2 :size="15" />
        <span>{{ t("dialog.delete") }}</span>
      </button>
    </div>
  </Teleport>
</template>



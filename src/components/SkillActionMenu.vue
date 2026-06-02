<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref } from "vue";
import { Link as LinkIcon, MoreHorizontal, Power, PowerOff, Trash2 } from "lucide-vue-next";
import type { ProjectRule, ScannedSkill } from "../types";

const props = withDefaults(defineProps<{
  skill: ScannedSkill;
  rule?: ProjectRule;
  busy?: boolean;
  showCategoryTitle?: boolean;
}>(), {
  rule: "inherit",
  busy: false,
  showCategoryTitle: false,
});

const emit = defineEmits<{
  "toggle-rule": [skillId: string];
  "remove-reference": [skillId: string, skillPath: string];
  "import-skill": [skillPath: string];
  "delete-unmanaged-skill": [skillId: string, skillName: string, skillPath: string];
}>();

const removeAriaLabel = computed(() =>
  props.showCategoryTitle ? "从项目移除技能引用" : "从 Agent 移除技能引用"
);

const openMenu = ref<{ x: number; y: number } | null>(null);
const menuRef = ref<HTMLElement | null>(null);
let closeListenerTimer: number | null = null;
const menuMargin = 8;
const fallbackMenuWidth = 148;

function closeMenu() {
  openMenu.value = null;
  if (closeListenerTimer !== null) {
    window.clearTimeout(closeListenerTimer);
    closeListenerTimer = null;
  }
  document.removeEventListener("click", closeMenu);
  document.removeEventListener("keydown", closeMenuOnEscape);
}

function closeMenuOnEscape(event: KeyboardEvent) {
  if (event.key === "Escape") closeMenu();
}

function clampMenuPosition(x: number, y: number, width: number, height: number) {
  const maxX = Math.max(menuMargin, window.innerWidth - width - menuMargin);
  const maxY = Math.max(menuMargin, window.innerHeight - height - menuMargin);
  return {
    x: Math.min(Math.max(menuMargin, x), maxX),
    y: Math.min(Math.max(menuMargin, y), maxY),
  };
}

async function openSkillMenu(event: MouseEvent) {
  closeMenu();
  const initialPosition = clampMenuPosition(event.clientX - fallbackMenuWidth, event.clientY, fallbackMenuWidth, 0);
  openMenu.value = initialPosition;
  await nextTick();
  const menuRect = menuRef.value?.getBoundingClientRect();
  if (openMenu.value && menuRect) {
    const menuWidth = menuRect.width || fallbackMenuWidth;
    const position = clampMenuPosition(event.clientX - menuWidth, event.clientY, menuWidth, menuRect.height);
    openMenu.value = position;
  }
  closeListenerTimer = window.setTimeout(() => {
    closeListenerTimer = null;
    document.addEventListener("click", closeMenu);
    document.addEventListener("keydown", closeMenuOnEscape);
  });
}

function runMenuAction(action: () => void) {
  action();
  closeMenu();
}

onBeforeUnmount(closeMenu);
</script>

<template>
  <button
    class="ghost-icon-button"
    type="button"
    :disabled="busy"
    aria-label="更多技能操作"
    title="更多技能操作"
    @click.stop="openSkillMenu"
  >
    <MoreHorizontal :size="16" />
  </button>

  <Teleport to="body">
    <div
      v-if="openMenu"
      ref="menuRef"
      class="global-context-menu"
      :style="{ left: `${openMenu.x}px`, top: `${openMenu.y}px` }"
      role="menu"
      @click.stop
    >
      <button
        v-if="skill.isManaged"
        type="button"
        role="menuitem"
        class="global-context-menu-item"
        :disabled="busy"
        @click="runMenuAction(() => emit('toggle-rule', skill.id))"
      >
        <Power v-if="rule === 'disable'" :size="15" />
        <PowerOff v-else :size="15" />
        <span>{{ rule === "disable" ? "打开" : "关闭" }}</span>
      </button>

      <button
        v-if="skill.isManaged"
        type="button"
        role="menuitem"
        class="global-context-menu-item global-context-menu-item--danger"
        :disabled="busy"
        :aria-label="removeAriaLabel"
        @click="runMenuAction(() => emit('remove-reference', skill.id, skill.path))"
      >
        <Trash2 :size="15" />
        <span>删除引用</span>
      </button>

      <button
        v-if="!skill.isManaged"
        type="button"
        role="menuitem"
        class="global-context-menu-item"
        :disabled="busy"
        @click="runMenuAction(() => emit('import-skill', skill.path))"
      >
        <LinkIcon :size="15" />
        <span>托管</span>
      </button>

      <button
        v-if="!skill.isManaged"
        type="button"
        role="menuitem"
        class="global-context-menu-item global-context-menu-item--danger"
        :disabled="busy"
        aria-label="删除未托管 skill 文件夹"
        @click="runMenuAction(() => emit('delete-unmanaged-skill', skill.id, skill.name, skill.path))"
      >
        <Trash2 :size="15" />
        <span>删除</span>
      </button>
    </div>
  </Teleport>
</template>

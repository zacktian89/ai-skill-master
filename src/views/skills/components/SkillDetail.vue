<script setup lang="ts">
import { computed, ref, nextTick, onBeforeUnmount } from "vue";
import { CircleHelp, Folder, Github, Plus, ShoppingBag, Trash2, MoreHorizontal, FolderOpen } from "lucide-vue-next";
import { openPath } from "@tauri-apps/plugin-opener";
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

const moreMenuOpen = ref<{ x: number; y: number } | null>(null);
const moreMenuRef = ref<HTMLElement | null>(null);
let moreMenuCloseTimer: number | null = null;
const menuMargin = 8;
const fallbackMenuWidth = 148;

function closeMoreMenu() {
  moreMenuOpen.value = null;
  if (moreMenuCloseTimer !== null) {
    window.clearTimeout(moreMenuCloseTimer);
    moreMenuCloseTimer = null;
  }
  document.removeEventListener("click", closeMoreMenu);
  document.removeEventListener("keydown", moreMenuOnEscape);
}

function moreMenuOnEscape(event: KeyboardEvent) {
  if (event.key === "Escape") closeMoreMenu();
}

function clampMenuPosition(x: number, y: number, width: number, height: number) {
  const maxX = Math.max(menuMargin, window.innerWidth - width - menuMargin);
  const maxY = Math.max(menuMargin, window.innerHeight - height - menuMargin);
  return {
    x: Math.min(Math.max(menuMargin, x), maxX),
    y: Math.min(Math.max(menuMargin, y), maxY),
  };
}

async function openMoreMenu(event: MouseEvent) {
  closeMoreMenu();
  const initialPosition = clampMenuPosition(event.clientX - fallbackMenuWidth, event.clientY, fallbackMenuWidth, 0);
  moreMenuOpen.value = initialPosition;
  await nextTick();
  const menuRect = moreMenuRef.value?.getBoundingClientRect();
  if (moreMenuOpen.value && menuRect) {
    const menuWidth = menuRect.width || fallbackMenuWidth;
    const position = clampMenuPosition(event.clientX - menuWidth, event.clientY, menuWidth, menuRect.height);
    moreMenuOpen.value = position;
  }
  moreMenuCloseTimer = window.setTimeout(() => {
    moreMenuCloseTimer = null;
    document.addEventListener("click", closeMoreMenu);
    document.addEventListener("keydown", moreMenuOnEscape);
  });
}

function runMoreMenuAction(action: () => void) {
  action();
  closeMoreMenu();
}

async function openSkillLibraryDirectory() {
  try {
    await openPath(props.selectedSkill.libraryPath);
  } catch (cause) {
    console.error(cause);
  }
}

onBeforeUnmount(closeMoreMenu);
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
            class="ghost-icon-button"
            type="button"
            :disabled="busy"
            aria-label="更多操作"
            title="更多操作"
            @click.stop="openMoreMenu"
          >
            <MoreHorizontal :size="16" />
          </button>
        </div>
      </div>

      <Teleport to="body">
        <div
          v-if="moreMenuOpen"
          ref="moreMenuRef"
          class="global-context-menu"
          :style="{ left: `${moreMenuOpen.x}px`, top: `${moreMenuOpen.y}px` }"
          role="menu"
          @click.stop
        >
          <button
            type="button"
            role="menuitem"
            class="global-context-menu-item"
            :disabled="busy"
            @click="runMoreMenuAction(() => $emit('open-add-reference'))"
          >
            <Plus :size="15" />
            <span>增加引用</span>
          </button>

          <button
            type="button"
            role="menuitem"
            class="global-context-menu-item"
            :disabled="busy"
            @click="runMoreMenuAction(() => $emit('delete-click'))"
          >
            <Trash2 :size="15" />
            <span>删除 Skill</span>
          </button>

          <button
            type="button"
            role="menuitem"
            class="global-context-menu-item"
            :disabled="busy"
            @click="runMoreMenuAction(openSkillLibraryDirectory)"
          >
            <FolderOpen :size="15" />
            <span>打开Skill目录</span>
          </button>
        </div>
      </Teleport>
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

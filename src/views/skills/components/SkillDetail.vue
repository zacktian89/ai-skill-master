<script setup lang="ts">
import { computed, ref, nextTick, onBeforeUnmount } from "vue";
import { CircleHelp, Folder, Github, Plus, ShoppingBag, Trash2, MoreHorizontal, FolderOpen, Download, Loader2 } from "lucide-vue-next";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import SkillDescriptionTab from "./SkillDescriptionTab.vue";
import SkillReferencesTab from "./SkillReferencesTab.vue";
import type { Skill, SkillSourceKind, SkillReferenceDetail } from "../../../types";
import { useI18n } from "../../../composables/useI18n";

type DetailTab = "references" | "description" | "readme";

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
  readmeMarkdown: string | null;
  isReadmeLoading: boolean;
  renderedReadme: string;
  activeDetailTab: DetailTab;
  busy: boolean;
  isStoreMode?: boolean;
  isInstalled?: boolean;
  importBusy?: boolean;
}>();

defineEmits<{
  "update:activeDetailTab": [tab: DetailTab];
  "delete-click": [];
  "open-add-reference": [];
  "open-delete-reference": [reference: SkillReferenceDetail];
  "download-click": [];
}>();

const { t } = useI18n();

const sourceIcons = {
  local: Folder,
  github: Github,
  openclawMarket: ShoppingBag,
  unknown: CircleHelp,
} as const;

const sourceLabels = computed(() => ({
  local: t("skills.sourceLocal"),
  github: t("skills.sourceGithub"),
  openclawMarket: t("skills.sourceMarket"),
  unknown: t("skills.sourceUnknown"),
}));

const sourceKind = computed<SkillSourceKind>(() => props.selectedSkill.source?.kind ?? "local");

const label = computed(() => {
  const rawLabel = props.selectedSkill.source?.label;
  if (rawLabel === "本地" || rawLabel === "Local") {
    return t("skills.sourceLocal");
  }
  if (rawLabel === "GitHub") {
    return t("skills.sourceGithub");
  }
  return rawLabel || sourceLabels.value[sourceKind.value];
});
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

const formattedGithubUrl = computed(() => {
  const url = props.selectedSkill.source?.url;
  if (!url) return "";
  
  let cleanUrl = url.trim();
  if (cleanUrl.startsWith("git@github.com:")) {
    cleanUrl = cleanUrl.replace("git@github.com:", "https://github.com/");
  } else if (cleanUrl.startsWith("git://github.com/")) {
    cleanUrl = cleanUrl.replace("git://github.com/", "https://github.com/");
  } else if (!/^https?:\/\//.test(cleanUrl) && cleanUrl.includes("/") && cleanUrl.split("/").length === 2) {
    cleanUrl = `https://github.com/${cleanUrl}`;
  }
  
  if (cleanUrl.endsWith(".git")) {
    cleanUrl = cleanUrl.slice(0, -4);
  }
  
  return cleanUrl;
});

async function openGithubUrl(url: string) {
  try {
    await openUrl(url);
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
            <a
              v-if="selectedSkill.source?.kind === 'github' && formattedGithubUrl"
              :href="formattedGithubUrl"
              class="github-link"
              :title="t('skills.openGithub')"
              @click.prevent="openGithubUrl(formattedGithubUrl)"
            >
              <Github :size="13" />
              <span>{{ t('skills.openGithub') }}</span>
            </a>
          </div>
        </div>
      </div>

      <div class="extension-command-panel">
        <div class="extension-actions">
          <button
            v-if="isStoreMode && !isInstalled"
            class="primary-button primary-button--sm"
            type="button"
            :disabled="importBusy"
            :aria-label="t('store.download')"
            @click="$emit('download-click')"
          >
            <Loader2 v-if="importBusy" :size="14" class="spin-animation" />
            <Download v-else :size="14" />
            <span>{{ importBusy ? 'downloading...' : 'download' }}</span>
          </button>
          <template v-else-if="isStoreMode && isInstalled">
            <button
              class="secondary-button secondary-button--sm"
              type="button"
              disabled
            >
              <span>{{ t('store.installed') }}</span>
            </button>
            <button
              class="ghost-icon-button ghost-icon-button--sm"
              type="button"
              :disabled="busy"
              :aria-label="t('skills.moreActions')"
              :title="t('skills.moreActions')"
              @click.stop="openMoreMenu"
            >
              <MoreHorizontal :size="14" />
            </button>
          </template>
          <button
            v-else
            class="ghost-icon-button"
            type="button"
            :disabled="busy"
            :aria-label="t('skills.moreActions')"
            :title="t('skills.moreActions')"
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
            <span>{{ t('reference.addReference') }}</span>
          </button>

          <button
            type="button"
            role="menuitem"
            class="global-context-menu-item"
            :disabled="busy"
            @click="runMoreMenuAction(() => $emit('delete-click'))"
          >
            <Trash2 :size="15" />
            <span>{{ t('skills.deleteSkill') }}</span>
          </button>

          <button
            type="button"
            role="menuitem"
            class="global-context-menu-item"
            :disabled="busy"
            @click="runMoreMenuAction(openSkillLibraryDirectory)"
          >
            <FolderOpen :size="15" />
            <span>{{ t('skills.openDirectory') }}</span>
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
        {{ t('skills.tabDetail') }}
      </button>
      <button
        v-if="selectedSkill.source?.kind === 'github'"
        class="detail-tab"
        :class="{ active: activeDetailTab === 'readme' }"
        type="button"
        @click="$emit('update:activeDetailTab', 'readme')"
      >
        {{ t('skills.tabReadme') }}
      </button>
      <button
        class="detail-tab"
        :class="{ active: activeDetailTab === 'references' }"
        type="button"
        @click="$emit('update:activeDetailTab', 'references')"
      >
        {{ t('skills.tabReferences') }}
      </button>
    </nav>

    <!-- Tab Panels -->
    <SkillReferencesTab
      v-if="activeDetailTab === 'references'"
      :selected-references="selectedReferences"
      :busy="busy"
      @open-delete-reference="$emit('open-delete-reference', $event)"
    />

    <section v-else-if="activeDetailTab === 'readme'" class="description-pane">
      <div v-if="isReadmeLoading" class="preview-loading">
        <span>{{ t('skills.loading') }}</span>
      </div>
      <div v-else-if="readmeMarkdown">
        <!-- Markdown Body -->
        <div class="markdown-body" v-html="renderedReadme"></div>
      </div>
      <p v-else class="description-empty">{{ t('skills.noDescription') }}</p>
    </section>

    <SkillDescriptionTab
      v-else
      :is-markdown-loading="isMarkdownLoading"
      :skill-markdown="skillMarkdown"
      :parsed-markdown="parsedMarkdown"
      :rendered-markdown="renderedMarkdown"
    />
  </div>
</template>

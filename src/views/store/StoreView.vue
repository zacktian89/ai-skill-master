<script setup lang="ts">
import { computed, inject, onMounted, ref, watch, nextTick, onBeforeUnmount } from "vue";
import { Download, Loader2, Search, Plus, Trash2, FolderOpen } from "lucide-vue-next";
import { marked } from "marked";
import { openPath } from "@tauri-apps/plugin-opener";
import SplitPane from "../../components/SplitPane.vue";
import ListPanel from "../../components/ListPanel.vue";
import AppLoadingAnimation from "../../components/AppLoadingAnimation.vue";
import SearchInput from "../../components/SearchInput.vue";
import { useI18n } from "../../composables/useI18n";
import { AppStoreKey } from "../../stores/useAppStore";
import * as api from "../../api";
import SkillDetail from "../skills/components/SkillDetail.vue";
import DeleteSkillDialog from "../skills/components/DeleteSkillDialog.vue";
import ReferenceDialogs from "../skills/components/ReferenceDialogs.vue";
import type {
  AppSnapshot,
  ImportSkillCandidate,
  StoreLeaderboardType,
  StoreSkill,
  Skill,
  SkillReferenceDetail,
  SkillTargetProfile,
} from "../../types";

const { t, locale } = useI18n();
const appStore = inject(AppStoreKey, null);

const props = defineProps<{
  snapshot: AppSnapshot;
}>();

const emit = defineEmits<{
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const snapshot = computed(() => appStore?.snapshot.value ?? props.snapshot);

const boardStorageKey = "skillmaster-store-board";
function getSavedBoard(): StoreLeaderboardType {
  if (typeof localStorage === "undefined") return "alltime";
  const saved = localStorage.getItem(boardStorageKey);
  if (saved === "hot" || saved === "trending" || saved === "alltime") {
    return saved as StoreLeaderboardType;
  }
  return "alltime";
}

const board = ref<StoreLeaderboardType>(getSavedBoard());
const query = ref("");
const sourceFilter = ref("all");
const loading = ref(false);
const hasLoadedOnce = ref(false);
const errorMessage = ref<string | null>(null);
const storeSkills = ref<StoreSkill[]>([]);
const selectedStoreSkillId = ref<string | null>(null);
const importBusy = ref(false);
const searchDebounceMs = 250;
let searchDebounceTimer: number | null = null;
let loadBoardRequestId = 0;

const filteredSkills = computed(() => {
  if (sourceFilter.value === "all") {
    return storeSkills.value;
  }
  return storeSkills.value.filter((skill) => skill.source === sourceFilter.value);
});

const showInitialLoading = computed(() => loading.value && !hasLoadedOnce.value);

const selectedStoreSkill = computed(() => {
  const match = filteredSkills.value.find((skill) => skill.id === selectedStoreSkillId.value);
  return match ?? filteredSkills.value[0] ?? null;
});

const installedKeys = computed(() => {
  const keys = new Set<string>();
  for (const skill of snapshot.value.state.skills) {
    const repoName = normalizeRepoName(skill.source?.url ?? null);
    if (!repoName) continue;
    keys.add(`${repoName}/${skill.id}`);
  }
  return keys;
});

function normalizeRepoName(url: string | null | undefined): string | null {
  if (!url) return null;
  const match = url.match(/github\.com[/:]([^/]+\/[^/.]+)(?:\.git)?/i);
  return match?.[1] ?? null;
}

function isInstalled(skill: StoreSkill): boolean {
  return installedKeys.value.has(`${skill.source}/${skill.skillId}`);
}

function candidatesForStoreSkill(
  candidates: ImportSkillCandidate[],
  skill: StoreSkill,
  sourceSubdir: string | null
): ImportSkillCandidate[] {
  if (sourceSubdir) {
    return candidates.filter((candidate) => candidate.relativePath === "." || candidate.id === skill.skillId);
  }
  return candidates.filter((candidate) => candidate.id === skill.skillId);
}

function githubImportSource(skill: StoreSkill, subdir: string | null = null) {
  return {
    kind: "github" as const,
    url: `https://github.com/${skill.source}.git`,
    ref: null,
    subdir,
  };
}

function storeSkillKey(skill: StoreSkill): string {
  return `${skill.source}/${skill.skillId}`;
}

function formatInstalls(count: number): string {
  const lang = locale.value === "zh" ? "zh-CN" : "en-US";
  return new Intl.NumberFormat(lang, {
    notation: "compact",
    maximumFractionDigits: 1,
  }).format(count);
}

function selectStoreSkill(skillId: string) {
  selectedStoreSkillId.value = skillId;
}

function applySnapshot(nextSnapshot: AppSnapshot) {
  if (appStore) {
    appStore.applySnapshot(nextSnapshot);
  } else {
    emit("snapshot", nextSnapshot);
  }
}

function reportError(error: unknown) {
  const message = String(error);
  errorMessage.value = message;
  if (appStore) {
    appStore.setError(message);
  } else {
    emit("error", message);
  }
}

async function loadBoard() {
  const requestId = ++loadBoardRequestId;
  const searchQuery = query.value.trim();
  const activeBoard = board.value;
  loading.value = true;
  errorMessage.value = null;
  try {
    const nextSkills = searchQuery
      ? await api.searchStoreSkills(searchQuery, 60)
      : await api.fetchStoreLeaderboard(activeBoard);
    if (requestId !== loadBoardRequestId) return;
    storeSkills.value = nextSkills;
    if (!nextSkills.some((skill) => skill.id === selectedStoreSkillId.value)) {
      selectedStoreSkillId.value = nextSkills[0]?.id ?? null;
    }
    if (sourceFilter.value !== "all" && !nextSkills.some((skill) => skill.source === sourceFilter.value)) {
      sourceFilter.value = "all";
    }
  } catch (error) {
    if (requestId !== loadBoardRequestId) return;
    reportError(error);
  } finally {
    if (requestId === loadBoardRequestId) {
      loading.value = false;
      hasLoadedOnce.value = true;
    }
  }
}

function scheduleSearchLoad() {
  if (searchDebounceTimer !== null) {
    window.clearTimeout(searchDebounceTimer);
  }
  searchDebounceTimer = window.setTimeout(() => {
    searchDebounceTimer = null;
    void loadBoard();
  }, searchDebounceMs);
}

async function prepareImport() {
  const skill = selectedStoreSkill.value;
  if (!skill) return;
  importBusy.value = true;
  errorMessage.value = null;
  try {
    const sourceSubdir = await resolveStoreSkillImportSubdir(skill);
    const source = githubImportSource(skill, sourceSubdir);
    const preview = await api.previewImportSkills(source);
    const candidates = candidatesForStoreSkill(preview.candidates, skill, sourceSubdir);
    const candidate = candidates.find((candidate) => candidate.status === "ready");
    if (!candidate) {
      throw new Error(candidates[0]?.message ?? t("store.skillNotFound"));
    }
    const nextSnapshot = await api.confirmImportSkills({ source, candidateIds: [candidate.candidateId] });
    applySnapshot(nextSnapshot);
  } catch (error) {
    reportError(error);
  } finally {
    importBusy.value = false;
  }
}

type DetailTab = "references" | "description" | "readme";
const activeDetailTab = ref<DetailTab>("description");

const deleteDialogOpen = ref(false);
const referenceDialogOpen = ref(false);
const referenceDialogMode = ref<"add" | "delete">("add");
const referenceToDelete = ref<SkillReferenceDetail | null>(null);

const targetProfiles = computed<SkillTargetProfile[]>(() => snapshot.value.targetProfiles ?? []);

function openAddReferenceDialog() {
  referenceDialogMode.value = "add";
  referenceToDelete.value = null;
  referenceDialogOpen.value = true;
}

function openDeleteReferenceDialog(reference: SkillReferenceDetail) {
  referenceDialogMode.value = "delete";
  referenceToDelete.value = reference;
  referenceDialogOpen.value = true;
}

function handleSnapshotSuccess(nextSnapshot: AppSnapshot) {
  applySnapshot(nextSnapshot);
}

const contextMenuOpen = ref<{ x: number; y: number; skill: StoreSkill } | null>(null);
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

async function handleStoreSkillContextMenu(event: MouseEvent, skill: StoreSkill) {
  event.preventDefault();
  closeContextMenu();
  
  // Select the skill first when right-clicking it
  selectStoreSkill(skill.id);
  
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

async function openSkillLibraryDirectory(path: string | undefined) {
  if (!path) return;
  try {
    await openPath(path);
  } catch (cause) {
    console.error(cause);
  }
}

function cleanupStoreView() {
  closeContextMenu();
  if (searchDebounceTimer !== null) {
    window.clearTimeout(searchDebounceTimer);
    searchDebounceTimer = null;
  }
  loadBoardRequestId++;
}

onBeforeUnmount(cleanupStoreView);

const matchedInstalledSkill = computed(() => {
  if (!selectedStoreSkill.value) return null;
  return snapshot.value.state.skills.find((skill) => {
    const repoName = normalizeRepoName(skill.source?.url ?? null);
    if (!repoName) return false;
    return `${repoName}/${skill.id}` === `${selectedStoreSkill.value.source}/${selectedStoreSkill.value.skillId}`;
  });
});

const selectedSkillRelativePath = computed(() => selectedStoreSkill.value?.skillId ?? null);

const virtualSkill = computed<Skill | null>(() => {
  if (!selectedStoreSkill.value) return null;
  if (matchedInstalledSkill.value) return matchedInstalledSkill.value;
  return {
    id: selectedStoreSkill.value.skillId,
    name: selectedStoreSkill.value.name,
    description: "",
    libraryPath: "",
    source: {
      kind: "github",
      label: "GitHub",
      url: `https://github.com/${selectedStoreSkill.value.source}.git`,
      ref: null,
      subdir: selectedStoreSkill.value.skillId,
    },
    references: [],
    managedLinks: {},
    conflict: null,
  };
});

const selectedIssues = computed(() => {
  if (!matchedInstalledSkill.value) return [];
  return snapshot.value.state.syncStatus.pendingActions
    .filter((item) => item.skillId === matchedInstalledSkill.value?.id && item.kind === "inspect")
    .map((item) => ({
      key: `${item.kind}-${item.target}-${item.message}`,
      title: t("skills.needHandle"),
      detail: `${item.message} · ${item.target}`,
    }));
});

function referencesForSkill(skill: Skill): SkillReferenceDetail[] {
  return (skill.references ?? []).map((reference) => ({
    id: reference.id,
    targetName: reference.targetName,
    symlinkPath: reference.targetPath,
    scope: reference.scope,
    status: reference.status,
    removable: true,
    legacyCodex: false,
  }));
}

const selectedReferences = computed(() => {
  if (!matchedInstalledSkill.value) return [];
  return referencesForSkill(matchedInstalledSkill.value);
});

const skillMarkdown = ref("");
const isMarkdownLoading = ref(false);
let markdownRequestId = 0;
const resolvedMarkdownPaths = new Map<string, string>();

type StoreSkillMarkdownResult = {
  text: string;
  path: string;
};

function storeSkillMarkdownPaths(relativePath: string): string[] {
  const paths = [relativePath];
  if (!relativePath.startsWith("skills/")) {
    paths.push(`skills/${relativePath}`);
  }
  if (!relativePath.startsWith(".agents/skills/")) {
    paths.push(`.agents/skills/${relativePath}`);
  }
  return [...new Set(paths)];
}

async function fetchStoreSkillMarkdown(ownerRepo: string, relativePath: string): Promise<StoreSkillMarkdownResult> {
  for (const branch of ["main", "master"]) {
    for (const path of storeSkillMarkdownPaths(relativePath)) {
      const url = `https://raw.githubusercontent.com/${ownerRepo}/${branch}/${path}/SKILL.md`;
      const res = await fetch(url);
      if (res.ok) {
        return { text: await res.text(), path: `${path}/SKILL.md` };
      }
    }
    const resolvedPath = await resolveSkillMarkdownPathFromGitHubTree(ownerRepo, branch, relativePath);
    if (resolvedPath) {
      const url = `https://raw.githubusercontent.com/${ownerRepo}/${branch}/${resolvedPath}`;
      const res = await fetch(url);
      if (res.ok) {
        return { text: await res.text(), path: resolvedPath };
      }
    }
    const matchedResult = await resolveSkillMarkdownFromFrontMatter(ownerRepo, branch, relativePath);
    if (matchedResult) {
      return matchedResult;
    }
  }
  throw new Error("Failed to fetch SKILL.md");
}

type GitHubTreeItem = {
  path?: string;
  type?: string;
};

async function resolveSkillMarkdownPathFromGitHubTree(
  ownerRepo: string,
  branch: string,
  skillId: string
): Promise<string | null> {
  const skillMarkdownPaths = await fetchGitHubSkillMarkdownPaths(ownerRepo, branch);
  if (!skillMarkdownPaths.length) return null;

  const prefixes = skillIdPrefixes(skillId);
  return (
    skillMarkdownPaths.find((path) => prefixes.some((prefix) => skillMarkdownDirectory(path) === prefix)) ??
    null
  );
}

function skillIdPrefixes(skillId: string): string[] {
  const parts = skillId.split("-").filter(Boolean);
  const prefixes: string[] = [];
  for (let length = parts.length; length >= 1; length--) {
    prefixes.push(parts.slice(0, length).join("-"));
  }
  return prefixes;
}

function skillMarkdownDirectory(path: string): string {
  const parts = path.split("/");
  return parts.length > 1 ? parts[parts.length - 2] ?? "" : "";
}

async function resolveSkillMarkdownFromFrontMatter(
  ownerRepo: string,
  branch: string,
  skillId: string
): Promise<StoreSkillMarkdownResult | null> {
  const paths = await fetchGitHubSkillMarkdownPaths(ownerRepo, branch);
  for (const path of paths) {
    const url = `https://raw.githubusercontent.com/${ownerRepo}/${branch}/${path}`;
    const res = await fetch(url);
    if (!res.ok) continue;
    const text = await res.text();
    if (parseFrontMatter(text).metadata.name === skillId) {
      return { text, path };
    }
  }
  return null;
}

async function fetchGitHubSkillMarkdownPaths(ownerRepo: string, branch: string): Promise<string[]> {
  const url = `https://api.github.com/repos/${ownerRepo}/git/trees/${branch}?recursive=1`;
  const res = await fetch(url);
  if (!res.ok) return [];

  const data = (await res.json()) as { tree?: GitHubTreeItem[] };
  return (data.tree ?? [])
    .map((item) => item.path ?? "")
    .filter((path) => path.endsWith("/SKILL.md") || path === "SKILL.md");
}

function markdownPathToSubdir(path: string): string | null {
  return path.endsWith("/SKILL.md") ? path.slice(0, -"/SKILL.md".length) : null;
}

async function resolveStoreSkillImportSubdir(skill: StoreSkill): Promise<string | null> {
  const key = storeSkillKey(skill);
  const cachedPath = resolvedMarkdownPaths.get(key);
  if (cachedPath) {
    return markdownPathToSubdir(cachedPath);
  }

  try {
    const result = await fetchStoreSkillMarkdown(skill.source, skill.skillId);
    resolvedMarkdownPaths.set(key, result.path);
    return markdownPathToSubdir(result.path);
  } catch {
    return null;
  }
}

async function loadSkillMarkdown() {
  const requestId = ++markdownRequestId;
  const skill = selectedStoreSkill.value;
  const installedSkill = matchedInstalledSkill.value;
  const relativePath = selectedSkillRelativePath.value;
  if (!skill) {
    skillMarkdown.value = "";
    return;
  }

  isMarkdownLoading.value = true;
  try {
    if (installedSkill) {
      const content = await api.readSkillFile(installedSkill.id);
      if (requestId !== markdownRequestId) return;
      skillMarkdown.value = content;
    } else if (relativePath) {
      const ownerRepo = skill.source;
      const result = await fetchStoreSkillMarkdown(ownerRepo, relativePath);
      if (requestId !== markdownRequestId) return;
      resolvedMarkdownPaths.set(storeSkillKey(skill), result.path);
      skillMarkdown.value = result.text;
    } else {
      skillMarkdown.value = "";
    }
  } catch (err) {
    if (requestId !== markdownRequestId) return;
    console.error(err);
    skillMarkdown.value = t("store.loadMarkdownFailed");
  } finally {
    if (requestId === markdownRequestId) {
      isMarkdownLoading.value = false;
    }
  }
}

function parseFrontMatter(content: string) {
  const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return { metadata: {} as Record<string, string>, body: content };
  const yamlStr = match[1];
  const body = content.slice(match[0].length).trim();
  const metadata: Record<string, string> = {};
  yamlStr.split(/\r?\n/).forEach((line) => {
    const idx = line.indexOf(":");
    if (idx > -1) {
      const key = line.slice(0, idx).trim();
      const value = line
        .slice(idx + 1)
        .trim()
        .replace(/^['"]|['"]$/g, "");
      if (key) metadata[key] = value;
    }
  });
  return { metadata, body };
}

const parsedMarkdown = computed(() => {
  const { metadata, body } = parseFrontMatter(skillMarkdown.value);
  return { metadata, body };
});

const renderedMarkdown = computed(() => {
  if (!parsedMarkdown.value.body) return "";
  try {
    return marked.parse(parsedMarkdown.value.body) as string;
  } catch (err) {
    console.error("Markdown 解析失败:", err);
    return parsedMarkdown.value.body;
  }
});

const readmeMarkdown = ref("");
const isReadmeLoading = ref(false);
let readmeRequestId = 0;

async function loadReadme() {
  const requestId = ++readmeRequestId;
  const skill = selectedStoreSkill.value;
  if (!skill) {
    readmeMarkdown.value = "";
    return;
  }

  isReadmeLoading.value = true;
  readmeMarkdown.value = "";

  const ownerRepo = skill.source;
  const urlMain = `https://raw.githubusercontent.com/${ownerRepo}/main/README.md`;
  const urlMaster = `https://raw.githubusercontent.com/${ownerRepo}/master/README.md`;

  try {
    const res = await fetch(urlMain);
    if (!res.ok) {
      const resMaster = await fetch(urlMaster);
      if (resMaster.ok) {
        const text = await resMaster.text();
        if (requestId !== readmeRequestId) return;
        readmeMarkdown.value = text;
        return;
      }
      throw new Error(`Failed to fetch readme: ${res.statusText}`);
    }
    const text = await res.text();
    if (requestId !== readmeRequestId) return;
    readmeMarkdown.value = text;
  } catch (err) {
    if (requestId !== readmeRequestId) return;
    console.error(err);
    readmeMarkdown.value = t("store.loadReadmeFailed");
  } finally {
    if (requestId === readmeRequestId) {
      isReadmeLoading.value = false;
    }
  }
}

const renderedReadme = computed(() => {
  if (!readmeMarkdown.value) return "";
  try {
    return marked.parse(readmeMarkdown.value) as string;
  } catch (err) {
    console.error("Markdown 解析失败:", err);
    return readmeMarkdown.value;
  }
});

watch(
  [selectedStoreSkill, selectedSkillRelativePath, activeDetailTab],
  async ([newSkill, , newTab]) => {
    if (newSkill && newTab === "description") {
      await loadSkillMarkdown();
    }
  },
  { immediate: true }
);

watch(
  [selectedStoreSkill, activeDetailTab],
  async ([newSkill, newTab]) => {
    if (newSkill && newTab === "readme") {
      await loadReadme();
    }
  },
  { immediate: true }
);

watch(selectedStoreSkill, (newSkill) => {
  if (activeDetailTab.value === "readme" && !newSkill) {
    activeDetailTab.value = "description";
  }
});

onMounted(loadBoard);

watch(board, (newBoard) => {
  if (typeof localStorage !== "undefined") {
    localStorage.setItem(boardStorageKey, newBoard);
  }
  if (!query.value.trim()) {
    void loadBoard();
  }
});

watch(query, () => {
  scheduleSearchLoad();
});
</script>

<template>
  <SplitPane class="store-view">
    <template #left>
      <ListPanel :items="filteredSkills" :has-search="true" :empty-text="t('store.empty')">
        <template #search-row>
          <div class="list-search-row">
            <div class="store-search-stack">
              <div class="store-search-row">
                <SearchInput v-model="query" type="search" :placeholder="t('store.searchPlaceholder')" />
                <button class="ghost-icon-button" type="button" :aria-label="t('store.searchPlaceholder')" @click="loadBoard">
                  <Loader2 v-if="loading" :size="16" class="spin-animation" />
                  <Search v-else :size="16" />
                </button>
              </div>
              <div class="segmented-control" :aria-label="t('store.marketSource')">
                <button type="button" :class="{ active: board === 'alltime' }" @click="board = 'alltime'">
                  {{ t("store.alltime") }}
                </button>
                <button type="button" :class="{ active: board === 'hot' }" @click="board = 'hot'">
                  {{ t("store.hot") }}
                </button>
                <button type="button" :class="{ active: board === 'trending' }" @click="board = 'trending'">
                  {{ t("store.trending") }}
                </button>
              </div>
            </div>
          </div>
        </template>

        <template #empty>
          <AppLoadingAnimation
            v-if="showInitialLoading"
            class="store-initial-loading"
            :rows="7"
            variant="panel"
          />
          <div v-else class="content-empty">{{ t("store.empty") }}</div>
        </template>

        <button
          v-for="skill in filteredSkills"
          :key="skill.id"
          class="list-row store-list-item"
          :class="{ active: selectedStoreSkill?.id === skill.id }"
          type="button"
          @click="selectStoreSkill(skill.id)"
          @contextmenu.prevent="handleStoreSkillContextMenu($event, skill)"
        >
          <div class="store-list-top">
            <strong>{{ skill.name }}</strong>
            <svg v-if="isInstalled(skill)" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="store-installed-icon">
              <circle cx="12" cy="12" r="10" />
              <path d="m7.5 12.5 3 3 6-6" stroke-width="3.5" />
            </svg>
          </div>
          <div class="store-list-meta">
            <code>{{ skill.source }}</code>
            <span class="store-install-count">
              {{ formatInstalls(skill.installs) }}
              <Download :size="12" />
            </span>
          </div>
        </button>
      </ListPanel>
    </template>

    <template #right>
      <SkillDetail
        v-if="virtualSkill"
        :selected-skill="virtualSkill"
        :selected-references="selectedReferences"
        :selected-issues="selectedIssues"
        :is-markdown-loading="isMarkdownLoading"
        :skill-markdown="skillMarkdown"
        :parsed-markdown="parsedMarkdown"
        :rendered-markdown="renderedMarkdown"
        :readme-markdown="readmeMarkdown"
        :is-readme-loading="isReadmeLoading"
        :rendered-readme="renderedReadme"
        :active-detail-tab="activeDetailTab"
        :busy="importBusy || deleteDialogOpen || referenceDialogOpen"
        :is-store-mode="true"
        :is-installed="isInstalled(selectedStoreSkill)"
        :import-busy="importBusy"
        @update:active-detail-tab="activeDetailTab = $event"
        @download-click="prepareImport"
        @delete-click="deleteDialogOpen = true"
        @open-add-reference="openAddReferenceDialog"
        @open-delete-reference="openDeleteReferenceDialog"
      />
      <div v-else class="content-empty">{{ t("store.selectDetail") }}</div>
    </template>
  </SplitPane>

  <!-- Dialogs -->
  <DeleteSkillDialog
    v-if="deleteDialogOpen && virtualSkill"
    :show="deleteDialogOpen"
    :skill-id="virtualSkill.id"
    :skill-name="virtualSkill.name"
    @close="deleteDialogOpen = false"
    @success="handleSnapshotSuccess"
  />

  <ReferenceDialogs
    v-if="referenceDialogOpen && virtualSkill"
    :mode="referenceDialogMode"
    :skill="virtualSkill"
    :reference-to-delete="referenceToDelete"
    :target-profiles="targetProfiles"
    @close="referenceDialogOpen = false"
    @success="handleSnapshotSuccess"
  />

  <Teleport to="body">
    <div
      v-if="contextMenuOpen"
      ref="contextMenuRef"
      class="global-context-menu"
      :style="{ left: `${contextMenuOpen.x}px`, top: `${contextMenuOpen.y}px` }"
      role="menu"
      @click.stop
    >
      <template v-if="isInstalled(contextMenuOpen.skill)">
        <button
          type="button"
          role="menuitem"
          class="global-context-menu-item"
          :disabled="importBusy"
          @click="runContextMenuAction(openAddReferenceDialog)"
        >
          <Plus :size="15" />
          <span>{{ t('reference.addReference') }}</span>
        </button>

        <button
          type="button"
          role="menuitem"
          class="global-context-menu-item"
          :disabled="importBusy"
          @click="runContextMenuAction(() => { deleteDialogOpen = true; })"
        >
          <Trash2 :size="15" />
          <span>{{ t('skills.deleteSkill') }}</span>
        </button>

        <button
          type="button"
          role="menuitem"
          class="global-context-menu-item"
          :disabled="importBusy"
          @click="runContextMenuAction(() => openSkillLibraryDirectory(matchedInstalledSkill?.libraryPath))"
        >
          <FolderOpen :size="15" />
          <span>{{ t('skills.openDirectory') }}</span>
        </button>
      </template>
      <template v-else>
        <button
          type="button"
          role="menuitem"
          class="global-context-menu-item"
          :disabled="importBusy"
          @click="runContextMenuAction(prepareImport)"
        >
          <Download :size="15" />
          <span>{{ t('store.download') }}</span>
        </button>
      </template>
    </div>
  </Teleport>
</template>

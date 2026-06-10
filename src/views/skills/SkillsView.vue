<script setup lang="ts">
import { computed, ref, watch, inject, nextTick, onBeforeUnmount } from "vue";
import { Plus, Folder, Github, FolderGit, ChevronRight, FolderOpen, Trash2, ListChecks, CheckSquare, X } from "lucide-vue-next";
import { marked } from "marked";
import { openPath } from "@tauri-apps/plugin-opener";
import SplitPane from "../../components/SplitPane.vue";
import ListPanel from "../../components/ListPanel.vue";
import SearchInput from "../../components/SearchInput.vue";
import { useI18n } from "../../composables/useI18n";

import SkillListItem from "./components/SkillListItem.vue";
import SkillDetail from "./components/SkillDetail.vue";
import ImportSkillDialog from "./components/ImportSkillDialog.vue";
import DeleteSkillDialog from "./components/DeleteSkillDialog.vue";
import BatchDeleteSkillsDialog from "./components/BatchDeleteSkillsDialog.vue";
import ReferenceDialogs from "./components/ReferenceDialogs.vue";

import type {
  AppSnapshot,
  PendingSyncAction,
  Skill,
  SkillTargetProfile,
  SkillReferenceDetail,
} from "../../types";
import { AppStoreKey } from "../../stores/useAppStore";
import { SelectionStoreKey } from "../../stores/useSelectionStore";
import { useAsyncAction } from "../../composables/useAsyncAction";
import { useSkillMarkdown } from "../../composables/useSkillMarkdown";

const { t } = useI18n();

type DetailTab = "references" | "description" | "readme";

const appStore = inject(AppStoreKey, null);
const selectionStore = inject(SelectionStoreKey, null);

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedSkillId: string | null;
}>();

const emit = defineEmits<{
  "select-skill": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const snapshot = computed(() => appStore?.snapshot.value ?? props.snapshot);
const selectedSkillId = computed({
  get: () => selectionStore?.selectedSkillId.value ?? props.selectedSkillId,
  set: (val) => {
    if (selectionStore) {
      selectionStore.setSelectedSkillId(val);
    } else {
      emit("select-skill", val);
    }
  }
});

const query = ref("");

const { busy } = useAsyncAction({
  onError: (err) => {
    if (appStore) appStore.setError(String(err));
    else emit("error", String(err));
  }
});

const deleteDialogOpen = ref(false);
const batchDeleteDialogOpen = ref(false);
const importDialogOpen = ref(false);
const referenceDialogOpen = ref(false);
const referenceDialogMode = ref<"add" | "delete">("add");
const referenceToDelete = ref<SkillReferenceDetail | null>(null);

const activeDetailTab = ref<DetailTab>("description");
const batchSelectionMode = ref(false);
const selectedSkillIds = ref<Set<string>>(new Set());

// Composable for Markdown
const {
  skillMarkdown,
  isMarkdownLoading,
  parsedMarkdown,
  renderedMarkdown
} = useSkillMarkdown(
  () => selectedSkillId.value,
  () => activeDetailTab.value
);

function actionsForSkill(skillId: string): PendingSyncAction[] {
  return snapshot.value.state.syncStatus.pendingActions.filter((item) => item.skillId === skillId);
}

function isReferenced(skill: Skill): boolean {
  if ((skill.references?.length ?? 0) > 0) return true;
  return snapshot.value.state.projects.some((project) => project.rules[skill.id] === "enable");
}

const skills = computed(() => {
  const normalized = query.value.trim().toLowerCase();
  return [...snapshot.value.state.skills]
    .filter((skill) => {
      if (!normalized) return true;
      return `${skill.name} ${skill.description} ${skill.id}`.toLowerCase().includes(normalized);
    })
    .sort((left, right) => left.name.localeCompare(right.name, "zh-CN"));
});

const storageKey = "skillmaster-group-by-github";

function readGroupByGitHub(): boolean {
  if (typeof localStorage === "undefined") return false;
  return localStorage.getItem(storageKey) === "true";
}

const groupByGitHub = ref(readGroupByGitHub());

watch(groupByGitHub, (newVal) => {
  if (typeof localStorage !== "undefined") {
    localStorage.setItem(storageKey, String(newVal));
  }
});

watch(
  () => snapshot.value.state.skills.map((skill) => skill.id).join("\n"),
  () => {
    const available = new Set(snapshot.value.state.skills.map((skill) => skill.id));
    selectedSkillIds.value = new Set([...selectedSkillIds.value].filter((skillId) => available.has(skillId)));
  }
);

function getGitHubRepo(url: string | null | undefined): string | null {
  if (!url) return null;
  const match = url.match(/(?:github\.com[/:])([^/]+)\/([^/.]+)(?:\.git)?/i);
  if (match) {
    return `${match[1]}/${match[2]}`;
  }
  return null;
}

interface GroupedSkills {
  repoName: string;
  isGitHub: boolean;
  skills: Skill[];
}

const groupedSkills = computed<GroupedSkills[]>(() => {
  if (!groupByGitHub.value) return [];
  const groupsMap: Record<string, Skill[]> = {};
  const localSkills: Skill[] = [];

  for (const skill of skills.value) {
    const gitHubRepo = getGitHubRepo(skill.source?.url);
    if (skill.source?.kind === "github" && gitHubRepo) {
      if (!groupsMap[gitHubRepo]) {
        groupsMap[gitHubRepo] = [];
      }
      groupsMap[gitHubRepo].push(skill);
    } else {
      localSkills.push(skill);
    }
  }

  const result: GroupedSkills[] = [];

  // Sort GitHub groups alphabetically
  const gitHubRepos = Object.keys(groupsMap).sort((a, b) => a.localeCompare(b, "en"));
  for (const repo of gitHubRepos) {
    result.push({
      repoName: repo,
      isGitHub: true,
      skills: groupsMap[repo],
    });
  }

  // Add local skills if any
  if (localSkills.length > 0) {
    result.push({
      repoName: t("skills.localGroup"),
      isGitHub: false,
      skills: localSkills,
    });
  }

  return result;
});

const collapsedGroups = ref<Record<string, boolean>>({});

function toggleGroup(repoName: string) {
  collapsedGroups.value[repoName] = !collapsedGroups.value[repoName];
}

const selectedSkill = computed(
  () => skills.value.find((skill) => skill.id === selectedSkillId.value) ?? skills.value[0] ?? null,
);

const selectedSkills = computed(() => {
  const selected = selectedSkillIds.value;
  return [...snapshot.value.state.skills]
    .filter((skill) => selected.has(skill.id))
    .sort((left, right) => left.name.localeCompare(right.name, "zh-CN"));
});

const selectedSkillIdList = computed(() => selectedSkills.value.map((skill) => skill.id));
const selectedSkillCount = computed(() => selectedSkillIdList.value.length);

const selectedIssues = computed(() => {
  if (!selectedSkill.value) return [];
  return actionsForSkill(selectedSkill.value.id)
    .filter((item) => item.kind === "inspect")
    .map((item) => ({
      key: `${item.kind}-${item.target}-${item.message}`,
      title: t("skills.needHandle"),
      detail: `${item.message} · ${item.target}`,
    }));
});

const selectedReferences = computed(() => {
  if (!selectedSkill.value) return [];
  return referencesForSkill(selectedSkill.value);
});

const targetProfiles = computed<SkillTargetProfile[]>(() => snapshot.value.targetProfiles ?? []);

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

function handleSnapshotSuccess(nextSnapshot: AppSnapshot) {
  if (appStore) appStore.applySnapshot(nextSnapshot);
  else emit("snapshot", nextSnapshot);
}

function toggleBatchSelectionMode() {
  batchSelectionMode.value = !batchSelectionMode.value;
  if (!batchSelectionMode.value) {
    selectedSkillIds.value = new Set();
  }
}

function toggleSkillSelection(skillId: string) {
  const next = new Set(selectedSkillIds.value);
  if (next.has(skillId)) next.delete(skillId);
  else next.add(skillId);
  selectedSkillIds.value = next;
}

function selectAllVisibleSkills() {
  const next = new Set(selectedSkillIds.value);
  for (const skill of skills.value) {
    next.add(skill.id);
  }
  selectedSkillIds.value = next;
}

function clearSelectedSkills() {
  selectedSkillIds.value = new Set();
}

function handleBatchDeleteSuccess(nextSnapshot: AppSnapshot) {
  handleSnapshotSuccess(nextSnapshot);
  selectedSkillIds.value = new Set();
  batchSelectionMode.value = false;
  if (!nextSnapshot.state.skills.some((skill) => skill.id === selectedSkillId.value)) {
    selectedSkillId.value = nextSnapshot.state.skills[0]?.id ?? null;
  }
}

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

const contextMenuOpen = ref<{ x: number; y: number; skill: Skill } | null>(null);
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

async function handleContextMenu(event: MouseEvent, skill: Skill) {
  event.preventDefault();
  closeContextMenu();
  
  // Select the skill first when right-clicking it
  selectedSkillId.value = skill.id;
  
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

async function openSkillLibraryDirectory(skill: Skill | undefined) {
  if (!skill) return;
  try {
    await openPath(skill.libraryPath);
  } catch (cause) {
    console.error(cause);
  }
}

onBeforeUnmount(closeContextMenu);

const readmeMarkdown = ref("");
const isReadmeLoading = ref(false);

function getGitHubRepoAndOwner(url: string | null | undefined): { owner: string; repo: string } | null {
  if (!url) return null;
  const match = url.match(/(?:github\.com[/:])([^/]+)\/([^/.]+)(?:\.git)?/i);
  if (match) {
    return { owner: match[1], repo: match[2] };
  }
  return null;
}

async function loadReadme() {
  const skill = selectedSkill.value;
  if (!skill || skill.source?.kind !== "github") {
    readmeMarkdown.value = "";
    return;
  }

  const info = getGitHubRepoAndOwner(skill.source.url);
  if (!info) {
    readmeMarkdown.value = "";
    return;
  }

  isReadmeLoading.value = true;
  readmeMarkdown.value = "";

  const branchOrCommit = skill.source.commit || skill.source.ref || "main";
  const urlMain = `https://raw.githubusercontent.com/${info.owner}/${info.repo}/${branchOrCommit}/README.md`;
  const urlMaster = `https://raw.githubusercontent.com/${info.owner}/${info.repo}/master/README.md`;

  try {
    const res = await fetch(urlMain);
    if (!res.ok) {
      if (res.status === 404 && !skill.source.commit && !skill.source.ref) {
        const resMaster = await fetch(urlMaster);
        if (resMaster.ok) {
          readmeMarkdown.value = await resMaster.text();
          return;
        }
      }
      throw new Error(`Failed to fetch readme: ${res.statusText}`);
    }
    readmeMarkdown.value = await res.text();
  } catch (err) {
    console.error(err);
    readmeMarkdown.value = `### 读取 README 失败\n\n无法从 GitHub 加载此技能的 README.md，请检查网络连接。`;
  } finally {
    isReadmeLoading.value = false;
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

// Watch selectedSkill to reset activeTab/reset detail description markdown loading
watch(selectedSkill, (newSkill) => {
  if (activeDetailTab.value === "readme" && newSkill?.source?.kind !== "github") {
    activeDetailTab.value = "description";
  }
});

// Watch selectedSkill and activeDetailTab to load README
watch(
  [selectedSkill, activeDetailTab],
  async ([newSkill, newTab]) => {
    if (newSkill && newTab === "readme") {
      await loadReadme();
    }
  },
  { immediate: true }
);
</script>

<template>
  <SplitPane class="skills-view">
    <template #left>
      <ListPanel :items="skills" :has-search="true" :empty-text="t('skills.empty')">
        <template #search-row>
          <div class="list-search-row">
            <button
              class="ghost-icon-button"
              :class="{ active: groupByGitHub }"
              type="button"
              :title="t('skills.groupByGithub')"
              :aria-label="t('skills.groupByGithub')"
              @click="groupByGitHub = !groupByGitHub"
            >
              <FolderGit :size="18" />
            </button>
            <SearchInput v-model="query" :placeholder="t('skills.searchPlaceholder')" />
            <button
              class="ghost-icon-button"
              :class="{ active: batchSelectionMode }"
              type="button"
              :title="t('skills.batchManage')"
              :aria-label="t('skills.batchManage')"
              @click="toggleBatchSelectionMode"
            >
              <ListChecks :size="18" />
            </button>
            <button
              class="icon-button"
              type="button"
              :disabled="busy"
              :aria-label="t('skills.addSkill')"
              @click="importDialogOpen = true"
            >
              <Plus :size="18" />
            </button>
          </div>
          <div v-if="batchSelectionMode" class="batch-selection-bar">
            <span>{{ t('skills.batchSelectedCount', { count: selectedSkillCount }) }}</span>
            <div class="batch-selection-actions">
              <button
                class="secondary-button secondary-button--sm"
                type="button"
                :disabled="!skills.length"
                @click="selectAllVisibleSkills"
              >
                <CheckSquare :size="14" />
                {{ t('skills.selectCurrentList') }}
              </button>
              <button
                class="secondary-button secondary-button--sm"
                type="button"
                :disabled="selectedSkillCount === 0"
                @click="clearSelectedSkills"
              >
                <X :size="14" />
                {{ t('skills.clearSelection') }}
              </button>
              <button
                class="danger-button danger-button--sm"
                type="button"
                :disabled="busy || selectedSkillCount === 0"
                :aria-label="t('skills.batchDelete')"
                @click="batchDeleteDialogOpen = true"
              >
                <Trash2 :size="14" />
                {{ t('skills.batchDelete') }}
              </button>
            </div>
          </div>
        </template>

        <template v-if="groupByGitHub">
          <div v-for="group in groupedSkills" :key="group.repoName" class="skill-group">
            <div
              class="skill-group-header"
              role="button"
              tabindex="0"
              @click="toggleGroup(group.repoName)"
              @keydown.enter="toggleGroup(group.repoName)"
              @keydown.space.prevent="toggleGroup(group.repoName)"
            >
              <span class="skill-group-chevron" :class="{ collapsed: collapsedGroups[group.repoName] }">
                <ChevronRight :size="14" />
              </span>
              <span class="skill-group-icon">
                <component :is="group.isGitHub ? Github : Folder" :size="14" />
              </span>
              <span class="skill-group-title" :title="group.repoName">{{ group.repoName }}</span>
              <span class="skill-group-count">{{ group.skills.length }}</span>
            </div>
            <div v-if="!collapsedGroups[group.repoName]" class="skill-group-items">
              <SkillListItem
                v-for="skill in group.skills"
                :key="skill.id"
                :skill="skill"
                :is-active="selectedSkill?.id === skill.id"
                :is-referenced="isReferenced(skill)"
                :selectable="batchSelectionMode"
                :selected="selectedSkillIds.has(skill.id)"
                @select="selectedSkillId = skill.id"
                @toggle-selected="toggleSkillSelection(skill.id)"
                @contextmenu="handleContextMenu($event, skill)"
              />
            </div>
          </div>
        </template>
        <template v-else>
          <SkillListItem
            v-for="skill in skills"
            :key="skill.id"
            :skill="skill"
            :is-active="selectedSkill?.id === skill.id"
            :is-referenced="isReferenced(skill)"
            :selectable="batchSelectionMode"
            :selected="selectedSkillIds.has(skill.id)"
            @select="selectedSkillId = skill.id"
            @toggle-selected="toggleSkillSelection(skill.id)"
            @contextmenu="handleContextMenu($event, skill)"
          />
        </template>
      </ListPanel>
    </template>

    <template #right>
      <SkillDetail
        v-if="selectedSkill"
        :selected-skill="selectedSkill"
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
        :busy="busy"
        @update:active-detail-tab="activeDetailTab = $event"
        @delete-click="deleteDialogOpen = true"
        @open-add-reference="openAddReferenceDialog"
        @open-delete-reference="openDeleteReferenceDialog"
      />

      <div v-else class="content-empty">{{ t("skills.selectDetail") }}</div>
    </template>
  </SplitPane>

  <!-- Dialogs -->
  <ImportSkillDialog
    :show="importDialogOpen"
    :busy="busy"
    @close="importDialogOpen = false"
    @success="handleSnapshotSuccess"
  />

  <DeleteSkillDialog
    v-if="deleteDialogOpen && selectedSkill"
    :show="deleteDialogOpen"
    :skill-id="selectedSkill.id"
    :skill-name="selectedSkill.name"
    @close="deleteDialogOpen = false"
    @success="handleSnapshotSuccess"
  />

  <BatchDeleteSkillsDialog
    v-if="batchDeleteDialogOpen && selectedSkillIdList.length"
    :show="batchDeleteDialogOpen"
    :skill-ids="selectedSkillIdList"
    @close="batchDeleteDialogOpen = false"
    @success="handleBatchDeleteSuccess"
  />

  <ReferenceDialogs
    v-if="referenceDialogOpen && selectedSkill"
    :mode="referenceDialogMode"
    :skill="selectedSkill"
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
      <button
        type="button"
        role="menuitem"
        class="global-context-menu-item"
        :disabled="busy"
        @click="runContextMenuAction(openAddReferenceDialog)"
      >
        <Plus :size="15" />
        <span>{{ t('reference.addReference') }}</span>
      </button>

      <button
        type="button"
        role="menuitem"
        class="global-context-menu-item"
        :disabled="busy"
        @click="runContextMenuAction(() => { deleteDialogOpen = true; })"
      >
        <Trash2 :size="15" />
        <span>{{ t('skills.deleteSkill') }}</span>
      </button>

      <button
        type="button"
        role="menuitem"
        class="global-context-menu-item"
        :disabled="busy"
        @click="runContextMenuAction(() => openSkillLibraryDirectory(contextMenuOpen?.skill))"
      >
        <FolderOpen :size="15" />
        <span>{{ t('skills.openDirectory') }}</span>
      </button>
    </div>
  </Teleport>
</template>

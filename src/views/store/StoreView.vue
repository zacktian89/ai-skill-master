<script setup lang="ts">
import { computed, inject, onMounted, ref, watch } from "vue";
import { Download, ExternalLink, Loader2, RefreshCw, Search } from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";
import { marked } from "marked";
import SplitPane from "../../components/SplitPane.vue";
import ListPanel from "../../components/ListPanel.vue";
import SearchInput from "../../components/SearchInput.vue";
import ModalDialog from "../../components/ModalDialog.vue";
import StatusTag from "../../components/StatusTag.vue";
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
  PendingSyncAction,
  SkillTargetProfile,
} from "../../types";

const { t } = useI18n();
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
const errorMessage = ref<string | null>(null);
const storeSkills = ref<StoreSkill[]>([]);
const selectedStoreSkillId = ref<string | null>(null);
const importDialogOpen = ref(false);
const importBusy = ref(false);
const importCandidates = ref<ImportSkillCandidate[]>([]);
const selectedCandidateIds = ref<string[]>([]);

const sourceOptions = computed(() => {
  const values = new Set(storeSkills.value.map((skill) => skill.source));
  return Array.from(values).sort((left, right) => left.localeCompare(right, "en"));
});

const filteredSkills = computed(() => {
  if (sourceFilter.value === "all") {
    return storeSkills.value;
  }
  return storeSkills.value.filter((skill) => skill.source === sourceFilter.value);
});

const selectedStoreSkill = computed(() => {
  const match = filteredSkills.value.find((skill) => skill.id === selectedStoreSkillId.value);
  return match ?? filteredSkills.value[0] ?? null;
});

const readyCandidates = computed(() =>
  importCandidates.value.filter((candidate) => candidate.status === "ready")
);

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

function selectStoreSkill(skillId: string) {
  selectedStoreSkillId.value = skillId;
}

function selectCandidate(candidateId: string, checked: boolean) {
  const next = new Set(selectedCandidateIds.value);
  if (checked) next.add(candidateId);
  else next.delete(candidateId);
  selectedCandidateIds.value = [...next];
}

function setAllCandidates(checked: boolean) {
  selectedCandidateIds.value = checked ? readyCandidates.value.map((candidate) => candidate.candidateId) : [];
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
  loading.value = true;
  errorMessage.value = null;
  try {
    const nextSkills = query.value.trim()
      ? await api.searchStoreSkills(query.value.trim(), 60)
      : await api.fetchStoreLeaderboard(board.value);
    storeSkills.value = nextSkills;
    if (!nextSkills.some((skill) => skill.id === selectedStoreSkillId.value)) {
      selectedStoreSkillId.value = nextSkills[0]?.id ?? null;
    }
    if (sourceFilter.value !== "all" && !nextSkills.some((skill) => skill.source === sourceFilter.value)) {
      sourceFilter.value = "all";
    }
  } catch (error) {
    reportError(error);
  } finally {
    loading.value = false;
  }
}

async function openRepository() {
  if (!selectedStoreSkill.value) return;
  const repoUrl = `https://github.com/${selectedStoreSkill.value.source}`;
  await openUrl(repoUrl);
}

async function prepareImport() {
  if (!selectedStoreSkill.value) return;
  importBusy.value = true;
  errorMessage.value = null;
  try {
    const preview = await api.previewImportSkills({
      kind: "github",
      url: `https://github.com/${selectedStoreSkill.value.source}.git`,
      ref: null,
      subdir: null,
    });
    importCandidates.value = preview.candidates;
    selectedCandidateIds.value = preview.candidates
      .filter((candidate) => candidate.status === "ready" && candidate.id === selectedStoreSkill.value.skillId)
      .map((candidate) => candidate.candidateId);
    importDialogOpen.value = true;
  } catch (error) {
    reportError(error);
  } finally {
    importBusy.value = false;
  }
}

async function confirmImport() {
  if (!selectedStoreSkill.value || selectedCandidateIds.value.length === 0) return;
  importBusy.value = true;
  try {
    const nextSnapshot = await api.confirmImportSkills({
      source: {
        kind: "github",
        url: `https://github.com/${selectedStoreSkill.value.source}.git`,
        ref: null,
        subdir: null,
      },
      candidateIds: selectedCandidateIds.value,
    });
    applySnapshot(nextSnapshot);
    importDialogOpen.value = false;
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

const matchedInstalledSkill = computed(() => {
  if (!selectedStoreSkill.value) return null;
  return snapshot.value.state.skills.find((skill) => {
    const repoName = normalizeRepoName(skill.source?.url ?? null);
    if (!repoName) return false;
    return `${repoName}/${skill.id}` === `${selectedStoreSkill.value.source}/${selectedStoreSkill.value.skillId}`;
  });
});

const loadingPreview = ref(false);
const previewError = ref<string | null>(null);
const selectedSkillRelativePath = ref<string | null>(null);
const selectedSkillDescription = ref("");

const virtualSkill = computed<Skill | null>(() => {
  if (!selectedStoreSkill.value) return null;
  if (matchedInstalledSkill.value) return matchedInstalledSkill.value;
  return {
    id: selectedStoreSkill.value.skillId,
    name: selectedStoreSkill.value.name,
    description: selectedSkillDescription.value || "",
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

async function loadSkillMarkdown() {
  if (!selectedStoreSkill.value) {
    skillMarkdown.value = "";
    return;
  }

  isMarkdownLoading.value = true;
  try {
    if (matchedInstalledSkill.value) {
      const content = await api.readSkillFile(matchedInstalledSkill.value.id);
      skillMarkdown.value = content;
    } else if (selectedSkillRelativePath.value) {
      const ownerRepo = selectedStoreSkill.value.source;
      const path = selectedSkillRelativePath.value;
      const url = `https://raw.githubusercontent.com/${ownerRepo}/main/${path}/SKILL.md`;
      const res = await fetch(url);
      if (!res.ok) {
        const urlMaster = `https://raw.githubusercontent.com/${ownerRepo}/master/${path}/SKILL.md`;
        const resMaster = await fetch(urlMaster);
        if (resMaster.ok) {
          skillMarkdown.value = await resMaster.text();
          return;
        }
        throw new Error(`Failed to fetch SKILL.md`);
      }
      skillMarkdown.value = await res.text();
    } else {
      skillMarkdown.value = "";
    }
  } catch (err) {
    console.error(err);
    skillMarkdown.value = t("store.loadMarkdownFailed");
  } finally {
    isMarkdownLoading.value = false;
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

async function loadReadme() {
  if (!selectedStoreSkill.value) {
    readmeMarkdown.value = "";
    return;
  }

  isReadmeLoading.value = true;
  readmeMarkdown.value = "";

  const ownerRepo = selectedStoreSkill.value.source;
  const urlMain = `https://raw.githubusercontent.com/${ownerRepo}/main/README.md`;
  const urlMaster = `https://raw.githubusercontent.com/${ownerRepo}/master/README.md`;

  try {
    const res = await fetch(urlMain);
    if (!res.ok) {
      const resMaster = await fetch(urlMaster);
      if (resMaster.ok) {
        readmeMarkdown.value = await resMaster.text();
        return;
      }
      throw new Error(`Failed to fetch readme: ${res.statusText}`);
    }
    readmeMarkdown.value = await res.text();
  } catch (err) {
    console.error(err);
    readmeMarkdown.value = t("store.loadReadmeFailed");
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

watch(
  selectedStoreSkill,
  async (newSkill) => {
    selectedSkillRelativePath.value = null;
    selectedSkillDescription.value = "";
    previewError.value = null;
    if (!newSkill) return;

    if (matchedInstalledSkill.value) {
      return;
    }

    loadingPreview.value = true;
    try {
      const preview = await api.previewImportSkills({
        kind: "github",
        url: `https://github.com/${newSkill.source}.git`,
        ref: null,
        subdir: null,
      });
      const match = preview.candidates.find((c) => c.id === newSkill.skillId);
      if (match) {
        selectedSkillRelativePath.value = match.relativePath;
        selectedSkillDescription.value = match.description;
      } else {
        previewError.value = t("store.skillNotFound");
      }
    } catch (err) {
      console.error(err);
      previewError.value = t("store.getDetailFailed");
    } finally {
      loadingPreview.value = false;
    }
  },
  { immediate: true }
);

watch(
  [selectedStoreSkill, selectedSkillRelativePath, activeDetailTab],
  async ([newSkill, newPath, newTab]) => {
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
  void loadBoard();
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

        <button
          v-for="skill in filteredSkills"
          :key="skill.id"
          class="list-row store-list-item"
          :class="{ active: selectedStoreSkill?.id === skill.id }"
          type="button"
          @click="selectStoreSkill(skill.id)"
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
              {{ skill.installs }}
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
        :is-markdown-loading="isMarkdownLoading || loadingPreview"
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

  <ModalDialog
    v-if="importDialogOpen"
    :title="t('store.importTitle')"
    card-class="import-modal"
    @close="importDialogOpen = false"
  >
    <div class="import-modal-body">
      <section class="import-results">
        <div class="import-results-head">
          <label class="import-check-all">
            <input
              type="checkbox"
              :checked="readyCandidates.length > 0 && selectedCandidateIds.length === readyCandidates.length"
              :disabled="readyCandidates.length === 0 || importBusy"
              @change="setAllCandidates(($event.target as HTMLInputElement).checked)"
            />
            <span>{{ t('importSkill.selectedCount', { selected: selectedCandidateIds.length, total: readyCandidates.length }) }}</span>
          </label>
        </div>

        <div v-if="importCandidates.length" class="import-candidate-list">
          <label
            v-for="candidate in importCandidates"
            :key="candidate.candidateId"
            class="import-candidate-row"
            :class="{ disabled: candidate.status !== 'ready' }"
          >
            <input
              type="checkbox"
              :checked="selectedCandidateIds.includes(candidate.candidateId)"
              :disabled="candidate.status !== 'ready' || importBusy"
              @change="selectCandidate(candidate.candidateId, ($event.target as HTMLInputElement).checked)"
            />
            <span class="import-candidate-main">
              <span class="import-candidate-top">
                <strong>{{ candidate.name }}</strong>
                <StatusTag :type="candidate.status === 'ready' ? 'healthy' : candidate.status">
                  {{ candidate.status === 'ready' ? t("importSkill.ready") : candidate.message }}
                </StatusTag>
              </span>
              <span class="import-candidate-meta">
                <code>{{ candidate.id }}</code>
                <span>{{ candidate.relativePath }}</span>
              </span>
              <small v-if="candidate.description">{{ candidate.description }}</small>
            </span>
          </label>
        </div>
      </section>
    </div>

    <template #footer>
      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="importBusy" @click="importDialogOpen = false">
          {{ t("dialog.cancel") }}
        </button>
        <button class="primary-button" :disabled="selectedCandidateIds.length === 0 || importBusy" @click="confirmImport">
          <Loader2 v-if="importBusy" :size="15" class="spin-animation" />
          <Download v-else :size="16" />
          {{ importBusy ? t("importSkill.importing") : t("importSkill.import") }}
        </button>
      </div>
    </template>
  </ModalDialog>

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
</template>

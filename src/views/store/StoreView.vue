<script setup lang="ts">
import { computed, inject, onMounted, ref, watch } from "vue";
import { Download, ExternalLink, Loader2, RefreshCw, Search } from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";
import SplitPane from "../../components/SplitPane.vue";
import ListPanel from "../../components/ListPanel.vue";
import SearchInput from "../../components/SearchInput.vue";
import ModalDialog from "../../components/ModalDialog.vue";
import StatusTag from "../../components/StatusTag.vue";
import { useI18n } from "../../composables/useI18n";
import { AppStoreKey } from "../../stores/useAppStore";
import * as api from "../../api";
import type {
  AppSnapshot,
  ImportSkillCandidate,
  StoreLeaderboardType,
  StoreSkill,
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

const board = ref<StoreLeaderboardType>("alltime");
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
    const subdir = skill.source?.subdir ?? skill.id;
    keys.add(`${repoName}/${subdir}`);
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
      subdir: selectedStoreSkill.value.skillId,
    });
    importCandidates.value = preview.candidates;
    selectedCandidateIds.value = preview.candidates
      .filter((candidate) => candidate.status === "ready")
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
        subdir: selectedStoreSkill.value.skillId,
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

onMounted(loadBoard);

watch(board, () => {
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
              <div class="segmented-control segmented-control--compact" :aria-label="t('store.marketSource')">
                <button type="button" :class="{ active: board === 'hot' }" @click="board = 'hot'">
                  {{ t("store.hot") }}
                </button>
                <button type="button" :class="{ active: board === 'trending' }" @click="board = 'trending'">
                  {{ t("store.trending") }}
                </button>
                <button type="button" :class="{ active: board === 'alltime' }" @click="board = 'alltime'">
                  {{ t("store.alltime") }}
                </button>
              </div>
              <div class="store-search-row">
                <SearchInput v-model="query" type="search" :placeholder="t('store.searchPlaceholder')" />
                <select v-model="sourceFilter" class="store-source-filter" :aria-label="t('store.sourceFilter')">
                  <option value="all">{{ t("store.sourceAll") }}</option>
                  <option v-for="source in sourceOptions" :key="source" :value="source">
                    {{ source }}
                  </option>
                </select>
                <button class="ghost-icon-button" type="button" :aria-label="t('store.refresh')" @click="loadBoard">
                  <RefreshCw :size="16" :class="{ 'spin-animation': loading }" />
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
            <StatusTag :type="isInstalled(skill) ? 'success' : 'offline'">
              {{ isInstalled(skill) ? t("store.installed") : t("store.notInstalled") }}
            </StatusTag>
          </div>
          <div class="store-list-meta">
            <code>{{ skill.source }}</code>
            <span>{{ t("store.installs", { count: skill.installs }) }}</span>
          </div>
        </button>
      </ListPanel>
    </template>

    <template #right>
      <div v-if="selectedStoreSkill" class="store-detail">
        <header class="extension-header">
          <div class="extension-identity">
            <div class="extension-icon">
              <Search :size="28" />
            </div>
            <div class="extension-title-group">
              <h2>{{ selectedStoreSkill.name }}</h2>
              <div class="extension-meta">
                <code>{{ selectedStoreSkill.skillId }}</code>
                <span>{{ selectedStoreSkill.source }}</span>
                <span>{{ t("store.installs", { count: selectedStoreSkill.installs }) }}</span>
              </div>
            </div>
          </div>

          <div class="store-detail-actions">
            <button class="secondary-button" type="button" @click="openRepository">
              <ExternalLink :size="16" />
              {{ t("store.openRepo") }}
            </button>
            <button
              class="primary-button"
              type="button"
              :disabled="importBusy"
              :aria-label="t('store.download')"
              @click="prepareImport"
            >
              <Loader2 v-if="importBusy" :size="16" class="spin-animation" />
              <Download v-else :size="16" />
              {{ importBusy ? t("store.downloading") : t("store.download") }}
            </button>
          </div>
        </header>

        <div class="description-pane store-description-pane">
          <section class="detail-section">
            <div class="meta-grid">
              <div class="meta-grid-item">
                <span class="meta-label">{{ t("store.marketSource") }}</span>
                <span class="meta-val">{{ selectedStoreSkill.source }}</span>
              </div>
              <div class="meta-grid-item">
                <span class="meta-label">{{ t("store.installsLabel") }}</span>
                <span class="meta-val">{{ selectedStoreSkill.installs }}</span>
              </div>
              <div class="meta-grid-item">
                <span class="meta-label">{{ t("store.download") }}</span>
                <span class="meta-val">{{ isInstalled(selectedStoreSkill) ? t("store.installed") : t("store.notInstalled") }}</span>
              </div>
              <div class="meta-grid-item full-width">
                <span class="meta-label">{{ t("store.openRepo") }}</span>
                <code class="meta-val">{{ `https://github.com/${selectedStoreSkill.source}` }}</code>
              </div>
            </div>
          </section>
        </div>
      </div>
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
</template>

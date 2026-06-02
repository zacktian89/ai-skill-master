<script setup lang="ts">
import { computed, ref, inject } from "vue";
import { Folder, Github, Plus, Loader2 } from "lucide-vue-next";
import * as api from "../../../api";
import { openDirectory } from "../../../utils/dialog";
import SearchInput from "../../../components/SearchInput.vue";
import StatusTag from "../../../components/StatusTag.vue";
import ModalDialog from "../../../components/ModalDialog.vue";
import { AppStoreKey } from "../../../stores/useAppStore";
import { useAsyncAction } from "../../../composables/useAsyncAction";
import type { AppSnapshot, ImportSkillCandidate, ImportSkillSource } from "../../../types";

type ImportSourceMode = "local" | "github";

defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  close: [];
  success: [nextSnapshot: AppSnapshot];
}>();

const appStore = inject(AppStoreKey, null);

const { busy, run: executeAsync } = useAsyncAction({
  onError: (err) => {
    if (appStore) appStore.setError(String(err));
  }
});

const importSourceMode = ref<ImportSourceMode>("local");
const importLocalPath = ref("");
const importGithubUrl = ref("");
const importGithubRef = ref("");
const importGithubSubdir = ref("");
const importCandidates = ref<ImportSkillCandidate[]>([]);
const selectedImportCandidateIds = ref<string[]>([]);
const importScanned = ref(false);
const currentStep = ref(1);

const importStatusLabels: Record<string, string> = {
  ready: "可导入",
  duplicate: "已存在",
  conflict: "冲突",
  invalid: "无效",
};

const importReadyCandidates = computed(() =>
  importCandidates.value.filter((candidate) => candidate.status === "ready")
);

const importSource = computed<ImportSkillSource | null>(() => {
  if (importSourceMode.value === "local") {
    const path = importLocalPath.value.trim();
    return path ? { kind: "local", path } : null;
  }
  let url = importGithubUrl.value.trim();
  if (!url) return null;
  
  // Expand owner/repo shorthand to standard GitHub URL
  if (!/^https?:\/\//.test(url) && !url.startsWith("git@") && url.includes("/") && url.split("/").length === 2) {
    url = `https://github.com/${url}.git`;
  }
  
  return {
    kind: "github",
    url,
    ref: importGithubRef.value.trim() || null,
    subdir: importGithubSubdir.value.trim() || null,
  };
});

const canScanImports = computed(() => Boolean(importSource.value) && !busy.value);
const canConfirmImports = computed(() => selectedImportCandidateIds.value.length > 0 && !busy.value);

function resetImportPreview() {
  importCandidates.value = [];
  selectedImportCandidateIds.value = [];
  importScanned.value = false;
  currentStep.value = 1;
}

function setImportSourceMode(mode: ImportSourceMode) {
  importSourceMode.value = mode;
  resetImportPreview();
}

async function selectImportLocalPath() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      importLocalPath.value = selected;
      resetImportPreview();
      // Auto-scan on selection
      await scanImportSource();
    }
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
  }
}

async function scanImportSource() {
  if (!importSource.value) return;
  await executeAsync(
    () => api.previewImportSkills(importSource.value!),
    (preview) => {
      importCandidates.value = preview.candidates;
      selectedImportCandidateIds.value = preview.candidates
        .filter((candidate) => candidate.status === "ready")
        .map((candidate) => candidate.candidateId);
      importScanned.value = true;
      currentStep.value = 2;
    }
  );
}

function toggleImportCandidate(candidate: ImportSkillCandidate, checked: boolean) {
  if (candidate.status !== "ready") return;
  const next = new Set(selectedImportCandidateIds.value);
  if (checked) {
    next.add(candidate.candidateId);
  } else {
    next.delete(candidate.candidateId);
  }
  selectedImportCandidateIds.value = [...next];
}

function importCandidateChecked(candidateId: string): boolean {
  return selectedImportCandidateIds.value.includes(candidateId);
}

function toggleAllImportCandidates(checked: boolean) {
  selectedImportCandidateIds.value = checked
    ? importReadyCandidates.value.map((candidate) => candidate.candidateId)
    : [];
}

async function confirmImportSkills() {
  if (!importSource.value || !selectedImportCandidateIds.value.length) return;
  await executeAsync(
    () => api.confirmImportSkills({
      source: importSource.value!,
      candidateIds: selectedImportCandidateIds.value,
    }),
    (nextSnapshot) => {
      emit("success", nextSnapshot);
      emit("close");
    }
  );
}
</script>

<template>
  <ModalDialog
    v-if="show"
    title="新增 Skill"
    card-class="import-modal"
    @close="$emit('close')"
  >
    <div class="import-modal-body">
      <!-- Step Indicator -->
      <div class="step-indicator">
        <div class="step" :class="{ active: currentStep === 1, completed: currentStep > 1 }">
          <span class="step-number">1</span>
          <span class="step-label">设置路径</span>
        </div>
        <div class="step-line" :class="{ completed: currentStep > 1 }"></div>
        <div class="step" :class="{ active: currentStep === 2 }">
          <span class="step-number">2</span>
          <span class="step-label">导入 Skill</span>
        </div>
      </div>

      <!-- Step 1: Set Path & Scan -->
      <div v-if="currentStep === 1" class="import-step-container">
        <div class="segmented-control import-source-tabs" aria-label="导入来源">
          <button
            type="button"
            :class="{ active: importSourceMode === 'local' }"
            @click="setImportSourceMode('local')"
          >
            <Folder :size="15" />
            本地
          </button>
          <button
            type="button"
            :class="{ active: importSourceMode === 'github' }"
            @click="setImportSourceMode('github')"
          >
            <Github :size="15" />
            GitHub
          </button>
        </div>

        <!-- Local Path Panel -->
        <div v-if="importSourceMode === 'local'" class="import-source-panel">
          <div class="import-path-row clickable-input-row" @click="!busy && selectImportLocalPath()">
            <SearchInput :model-value="importLocalPath" readonly placeholder="选择本地 Skill 文件夹..." />
            <button class="secondary-button" type="button" :disabled="busy" @click.stop="selectImportLocalPath">
              <Folder :size="16" />
              选择
            </button>
          </div>
        </div>

        <!-- GitHub URL Panel -->
        <div v-else class="import-source-panel">
          <div class="import-path-row">
            <SearchInput
              v-model="importGithubUrl"
              type="url"
              placeholder="输入 GitHub 仓库 URL 或 owner/repo (如: owner/repo)"
              @input="resetImportPreview"
              @keyup.enter="scanImportSource"
            />
            <button class="primary-button scan-btn" type="button" :disabled="!canScanImports" @click="scanImportSource">
              <Loader2 v-if="busy" :size="15" class="spin-animation" />
              <span v-if="busy">扫描中...</span>
              <span v-else>扫描</span>
            </button>
          </div>
          
          <div class="import-github-grid">
            <div class="input-group">
              <span class="input-label">分支 / 标签 / 提交号 (可选)</span>
              <SearchInput v-model="importGithubRef" placeholder="例如: main" @input="resetImportPreview" />
            </div>
            <div class="input-group">
              <span class="input-label">指定子目录 (可选)</span>
              <SearchInput v-model="importGithubSubdir" placeholder="例如: skills/my-skill" @input="resetImportPreview" />
            </div>
          </div>
        </div>
      </div>

      <!-- Step 2: Import Scanned Candidates -->
      <section v-else-if="currentStep === 2" class="import-results">
        <div class="import-results-head">
          <label class="import-check-all">
            <input
              type="checkbox"
              :checked="importReadyCandidates.length > 0 && selectedImportCandidateIds.length === importReadyCandidates.length"
              :disabled="importReadyCandidates.length === 0 || busy"
              @change="toggleAllImportCandidates(($event.target as HTMLInputElement).checked)"
            />
            <span>已选 {{ selectedImportCandidateIds.length }}/{{ importReadyCandidates.length }}</span>
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
              :checked="importCandidateChecked(candidate.candidateId)"
              :disabled="candidate.status !== 'ready' || busy"
              @change="toggleImportCandidate(candidate, ($event.target as HTMLInputElement).checked)"
            />
            <span class="import-candidate-main">
              <span class="import-candidate-top">
                <strong>{{ candidate.name }}</strong>
                <StatusTag :type="candidate.status === 'ready' ? 'healthy' : candidate.status">
                  {{ importStatusLabels[candidate.status] }}
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

        <div v-else class="content-empty content-empty--compact">没有找到 skill。</div>
      </section>
    </div>

    <template #footer>
      <div class="button-row button-row--end">
        <template v-if="currentStep === 1">
          <button class="secondary-button" :disabled="busy" @click="$emit('close')">取消</button>
          <button
            v-if="importScanned"
            class="primary-button"
            :disabled="busy"
            @click="currentStep = 2"
          >
            下一步
          </button>
        </template>
        <template v-else-if="currentStep === 2">
          <button class="secondary-button" :disabled="busy" @click="currentStep = 1">上一步</button>
          <button class="primary-button" :disabled="!canConfirmImports" @click="confirmImportSkills">
            <Loader2 v-if="busy" :size="15" class="spin-animation" />
            <Plus v-else :size="16" />
            <span v-if="busy">导入中...</span>
            <span v-else>导入{{ selectedImportCandidateIds.length > 0 ? ` (${selectedImportCandidateIds.length})` : '' }}</span>
          </button>
        </template>
      </div>
    </template>
  </ModalDialog>
</template>

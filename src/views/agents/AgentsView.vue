<script setup lang="ts">
import { computed, ref, watch, inject, nextTick, onBeforeUnmount } from "vue";
import {
  UserPlus,
  Plus,
  Trash2,
  X,
  FolderOpen,
  RefreshCw,
  AlertTriangle,
  MoreHorizontal,
} from "lucide-vue-next";
import { openPath } from "@tauri-apps/plugin-opener";
import * as api from "../../api";
import AgentIcon from "../../components/icons/AgentIcon.vue";
import { openDirectory } from "../../utils/dialog";
import SplitPane from "../../components/SplitPane.vue";
import ListPanel from "../../components/ListPanel.vue";
import SearchInput from "../../components/SearchInput.vue";
import ModalDialog from "../../components/ModalDialog.vue";
import ScannedSkillList from "../../components/ScannedSkillList.vue";
import SkillPreviewPanel from "../../components/SkillPreviewPanel.vue";
import type { AppSnapshot, Agent, ScannedSkill } from "../../types";
import { AppStoreKey } from "../../stores/useAppStore";
import { SelectionStoreKey } from "../../stores/useSelectionStore";
import { useAsyncAction } from "../../composables/useAsyncAction";
import { useSkillScanner } from "../../composables/useSkillScanner";
import { useSkillPicker } from "../../composables/useSkillPicker";
import { useSkillMarkdown } from "../../composables/useSkillMarkdown";
import { useI18n } from "../../composables/useI18n";

const { t } = useI18n();

const appStore = inject(AppStoreKey, null);
const selectionStore = inject(SelectionStoreKey, null);

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedAgentId: string | null;
}>();

const emit = defineEmits<{
  "select-agent": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const snapshot = computed(() => appStore?.snapshot.value ?? props.snapshot);
const selectedAgentId = computed({
  get: () => selectionStore?.selectedAgentId.value ?? props.selectedAgentId,
  set: (val) => {
    if (selectionStore) {
      selectionStore.setSelectedAgentId(val);
    } else {
      emit("select-agent", val);
    }
  }
});

const agentQuery = ref("");
const skillQuery = ref("");

// Composable for Async Actions
const { busy, run: executeAsync } = useAsyncAction({
  onError: (err) => {
    if (appStore) appStore.setError(String(err));
    else emit("error", String(err));
  }
});

// Add Agent Dialog State
const addAgentDialogOpen = ref(false);
const selectedPresetIndex = ref<number>(0);
const inputAgentName = ref("");
const inputAgentPath = ref("");

// Add Skill Dialog State (linking library skills to selected agent)
const addSkillDialogOpen = ref(false);
const deleteAgentDialogOpen = ref(false);
const pendingReferenceRemoval = ref<{ skillId: string; skillPath: string } | null>(null);
const pendingUnmanagedSkillDeletion = ref<{ skillId: string; skillName: string; skillPath: string } | null>(null);
const previewSkill = ref<ScannedSkill | null>(null);
const listSectionRef = ref<HTMLElement | null>(null);
const lastListScrollTop = ref(0);

const PRESET_AGENTS = [
  { name: "Codex", defaultPath: "~/.codex/skills", targetName: "Codex" },
  { name: "Claude Code", defaultPath: "~/.claude/skills", targetName: "Claude Code" },
  { name: "Gemini CLI", defaultPath: "~/.gemini/config/skills", targetName: "Gemini CLI" },
  { name: "GitHub Copilot", defaultPath: "~/.copilot/skills", targetName: "GitHub Copilot" },
  { name: "Cursor", defaultPath: "~/.cursor/skills", targetName: "Cursor" },
  { name: "WorkBuddy", defaultPath: "~/.workbuddy/skills", targetName: "WorkBuddy" },
  { name: "Windsurf", defaultPath: "~/.codeium/windsurf/skills", targetName: "Windsurf" },
  { name: "Kiro", defaultPath: "~/.kiro/skills", targetName: "Kiro" },
  { name: "OpenCode", defaultPath: "~/.config/opencode/skill", targetName: "OpenCode" },
  { name: "CodeBuddy", defaultPath: "~/.codebuddy/skills", targetName: "CodeBuddy" },
  { name: "自定义 Agent", defaultPath: "", targetName: "自定义" },
];

// Compute active skills count for each agent
function agentSkillCount(agent: Agent): number {
  return Object.values(agent.rules).filter((rule) => rule === "enable").length;
}

const agents = computed(() => {
  const normalized = agentQuery.value.trim().toLowerCase();
  const list = snapshot.value.state.agents || [];
  return [...list]
    .filter((agent) => {
      if (!normalized) return true;
      return `${agent.name} ${agent.path}`.toLowerCase().includes(normalized);
    })
    .sort((left, right) => {
      const skillBias = agentSkillCount(right) - agentSkillCount(left);
      return skillBias || left.name.localeCompare(right.name, "zh-CN");
    });
});

const selectedAgent = computed(
  () => agents.value.find((agent) => agent.id === selectedAgentId.value) ?? agents.value[0] ?? null
);

async function run(action: () => Promise<AppSnapshot>) {
  await executeAsync(action, (next) => {
    if (appStore) appStore.applySnapshot(next);
    else emit("snapshot", next);
  });
}

// Add Agent functions
function openAddAgentDialog() {
  selectedPresetIndex.value = 0;
  inputAgentName.value = PRESET_AGENTS[0].name;
  inputAgentPath.value = PRESET_AGENTS[0].defaultPath;
  addAgentDialogOpen.value = true;
}

// Close Agent dialog
function closeAddAgentDialog() {
  addAgentDialogOpen.value = false;
}

function selectPreset(index: number) {
  selectedPresetIndex.value = index;
  inputAgentName.value = PRESET_AGENTS[index].name === "自定义 Agent" ? "" : PRESET_AGENTS[index].name;
  inputAgentPath.value = PRESET_AGENTS[index].defaultPath;
}

async function browseAgentPath() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      inputAgentPath.value = selected;
    }
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
    else emit("error", String(cause));
  }
}

async function confirmAddAgent() {
  if (!inputAgentName.value.trim() || !inputAgentPath.value.trim()) {
    if (appStore) appStore.setError(t("agents.nameAndPathRequired"));
    else emit("error", t("agents.nameAndPathRequired"));
    return;
  }
  await executeAsync(
    () => api.addAgent(inputAgentName.value.trim(), inputAgentPath.value.trim()),
    (next) => {
      if (appStore) appStore.applySnapshot(next);
      else emit("snapshot", next);
      const newAgent = next.state.agents.find(
        (a) => !snapshot.value.state.agents?.some((oldAgent) => oldAgent.id === a.id)
      );
      if (newAgent) {
        selectedAgentId.value = newAgent.id;
      }
      closeAddAgentDialog();
    }
  );
}

function openDeleteAgentDialog() {
  if (!selectedAgent.value) return;
  deleteAgentDialogOpen.value = true;
}

async function confirmDeleteAgent() {
  if (!selectedAgent.value) return;
  await executeAsync(
    () => api.deleteAgent(selectedAgent.value!.id),
    (next) => {
      if (appStore) appStore.applySnapshot(next);
      else emit("snapshot", next);
      selectedAgentId.value = next.state.agents?.[0]?.id ?? null;
      deleteAgentDialogOpen.value = false;
    }
  );
}

// Composable for Skill Scanner
const {
  scannedCategories,
  scanning,
  scannedSkillsCount,
  loadScan,
  refreshScan
} = useSkillScanner(
  () => selectedAgent.value?.path,
  api.scanAgentSkills,
  {
    onError: (err) => {
      if (appStore) appStore.setError(String(err));
      else emit("error", String(err));
    }
  }
);

// Composable for Skill Picker
const {
  addSkillQuery,
  selectedSkillIds,
  filteredLibrarySkills,
  toggleAllLibrarySkills,
  resetPicker
} = useSkillPicker(() => snapshot.value.state.skills || []);

// Skill dialog functions
function openAddSkillDialog() {
  resetPicker();
  addSkillDialogOpen.value = true;
}

function closeAddSkillDialog() {
  addSkillDialogOpen.value = false;
}

async function confirmAddSkillReferences() {
  if (!selectedAgent.value || selectedSkillIds.value.length === 0) return;
  await executeAsync(
    async () => {
      let currentSnapshot = snapshot.value;
      for (const skillId of selectedSkillIds.value) {
        // 1. Link references
        currentSnapshot = await api.addSkillReference({
          skillId,
          targetName: selectedAgent.value!.name,
          rootPath: selectedAgent.value!.path,
          scope: "user" as const,
          overwrite: true,
        });
        // 2. Set rule to enable explicitly
        currentSnapshot = await api.setAgentRule({
          agentId: selectedAgent.value!.id,
          skillId,
          rule: "enable",
        });
      }
      if (appStore) appStore.applySnapshot(currentSnapshot);
      else emit("snapshot", currentSnapshot);
      closeAddSkillDialog();
      await refreshScan();
    }
  );
}

// Rules functions
async function toggleSkillRule(skillId: string) {
  if (!selectedAgent.value) return;
  const isCurrentlyDisabled = selectedAgent.value.rules[skillId] === "disable";
  const newRule = isCurrentlyDisabled ? "enable" : "disable";
  await run(() =>
    api.setAgentRule({
      agentId: selectedAgent.value!.id,
      skillId,
      rule: newRule,
    })
  );
  await loadScan();
}

const conflictState = ref<{ skillId: string; libraryName: string; projectName: string; skillPath: string } | null>(null);

const filteredScannedCategories = computed(() => {
  const normalized = skillQuery.value.trim().toLowerCase();
  if (!normalized) return scannedCategories.value;
  return scannedCategories.value
    .map((category) => {
      const skills = category.skills.filter((skill) => {
        return (
          skill.name.toLowerCase().includes(normalized) ||
          skill.id.toLowerCase().includes(normalized) ||
          (skill.description && skill.description.toLowerCase().includes(normalized)) ||
          skill.path.toLowerCase().includes(normalized)
        );
      });
      return { ...category, skills };
    })
    .filter((category) => category.skills.length > 0);
});

const previewLibrarySkill = computed(
  () => snapshot.value.state.skills.find((skill) => skill.id === previewSkill.value?.id) ?? null
);

const {
  skillMarkdown,
  isMarkdownLoading,
  parsedMarkdown,
  renderedMarkdown,
} = useSkillMarkdown(
  () => previewLibrarySkill.value?.id ?? null,
  undefined,
  () => (previewLibrarySkill.value ? null : previewSkill.value?.path ?? null)
);

function findDetailScrollContainer() {
  return listSectionRef.value?.closest(".detail-panel") as HTMLElement | null;
}

function openSkillPreview(skill: ScannedSkill) {
  lastListScrollTop.value = findDetailScrollContainer()?.scrollTop ?? 0;
  previewSkill.value = skill;
}

async function closeSkillPreview() {
  previewSkill.value = null;
  await nextTick();
  const container = findDetailScrollContainer();
  if (container) {
    container.scrollTop = lastListScrollTop.value;
  }
}

function findReferenceIdForScannedSkill(skillId: string, skillPath: string): string | null {
  const skill = snapshot.value.state.skills.find((s) => s.id === skillId);
  if (!skill || !skill.references) return null;
  return (
    skill.references.find(
      (r) => r.targetPath.replace(/[\\/]+/g, "/").toLowerCase() === skillPath.replace(/[\\/]+/g, "/").toLowerCase()
    )?.id ?? null
  );
}

async function removeManagedSkillReference(skillId: string, skillPath: string) {
  pendingReferenceRemoval.value = { skillId, skillPath };
}

async function confirmRemoveManagedSkillReference() {
  if (!pendingReferenceRemoval.value) return;
  const { skillId, skillPath } = pendingReferenceRemoval.value;
  const refId = findReferenceIdForScannedSkill(skillId, skillPath);
  if (!refId) {
    if (appStore) appStore.setError(t("reference.pathConflictError"));
    else emit("error", t("reference.pathConflictError"));
    pendingReferenceRemoval.value = null;
    return;
  }
  // Remove reference (deletes link physically)
  await run(() => api.removeSkillReference(refId, true));
  // Clean up rule
  if (selectedAgent.value) {
    await run(() =>
      api.setAgentRule({
        agentId: selectedAgent.value!.id,
        skillId,
        rule: "inherit", // inherit effectively clears it from rules map
      })
    );
  }
  pendingReferenceRemoval.value = null;
  await refreshScan();
}

function deleteUnmanagedSkill(skillId: string, skillName: string, skillPath: string) {
  pendingUnmanagedSkillDeletion.value = { skillId, skillName, skillPath };
}

async function confirmDeleteUnmanagedSkill() {
  if (!pendingUnmanagedSkillDeletion.value) return;
  await run(() => api.deleteUnmanagedSkill(pendingUnmanagedSkillDeletion.value!.skillPath));
  pendingUnmanagedSkillDeletion.value = null;
  await refreshScan();
}

async function handleImportSkill(skillPath: string, strategy?: "overwrite" | "keep_existing") {
  if (!selectedAgent.value) return;
  await executeAsync(
    () => api.importProjectSkill(selectedAgent.value!.name, skillPath, strategy),
    async (result) => {
      if (result.type === "success") {
        if (appStore) appStore.applySnapshot(result.snapshot);
        else emit("snapshot", result.snapshot);
        conflictState.value = null;
        await refreshScan();
      } else if (result.type === "conflict") {
        conflictState.value = {
          skillId: result.skillId,
          libraryName: result.libraryName,
          projectName: result.projectName,
          skillPath,
        };
      }
    }
  );
}

watch(
  () => selectedAgent.value?.id,
  (newId, oldId) => {
    if (newId !== oldId) {
      scannedCategories.value = [];
      previewSkill.value = null;
    }
    loadScan();
  },
  { immediate: true }
);

// More menu state for the top-right header action
const headerMenuOpen = ref<{ x: number; y: number } | null>(null);
const headerMenuRef = ref<HTMLElement | null>(null);
let headerMenuCloseTimer: number | null = null;
const menuMargin = 8;
const fallbackMenuWidth = 148;

function closeHeaderMenu() {
  headerMenuOpen.value = null;
  if (headerMenuCloseTimer !== null) {
    window.clearTimeout(headerMenuCloseTimer);
    headerMenuCloseTimer = null;
  }
  document.removeEventListener("click", closeHeaderMenu);
  document.removeEventListener("keydown", headerMenuOnEscape);
}

function headerMenuOnEscape(event: KeyboardEvent) {
  if (event.key === "Escape") closeHeaderMenu();
}

function clampMenuPosition(x: number, y: number, width: number, height: number) {
  const maxX = Math.max(menuMargin, window.innerWidth - width - menuMargin);
  const maxY = Math.max(menuMargin, window.innerHeight - height - menuMargin);
  return {
    x: Math.min(Math.max(menuMargin, x), maxX),
    y: Math.min(Math.max(menuMargin, y), maxY),
  };
}

async function openHeaderMenu(event: MouseEvent) {
  closeHeaderMenu();
  const initialPosition = clampMenuPosition(event.clientX - fallbackMenuWidth, event.clientY, fallbackMenuWidth, 0);
  headerMenuOpen.value = initialPosition;
  await nextTick();
  const menuRect = headerMenuRef.value?.getBoundingClientRect();
  if (headerMenuOpen.value && menuRect) {
    const menuWidth = menuRect.width || fallbackMenuWidth;
    const position = clampMenuPosition(event.clientX - menuWidth, event.clientY, menuWidth, menuRect.height);
    headerMenuOpen.value = position;
  }
  headerMenuCloseTimer = window.setTimeout(() => {
    headerMenuCloseTimer = null;
    document.addEventListener("click", closeHeaderMenu);
    document.addEventListener("keydown", headerMenuOnEscape);
  });
}

function runHeaderMenuAction(action: () => void) {
  action();
  closeHeaderMenu();
}

async function openAgentSkillDirectory() {
  if (!selectedAgent.value) return;
  try {
    await openPath(selectedAgent.value.path);
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
    else emit("error", String(cause));
  }
}

onBeforeUnmount(closeHeaderMenu);
</script>

<template>
  <SplitPane class="agents-view">
    <template #left>
      <ListPanel :items="agents" :has-search="true" :empty-text="t('agents.empty')">
        <template #search-row>
          <div class="list-search-row">
            <SearchInput v-model="agentQuery" :placeholder="t('agents.searchPlaceholder')" />
            <button class="icon-button" type="button" :disabled="busy" :aria-label="t('agents.addAgent')" @click="openAddAgentDialog">
              <UserPlus :size="18" />
            </button>
          </div>
        </template>

        <button
          v-for="agent in agents"
          :key="agent.id"
          class="list-row list-row--agent"
          :class="{ active: selectedAgent?.id === agent.id }"
          @click="emit('select-agent', agent.id)"
        >
          <div class="list-row-agent-icon">
            <AgentIcon :name="agent.name" :size="20" />
          </div>
          <div class="list-row-copy">
            <strong>{{ agent.name }}</strong>
          </div>
        </button>
      </ListPanel>
    </template>

    <template #right>
      <template v-if="selectedAgent">
        <SkillPreviewPanel
          v-if="previewSkill"
          :skill="previewSkill"
          :library-skill="previewLibrarySkill"
          :rule="selectedAgent.rules[previewSkill.id]"
          :busy="busy"
          :show-category-title="false"
          :is-markdown-loading="isMarkdownLoading"
          :skill-markdown="skillMarkdown"
          :parsed-markdown="parsedMarkdown"
          :rendered-markdown="renderedMarkdown"
          @back="closeSkillPreview"
          @toggle-rule="toggleSkillRule"
          @remove-reference="removeManagedSkillReference"
          @import-skill="handleImportSkill"
          @delete-unmanaged-skill="deleteUnmanagedSkill"
        />
        <template v-else>
          <div class="detail-header">
            <div>
              <h2>{{ selectedAgent.name }}</h2>
              <p>{{ selectedAgent.path }}</p>
            </div>
            <div class="detail-actions">
              <button
                class="ghost-icon-button"
                type="button"
                :disabled="busy"
                :aria-label="t('projects.moreActions')"
                :title="t('projects.moreActions')"
                @click.stop="openHeaderMenu"
              >
                <MoreHorizontal :size="16" />
              </button>
            </div>
          </div>

          <Teleport to="body">
            <div
              v-if="headerMenuOpen"
              ref="headerMenuRef"
              class="global-context-menu"
              :style="{ left: `${headerMenuOpen.x}px`, top: `${headerMenuOpen.y}px` }"
              role="menu"
              @click.stop
            >
              <button
                type="button"
                role="menuitem"
                class="global-context-menu-item"
                :disabled="busy"
                @click="runHeaderMenuAction(openAddSkillDialog)"
              >
                <Plus :size="15" />
                <span>{{ t('projects.addSkill') }}</span>
              </button>

              <button
                type="button"
                role="menuitem"
                class="global-context-menu-item"
                :disabled="busy"
                @click="runHeaderMenuAction(openDeleteAgentDialog)"
              >
                <Trash2 :size="15" />
                <span>{{ t('projects.cancelManage') }}</span>
              </button>

              <button
                type="button"
                role="menuitem"
                class="global-context-menu-item"
                :disabled="busy"
                @click="runHeaderMenuAction(openAgentSkillDirectory)"
              >
                <FolderOpen :size="15" />
                <span>{{ t('agents.openAgentDir') }}</span>
              </button>
            </div>
          </Teleport>

          <section ref="listSectionRef" class="detail-section">
            <div class="project-skill-toolbar">
              <span class="search-row-count">{{ scannedSkillsCount }}</span>
              <SearchInput v-model="skillQuery" :placeholder="t('projects.searchSkillPlaceholder')" class="detail-search-input" />
              <button class="ghost-icon-button" type="button" :disabled="busy || scanning" :aria-label="t('projects.rescan')" :title="t('projects.rescan')" @click="refreshScan">
                <RefreshCw :size="14" :class="{ 'spin-animation': scanning }" />
              </button>
            </div>

            <ScannedSkillList
              v-if="filteredScannedCategories.length && filteredScannedCategories[0].skills.length"
              :categories="filteredScannedCategories"
              :rules="selectedAgent.rules"
              :busy="busy"
              :show-category-title="false"
              :show-add-button="false"
              :show-disabled-badge="true"
              @preview-skill="openSkillPreview"
              @toggle-rule="toggleSkillRule"
              @remove-reference="removeManagedSkillReference"
              @import-skill="handleImportSkill"
              @delete-unmanaged-skill="deleteUnmanagedSkill"
            />
            <div v-else class="content-empty content-empty--padding-lg">{{ t('agents.noSkillsScanned') }}</div>
          </section>
        </template>
      </template>

      <div v-else class="content-empty">{{ t('agents.selectAgentDetail') }}</div>
    </template>
  </SplitPane>

  <!-- Add Agent Dialog -->
  <ModalDialog
    v-if="addAgentDialogOpen"
    :title="t('agents.addAgent')"
    card-class="modal-card--agent"
    @close="closeAddAgentDialog"
  >
    <div class="modal-step-section modal-step-section--scroll">
      <div class="target-grid target-grid--agent-presets">
        <button
          v-for="(preset, index) in PRESET_AGENTS"
          :key="preset.name"
          class="target-tile"
          :class="{ active: selectedPresetIndex === index }"
          type="button"
          :disabled="busy"
          @click="selectPreset(index)"
        >
          <span class="target-tile-icon" aria-hidden="true">
            <AgentIcon :name="preset.targetName" :size="20" />
          </span>
          <strong class="preset-name">{{ preset.name === '自定义 Agent' ? t('agents.presetCustomAgent') : preset.name }}</strong>
        </button>
      </div>

      <div class="agent-form-grid">
        <div class="field-stack">
          <label>{{ t('agents.agentNameLabel') }}</label>
          <SearchInput v-model="inputAgentName" :placeholder="t('agents.agentNamePlaceholder')" />
        </div>

        <div class="field-stack">
          <label>{{ t('agents.agentPathLabel') }}</label>
          <div class="path-input-row">
            <SearchInput v-model="inputAgentPath" :placeholder="t('agents.agentPathPlaceholder')" />
            <button
              class="secondary-button"
              type="button"
              :disabled="busy"
              @click="browseAgentPath"
            >
              <FolderOpen :size="16" />
              <span>{{ t('agents.browseButton') }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="button-row button-row--end modal-footer-row">
        <button class="secondary-button" :disabled="busy" @click="closeAddAgentDialog">{{ t('dialog.cancel') }}</button>
        <button class="primary-button" :disabled="busy || !inputAgentName.trim() || !inputAgentPath.trim()" @click="confirmAddAgent">
          {{ t('dialog.confirm') }}
        </button>
      </div>
    </template>
  </ModalDialog>

  <ModalDialog
    v-if="deleteAgentDialogOpen && selectedAgent"
    :title="t('deleteAgent.title')"
    @close="deleteAgentDialogOpen = false"
  >
    <p class="modal-note">
      {{ t('deleteAgent.note', { name: selectedAgent.name }) }}
    </p>
    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="deleteAgentDialogOpen = false">{{ t('dialog.cancel') }}</button>
        <button class="danger-button" :disabled="busy" @click="confirmDeleteAgent">
          {{ t('deleteAgent.deleteAgentButton') }}
        </button>
      </div>
    </template>
  </ModalDialog>

  <ModalDialog
    v-if="pendingReferenceRemoval"
    :title="t('deleteRefProj.title')"
    @close="pendingReferenceRemoval = null"
  >
    <p class="modal-note">
      {{ t('deleteRefProj.noteAgent') }}
    </p>
    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="pendingReferenceRemoval = null">{{ t('dialog.cancel') }}</button>
        <button class="danger-button" :disabled="busy" @click="confirmRemoveManagedSkillReference">
          {{ t('deleteRefProj.removeReferenceButton') }}
        </button>
      </div>
    </template>
  </ModalDialog>

  <ModalDialog
    v-if="pendingUnmanagedSkillDeletion"
    :title="t('deleteUnmanaged.title')"
    @close="pendingUnmanagedSkillDeletion = null"
  >
    <p class="modal-note">
      {{ t('deleteUnmanaged.note', { name: pendingUnmanagedSkillDeletion.skillName }) }}
    </p>
    <dl class="modal-summary">
      <dt>{{ t('deleteUnmanaged.path') }}</dt>
      <dd>{{ pendingUnmanagedSkillDeletion.skillPath }}</dd>
    </dl>
    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="pendingUnmanagedSkillDeletion = null">{{ t('dialog.cancel') }}</button>
        <button class="danger-button" :disabled="busy" @click="confirmDeleteUnmanagedSkill">
          {{ t('deleteUnmanaged.deleteFolder') }}
        </button>
      </div>
    </template>
  </ModalDialog>

  <!-- Add Skill to Agent Dialog -->
  <ModalDialog
    v-if="addSkillDialogOpen && selectedAgent"
    :title="t('agents.linkSkillsTo', { name: selectedAgent.name })"
    card-class="modal-card--compact"
    @close="closeAddSkillDialog"
  >
    <div class="modal-step-section">
      <!-- Search box inside dialog -->
      <SearchInput v-model="addSkillQuery" :placeholder="t('projects.searchSkillPlaceholder')" class="dialog-search-input" />

      <!-- Check all / Selected count -->
      <div class="dialog-checklist-header">
        <label class="check-all-label">
          <input
            type="checkbox"
            :checked="filteredLibrarySkills.length > 0 && selectedSkillIds.length === filteredLibrarySkills.length"
            :disabled="filteredLibrarySkills.length === 0 || busy"
            @change="toggleAllLibrarySkills(($event.target as HTMLInputElement).checked)"
          />
          <span>{{ t('projects.checkAll') }}</span>
        </label>
        <span>{{ t('projects.selectedSkillsCount', { selected: selectedSkillIds.length, total: filteredLibrarySkills.length }) }}</span>
      </div>

      <!-- Skill list with checkboxes -->
      <div v-if="filteredLibrarySkills.length" class="project-skill-picker">
        <label
          v-for="skill in filteredLibrarySkills"
          :key="skill.id"
          class="dialog-skill-pick-row"
          :class="{ disabled: busy }"
        >
          <input
            type="checkbox"
            :value="skill.id"
            v-model="selectedSkillIds"
            :disabled="busy"
          />
          <span class="dialog-skill-info">
            <strong>{{ skill.name }}</strong>
            <small>
              <code>{{ skill.id }}</code>
              <span v-if="skill.description"> · {{ skill.description }}</span>
            </small>
          </span>
        </label>
      </div>
      <div v-else class="content-empty content-empty--compact">{{ t('projects.noMatchingSkills') }}</div>
    </div>

    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="closeAddSkillDialog">{{ t('dialog.cancel') }}</button>
        <button
          class="primary-button"
          :disabled="busy || selectedSkillIds.length === 0"
          @click="confirmAddSkillReferences"
        >
          {{ t('dialog.confirm') }}
        </button>
      </div>
    </template>
  </ModalDialog>

  <!-- Conflict Modal Dialog -->
  <ModalDialog
    v-if="conflictState"
    :is-conflict="true"
    @close="conflictState = null"
  >
    <template #header>
      <div class="modal-title-row header-warning-title">
        <div class="conflict-title-container">
          <AlertTriangle :size="20" />
          <h2 class="conflict-modal-title">{{ t('projects.importConflictTitle') }}</h2>
        </div>
        <button class="ghost-icon-button" type="button" :aria-label="t('dialog.close')" @click="conflictState = null">
          <X :size="16" />
        </button>
      </div>
    </template>

    <div class="conflict-modal-body">
      <p>{{ t('projects.importConflictDesc', { id: conflictState.skillId }) }}</p>
      
      <div class="conflict-compare-box">
        <div class="conflict-compare-item">
          <span class="conflict-compare-label">{{ t('projects.projectLocalVersion') }}</span>
          <span class="conflict-compare-value">{{ conflictState.projectName }}</span>
        </div>
        <div class="conflict-compare-item">
          <span class="conflict-compare-label">{{ t('projects.libraryExistingVersion') }}</span>
          <span class="conflict-compare-value">{{ conflictState.libraryName }}</span>
        </div>
      </div>
      
      <p class="conflict-explanation">
        {{ t('projects.conflictExplain') }}
      </p>
    </div>

    <template #footer>
      <div class="conflict-modal-footer">
        <button class="ghost-button" :disabled="busy" @click="conflictState = null">
          {{ t('dialog.cancel') }}
        </button>
        <button class="primary-button" :disabled="busy" @click="handleImportSkill(conflictState.skillPath, 'keep_existing')">
          {{ t('projects.keepExisting') }}
        </button>
        <button class="primary-button danger-override-btn" :disabled="busy" @click="handleImportSkill(conflictState.skillPath, 'overwrite')">
          {{ t('projects.overwriteExisting') }}
        </button>
      </div>
    </template>
  </ModalDialog>
</template>



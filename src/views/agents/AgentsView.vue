<script setup lang="ts">
import { computed, ref, watch, inject } from "vue";
import {
  FolderPlus,
  Plus,
  Trash2,
  X,
  FolderOpen,
  RefreshCw,
  AlertTriangle,
} from "lucide-vue-next";
import * as api from "../../api";
import AgentIcon from "../../components/icons/AgentIcon.vue";
import { openDirectory } from "../../utils/dialog";
import SplitPane from "../../components/SplitPane.vue";
import ListPanel from "../../components/ListPanel.vue";
import SearchInput from "../../components/SearchInput.vue";
import ModalDialog from "../../components/ModalDialog.vue";
import ScannedSkillList from "../../components/ScannedSkillList.vue";
import type { AppSnapshot, Agent } from "../../types";
import { AppStoreKey } from "../../stores/useAppStore";
import { SelectionStoreKey } from "../../stores/useSelectionStore";
import { useAsyncAction } from "../../composables/useAsyncAction";
import { useSkillScanner } from "../../composables/useSkillScanner";
import { useSkillPicker } from "../../composables/useSkillPicker";

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

const PRESET_AGENTS = [
  { name: "Codex", defaultPath: "~/.agents/skills", targetName: "Codex" },
  { name: "Claude Code", defaultPath: "~/.claude/skills", targetName: "Claude Code" },
  { name: "Gemini CLI", defaultPath: "~/.gemini/config/skills", targetName: "Gemini CLI" },
  { name: "GitHub Copilot", defaultPath: "~/.copilot/skills", targetName: "GitHub Copilot" },
  { name: "Cursor", defaultPath: "~/.cursor/skills", targetName: "Cursor" },
  { name: "WorkBuddy", defaultPath: "~/.workbuddy/skills", targetName: "WorkBuddy" },
  { name: "Windsurf", defaultPath: "~/.codeium/windsurf/skills", targetName: "Windsurf" },
  { name: "Kiro", defaultPath: "~/.kiro/skills", targetName: "Kiro" },
  { name: "OpenCode", defaultPath: "~/.config/opencode/skill", targetName: "OpenCode" },
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
    if (appStore) appStore.setError("请填写 Agent 名称和目标路径");
    else emit("error", "请填写 Agent 名称和目标路径");
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
    if (appStore) appStore.setError("无法找到该引用的记录，请确认该技能已在技能详情的引用列表中注册。");
    else emit("error", "无法找到该引用的记录，请确认该技能已在技能详情的引用列表中注册。");
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
    (result) => {
      if (result.type === "success") {
        if (appStore) appStore.applySnapshot(result.snapshot);
        else emit("snapshot", result.snapshot);
        conflictState.value = null;
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
    }
    loadScan();
  },
  { immediate: true }
);
</script>

<template>
  <SplitPane>
    <template #left>
      <ListPanel :items="agents" :has-search="true" empty-text="没有匹配的 Agent。">
        <template #search-row>
          <div class="list-search-row">
            <SearchInput v-model="agentQuery" placeholder="搜索 Agent 名称或路径" />
            <button class="icon-button" type="button" :disabled="busy" aria-label="添加 Agent" @click="openAddAgentDialog">
              <FolderPlus :size="18" />
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
            <small>{{ agent.path }}</small>
          </div>
        </button>
      </ListPanel>
    </template>

    <template #right>
      <template v-if="selectedAgent">
        <div class="detail-header">
          <div>
            <h2>{{ selectedAgent.name }}</h2>
            <p>{{ selectedAgent.path }}</p>
          </div>
          <div class="detail-actions">
            <button class="primary-button" :disabled="busy" aria-label="添加技能" @click="openAddSkillDialog">
              <Plus :size="16" />
            </button>
            <button
              class="danger-button danger-button--icon"
              :disabled="busy"
              aria-label="删除 Agent"
              title="删除 Agent"
              @click="openDeleteAgentDialog"
            >
              <Trash2 :size="16" />
            </button>
          </div>
        </div>

        <section class="detail-section">
          <div class="project-skill-toolbar">
            <span class="search-row-count">{{ scannedSkillsCount }}</span>
            <SearchInput v-model="skillQuery" placeholder="搜索技能" class="detail-search-input" />
            <button class="ghost-icon-button" type="button" :disabled="busy || scanning" aria-label="重新扫描" title="重新扫描" @click="refreshScan">
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
            @toggle-rule="toggleSkillRule"
            @remove-reference="removeManagedSkillReference"
            @import-skill="handleImportSkill"
            @delete-unmanaged-skill="deleteUnmanagedSkill"
          />
          <div v-else class="content-empty" style="padding: 24px 0;">此 Agent 下尚未扫描到任何技能。</div>
        </section>
      </template>

      <div v-else class="content-empty">选择左侧 Agent 查看技能列表。</div>
    </template>
  </SplitPane>

  <!-- Add Agent Dialog -->
  <!-- Add Agent Dialog -->
  <ModalDialog
    v-if="addAgentDialogOpen"
    title="添加 Agent"
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
          <strong style="font-size: 12px;">{{ preset.name }}</strong>
        </button>
      </div>

      <div class="agent-form-grid">
        <div class="field-stack">
          <label>Agent 名称</label>
          <SearchInput v-model="inputAgentName" placeholder="输入 Agent 名称" />
        </div>

        <div class="field-stack">
          <label>技能引用根目录 (skills 目录)</label>
          <div class="path-input-row">
            <SearchInput v-model="inputAgentPath" placeholder="输入或浏览目录路径（可使用 ~ 开头）" />
            <button
              class="secondary-button"
              type="button"
              :disabled="busy"
              @click="browseAgentPath"
            >
              <FolderOpen :size="16" />
              <span>浏览</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="button-row button-row--end modal-footer-row">
        <button class="secondary-button" :disabled="busy" @click="closeAddAgentDialog">取消</button>
        <button class="primary-button" :disabled="busy || !inputAgentName.trim() || !inputAgentPath.trim()" @click="confirmAddAgent">
          确定
        </button>
      </div>
    </template>
  </ModalDialog>

  <ModalDialog
    v-if="deleteAgentDialogOpen && selectedAgent"
    title="删除 Agent"
    @close="deleteAgentDialogOpen = false"
  >
    <p class="modal-note">
      确认删除 Agent "{{ selectedAgent.name }}" 吗？这不会影响其目录下的实际技能文件。
    </p>
    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="deleteAgentDialogOpen = false">取消</button>
        <button class="danger-button" :disabled="busy" @click="confirmDeleteAgent">
          删除 Agent
        </button>
      </div>
    </template>
  </ModalDialog>

  <ModalDialog
    v-if="pendingReferenceRemoval"
    title="移除技能引用"
    @close="pendingReferenceRemoval = null"
  >
    <p class="modal-note">
      确认从 Agent 中移除这个技能引用吗？这会删除对应的托管链接，不会删除技能库中的 skill。
    </p>
    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="pendingReferenceRemoval = null">取消</button>
        <button class="danger-button" :disabled="busy" @click="confirmRemoveManagedSkillReference">
          移除引用
        </button>
      </div>
    </template>
  </ModalDialog>

  <ModalDialog
    v-if="pendingUnmanagedSkillDeletion"
    title="删除未托管 Skill"
    @close="pendingUnmanagedSkillDeletion = null"
  >
    <p class="modal-note">
      确认删除 "{{ pendingUnmanagedSkillDeletion.skillName }}" 吗？这会删除磁盘上的 skill 文件夹。
    </p>
    <dl class="modal-summary">
      <dt>路径</dt>
      <dd>{{ pendingUnmanagedSkillDeletion.skillPath }}</dd>
    </dl>
    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="pendingUnmanagedSkillDeletion = null">取消</button>
        <button class="danger-button" :disabled="busy" @click="confirmDeleteUnmanagedSkill">
          删除文件夹
        </button>
      </div>
    </template>
  </ModalDialog>

  <!-- Add Skill to Agent Dialog -->
  <ModalDialog
    v-if="addSkillDialogOpen && selectedAgent"
    :title="`关联技能到 ${selectedAgent.name}`"
    card-class="modal-card--compact"
    @close="closeAddSkillDialog"
  >
    <div class="modal-step-section">
      <!-- Search box inside dialog -->
      <SearchInput v-model="addSkillQuery" placeholder="搜索技能" class="dialog-search-input" />

      <!-- Check all / Selected count -->
      <div class="dialog-checklist-header">
        <label style="display: flex; align-items: center; gap: 6px; cursor: pointer;">
          <input
            type="checkbox"
            :checked="filteredLibrarySkills.length > 0 && selectedSkillIds.length === filteredLibrarySkills.length"
            :disabled="filteredLibrarySkills.length === 0 || busy"
            @change="toggleAllLibrarySkills(($event.target as HTMLInputElement).checked)"
          />
          <span>全选</span>
        </label>
        <span>已选择 {{ selectedSkillIds.length }} / {{ filteredLibrarySkills.length }} 个技能</span>
      </div>

      <!-- Skill list with checkboxes -->
      <div v-if="filteredLibrarySkills.length" class="project-skill-picker" style="max-height: 250px; overflow-y: auto; border: 1px solid var(--border-default); border-radius: 8px; padding: 4px;">
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
            style="margin-top: 3px;"
          />
          <span style="display: flex; flex-direction: column; gap: 2px;">
            <strong>{{ skill.name }}</strong>
            <small style="font-size: 11px; color: var(--text-secondary);">
              <code>{{ skill.id }}</code>
              <span v-if="skill.description"> · {{ skill.description }}</span>
            </small>
          </span>
        </label>
      </div>
      <div v-else class="content-empty content-empty--compact">没有匹配的技能。</div>
    </div>

    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="closeAddSkillDialog">取消</button>
        <button class="primary-button" :disabled="busy || selectedSkillIds.length === 0" @click="confirmAddSkillReferences">
          确定
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
        <div style="display: flex; align-items: center; gap: 8px; color: var(--warning-text);">
          <AlertTriangle :size="20" />
          <h2 class="conflict-modal-title">导入冲突检测</h2>
        </div>
        <button class="ghost-icon-button" type="button" aria-label="关闭" @click="conflictState = null">
          <X :size="16" />
        </button>
      </div>
    </template>

    <div class="conflict-modal-body">
      <p>技能库中已存在同名技能 (ID: <code>{{ conflictState.skillId }}</code>)，请选择处理策略：</p>
      
      <div class="conflict-compare-box">
        <div class="conflict-compare-item">
          <span class="conflict-compare-label">本地版本名称：</span>
          <span class="conflict-compare-value">{{ conflictState.projectName }}</span>
        </div>
        <div class="conflict-compare-item">
          <span class="conflict-compare-label">技能库已有版本名称：</span>
          <span class="conflict-compare-value">{{ conflictState.libraryName }}</span>
        </div>
      </div>
      
      <p style="font-size: 13px; color: var(--text-muted); margin: 0;">
        <strong>覆盖已有</strong>：使用此本地的版本覆盖统一技能库中的版本（建议在本地版本有最新修改时使用）。<br/>
        <strong>保留已有</strong>：保留技能库中的现有版本，丢弃本地的修改，直接将该目录链接至技能库现有技能。
      </p>
    </div>

    <template #footer>
      <div class="conflict-modal-footer">
        <button class="ghost-button" :disabled="busy" @click="conflictState = null">
          取消
        </button>
        <button class="primary-button" :disabled="busy" @click="handleImportSkill(conflictState.skillPath, 'keep_existing')">
          保留已有
        </button>
        <button class="primary-button danger-override-btn" :disabled="busy" @click="handleImportSkill(conflictState.skillPath, 'overwrite')">
          覆盖已有
        </button>
      </div>
    </template>
  </ModalDialog>
</template>

<style scoped>
.list-search-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 30px;
  gap: 8px;
  align-items: center;
}

.search-row-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 30px;
  font-family: ui-monospace, monospace;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: 6px;
  flex-shrink: 0;
}

.detail-search-input {
  flex: 1;
}

.dialog-search-input {
  margin-bottom: 12px;
}

.dialog-checklist-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  padding: 0 4px;
}

.dialog-skill-pick-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 8px;
  border-bottom: 1px solid var(--border-default);
  cursor: pointer;
  transition: background 0.15s;
}

.dialog-skill-pick-row:hover {
  background: var(--bg-hover);
}

.dialog-skill-pick-row.disabled {
  pointer-events: none;
}

.dialog-footer-row {
  margin-top: 18px;
  border-top: 1px solid var(--border-default);
  padding-top: 14px;
}

.header-warning-title {
  margin-bottom: 12px;
}

.danger-override-btn {
  background-color: var(--danger-bg);
  color: var(--danger-text);
  border-color: var(--danger-border);
}
</style>

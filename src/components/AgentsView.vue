<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  FolderPlus,
  Plus,
  Trash2,
  X,
  FolderOpen,
  RefreshCw,
  AlertTriangle,
  SquareTerminal,
  Bot,
  Github,
  Code2,
  Cpu,
  CircleHelp,
  Folder,
} from "lucide-vue-next";
import * as api from "../api";
import { openDirectory } from "../dialog";
import type { AppSnapshot, Agent, ScannedCategory } from "../types";

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedAgentId: string | null;
}>();

const emit = defineEmits<{
  "select-agent": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const agentQuery = ref("");
const skillQuery = ref("");
const busy = ref(false);

// Add Agent Dialog State
const addAgentDialogOpen = ref(false);
const selectedPresetIndex = ref<number>(0);
const inputAgentName = ref("");
const inputAgentPath = ref("");

// Add Skill Dialog State (linking library skills to selected agent)
const addSkillDialogOpen = ref(false);
const addSkillQuery = ref("");
const selectedSkillIds = ref<string[]>([]);

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

const targetIcons: Record<string, any> = {
  Codex: SquareTerminal,
  "Claude Code": Bot,
  "Gemini CLI": Bot,
  "GitHub Copilot": Github,
  Cursor: Code2,
  WorkBuddy: Bot,
  Windsurf: Github,
  Kiro: Cpu,
  OpenCode: CircleHelp,
  自定义: Folder,
};

function iconForTarget(targetName: string) {
  return targetIcons[targetName] ?? CircleHelp;
}

// Compute active skills count for each agent
function agentSkillCount(agent: Agent): number {
  return Object.values(agent.rules).filter((rule) => rule === "enable").length;
}

const agents = computed(() => {
  const normalized = agentQuery.value.trim().toLowerCase();
  const list = props.snapshot.state.agents || [];
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
  () => agents.value.find((agent) => agent.id === props.selectedAgentId) ?? agents.value[0] ?? null
);

async function run(action: () => Promise<AppSnapshot>) {
  busy.value = true;
  try {
    emit("snapshot", await action());
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

// Add Agent functions
function openAddAgentDialog() {
  selectedPresetIndex.value = 0;
  inputAgentName.value = PRESET_AGENTS[0].name;
  inputAgentPath.value = PRESET_AGENTS[0].defaultPath;
  addAgentDialogOpen.value = true;
}

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
    emit("error", String(cause));
  }
}

async function confirmAddAgent() {
  if (!inputAgentName.value.trim() || !inputAgentPath.value.trim()) {
    emit("error", "请填写 Agent 名称和目标路径");
    return;
  }
  busy.value = true;
  try {
    const next = await api.addAgent(inputAgentName.value.trim(), inputAgentPath.value.trim());
    emit("snapshot", next);
    // Auto-select the newly added agent by looking at differences
    const newAgent = next.state.agents.find(
      (a) => !props.snapshot.state.agents?.some((oldAgent) => oldAgent.id === a.id)
    );
    if (newAgent) {
      emit("select-agent", newAgent.id);
    }
    closeAddAgentDialog();
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

async function handleDeleteAgent() {
  if (!selectedAgent.value) return;
  if (!confirm(`确认删除 Agent "${selectedAgent.value.name}" 吗？这不会影响其目录下的实际技能文件。`)) return;
  busy.value = true;
  try {
    const next = await api.deleteAgent(selectedAgent.value.id);
    emit("snapshot", next);
    emit("select-agent", next.state.agents?.[0]?.id ?? null);
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

// Skill dialog functions
function openAddSkillDialog() {
  selectedSkillIds.value = [];
  addSkillQuery.value = "";
  addSkillDialogOpen.value = true;
}

function closeAddSkillDialog() {
  addSkillDialogOpen.value = false;
}

const filteredLibrarySkills = computed(() => {
  const normalized = addSkillQuery.value.trim().toLowerCase();
  const allSkills = props.snapshot.state.skills || [];
  if (!normalized) return allSkills;
  return allSkills.filter(
    (skill) =>
      skill.name.toLowerCase().includes(normalized) ||
      skill.id.toLowerCase().includes(normalized) ||
      skill.description.toLowerCase().includes(normalized)
  );
});

function toggleAllLibrarySkills(checked: boolean) {
  if (checked) {
    selectedSkillIds.value = filteredLibrarySkills.value.map((s) => s.id);
  } else {
    selectedSkillIds.value = [];
  }
}

async function confirmAddSkillReferences() {
  if (!selectedAgent.value || selectedSkillIds.value.length === 0) return;
  busy.value = true;
  try {
    let currentSnapshot = props.snapshot;
    for (const skillId of selectedSkillIds.value) {
      // 1. Link references
      currentSnapshot = await api.addSkillReference({
        skillId,
        targetName: selectedAgent.value.name,
        rootPath: selectedAgent.value.path,
        scope: "user" as const,
        overwrite: true,
      });
      // 2. Set rule to enable explicitly
      currentSnapshot = await api.setAgentRule({
        agentId: selectedAgent.value.id,
        skillId,
        rule: "enable",
      });
    }
    emit("snapshot", currentSnapshot);
    closeAddSkillDialog();
    await refreshScan();
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
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
  await refreshScan();
}

// Scanning functions
const scannedCategories = ref<ScannedCategory[]>([]);
const scanning = ref(false);
const conflictState = ref<{ skillId: string; libraryName: string; projectName: string; skillPath: string } | null>(null);

async function refreshScan() {
  if (!selectedAgent.value) return;
  scanning.value = true;
  try {
    scannedCategories.value = await api.scanAgentSkills(selectedAgent.value.path);
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    scanning.value = false;
  }
}

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
  const skill = props.snapshot.state.skills.find((s) => s.id === skillId);
  if (!skill || !skill.references) return null;
  return (
    skill.references.find(
      (r) => r.targetPath.replace(/[\\/]+/g, "/").toLowerCase() === skillPath.replace(/[\\/]+/g, "/").toLowerCase()
    )?.id ?? null
  );
}

async function removeManagedSkillReference(skillId: string, skillPath: string) {
  const refId = findReferenceIdForScannedSkill(skillId, skillPath);
  if (!refId) {
    emit("error", "无法找到该引用的记录，请确认该技能已在技能详情的引用列表中注册。");
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
  await refreshScan();
}

async function handleImportSkill(skillPath: string, strategy?: "overwrite" | "keep_existing") {
  if (!selectedAgent.value) return;
  busy.value = true;
  try {
    const result = await api.importProjectSkill(selectedAgent.value.name, skillPath, strategy);
    if (result.type === "success") {
      emit("snapshot", result.snapshot);
      conflictState.value = null;
    } else if (result.type === "conflict") {
      conflictState.value = {
        skillId: result.skillId,
        libraryName: result.libraryName,
        projectName: result.projectName,
        skillPath,
      };
    }
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

watch(
  () => selectedAgent.value?.id,
  (newId, oldId) => {
    if (newId !== oldId) {
      scannedCategories.value = [];
    }
    refreshScan();
  },
  { immediate: true }
);

watch(
  () => props.snapshot.state.skills,
  () => {
    refreshScan();
  }
);
</script>

<template>
  <div class="split-content">
    <section class="list-panel">
      <div class="list-panel-head">
        <div class="list-search-row">
          <input v-model="agentQuery" class="search-input" placeholder="搜索 Agent 名称或路径" />
          <button class="icon-button" type="button" :disabled="busy" aria-label="添加 Agent" @click="openAddAgentDialog">
            <FolderPlus :size="18" />
          </button>
        </div>
      </div>

      <div v-if="agents.length" class="list-stack">
        <button
          v-for="agent in agents"
          :key="agent.id"
          class="list-row"
          :class="{ active: selectedAgent?.id === agent.id }"
          @click="emit('select-agent', agent.id)"
        >
          <div class="list-row-copy">
            <strong>{{ agent.name }}</strong>
            <small>{{ agent.path }}</small>
          </div>
        </button>
      </div>

      <div v-else class="content-empty">没有匹配的 Agent。</div>
    </section>

    <section class="detail-panel">
      <template v-if="selectedAgent">
        <div class="detail-header">
          <div>
            <h2>{{ selectedAgent.name }}</h2>
            <p>{{ selectedAgent.path }}</p>
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="ghost-icon-button ghost-icon-button--danger" :disabled="busy" aria-label="删除 Agent" @click="handleDeleteAgent">
              <Trash2 :size="16" />
            </button>
            <button class="primary-button" :disabled="busy" aria-label="添加技能" @click="openAddSkillDialog">
              <Plus :size="16" />
            </button>
          </div>
        </div>

        <section class="detail-section">
          <div class="project-skill-toolbar" style="display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 14px;">
            <input v-model="skillQuery" class="search-input" placeholder="搜索技能" style="flex: 1;" />
            <button class="ghost-icon-button" type="button" :disabled="busy || scanning" aria-label="重新扫描" title="重新扫描" @click="refreshScan">
              <RefreshCw :size="14" :class="{ 'spin-animation': scanning }" />
            </button>
          </div>

          <div v-if="filteredScannedCategories.length && filteredScannedCategories[0].skills.length" class="scanned-skills-list" style="border-left: none; padding-left: 0;">
            <article v-for="skill in filteredScannedCategories[0].skills" :key="skill.path" class="project-skill-row">
              <div class="project-skill-copy">
                <div style="display: flex; align-items: center; gap: 8px;">
                  <strong>{{ skill.name }}</strong>
                  <span v-if="selectedAgent.rules[skill.id] === 'disable'" class="badge badge--error" style="font-size: 10px; padding: 2px 6px;">已停用</span>
                </div>
                <small>
                  <code>{{ skill.id }}</code>
                  <span v-if="skill.description"> · {{ skill.description }}</span>
                </small>
              </div>

              <div class="project-skill-actions">
                <template v-if="skill.isManaged">
                  <label class="switch-toggle" title="启用/停用技能（软链接同步）" style="margin-right: 4px;">
                    <input
                      type="checkbox"
                      :checked="selectedAgent.rules[skill.id] !== 'disable'"
                      :disabled="busy"
                      @change="toggleSkillRule(skill.id)"
                    />
                    <span class="switch-slider"></span>
                  </label>

                  <button
                    class="ghost-icon-button ghost-icon-button--danger"
                    type="button"
                    :disabled="busy"
                    aria-label="从 Agent 移除技能引用"
                    title="从 Agent 移除技能引用"
                    @click="removeManagedSkillReference(skill.id, skill.path)"
                  >
                    <Trash2 :size="15" />
                  </button>
                </template>

                <template v-else>
                  <button
                    class="primary-button"
                    type="button"
                    :disabled="busy"
                    style="font-size: 12px; height: 28px; padding: 0 10px;"
                    @click="handleImportSkill(skill.path)"
                  >
                    托管
                  </button>
                </template>
              </div>
            </article>
          </div>
          <div v-else class="content-empty" style="padding: 24px 0;">此 Agent 下尚未扫描到任何技能。</div>

        </section>
      </template>

      <div v-else class="content-empty">选择左侧 Agent 查看技能列表。</div>
    </section>
  </div>

  <!-- Add Agent Dialog -->
  <div v-if="addAgentDialogOpen" class="modal-backdrop" @click.self="closeAddAgentDialog">
    <section class="modal-card modal-card--agent">
      <div class="modal-title-row">
        <h2>添加 Agent</h2>
        <button class="ghost-icon-button" type="button" aria-label="关闭" @click="closeAddAgentDialog">
          <X :size="16" />
        </button>
      </div>

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
              <component :is="iconForTarget(preset.targetName)" :size="20" />
            </span>
            <strong style="font-size: 12px;">{{ preset.name }}</strong>
          </button>
        </div>

        <div class="agent-form-grid">
          <div class="field-stack">
            <label>Agent 名称</label>
            <input v-model="inputAgentName" class="search-input" placeholder="输入 Agent 名称" />
          </div>

          <div class="field-stack">
            <label>技能引用根目录 (skills 目录)</label>
            <div class="path-input-row">
              <input v-model="inputAgentPath" class="search-input" placeholder="输入或浏览目录路径（可使用 ~ 开头）" />
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

      <div class="button-row button-row--end modal-footer-row">
        <button class="secondary-button" :disabled="busy" @click="closeAddAgentDialog">取消</button>
        <button class="primary-button" :disabled="busy || !inputAgentName.trim() || !inputAgentPath.trim()" @click="confirmAddAgent">
          确定
        </button>
      </div>
    </section>
  </div>

  <!-- Add Skill to Agent Dialog -->
  <div v-if="addSkillDialogOpen && selectedAgent" class="modal-backdrop" @click.self="closeAddSkillDialog">
    <section class="modal-card modal-card--compact">
      <div class="modal-title-row" style="margin-bottom: 16px;">
        <h2>关联技能到 {{ selectedAgent.name }}</h2>
        <button class="ghost-icon-button" type="button" aria-label="关闭" @click="closeAddSkillDialog">
          <X :size="16" />
        </button>
      </div>

      <div class="modal-step-section">
        <!-- Search box inside dialog -->
        <input v-model="addSkillQuery" class="search-input" placeholder="搜索技能" style="margin-bottom: 12px;" />

        <!-- Check all / Selected count -->
        <div style="display: flex; align-items: center; justify-content: space-between; font-size: 12px; color: var(--text-secondary); margin-bottom: 8px; padding: 0 4px;">
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
            class="project-skill-pick-row"
            style="display: flex; align-items: flex-start; gap: 10px; padding: 8px; border-bottom: 1px solid var(--border-default); cursor: pointer; transition: background 0.15s;"
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

      <div class="button-row button-row--end" style="margin-top: 18px; border-top: 1px solid var(--border-default); padding-top: 14px;">
        <button class="secondary-button" :disabled="busy" @click="closeAddSkillDialog">取消</button>
        <button class="primary-button" :disabled="busy || selectedSkillIds.length === 0" @click="confirmAddSkillReferences">
          确定
        </button>
      </div>
    </section>
  </div>

  <!-- Conflict Modal Dialog -->
  <div v-if="conflictState" class="modal-backdrop" @click.self="conflictState = null">
    <section class="conflict-modal-card">
      <div class="modal-title-row" style="margin-bottom: 12px;">
        <div style="display: flex; align-items: center; gap: 8px; color: var(--warning-text);">
          <AlertTriangle :size="20" />
          <h2 class="conflict-modal-title">导入冲突检测</h2>
        </div>
        <button class="ghost-icon-button" type="button" aria-label="关闭" @click="conflictState = null">
          <X :size="16" />
        </button>
      </div>

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

      <div class="conflict-modal-footer">
        <button class="ghost-button" :disabled="busy" @click="conflictState = null">
          取消
        </button>
        <button class="primary-button" :disabled="busy" @click="handleImportSkill(conflictState.skillPath, 'keep_existing')">
          保留已有
        </button>
        <button class="primary-button" :disabled="busy" @click="handleImportSkill(conflictState.skillPath, 'overwrite')" style="background-color: var(--danger-bg); color: var(--danger-text); border-color: var(--danger-border);">
          覆盖已有
        </button>
      </div>
    </section>
  </div>
</template>

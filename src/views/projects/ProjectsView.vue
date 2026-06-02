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
import type { AppSnapshot, Project, ProjectRule, ScannedCategory, ReferenceScope } from "../../types";
import { AppStoreKey } from "../../stores/useAppStore";
import { SelectionStoreKey } from "../../stores/useSelectionStore";
import { useAsyncAction } from "../../composables/useAsyncAction";
import { useSkillScanner } from "../../composables/useSkillScanner";
import { useSkillPicker } from "../../composables/useSkillPicker";

const appStore = inject(AppStoreKey, null);
const selectionStore = inject(SelectionStoreKey, null);

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedProjectId: string | null;
}>();

const emit = defineEmits<{
  "select-project": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const snapshot = computed(() => appStore?.snapshot.value ?? props.snapshot);
const selectedProjectId = computed({
  get: () => selectionStore?.selectedProjectId.value ?? props.selectedProjectId,
  set: (val) => {
    if (selectionStore) {
      selectionStore.setSelectedProjectId(val);
    } else {
      emit("select-project", val);
    }
  }
});

const projectQuery = ref("");
const skillQuery = ref("");
const addSkillDialogOpen = ref(false);
const selectedAddDir = ref("");
const selectedAddScope = ref<ReferenceScope>("project");
const selectedAddTargetName = ref("");
const deleteProjectDialogOpen = ref(false);
const pendingReferenceRemoval = ref<{ skillId: string; skillPath: string } | null>(null);

// Composable for Async Actions
const { busy, run: executeAsync } = useAsyncAction({
  onError: (err) => {
    if (appStore) appStore.setError(String(err));
    else emit("error", String(err));
  }
});

function projectSkillCount(project: Project): number {
  return Object.values(project.rules).filter((rule) => rule === "enable" || rule === "disable").length;
}

const projects = computed(() => {
  const normalized = projectQuery.value.trim().toLowerCase();
  return [...snapshot.value.state.projects]
    .filter((project) => {
      if (!normalized) return true;
      return `${project.name} ${project.path}`.toLowerCase().includes(normalized);
    })
    .sort((left, right) => {
      const skillBias = projectSkillCount(right) - projectSkillCount(left);
      return skillBias || left.name.localeCompare(right.name, "zh-CN");
    });
});

const selectedProject = computed(
  () => projects.value.find((project) => project.id === selectedProjectId.value) ?? projects.value[0] ?? null
);

const projectProfiles = computed(() => {
  if (!selectedProject.value) return [];
  const projectRoot = selectedProject.value.path;
  return (snapshot.value.targetProfiles || []).map((profile) => {
    let relPath = "";
    switch (profile.targetName) {
      case "Codex":
        relPath = ".agents/skills";
        break;
      case "Claude Code":
        relPath = ".claude/skills";
        break;
      case "GitHub Copilot":
        relPath = ".copilot/skills";
        break;
      case "Cursor":
        relPath = ".cursor/skills";
        break;
      case "Windsurf":
        relPath = ".codeium/windsurf/skills";
        break;
      case "Kiro":
        relPath = ".kiro/skills";
        break;
      default:
        const match = profile.rootPath.match(/[\\/](\.[^\\/]+[\\/].*)$/);
        if (match) {
          relPath = match[1];
        } else {
          relPath = "skills";
        }
    }
    const fullPath = `${projectRoot}/${relPath}`.replace(/[\\/]+/g, "/");
    return {
      ...profile,
      rootPath: fullPath,
      scope: "project" as const,
    };
  });
});

async function run(action: () => Promise<AppSnapshot>) {
  await executeAsync(action, (next) => {
    if (appStore) appStore.applySnapshot(next);
    else emit("snapshot", next);
  });
}

async function addProject() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      const parts = selected.split(/[\\/]/).filter(Boolean);
      const name = parts[parts.length - 1] ?? selected;
      await run(() => api.addProject({ name, path: selected }));
    }
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
    else emit("error", String(cause));
  }
}

// Composable for Skill Scanner
const {
  scannedCategories,
  scanning,
  scannedSkillsCount,
  refreshScan
} = useSkillScanner(
  () => selectedProject.value?.path,
  api.scanProjectSkills,
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

function openAddSkillDialog() {
  selectedAddDir.value = "";
  selectedAddScope.value = "project";
  selectedAddTargetName.value = "";
  resetPicker();
  addSkillDialogOpen.value = true;
}

function openAddSkillDialogForCategory(category: ScannedCategory) {
  const rootPath = `${category.path}/skills`.replace(/[\\/]+/g, "/");
  selectedAddDir.value = rootPath;
  resetPicker();

  const matchingProfile = projectProfiles.value.find(
    (profile) => profile.rootPath.replace(/[\\/]+/g, "/").toLowerCase() === rootPath.toLowerCase()
  );

  if (matchingProfile) {
    selectedAddScope.value = matchingProfile.scope;
    selectedAddTargetName.value = matchingProfile.targetName;
  } else {
    const projectPath = selectedProject.value?.path || "";
    const isInsideProject = rootPath.toLowerCase().startsWith(projectPath.replace(/[\\/]+/g, "/").toLowerCase());
    if (isInsideProject) {
      selectedAddScope.value = "project";
      const nameMap: Record<string, string> = {
        ".agent": "Codex",
        ".agents": "Codex",
        ".claude": "Claude Code",
        ".copilot": "GitHub Copilot",
        ".cursor": "Cursor",
        ".codeium/windsurf": "Windsurf",
        ".kiro": "Kiro",
        ".opencode": "OpenCode",
      };
      const mappedName = nameMap[category.name];
      if (mappedName) {
        selectedAddTargetName.value = mappedName;
      } else {
        selectedAddTargetName.value = category.name === "." ? (selectedProject.value?.name || "项目目录") : category.name;
      }
    } else {
      selectedAddScope.value = "custom";
      selectedAddTargetName.value = "自定义目录";
    }
  }

  addSkillDialogOpen.value = true;
}

function closeAddSkillDialog() {
  addSkillDialogOpen.value = false;
  selectedAddDir.value = "";
  selectedAddScope.value = "project";
  selectedAddTargetName.value = "";
  resetPicker();
}

function openDeleteProjectDialog() {
  if (!selectedProject.value) return;
  deleteProjectDialogOpen.value = true;
}

async function confirmDeleteProject() {
  if (!selectedProject.value) return;
  await executeAsync(
    () => api.deleteProject(selectedProject.value!.id),
    (next) => {
      if (appStore) appStore.applySnapshot(next);
      else emit("snapshot", next);
      selectedProjectId.value = next.state.projects[0]?.id ?? null;
      deleteProjectDialogOpen.value = false;
    }
  );
}

function selectAddProfile(profile: any) {
  selectedAddDir.value = profile.rootPath;
  selectedAddScope.value = profile.scope;
  selectedAddTargetName.value = profile.targetName;
}

async function selectCustomAddDir() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      selectedAddDir.value = selected;
      const projectPath = selectedProject.value?.path || "";
      const isInsideProject = selected.replace(/[\\/]+/g, "/").toLowerCase().startsWith(projectPath.replace(/[\\/]+/g, "/").toLowerCase());
      if (isInsideProject) {
        selectedAddScope.value = "project";
        selectedAddTargetName.value = selectedProject.value?.name || "项目目录";
      } else {
        selectedAddScope.value = "custom";
        selectedAddTargetName.value = "自定义目录";
      }
    }
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
    else emit("error", String(cause));
  }
}

async function confirmAddSkillReferences() {
  if (!selectedAddDir.value || selectedSkillIds.value.length === 0) return;
  await executeAsync(
    async () => {
      let currentSnapshot = snapshot.value;
      for (const skillId of selectedSkillIds.value) {
        const request = {
          skillId,
          targetName: selectedAddTargetName.value,
          rootPath: selectedAddDir.value,
          scope: selectedAddScope.value,
          overwrite: true,
        };
        currentSnapshot = await api.addSkillReference(request);
      }
      if (appStore) appStore.applySnapshot(currentSnapshot);
      else emit("snapshot", currentSnapshot);
      closeAddSkillDialog();
      await refreshScan();
    }
  );
}

function setSkillRule(skillId: string, rule: ProjectRule) {
  if (!selectedProject.value) return;
  return run(() => api.setProjectRule({ projectId: selectedProject.value!.id, skillId, rule }));
}

async function toggleSkillRule(skillId: string) {
  if (!selectedProject.value) return;
  const isCurrentlyDisabled = selectedProject.value.rules[skillId] === "disable";
  const newRule = isCurrentlyDisabled ? "enable" : "disable";
  await setSkillRule(skillId, newRule);
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
  const skill = snapshot.value.state.skills.find(s => s.id === skillId);
  if (!skill || !skill.references) return null;
  return skill.references.find(r => r.targetPath.replace(/[\\/]+/g, "/").toLowerCase() === skillPath.replace(/[\\/]+/g, "/").toLowerCase())?.id ?? null;
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
  await run(() => api.removeSkillReference(refId, true));
  pendingReferenceRemoval.value = null;
  await refreshScan();
}

watch(
  () => selectedProject.value?.id,
  (newId, oldId) => {
    if (newId !== oldId) {
      scannedCategories.value = [];
    }
    refreshScan();
  },
  { immediate: true }
);

watch(
  () => snapshot.value.state.skills,
  () => {
    refreshScan();
  }
);

async function handleImportSkill(skillPath: string, strategy?: "overwrite" | "keep_existing") {
  if (!selectedProject.value) return;
  await executeAsync(
    () => api.importProjectSkill(
      selectedProject.value!.name,
      skillPath,
      strategy
    ),
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
</script>

<template>
  <SplitPane>
    <template #left>
      <ListPanel :items="projects" :has-search="true" empty-text="没有匹配的项目。">
        <template #search-row>
          <div class="list-search-row">
            <span class="search-row-count">{{ snapshot.state.projects.length }}</span>
            <SearchInput v-model="projectQuery" placeholder="搜索项目名称或路径" />
            <button class="icon-button" type="button" :disabled="busy" aria-label="添加项目" @click="addProject">
              <FolderPlus :size="18" />
            </button>
          </div>
        </template>

        <button
          v-for="project in projects"
          :key="project.id"
          class="list-row"
          :class="{ active: selectedProject?.id === project.id }"
          @click="emit('select-project', project.id)"
        >
          <div class="list-row-copy">
            <strong>{{ project.name }}</strong>
            <small>{{ project.path }}</small>
          </div>
        </button>
      </ListPanel>
    </template>

    <template #right>
      <template v-if="selectedProject">
        <div class="detail-header">
          <div>
            <h2>{{ selectedProject.name }}</h2>
            <p>{{ selectedProject.path }}</p>
          </div>
          <div class="detail-actions">
            <button class="primary-button" :disabled="busy" aria-label="添加" @click="openAddSkillDialog">
              <Plus :size="16" />
            </button>
            <button
              class="danger-button danger-button--icon"
              :disabled="busy"
              aria-label="删除项目"
              title="删除项目"
              @click="openDeleteProjectDialog"
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
            v-if="filteredScannedCategories.length"
            :categories="filteredScannedCategories"
            :rules="selectedProject.rules"
            :busy="busy"
            :show-category-title="true"
            :show-add-button="true"
            @add-skill-click="openAddSkillDialogForCategory"
            @toggle-rule="toggleSkillRule"
            @remove-reference="removeManagedSkillReference"
            @import-skill="handleImportSkill"
          />
        </section>
      </template>

      <div v-else class="content-empty">选择左侧项目查看技能列表。</div>
    </template>
  </SplitPane>

  <ModalDialog
    v-if="addSkillDialogOpen && selectedProject"
    title="添加技能"
    card-class="modal-card--compact"
    @close="closeAddSkillDialog"
  >
    <!-- Step 1: Select directory if selectedAddDir is empty -->
    <div v-if="!selectedAddDir" class="modal-step-section">
      <p class="modal-instruction-text" style="font-size: 13px; color: var(--text-secondary); margin-bottom: 12px;">
        请选择要添加技能引用的目标目录：
      </p>

      <!-- Quick Select Agent Profiles -->
      <div class="target-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: 10px; margin-bottom: 16px;">
        <button
          v-for="profile in projectProfiles"
          :key="profile.id"
          class="target-tile"
          type="button"
          :disabled="busy"
          @click="selectAddProfile(profile)"
        >
          <span class="target-tile-icon" aria-hidden="true">
            <AgentIcon :name="profile.targetName" :size="20" />
          </span>
          <strong style="font-size: 13px;">{{ profile.targetName }}</strong>
          <small style="font-size: 10px; color: var(--text-muted); font-family: monospace; display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 100px;">
            {{ profile.rootPath }}
          </small>
        </button>
      </div>

      <!-- Custom Directory Picker Button -->
      <button
        class="target-custom-button"
        type="button"
        :disabled="busy"
        @click="selectCustomAddDir"
      >
        <FolderOpen :size="16" />
        选择自定义目录
      </button>
    </div>

    <!-- Step 2: Show selected path and skill checklist if selectedAddDir is NOT empty -->
    <div v-else class="modal-step-section">
      <!-- Header displaying chosen path -->
      <div class="chosen-path-box">
        <div class="chosen-path-copy">
          <span class="chosen-path-label">目标引用目录:</span>
          <code class="chosen-path-value">
            {{ selectedAddDir }}
          </code>
        </div>
        <button
          class="secondary-button chosen-path-edit-btn"
          type="button"
          :disabled="busy"
          @click="selectedAddDir = ''"
        >
          修改目录
        </button>
      </div>

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
        <button
          v-if="selectedAddDir"
          class="primary-button"
          :disabled="busy || selectedSkillIds.length === 0"
          @click="confirmAddSkillReferences"
        >
          确定
        </button>
      </div>
    </template>
  </ModalDialog>

  <ModalDialog
    v-if="deleteProjectDialogOpen && selectedProject"
    title="删除项目"
    @close="deleteProjectDialogOpen = false"
  >
    <p class="modal-note">
      确认删除项目 "{{ selectedProject.name }}" 吗？这只会移除 SkillMaster 中的项目记录，不会删除磁盘上的项目目录。
    </p>
    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="deleteProjectDialogOpen = false">取消</button>
        <button class="danger-button" :disabled="busy" @click="confirmDeleteProject">
          删除项目
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
      确认从项目中移除这个技能引用吗？这会删除对应的托管链接，不会删除技能库中的 skill。
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
          <span class="conflict-compare-label">项目本地版本名称：</span>
          <span class="conflict-compare-value">{{ conflictState.projectName }}</span>
        </div>
        <div class="conflict-compare-item">
          <span class="conflict-compare-label">技能库已有版本名称：</span>
          <span class="conflict-compare-value">{{ conflictState.libraryName }}</span>
        </div>
      </div>
      
      <p style="font-size: 13px; color: var(--text-muted); margin: 0;">
        <strong>覆盖已有</strong>：使用本项目本地的版本覆盖统一技能库中的版本（建议在本项目版本有最新修改时使用）。<br/>
        <strong>保留已有</strong>：保留技能库中的现有版本，丢弃项目中的修改，直接将本项目该目录链接至技能库现有技能。
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
  grid-template-columns: auto minmax(0, 1fr) 30px;
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

.target-custom-button {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px;
  border: 1px dashed var(--border-default);
  border-radius: 8px;
  background: none;
  color: var(--text-primary);
  cursor: pointer;
}

.chosen-path-box {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--bg-panel-muted);
  padding: 8px 12px;
  border: 1px solid var(--border-default);
  border-radius: 8px;
  margin-bottom: 14px;
}

.chosen-path-copy {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
  flex: 1;
  margin-right: 8px;
}

.chosen-path-label {
  font-size: 11px;
  color: var(--text-secondary);
}

.chosen-path-value {
  font-size: 12px;
  color: var(--text-primary);
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  font-family: monospace;
}

.chosen-path-edit-btn {
  font-size: 11px;
  height: 24px;
  padding: 0 8px;
  min-height: 24px;
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

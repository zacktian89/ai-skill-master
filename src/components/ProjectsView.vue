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
} from "lucide-vue-next";
import * as api from "../api";
import AgentIcon from "./AgentIcon.vue";
import { openDirectory } from "../dialog";
import { useScrollableList } from "../useScrollableList";
import type { AppSnapshot, Project, ProjectRule, ScannedCategory, ReferenceScope } from "../types";

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedProjectId: string | null;
}>();

const emit = defineEmits<{
  "select-project": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const projectQuery = ref("");
const skillQuery = ref("");
const addSkillDialogOpen = ref(false);
const addSkillQuery = ref("");
const busy = ref(false);

const selectedAddDir = ref("");
const selectedAddScope = ref<ReferenceScope>("project");
const selectedAddTargetName = ref("");
const selectedSkillIds = ref<string[]>([]);



function projectSkillCount(project: Project): number {
  return Object.values(project.rules).filter((rule) => rule === "enable" || rule === "disable").length;
}

const projects = computed(() => {
  const normalized = projectQuery.value.trim().toLowerCase();
  return [...props.snapshot.state.projects]
    .filter((project) => {
      if (!normalized) return true;
      return `${project.name} ${project.path}`.toLowerCase().includes(normalized);
    })
    .sort((left, right) => {
      const skillBias = projectSkillCount(right) - projectSkillCount(left);
      return skillBias || left.name.localeCompare(right.name, "zh-CN");
    });
});

const { listStackRef, listStackScrollable } = useScrollableList(projects);

const selectedProject = computed(
    () => projects.value.find((project) => project.id === props.selectedProjectId) ?? projects.value[0] ?? null
);

const projectProfiles = computed(() => {
  if (!selectedProject.value) return [];
  const projectRoot = selectedProject.value.path;
  return (props.snapshot.targetProfiles || []).map((profile) => {
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
  busy.value = true;
  try {
    emit("snapshot", await action());
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
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
    emit("error", String(cause));
  }
}

function openAddSkillDialog() {
  selectedAddDir.value = "";
  selectedAddScope.value = "project";
  selectedAddTargetName.value = "";
  selectedSkillIds.value = [];
  addSkillQuery.value = "";
  addSkillDialogOpen.value = true;
}

function openAddSkillDialogForCategory(category: ScannedCategory) {
  const rootPath = `${category.path}/skills`.replace(/[\\/]+/g, "/");
  selectedAddDir.value = rootPath;
  selectedSkillIds.value = [];
  addSkillQuery.value = "";

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
  selectedSkillIds.value = [];
  addSkillQuery.value = "";
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
    emit("error", String(cause));
  }
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
  if (!selectedAddDir.value || selectedSkillIds.value.length === 0) return;
  busy.value = true;
  try {
    let currentSnapshot = props.snapshot;
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
    emit("snapshot", currentSnapshot);
    closeAddSkillDialog();
    await refreshScan();
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
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

const scannedCategories = ref<ScannedCategory[]>([]);
const scannedSkillsCount = computed(() => {
  return scannedCategories.value.reduce((acc, cat) => acc + cat.skills.length, 0);
});
const scanning = ref(false);
const conflictState = ref<{ skillId: string; libraryName: string; projectName: string; skillPath: string } | null>(null);

async function refreshScan() {
  if (!selectedProject.value) return;
  scanning.value = true;
  try {
    scannedCategories.value = await api.scanProjectSkills(selectedProject.value.path);
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
  const skill = props.snapshot.state.skills.find(s => s.id === skillId);
  if (!skill || !skill.references) return null;
  return skill.references.find(r => r.targetPath.replace(/[\\/]+/g, "/").toLowerCase() === skillPath.replace(/[\\/]+/g, "/").toLowerCase())?.id ?? null;
}

async function removeManagedSkillReference(skillId: string, skillPath: string) {
  const refId = findReferenceIdForScannedSkill(skillId, skillPath);
  if (!refId) {
    emit("error", "无法找到该引用的记录，请确认该技能已在技能详情的引用列表中注册。");
    return;
  }
  await run(() => api.removeSkillReference(refId, true));
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
  () => props.snapshot.state.skills,
  () => {
    refreshScan();
  }
);

async function handleImportSkill(skillPath: string, strategy?: "overwrite" | "keep_existing") {
  if (!selectedProject.value) return;
  busy.value = true;
  try {
    const result = await api.importProjectSkill(
      selectedProject.value.name,
      skillPath,
      strategy
    );
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
</script>

<template>
  <div class="split-content">
    <section class="list-panel">
      <div class="list-panel-head">
        <div class="list-search-row" style="display: grid; grid-template-columns: auto minmax(0, 1fr) 30px; gap: 8px; align-items: center;">
          <span class="search-row-count" style="display: inline-flex; align-items: center; justify-content: center; width: 32px; height: 30px; font-family: ui-monospace, monospace; font-size: 12px; font-weight: 600; color: var(--text-secondary); background: var(--bg-input); border: 1px solid var(--border-default); border-radius: 6px; flex-shrink: 0;">{{ snapshot.state.projects.length }}</span>
          <input v-model="projectQuery" class="search-input" placeholder="搜索项目名称或路径" />
          <button class="icon-button" type="button" :disabled="busy" aria-label="添加项目" @click="addProject">
            <FolderPlus :size="18" />
          </button>
        </div>
      </div>

      <div v-if="projects.length" ref="listStackRef" class="list-stack" :class="{ 'list-stack--scrollable': listStackScrollable }">
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
      </div>

      <div v-else class="content-empty">没有匹配的项目。</div>
    </section>

    <section class="detail-panel">
      <template v-if="selectedProject">
        <div class="detail-header">
          <div>
            <h2>{{ selectedProject.name }}</h2>
            <p>{{ selectedProject.path }}</p>
          </div>
          <button class="primary-button" :disabled="busy" aria-label="添加" @click="openAddSkillDialog">
            <Plus :size="16" />
          </button>
        </div>

        <section class="detail-section">
          <div class="project-skill-toolbar" style="display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 14px;">
            <span class="search-row-count" style="display: inline-flex; align-items: center; justify-content: center; width: 32px; height: 30px; font-family: ui-monospace, monospace; font-size: 12px; font-weight: 600; color: var(--text-secondary); background: var(--bg-input); border: 1px solid var(--border-default); border-radius: 6px; flex-shrink: 0;">{{ scannedSkillsCount }}</span>
            <input v-model="skillQuery" class="search-input" placeholder="搜索技能" style="flex: 1;" />
            <button class="ghost-icon-button" type="button" :disabled="busy || scanning" aria-label="重新扫描" title="重新扫描" @click="refreshScan">
              <RefreshCw :size="14" :class="{ 'spin-animation': scanning }" />
            </button>
          </div>

          <div v-if="filteredScannedCategories.length" class="scanned-categories-list">
            <div v-for="category in filteredScannedCategories" :key="category.path" class="scanned-category-item" style="margin-bottom: 20px;">
              <div class="scanned-category-title" style="margin-bottom: 8px; font-weight: 600; font-size: 13px; color: var(--text-muted); display: flex; align-items: center; justify-content: space-between; width: 100%;">
                <div style="display: flex; align-items: center; gap: 6px;">
                  <span>📁 模块:</span>
                  <code>{{ category.name }}</code>
                </div>
                <button
                  class="ghost-icon-button"
                  type="button"
                  :disabled="busy"
                  aria-label="向此模块添加skill"
                  title="向此模块添加skill"
                  style="width: 24px; height: 24px; border-radius: 6px;"
                  @click="openAddSkillDialogForCategory(category)"
                >
                  <Plus :size="12" />
                </button>
              </div>

              <div class="scanned-skills-list">
                <article v-for="skill in category.skills" :key="skill.path" class="project-skill-row">
                  <div class="project-skill-copy">
                    <div style="display: flex; align-items: center; gap: 8px;">
                      <strong>{{ skill.name }}</strong>
                    </div>
                    <small>
                      <code>{{ skill.id }}</code>
                      <span v-if="skill.description"> · {{ skill.description }}</span>
                    </small>
                  </div>

                  <div class="project-skill-actions">
                    <!-- If managed, show Enable/Disable switch + Delete reference button -->
                    <template v-if="skill.isManaged">
                      <label class="switch-toggle" title="控制是否启用此技能" style="margin-right: 4px;">
                        <input
                          type="checkbox"
                          :checked="selectedProject.rules[skill.id] !== 'disable'"
                          :disabled="busy"
                          @change="toggleSkillRule(skill.id)"
                        />
                        <span class="switch-slider"></span>
                      </label>

                      <button
                        class="ghost-icon-button ghost-icon-button--danger"
                        type="button"
                        :disabled="busy"
                        aria-label="从项目移除技能引用"
                        title="从项目移除技能引用"
                        @click="removeManagedSkillReference(skill.id, skill.path)"
                      >
                        <Trash2 :size="15" />
                      </button>
                    </template>

                    <!-- If unmanaged, show 托管 button -->
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
            </div>
          </div>

        </section>
      </template>

      <div v-else class="content-empty">选择左侧项目查看技能列表。</div>
    </section>
  </div>

  <div v-if="addSkillDialogOpen && selectedProject" class="modal-backdrop" @click.self="closeAddSkillDialog">
    <section class="modal-card modal-card--compact">
      <div class="modal-title-row" style="margin-bottom: 16px;">
        <div>
          <h2>添加技能</h2>
        </div>
        <button class="ghost-icon-button" type="button" aria-label="关闭" @click="closeAddSkillDialog">
          <X :size="16" />
        </button>
      </div>

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
          style="width: 100%; display: flex; align-items: center; justify-content: center; gap: 8px; padding: 10px; border: 1px dashed var(--border-default); border-radius: 8px; background: none; color: var(--text-primary); cursor: pointer;"
        >
          <FolderOpen :size="16" />
          选择自定义目录
        </button>
      </div>

      <!-- Step 2: Show selected path and skill checklist if selectedAddDir is NOT empty -->
      <div v-else class="modal-step-section">
        <!-- Header displaying chosen path -->
        <div style="display: flex; justify-content: space-between; align-items: center; background: var(--bg-panel-muted); padding: 8px 12px; border: 1px solid var(--border-default); border-radius: 8px; margin-bottom: 14px;">
          <div style="display: flex; flex-direction: column; gap: 2px; overflow: hidden; flex: 1; margin-right: 8px;">
            <span style="font-size: 11px; color: var(--text-secondary);">目标引用目录:</span>
            <code style="font-size: 12px; color: var(--text-primary); text-overflow: ellipsis; overflow: hidden; white-space: nowrap; font-family: monospace;">
              {{ selectedAddDir }}
            </code>
          </div>
          <button
            class="secondary-button"
            type="button"
            style="font-size: 11px; height: 24px; padding: 0 8px; min-height: 24px;"
            :disabled="busy"
            @click="selectedAddDir = ''"
          >
            修改目录
          </button>
        </div>

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
        <button
          v-if="selectedAddDir"
          class="primary-button"
          :disabled="busy || selectedSkillIds.length === 0"
          @click="confirmAddSkillReferences"
        >
          确定
        </button>
      </div>
    </section>
  </div>

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

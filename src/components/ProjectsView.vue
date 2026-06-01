<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { FolderPlus, Plus, Trash2, X, FolderOpen, RefreshCw, AlertTriangle } from "lucide-vue-next";
import * as api from "../api";
import { openDirectory } from "../dialog";
import type { AppSnapshot, Project, ProjectRule, Skill, ScannedCategory } from "../types";

type ProjectSkillRule = Exclude<ProjectRule, "inherit">;

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

function projectSkillCount(project: Project): number {
  return Object.values(project.rules).filter((rule) => rule === "enable" || rule === "disable").length;
}


function projectRuleLabel(rule: ProjectRule): string {
  return rule === "disable" ? "停用" : "启用";
}

function projectRuleTone(rule: ProjectRule): "success" | "muted" {
  return rule === "disable" ? "muted" : "success";
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

const selectedProject = computed(
  () => projects.value.find((project) => project.id === props.selectedProjectId) ?? projects.value[0] ?? null,
);

const allSkills = computed(() => [...props.snapshot.state.skills].sort((left, right) => left.name.localeCompare(right.name, "zh-CN")));

const projectSkillRows = computed(() => {
  if (!selectedProject.value) return [];
  const normalized = skillQuery.value.trim().toLowerCase();
  return Object.entries(selectedProject.value.rules)
    .map(([skillId, rule]) => {
      if (rule !== "enable" && rule !== "disable") return null;
      const skill = props.snapshot.state.skills.find((item) => item.id === skillId);
      if (!skill) return null;
      return { skill, rule: rule as ProjectSkillRule };
    })
    .filter((item): item is { skill: Skill; rule: ProjectSkillRule } => Boolean(item))
    .filter(({ skill }) => {
      if (!normalized) return true;
      return `${skill.name} ${skill.description} ${skill.id}`.toLowerCase().includes(normalized);
    })
    .sort((left, right) => left.skill.name.localeCompare(right.skill.name, "zh-CN"));
});

const availableSkills = computed(() => {
  if (!selectedProject.value) return [];
  const normalized = addSkillQuery.value.trim().toLowerCase();
  const configured = new Set(Object.keys(selectedProject.value.rules));
  return allSkills.value
    .filter((skill) => !configured.has(skill.id))
    .filter((skill) => {
      if (!normalized) return true;
      return `${skill.name} ${skill.description} ${skill.id}`.toLowerCase().includes(normalized);
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
  addSkillQuery.value = "";
  addSkillDialogOpen.value = true;
}

function closeAddSkillDialog() {
  addSkillDialogOpen.value = false;
  addSkillQuery.value = "";
}

async function addSkillToProject(skillId: string) {
  if (!selectedProject.value) return;
  await run(() =>
    api.setProjectRule({
      projectId: selectedProject.value!.id,
      skillId,
      rule: "enable",
    }),
  );
  closeAddSkillDialog();
}

function setSkillRule(skillId: string, rule: ProjectSkillRule) {
  if (!selectedProject.value) return;
  return run(() => api.setProjectRule({ projectId: selectedProject.value!.id, skillId, rule }));
}

function removeSkillFromProject(skillId: string) {
  if (!selectedProject.value) return;
  return run(() => api.setProjectRule({ projectId: selectedProject.value!.id, skillId, rule: "inherit" }));
}

const scannedCategories = ref<ScannedCategory[]>([]);
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

watch(
  () => [selectedProject.value?.id, props.snapshot.state.skills],
  () => {
    scannedCategories.value = [];
    refreshScan();
  },
  { immediate: true }
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
        <div class="list-search-row">
          <input v-model="projectQuery" class="search-input" placeholder="搜索项目名称或路径" />
          <button class="icon-button" type="button" :disabled="busy" aria-label="添加项目" @click="addProject">
            <FolderPlus :size="18" />
          </button>
        </div>
      </div>

      <div v-if="projects.length" class="list-stack">
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
          <div class="list-row-meta">
            <span class="status-tag" :class="projectSkillCount(project) ? 'status-tag--neutral' : 'status-tag--muted'">
              {{ projectSkillCount(project) }} 个技能
            </span>
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
          <button class="primary-button" :disabled="busy" @click="openAddSkillDialog">
            <Plus :size="16" />
            添加
          </button>
        </div>

        <section class="detail-section">
          <div class="project-skill-toolbar">
            <input v-model="skillQuery" class="search-input" placeholder="搜索技能" />
          </div>

          <div v-if="projectSkillRows.length" class="project-skill-list">
            <article v-for="item in projectSkillRows" :key="item.skill.id" class="project-skill-row">
              <div class="project-skill-copy">
                <strong>{{ item.skill.name }}</strong>
                <small>
                  <code>{{ item.skill.id }}</code>
                  <span v-if="item.skill.description">{{ item.skill.description }}</span>
                </small>
              </div>

              <div class="project-skill-actions">
                <span
                  class="status-tag"
                  :class="[
                    `status-tag--${projectRuleTone(item.rule)}`,
                    busy ? 'status-tag--disabled' : 'status-tag--interactive'
                  ]"
                  @click="!busy && setSkillRule(item.skill.id, item.rule === 'enable' ? 'disable' : 'enable')"
                >
                  {{ projectRuleLabel(item.rule) }}
                </span>
                <button
                  class="ghost-icon-button ghost-icon-button--danger"
                  type="button"
                  :disabled="busy"
                  aria-label="从项目移除技能"
                  title="从项目移除技能"
                  @click="removeSkillFromProject(item.skill.id)"
                >
                  <Trash2 :size="15" />
                </button>
              </div>
            </article>
          </div>

          <div v-else class="content-empty content-empty--compact">
            这个项目还没有技能。
          </div>
        </section>

        <!-- 本地目录扫描区域 -->
        <section class="detail-section detail-section-divider">
          <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px;">
            <h3 style="margin: 0; font-size: 14px; font-weight: 600; color: var(--text-primary); display: flex; align-items: center; gap: 6px;">
              <FolderOpen :size="16" />
              本地模块目录扫描
            </h3>
            <button class="ghost-icon-button" type="button" :disabled="busy || scanning" aria-label="重新扫描" title="重新扫描" @click="refreshScan">
              <RefreshCw :size="14" :class="{ 'spin-animation': scanning }" />
            </button>
          </div>

          <div v-if="scannedCategories.length" class="scanned-categories-list">
            <div v-for="category in scannedCategories" :key="category.path" class="scanned-category-item">
              <div class="scanned-category-title">
                <span>📁 模块:</span>
                <code>{{ category.name }}</code>
              </div>

              <div class="scanned-skills-list">
                <article v-for="skill in category.skills" :key="skill.path" class="project-skill-row">
                  <div class="project-skill-copy">
                    <div style="display: flex; align-items: center; gap: 8px;">
                      <strong>{{ skill.name }}</strong>
                      <span class="status-tag" :class="skill.isManaged ? 'status-tag--success' : 'status-tag--neutral'">
                        {{ skill.isManaged ? '已管理' : '未管理' }}
                      </span>
                    </div>
                    <small>
                      <code>{{ skill.id }}</code>
                      <span v-if="skill.description"> · {{ skill.description }}</span>
                    </small>
                    <div style="font-size: 11px; color: var(--text-muted); margin-top: 4px; font-family: monospace; word-break: break-all;">
                      路径: {{ skill.path }}
                    </div>
                  </div>

                  <div class="project-skill-actions" v-if="!skill.isManaged">
                    <button
                      class="primary-button"
                      type="button"
                      :disabled="busy"
                      style="font-size: 12px; height: 28px; padding: 0 10px;"
                      @click="handleImportSkill(skill.path)"
                    >
                      导入并引用
                    </button>
                  </div>
                </article>
              </div>
            </div>
          </div>

          <div v-else-if="scanning" class="content-empty content-empty--compact">
            正在扫描本地技能目录...
          </div>
          <div v-else class="content-empty content-empty--compact">
            未在当前项目目录下扫描到任何含有 skills 文件夹的模块。
          </div>
        </section>
      </template>

      <div v-else class="content-empty">选择左侧项目查看技能列表。</div>
    </section>
  </div>

  <div v-if="addSkillDialogOpen && selectedProject" class="modal-backdrop" @click.self="closeAddSkillDialog">
    <section class="modal-card modal-card--compact">
      <div class="modal-title-row">
        <div>
          <h2>添加技能</h2>
        </div>
        <button class="ghost-icon-button" type="button" aria-label="关闭" @click="closeAddSkillDialog">
          <X :size="16" />
        </button>
      </div>

      <input v-model="addSkillQuery" class="search-input" placeholder="搜索可添加技能" />

      <div v-if="availableSkills.length" class="project-skill-picker">
        <button
          v-for="skill in availableSkills"
          :key="skill.id"
          class="project-skill-pick-row"
          type="button"
          :disabled="busy"
          @click="addSkillToProject(skill.id)"
        >
          <span>
            <strong>{{ skill.name }}</strong>
            <small>
              <code>{{ skill.id }}</code>
              <template v-if="skill.description"> · {{ skill.description }}</template>
            </small>
          </span>
          <Plus :size="16" />
        </button>
      </div>

      <div v-else class="content-empty content-empty--compact">没有可添加的技能。</div>
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

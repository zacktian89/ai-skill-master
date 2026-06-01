<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderPlus, Plus, Trash2, X } from "lucide-vue-next";
import * as api from "../api";
import { openDirectory } from "../dialog";
import type { AppSnapshot, Project, ProjectRule, Skill } from "../types";

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

function enabledProjectSkills(project: Project): number {
  return Object.values(project.rules).filter((rule) => rule === "enable").length;
}

function disabledProjectSkills(project: Project): number {
  return Object.values(project.rules).filter((rule) => rule === "disable").length;
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
</script>

<template>
  <div class="split-content">
    <section class="list-panel">
      <div class="list-panel-head">
        <div class="toolbar toolbar--stack">
          <input v-model="projectQuery" class="search-input" placeholder="搜索项目名称或路径" />
          <button class="primary-button" :disabled="busy" @click="addProject">
            <FolderPlus :size="16" />
            添加项目
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
            添加技能
          </button>
        </div>

        <section class="project-skill-summary" aria-label="项目技能统计">
          <div>
            <strong>{{ projectSkillCount(selectedProject) }}</strong>
            <span>技能</span>
          </div>
          <div>
            <strong>{{ enabledProjectSkills(selectedProject) }}</strong>
            <span>启用</span>
          </div>
          <div>
            <strong>{{ disabledProjectSkills(selectedProject) }}</strong>
            <span>停用</span>
          </div>
        </section>

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
                <span class="status-tag" :class="`status-tag--${projectRuleTone(item.rule)}`">
                  {{ projectRuleLabel(item.rule) }}
                </span>
                <div class="segmented-control segmented-control--binary">
                  <button :class="{ active: item.rule === 'enable' }" :disabled="busy" @click="setSkillRule(item.skill.id, 'enable')">
                    启用
                  </button>
                  <button :class="{ active: item.rule === 'disable' }" :disabled="busy" @click="setSkillRule(item.skill.id, 'disable')">
                    停用
                  </button>
                </div>
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
</template>

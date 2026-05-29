<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderPlus, RotateCcw, Trash2 } from "lucide-vue-next";
import * as api from "../api";
import { openDirectory } from "../dialog";
import { ruleLabel, type AppSnapshot, type Project, type ProjectRule, type Skill } from "../types";

type ProjectFilter = "all" | "current" | "overrides" | "empty";

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedProjectId: string | null;
}>();

const emit = defineEmits<{
  "select-project": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const query = ref("");
const filter = ref<ProjectFilter>("all");
const busy = ref(false);
const confirmAction = ref<"reset" | "delete" | null>(null);

function overrideCount(project: Project): number {
  return Object.keys(project.rules).length;
}

function enabledOverrides(project: Project): number {
  return Object.values(project.rules).filter((rule) => rule === "enable").length;
}

function disabledOverrides(project: Project): number {
  return Object.values(project.rules).filter((rule) => rule === "disable").length;
}

function matchesFilter(project: Project): boolean {
  if (filter.value === "current") return props.snapshot.state.currentProjectId === project.id;
  if (filter.value === "overrides") return overrideCount(project) > 0;
  if (filter.value === "empty") return overrideCount(project) === 0;
  return true;
}

const projects = computed(() => {
  const normalized = query.value.trim().toLowerCase();
  return [...props.snapshot.state.projects]
    .filter((project) => {
      if (!matchesFilter(project)) return false;
      if (!normalized) return true;
      return `${project.name} ${project.path}`.toLowerCase().includes(normalized);
    })
    .sort((left, right) => {
      const currentBias =
        Number(props.snapshot.state.currentProjectId === right.id) - Number(props.snapshot.state.currentProjectId === left.id);
      if (currentBias) return currentBias;
      const overrideBias = overrideCount(right) - overrideCount(left);
      return overrideBias || left.name.localeCompare(right.name, "zh-CN");
    });
});

const selectedProject = computed(
  () => projects.value.find((project) => project.id === props.selectedProjectId) ?? projects.value[0] ?? null,
);

const allSkills = computed(() => [...props.snapshot.state.skills].sort((left, right) => left.name.localeCompare(right.name, "zh-CN")));

const overrideRows = computed(() => {
  if (!selectedProject.value) return [];
  return Object.entries(selectedProject.value.rules)
    .map(([skillId, rule]) => {
      const skill = props.snapshot.state.skills.find((item) => item.id === skillId);
      if (!skill) return null;
      return { skill, rule };
    })
    .filter((item): item is { skill: Skill; rule: ProjectRule } => Boolean(item))
    .sort((left, right) => left.skill.name.localeCompare(right.skill.name, "zh-CN"));
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

function setRule(skillId: string, rule: ProjectRule) {
  if (!selectedProject.value) return;
  return run(() => api.setProjectRule({ projectId: selectedProject.value!.id, skillId, rule }));
}

async function confirmCurrentAction() {
  if (!selectedProject.value || !confirmAction.value) return;
  if (confirmAction.value === "reset") {
    await run(() => api.resetProjectRules(selectedProject.value!.id));
  } else {
    await run(() => api.deleteProject(selectedProject.value!.id));
  }
  confirmAction.value = null;
}

function projectTags(project: Project) {
  const tags = [];
  if (props.snapshot.state.currentProjectId === project.id) {
    tags.push({ label: "当前项目", tone: "brand" });
  }
  tags.push({
    label: overrideCount(project) ? `${overrideCount(project)} 条覆盖` : "无覆盖",
    tone: overrideCount(project) ? "neutral" : "muted",
  });
  return tags;
}
</script>

<template>
  <div class="split-content">
    <section class="list-panel">
      <div class="panel-header">
        <div>
          <p class="eyebrow">Projects</p>
          <h1 class="panel-title">项目规则</h1>
        </div>
        <span class="panel-count">{{ projects.length }}</span>
      </div>

      <div class="toolbar toolbar--stack">
        <input v-model="query" class="search-input" placeholder="搜索项目名称或路径" />

        <div class="toolbar-row">
          <div class="filter-group">
            <button class="filter-chip" :class="{ active: filter === 'all' }" @click="filter = 'all'">全部项目</button>
            <button class="filter-chip" :class="{ active: filter === 'current' }" @click="filter = 'current'">
              当前上下文
            </button>
            <button class="filter-chip" :class="{ active: filter === 'overrides' }" @click="filter = 'overrides'">
              有覆盖规则
            </button>
            <button class="filter-chip" :class="{ active: filter === 'empty' }" @click="filter = 'empty'">无覆盖</button>
          </div>
        </div>

        <div class="toolbar-row">
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
            <span
              v-for="tag in projectTags(project)"
              :key="`${project.id}-${tag.label}`"
              class="status-tag"
              :class="`status-tag--${tag.tone}`"
            >
              {{ tag.label }}
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
            <p class="eyebrow">Project Detail</p>
            <h2>{{ selectedProject.name }}</h2>
            <p>{{ selectedProject.path }}</p>
          </div>
          <div class="tag-row">
            <span
              v-for="tag in projectTags(selectedProject)"
              :key="`detail-${tag.label}`"
              class="status-tag"
              :class="`status-tag--${tag.tone}`"
            >
              {{ tag.label }}
            </span>
          </div>
        </div>

        <section class="detail-section">
          <div class="section-heading">
            <h3>概览</h3>
          </div>
          <dl class="detail-kv">
            <div>
              <dt>项目名称</dt>
              <dd>{{ selectedProject.name }}</dd>
            </div>
            <div>
              <dt>当前上下文</dt>
              <dd>{{ snapshot.state.currentProjectId === selectedProject.id ? "已设为当前项目" : "未设为当前项目" }}</dd>
            </div>
            <div>
              <dt>覆盖规则</dt>
              <dd>{{ overrideCount(selectedProject) }} 条</dd>
            </div>
          </dl>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>上下文控制</h3>
          </div>
          <div class="button-row">
            <button class="primary-button" :disabled="busy" @click="run(() => api.setCurrentProject(selectedProject!.id))">
              设为当前项目
            </button>
            <button class="secondary-button" :disabled="busy" @click="run(() => api.setCurrentProject(null))">
              清除当前项目
            </button>
          </div>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>规则摘要</h3>
          </div>
          <div class="stat-grid">
            <div class="stat-card">
              <strong>{{ overrideCount(selectedProject) }}</strong>
              <span>覆盖项</span>
            </div>
            <div class="stat-card">
              <strong>{{ enabledOverrides(selectedProject) }}</strong>
              <span>在此项目启用</span>
            </div>
            <div class="stat-card">
              <strong>{{ disabledOverrides(selectedProject) }}</strong>
              <span>在此项目停用</span>
            </div>
          </div>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>已覆盖规则</h3>
          </div>
          <div v-if="overrideRows.length" class="rule-stack">
            <div v-for="item in overrideRows" :key="item.skill.id" class="rule-card">
              <div class="rule-copy">
                <strong>{{ item.skill.name }}</strong>
                <small>{{ ruleLabel(item.rule) }}</small>
              </div>
              <div class="segmented-control">
                <button :class="{ active: item.rule === 'inherit' }" :disabled="busy" @click="setRule(item.skill.id, 'inherit')">
                  跟随默认
                </button>
                <button :class="{ active: item.rule === 'enable' }" :disabled="busy" @click="setRule(item.skill.id, 'enable')">
                  启用
                </button>
                <button :class="{ active: item.rule === 'disable' }" :disabled="busy" @click="setRule(item.skill.id, 'disable')">
                  停用
                </button>
              </div>
            </div>
          </div>
          <div v-else class="content-empty content-empty--compact">这个项目还没有覆盖规则。</div>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>全部规则</h3>
          </div>
          <details class="expander">
            <summary>展开全部规则</summary>
            <div v-if="allSkills.length" class="rule-stack">
              <div v-for="skill in allSkills" :key="skill.id" class="rule-card">
                <div class="rule-copy">
                  <strong>{{ skill.name }}</strong>
                  <small>{{ ruleLabel(selectedProject.rules[skill.id]) }} · {{ skill.defaultEnabled ? "默认启用" : "默认停用" }}</small>
                </div>
                <div class="segmented-control">
                  <button
                    :class="{ active: (selectedProject.rules[skill.id] ?? 'inherit') === 'inherit' }"
                    :disabled="busy"
                    @click="setRule(skill.id, 'inherit')"
                  >
                    跟随默认
                  </button>
                  <button
                    :class="{ active: (selectedProject.rules[skill.id] ?? 'inherit') === 'enable' }"
                    :disabled="busy"
                    @click="setRule(skill.id, 'enable')"
                  >
                    启用
                  </button>
                  <button
                    :class="{ active: (selectedProject.rules[skill.id] ?? 'inherit') === 'disable' }"
                    :disabled="busy"
                    @click="setRule(skill.id, 'disable')"
                  >
                    停用
                  </button>
                </div>
              </div>
            </div>
            <div v-else class="content-empty content-empty--compact">当前没有可配置的 skill。</div>
          </details>
        </section>

        <section class="detail-section detail-section--danger">
          <div class="section-heading">
            <h3>低频操作</h3>
          </div>
          <div class="button-row">
            <button class="secondary-button" :disabled="busy || !overrideCount(selectedProject)" @click="confirmAction = 'reset'">
              <RotateCcw :size="16" />
              重置全部覆盖规则
            </button>
            <button class="danger-button" :disabled="busy" @click="confirmAction = 'delete'">
              <Trash2 :size="16" />
              删除项目
            </button>
          </div>
        </section>
      </template>

      <div v-else class="content-empty">选择左侧项目查看规则详情。</div>
    </section>
  </div>

  <div v-if="confirmAction && selectedProject" class="modal-backdrop" @click.self="confirmAction = null">
    <section class="modal-card">
      <div class="detail-header">
        <div>
          <p class="eyebrow">{{ confirmAction === "reset" ? "Reset Rules" : "Delete Project" }}</p>
          <h2>{{ confirmAction === "reset" ? "重置覆盖规则" : `删除 ${selectedProject.name}` }}</h2>
          <p>
            {{
              confirmAction === "reset" ? "这会把所有项目规则改回跟随默认。" : "这会移除项目记录。"
            }}
          </p>
        </div>
      </div>

      <dl class="detail-kv">
        <div>
          <dt>项目路径</dt>
          <dd>{{ selectedProject.path }}</dd>
        </div>
        <div>
          <dt>覆盖规则</dt>
          <dd>{{ overrideCount(selectedProject) }} 条</dd>
        </div>
      </dl>

      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="confirmAction = null">取消</button>
        <button
          :class="confirmAction === 'reset' ? 'secondary-button' : 'danger-button'"
          :disabled="busy"
          @click="confirmCurrentAction"
        >
          {{ confirmAction === "reset" ? "确认重置" : "确认删除" }}
        </button>
      </div>
    </section>
  </div>
</template>

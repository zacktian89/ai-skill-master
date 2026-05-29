<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderPlus, RefreshCw, Trash2 } from "lucide-vue-next";
import * as api from "../api";
import { openDirectory } from "../dialog";
import type { AppSnapshot, DeleteSkillPreview, PendingSyncAction, Skill } from "../types";

type SkillFilter = "all" | "enabled" | "pending" | "issues" | "disabled";
type SkillSort = "name" | "status" | "sync";

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedSkillId: string | null;
}>();

const emit = defineEmits<{
  "select-skill": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const query = ref("");
const filter = ref<SkillFilter>("all");
const sort = ref<SkillSort>("status");
const busy = ref(false);
const deletePreview = ref<DeleteSkillPreview | null>(null);
const deleteDialogOpen = ref(false);

function actionsForSkill(skillId: string): PendingSyncAction[] {
  return props.snapshot.state.syncStatus.pendingActions.filter((item) => item.skillId === skillId);
}

const currentProject = computed(
  () =>
    props.snapshot.state.projects.find((project) => project.id === props.snapshot.state.currentProjectId) ?? null,
);

function effectiveEnabled(skill: Skill): boolean {
  const rule = currentProject.value?.rules[skill.id];
  if (rule === "enable") return true;
  if (rule === "disable") return false;
  return skill.defaultEnabled;
}

function hasBlockingIssue(skill: Skill): boolean {
  return Boolean(skill.conflict) || actionsForSkill(skill.id).some((item) => item.kind === "inspect");
}

function hasPendingApply(skill: Skill): boolean {
  if (hasBlockingIssue(skill)) return false;
  const actions = actionsForSkill(skill.id);
  if (actions.some((item) => item.kind === "create" || item.kind === "remove")) return true;
  return effectiveEnabled(skill) && !skill.managedLinks.codex;
}

function matchesFilter(skill: Skill): boolean {
  if (filter.value === "enabled") return effectiveEnabled(skill);
  if (filter.value === "disabled") return !effectiveEnabled(skill);
  if (filter.value === "issues") return hasBlockingIssue(skill);
  if (filter.value === "pending") return hasPendingApply(skill);
  return true;
}

function statusWeight(skill: Skill): number {
  if (hasBlockingIssue(skill)) return 0;
  if (hasPendingApply(skill)) return 1;
  if (effectiveEnabled(skill)) return 2;
  return 3;
}

const globalApplyState = computed(() => {
  const issueCount = props.snapshot.diagnostics.filter((item) => item.level === "error").length;
  const pendingCount = props.snapshot.state.syncStatus.pendingActions.filter((item) => item.kind !== "inspect").length;

  if (issueCount) {
    return {
      tone: "danger",
      label: "需处理",
      message: `有 ${issueCount} 个问题需要处理，处理后再应用到 Codex。`,
    };
  }

  if (pendingCount) {
    return {
      tone: "warning",
      label: "待应用",
      message: `有 ${pendingCount} 项改动待应用到 Codex。`,
    };
  }

  return {
    tone: props.snapshot.codexConnected ? "success" : "neutral",
    label: props.snapshot.codexConnected ? "已应用" : "未连接",
    message: props.snapshot.codexConnected ? "当前配置已应用到 Codex。" : "请先连接 Codex 目录。",
  };
});

const skills = computed(() => {
  const normalized = query.value.trim().toLowerCase();
  return [...props.snapshot.state.skills]
    .filter((skill) => {
      if (!matchesFilter(skill)) return false;
      if (!normalized) return true;
      return `${skill.name} ${skill.description} ${skill.id}`.toLowerCase().includes(normalized);
    })
    .sort((left, right) => {
      if (sort.value === "name") return left.name.localeCompare(right.name, "zh-CN");
      if (sort.value === "sync") return Number(hasPendingApply(right)) - Number(hasPendingApply(left));
      const diff = statusWeight(left) - statusWeight(right);
      return diff || left.name.localeCompare(right.name, "zh-CN");
    });
});

const selectedSkill = computed(
  () => skills.value.find((skill) => skill.id === props.selectedSkillId) ?? skills.value[0] ?? null,
);

const selectedIssues = computed(() => {
  if (!selectedSkill.value) return [];
  const issues = actionsForSkill(selectedSkill.value.id)
    .filter((item) => item.kind === "inspect")
    .map((item) => ({
      key: `${item.kind}-${item.target}-${item.message}`,
      title: "需要处理",
      detail: `${item.message} · ${item.target}`,
    }));
  if (selectedSkill.value.conflict) {
    issues.unshift({
      key: `conflict-${selectedSkill.value.conflict.path}`,
      title: "内容冲突",
      detail: `${selectedSkill.value.conflict.message} · ${selectedSkill.value.conflict.path}`,
    });
  }
  return issues;
});

function applyState(skill: Skill) {
  if (hasBlockingIssue(skill)) return { label: "需处理", tone: "danger" };
  if (hasPendingApply(skill)) return { label: "待应用", tone: "warning" };
  if (props.snapshot.codexConnected) return { label: "已应用", tone: "success" };
  return { label: "未连接", tone: "neutral" };
}

function skillTags(skill: Skill) {
  const tags = [
    {
      label: effectiveEnabled(skill) ? "当前启用" : "当前停用",
      tone: effectiveEnabled(skill) ? "brand" : "neutral",
    },
  ];
  tags.push(applyState(skill));
  return tags;
}

function primarySkillState(skill: Skill) {
  if (hasBlockingIssue(skill)) return { label: "需处理", tone: "danger" };
  return effectiveEnabled(skill)
    ? { label: "启用中", tone: "brand" }
    : { label: "已停用", tone: "neutral" };
}

function listSummary(skill: Skill): string {
  const summary = [skill.defaultEnabled ? "默认启用" : "默认停用"];

  if (hasBlockingIssue(skill)) {
    summary.push("处理问题后才能应用");
  } else if (hasPendingApply(skill)) {
    summary.push("有改动待应用");
  } else if (effectiveEnabled(skill)) {
    summary.push("当前会生效");
  } else {
    summary.push("当前不会生效");
  }

  return summary.join(" · ");
}

function syncSummary(skill: Skill): string {
  if (hasBlockingIssue(skill)) return "存在问题，处理后再应用到 Codex。";
  if (hasPendingApply(skill)) return "有改动待应用到 Codex。";
  if (!props.snapshot.codexConnected) return "请先连接 Codex 目录。";
  return "当前配置已应用到 Codex。";
}

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

async function importSkill() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await run(() => api.importSkill(selected));
    }
  } catch (cause) {
    emit("error", String(cause));
  }
}

async function openDeleteDialog() {
  if (!selectedSkill.value) return;
  busy.value = true;
  try {
    deletePreview.value = await api.previewDeleteSkill(selectedSkill.value.id);
    deleteDialogOpen.value = true;
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

async function confirmDelete() {
  if (!deletePreview.value) return;
  await run(() => api.deleteSkill(deletePreview.value!.skillId));
  deleteDialogOpen.value = false;
  deletePreview.value = null;
}

function closeDeleteDialog() {
  deleteDialogOpen.value = false;
  deletePreview.value = null;
}
</script>

<template>
  <div class="split-content">
    <section class="list-panel">
      <div class="panel-header">
        <div>
          <p class="eyebrow">Skills</p>
          <h1 class="panel-title">技能库</h1>
        </div>
        <span class="panel-count">{{ skills.length }}</span>
      </div>

      <div class="toolbar toolbar--stack">
        <input v-model="query" class="search-input" placeholder="搜索名称、ID 或描述" />

        <div class="toolbar-row">
          <div class="filter-group">
            <button class="filter-chip" :class="{ active: filter === 'all' }" @click="filter = 'all'">全部</button>
            <button class="filter-chip" :class="{ active: filter === 'enabled' }" @click="filter = 'enabled'">
              启用中
            </button>
            <button class="filter-chip" :class="{ active: filter === 'pending' }" @click="filter = 'pending'">
              待应用
            </button>
            <button class="filter-chip" :class="{ active: filter === 'issues' }" @click="filter = 'issues'">
              需处理
            </button>
            <button class="filter-chip" :class="{ active: filter === 'disabled' }" @click="filter = 'disabled'">
              已停用
            </button>
          </div>

          <select v-model="sort" class="select-input">
            <option value="status">状态优先</option>
            <option value="name">名称</option>
            <option value="sync">同步优先</option>
          </select>
        </div>

        <div class="toolbar-row">
          <button class="primary-button" :disabled="busy" @click="importSkill">
            <FolderPlus :size="16" />
            导入 Skill
          </button>
          <button class="secondary-button" :disabled="busy" @click="run(api.syncCodex)">
            <RefreshCw :size="16" />
            应用到 Codex
          </button>
        </div>

        <div class="inline-panel" :class="`inline-panel--${globalApplyState.tone}`">
          <strong>{{ globalApplyState.label }}</strong>
          <span>{{ globalApplyState.message }}</span>
        </div>
      </div>

      <div v-if="skills.length" class="list-stack">
        <button
          v-for="skill in skills"
          :key="skill.id"
          class="list-row"
          :class="{ active: selectedSkill?.id === skill.id }"
          @click="emit('select-skill', skill.id)"
        >
          <div class="list-row-main">
            <div class="list-row-top">
              <strong>{{ skill.name }}</strong>
              <span class="status-tag list-row-state" :class="`status-tag--${primarySkillState(skill).tone}`">
                {{ primarySkillState(skill).label }}
              </span>
            </div>
            <div class="list-row-bottom">
              <code class="skill-id-badge">{{ skill.id }}</code>
              <span class="list-row-summary">{{ listSummary(skill) }}</span>
            </div>
          </div>
        </button>
      </div>

      <div v-else class="content-empty">没有匹配的 skill。</div>
    </section>

    <section class="detail-panel">
      <template v-if="selectedSkill">
        <div class="detail-header">
          <div>
            <p class="eyebrow">Skill Detail</p>
            <h2>{{ selectedSkill.name }}</h2>
            <p>{{ selectedSkill.description || "没有描述，使用 ID 作为识别信息。" }}</p>
          </div>
          <div class="tag-row">
            <span
              v-for="tag in skillTags(selectedSkill)"
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
              <dt>Skill ID</dt>
              <dd>{{ selectedSkill.id }}</dd>
            </div>
            <div>
              <dt>当前状态</dt>
              <dd>{{ primarySkillState(selectedSkill).label }}</dd>
            </div>
            <div>
              <dt>应用状态</dt>
              <dd>{{ applyState(selectedSkill).label }}</dd>
            </div>
            <div>
              <dt>默认规则</dt>
              <dd>{{ selectedSkill.defaultEnabled ? "默认启用" : "默认停用" }}</dd>
            </div>
          </dl>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>规则与应用</h3>
          </div>
          <div class="action-card">
            <label class="toggle-row">
              <span>
                <strong>默认启用</strong>
                <small>关闭后仅在项目覆盖时启用</small>
              </span>
              <input
                type="checkbox"
                :checked="selectedSkill.defaultEnabled"
                :disabled="busy"
                @change="run(() => api.setDefaultEnabled(selectedSkill!.id, ($event.target as HTMLInputElement).checked))"
              />
            </label>
            <div class="button-row">
              <button class="secondary-button" :disabled="busy" @click="run(api.syncCodex)">
                <RefreshCw :size="16" />
                应用到 Codex
              </button>
            </div>
            <p class="inline-copy">{{ syncSummary(selectedSkill) }}</p>
          </div>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>高级信息</h3>
          </div>
          <dl class="detail-kv detail-kv--wide">
            <div>
              <dt>技能库路径</dt>
              <dd>{{ selectedSkill.libraryPath }}</dd>
            </div>
            <div>
              <dt>Codex 目标</dt>
              <dd>{{ selectedSkill.managedLinks.codex || "当前还没有应用目标" }}</dd>
            </div>
          </dl>
        </section>

        <section v-if="selectedIssues.length" class="detail-section detail-section--danger">
          <div class="section-heading">
            <h3>问题与修复</h3>
          </div>
          <div class="issue-list">
            <div v-for="issue in selectedIssues" :key="issue.key" class="issue-card">
              <strong>{{ issue.title }}</strong>
              <p>{{ issue.detail }}</p>
            </div>
          </div>
        </section>

        <section class="detail-section detail-section--danger">
          <div class="section-heading">
            <h3>危险操作</h3>
          </div>
          <button class="danger-button" :disabled="busy" @click="openDeleteDialog">
            <Trash2 :size="16" />
            删除 Skill
          </button>
        </section>
      </template>

      <div v-else class="content-empty">选择左侧 skill 查看详情。</div>
    </section>
  </div>

  <div v-if="deleteDialogOpen && deletePreview" class="modal-backdrop" @click.self="closeDeleteDialog">
    <section class="modal-card">
      <div class="detail-header">
        <div>
          <p class="eyebrow">Confirm Delete</p>
          <h2>删除 {{ deletePreview.skillName }}</h2>
          <p>确认前不会修改任何文件。</p>
        </div>
      </div>

      <dl class="detail-kv detail-kv--wide">
        <div>
          <dt>技能库目录</dt>
          <dd>{{ deletePreview.libraryPath }}</dd>
        </div>
        <div>
          <dt>托管链接</dt>
          <dd>
            <template v-if="deletePreview.managedLinkTargets.length">
              <div class="meta-stack">
                <span v-for="target in deletePreview.managedLinkTargets" :key="target">{{ target }}</span>
              </div>
            </template>
            <template v-else>无</template>
          </dd>
        </div>
        <div>
          <dt>项目规则影响</dt>
          <dd>
            <template v-if="deletePreview.affectedProjects.length">
              <div class="meta-stack">
                <span v-for="project in deletePreview.affectedProjects" :key="project.projectId">
                  {{ project.projectName }} · {{ project.projectPath }}
                </span>
              </div>
            </template>
            <template v-else>无</template>
          </dd>
        </div>
      </dl>

      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="closeDeleteDialog">取消</button>
        <button class="danger-button" :disabled="busy" @click="confirmDelete">
          <Trash2 :size="16" />
          确认删除
        </button>
      </div>
    </section>
  </div>
</template>

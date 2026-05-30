<script setup lang="ts">
import { computed, ref } from "vue";
import { CircleHelp, Folder, Github, Plus, RefreshCw, ShoppingBag, Trash2 } from "lucide-vue-next";
import * as api from "../api";
import { openDirectory } from "../dialog";
import type { AppSnapshot, DeleteSkillPreview, PendingSyncAction, Skill, SkillSourceKind } from "../types";

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
const busy = ref(false);
const deletePreview = ref<DeleteSkillPreview | null>(null);
const deleteDialogOpen = ref(false);

const sourceIcons = {
  local: Folder,
  github: Github,
  openclawMarket: ShoppingBag,
  unknown: CircleHelp,
} satisfies Record<SkillSourceKind, unknown>;

const sourceLabels = {
  local: "本地",
  github: "GitHub",
  openclawMarket: "OpenClaw Market",
  unknown: "未知来源",
} satisfies Record<SkillSourceKind, string>;

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
  return linkEnabled(skill);
}

function hasBlockingIssue(skill: Skill): boolean {
  return Boolean(skill.conflict) || actionsForSkill(skill.id).some((item) => item.kind === "inspect");
}

function hasPendingApply(skill: Skill): boolean {
  if (hasBlockingIssue(skill)) return false;
  const actions = actionsForSkill(skill.id);
  return actions.some((item) => item.kind === "create" || item.kind === "remove");
}

function isReferenced(skill: Skill): boolean {
  if (linkEnabled(skill)) return true;
  return props.snapshot.state.projects.some((project) => project.rules[skill.id] === "enable");
}

function linkEnabled(skill: Skill): boolean {
  return Boolean(skill.managedLinks.codex);
}

function sourceKind(skill: Skill): SkillSourceKind {
  return skill.source?.kind ?? "local";
}

function sourceLabel(skill: Skill): string {
  const kind = sourceKind(skill);
  return skill.source?.label || sourceLabels[kind];
}

const skills = computed(() => {
  const normalized = query.value.trim().toLowerCase();
  return [...props.snapshot.state.skills]
    .filter((skill) => {
      if (!normalized) return true;
      return `${skill.name} ${skill.description} ${skill.id}`.toLowerCase().includes(normalized);
    })
    .sort((left, right) => left.name.localeCompare(right.name, "zh-CN"));
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
  return linkEnabled(skill)
    ? { label: "已链接", tone: "success" }
    : { label: "未链接", tone: "neutral" };
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

function syncSummary(skill: Skill): string {
  if (hasBlockingIssue(skill)) return "存在问题，处理后再应用到 Codex。";
  if (hasPendingApply(skill)) return "有改动待应用到 Codex。";
  return linkEnabled(skill) ? "Codex 中存在托管链接。" : "Codex 中没有托管链接。";
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
        <div class="list-search-row">
          <input v-model="query" class="search-input" placeholder="搜索已安装 Skill" />
          <button class="icon-button" type="button" :disabled="busy" aria-label="按文件夹添加 Skill" @click="importSkill">
            <Plus :size="18" />
          </button>
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
              <span
                class="reference-dot"
                :class="isReferenced(skill) ? 'reference-dot--active' : 'reference-dot--idle'"
                :aria-label="isReferenced(skill) ? '已引用' : '未引用'"
                :title="isReferenced(skill) ? '已引用' : '未引用'"
                role="img"
              ></span>
            </div>
            <div class="list-row-bottom">
              <span class="source-icon" :title="sourceLabel(skill)" :aria-label="sourceLabel(skill)" role="img">
                <component :is="sourceIcons[sourceKind(skill)]" :size="13" />
              </span>
              <code class="skill-id-badge">{{ skill.id }}</code>
            </div>
          </div>
        </button>
      </div>

      <div v-else class="content-empty">没有匹配的 skill。</div>
    </section>

    <section class="detail-panel">
      <template v-if="selectedSkill">
        <div class="extension-detail">
          <header class="extension-header">
            <div class="extension-identity">
              <div class="extension-icon" :title="sourceLabel(selectedSkill)">
                <component :is="sourceIcons[sourceKind(selectedSkill)]" :size="28" />
              </div>
              <div class="extension-title-group">
                <p class="eyebrow">Skill Detail</p>
                <h2>{{ selectedSkill.name }}</h2>
                <div class="extension-meta">
                  <code>{{ selectedSkill.id }}</code>
                  <span>{{ sourceLabel(selectedSkill) }}</span>
                  <span>{{ syncSummary(selectedSkill) }}</span>
                </div>
              </div>
            </div>

            <div class="extension-command-panel">
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
              <div class="extension-actions">
                <label class="switch-control">
                  <span>启用</span>
                  <input
                    type="checkbox"
                    :checked="linkEnabled(selectedSkill)"
                    :disabled="busy"
                    @change="run(() => api.setSkillLinkEnabled(selectedSkill!.id, ($event.target as HTMLInputElement).checked))"
                  />
                </label>
                <button class="secondary-button" :disabled="busy" @click="run(api.syncCodex)">
                  <RefreshCw :size="16" />
                  同步
                </button>
                <button class="danger-button danger-button--icon" :disabled="busy" aria-label="删除 Skill" @click="openDeleteDialog">
                  <Trash2 :size="16" />
                </button>
              </div>
            </div>
          </header>

          <div v-if="selectedIssues.length" class="issue-strip">
            <div v-for="issue in selectedIssues" :key="issue.key">
              <strong>{{ issue.title }}</strong>
              <span>{{ issue.detail }}</span>
            </div>
          </div>

          <nav class="detail-tabs" aria-label="Skill detail tabs">
            <button class="detail-tab active" type="button">Description</button>
          </nav>

          <section class="description-pane">
            <p v-if="selectedSkill.description">{{ selectedSkill.description }}</p>
            <p v-else class="description-empty">暂无描述。</p>
          </section>
        </div>
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

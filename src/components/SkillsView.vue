<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderPlus, RefreshCw, Trash2 } from "lucide-vue-next";
import * as api from "../api";
import { openDirectory } from "../dialog";
import type { AppSnapshot, DeleteSkillPreview, PendingSyncAction, Skill } from "../types";

type SkillFilter = "all" | "enabled" | "conflict" | "unsynced" | "disabled";
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

function hasPendingSync(skill: Skill): boolean {
  return actionsForSkill(skill.id).length > 0 || !skill.managedLinks.codex;
}

function matchesFilter(skill: Skill): boolean {
  if (filter.value === "enabled") return skill.defaultEnabled;
  if (filter.value === "disabled") return !skill.defaultEnabled;
  if (filter.value === "conflict") return Boolean(skill.conflict);
  if (filter.value === "unsynced") return hasPendingSync(skill);
  return true;
}

function statusWeight(skill: Skill): number {
  if (skill.conflict) return 0;
  if (hasPendingSync(skill)) return 1;
  if (skill.defaultEnabled) return 2;
  return 3;
}

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
      if (sort.value === "sync") return Number(hasPendingSync(left)) - Number(hasPendingSync(right));
      const diff = statusWeight(left) - statusWeight(right);
      return diff || left.name.localeCompare(right.name, "zh-CN");
    });
});

const selectedSkill = computed(
  () => skills.value.find((skill) => skill.id === props.selectedSkillId) ?? skills.value[0] ?? null,
);

const selectedIssues = computed(() => {
  if (!selectedSkill.value) return [];
  const issues = actionsForSkill(selectedSkill.value.id).map((item) => ({
    key: `${item.kind}-${item.target}-${item.message}`,
    title: item.kind === "remove" ? "待移除托管链接" : item.kind === "inspect" ? "需要人工检查" : "待创建托管链接",
    detail: `${item.message} · ${item.target}`,
  }));
  if (selectedSkill.value.conflict) {
    issues.unshift({
      key: `conflict-${selectedSkill.value.conflict.path}`,
      title: "存在冲突",
      detail: `${selectedSkill.value.conflict.message} · ${selectedSkill.value.conflict.path}`,
    });
  }
  if (!selectedSkill.value.managedLinks.codex) {
    issues.push({
      key: "missing-link",
      title: "尚未同步到 Codex",
      detail: "当前 skill 还没有托管链接。",
    });
  }
  return issues;
});

function skillTags(skill: Skill) {
  const tags = [
    {
      label: skill.defaultEnabled ? "默认启用" : "默认停用",
      tone: skill.defaultEnabled ? "brand" : "neutral",
    },
  ];

  if (skill.conflict) {
    tags.push({ label: "有冲突", tone: "danger" });
  } else if (hasPendingSync(skill)) {
    tags.push({ label: "待同步", tone: "warning" });
  } else {
    tags.push({ label: "已同步", tone: "success" });
  }

  return tags;
}

function syncSummary(skill: Skill): string {
  const actions = actionsForSkill(skill.id);
  if (skill.conflict) return "存在冲突，处理后再同步。";
  if (actions.length) return actions[0]?.message ?? "有待处理同步操作。";
  if (!skill.managedLinks.codex) return "尚未建立 Codex 托管链接。";
  return props.snapshot.state.syncStatus.message || "已同步到 Codex。";
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
        <input v-model="query" class="search-input" placeholder="搜索 skill 名称、描述或 ID" />

        <div class="toolbar-row">
          <div class="filter-group">
            <button class="filter-chip" :class="{ active: filter === 'all' }" @click="filter = 'all'">全部</button>
            <button class="filter-chip" :class="{ active: filter === 'enabled' }" @click="filter = 'enabled'">
              已启用
            </button>
            <button class="filter-chip" :class="{ active: filter === 'conflict' }" @click="filter = 'conflict'">
              有冲突
            </button>
            <button class="filter-chip" :class="{ active: filter === 'unsynced' }" @click="filter = 'unsynced'">
              未同步
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
            同步
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
          <div class="list-row-copy">
            <strong>{{ skill.name }}</strong>
            <small>{{ skill.description || skill.id }}</small>
          </div>
          <div class="list-row-meta">
            <span
              v-for="tag in skillTags(skill)"
              :key="`${skill.id}-${tag.label}`"
              class="status-tag"
              :class="`status-tag--${tag.tone}`"
            >
              {{ tag.label }}
            </span>
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
              <dt>默认状态</dt>
              <dd>{{ selectedSkill.defaultEnabled ? "默认启用" : "默认停用" }}</dd>
            </div>
            <div>
              <dt>Codex</dt>
              <dd>{{ selectedSkill.managedLinks.codex ? "已托管" : "未同步" }}</dd>
            </div>
          </dl>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>启用与同步</h3>
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
                手动同步
              </button>
            </div>
            <p class="inline-copy">{{ syncSummary(selectedSkill) }}</p>
          </div>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>路径与来源</h3>
          </div>
          <dl class="detail-kv detail-kv--wide">
            <div>
              <dt>技能库路径</dt>
              <dd>{{ selectedSkill.libraryPath }}</dd>
            </div>
            <div>
              <dt>托管链接</dt>
              <dd>{{ selectedSkill.managedLinks.codex || "尚未创建 Codex 托管链接" }}</dd>
            </div>
          </dl>
        </section>

        <section v-if="selectedIssues.length" class="detail-section detail-section--danger">
          <div class="section-heading">
            <h3>异常与冲突</h3>
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

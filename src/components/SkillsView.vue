<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { marked } from "marked";
import {
  CircleHelp,
  Folder,
  Github,
  List,
  Network,
  Plus,
  ShoppingBag,
  Trash2,
  X,
} from "lucide-vue-next";
import * as api from "../api";
import AgentIcon from "./AgentIcon.vue";
import { openDirectory } from "../dialog";
import { useScrollableList } from "../useScrollableList";
import type {
  AddSkillReferenceRequest,
  AppSnapshot,
  DeleteSkillPreview,
  ImportSkillCandidate,
  ImportSkillSource,
  PendingSyncAction,
  ReferenceScope,
  ReferenceStatus,
  Skill,
  SkillSourceKind,
  SkillTargetProfile,
} from "../types";

type DetailTab = "references" | "description";
type ReferenceViewMode = "list" | "graph";
type ImportSourceMode = "local" | "github";

interface SkillReference {
  id: string;
  targetName: string;
  symlinkPath: string;
  scope: ReferenceScope;
  status: ReferenceStatus;
  removable: boolean;
  legacyCodex: boolean;
}

interface PendingReferenceTarget {
  targetName: string;
  rootPath: string;
  scope: ReferenceScope;
}

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
const importDialogOpen = ref(false);
const importSourceMode = ref<ImportSourceMode>("local");
const importLocalPath = ref("");
const importGithubUrl = ref("");
const importGithubRef = ref("");
const importGithubSubdir = ref("");
const importCandidates = ref<ImportSkillCandidate[]>([]);
const selectedImportCandidateIds = ref<string[]>([]);
const importScanned = ref(false);
const addReferenceDialogOpen = ref(false);
const pendingReferenceTarget = ref<PendingReferenceTarget | null>(null);
const overwriteReferenceRequest = ref<AddSkillReferenceRequest | null>(null);
const deleteReferenceDialogOpen = ref(false);
const referenceToDelete = ref<SkillReference | null>(null);
const removeReferenceConflictRequest = ref<{ referenceId: string; symlinkPath: string } | null>(null);
const activeDetailTab = ref<DetailTab>("description");
const referenceViewMode = ref<ReferenceViewMode>("list");

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



const scopeLabels = {
  user: "个人目录",
  project: "项目目录",
  custom: "自定义目录",
} satisfies Record<ReferenceScope, string>;

const referenceStatusLabels = {
  healthy: "正常",
  missing: "缺失",
  conflict: "冲突",
  stale: "失效",
} satisfies Record<ReferenceStatus, string>;

const importStatusLabels = {
  ready: "可导入",
  duplicate: "已存在",
  conflict: "冲突",
  invalid: "无效",
} satisfies Record<string, string>;

function actionsForSkill(skillId: string): PendingSyncAction[] {
  return props.snapshot.state.syncStatus.pendingActions.filter((item) => item.skillId === skillId);
}

function isReferenced(skill: Skill): boolean {
  if (linkEnabled(skill) || (skill.references?.length ?? 0) > 0) return true;
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

const { listStackRef, listStackScrollable } = useScrollableList(skills);

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

const selectedReferences = computed(() => {
  if (!selectedSkill.value) return [];
  return referencesForSkill(selectedSkill.value);
});

const targetProfiles = computed<SkillTargetProfile[]>(() => props.snapshot.targetProfiles ?? []);

const pendingReferencePath = computed(() => {
  if (!selectedSkill.value || !pendingReferenceTarget.value) return "";
  return joinPath(pendingReferenceTarget.value.rootPath, selectedSkill.value.id);
});

const addReferenceTitle = computed(() => {
  if (overwriteReferenceRequest.value) return "覆盖引用链接";
  return pendingReferenceTarget.value ? "确认新增引用" : "新增引用";
});

const importReadyCandidates = computed(() => importCandidates.value.filter((candidate) => candidate.status === "ready"));

const importSource = computed<ImportSkillSource | null>(() => {
  if (importSourceMode.value === "local") {
    const path = importLocalPath.value.trim();
    return path ? { kind: "local", path } : null;
  }
  const url = importGithubUrl.value.trim();
  if (!url) return null;
  return {
    kind: "github",
    url,
    ref: importGithubRef.value.trim() || null,
    subdir: importGithubSubdir.value.trim() || null,
  };
});

const canScanImports = computed(() => Boolean(importSource.value) && !busy.value);

const canConfirmImports = computed(() => selectedImportCandidateIds.value.length > 0 && !busy.value);

function referencesForSkill(skill: Skill): SkillReference[] {
  const actions = actionsForSkill(skill.id);
  const inspectAction = actions.find((item) => item.kind === "inspect");
  const createAction = actions.find((item) => item.kind === "create");

  if (skill.conflict) {
    return [
      {
        id: `codex-conflict-${skill.conflict.path}`,
        targetName: "Codex",
        symlinkPath: skill.conflict.path,
        scope: "user",
        status: "conflict",
        removable: false,
        legacyCodex: true,
      },
    ];
  }

  const references = (skill.references ?? []).map((reference) => ({
    id: reference.id,
    targetName: reference.targetName,
    symlinkPath: reference.targetPath,
    scope: reference.scope,
    status: reference.status,
    removable: true,
    legacyCodex: false,
  }));

  if (inspectAction) {
    return references.concat([
      {
        id: `codex-inspect-${inspectAction.target}`,
        targetName: "Codex",
        symlinkPath: inspectAction.target,
        scope: "user",
        status: "conflict",
        removable: false,
        legacyCodex: true,
      },
    ]);
  }

  if (skill.managedLinks.codex) {
    return references.concat([
      {
        id: `codex-${skill.managedLinks.codex}`,
        targetName: "Codex",
        symlinkPath: skill.managedLinks.codex,
        scope: "user",
        status: "healthy",
        removable: false,
        legacyCodex: true,
      },
    ]);
  }

  if (createAction) {
    return references.concat([
      {
        id: `codex-create-${createAction.target}`,
        targetName: "Codex",
        symlinkPath: createAction.target,
        scope: "user",
        status: "missing",
        removable: false,
        legacyCodex: true,
      },
    ]);
  }

  return references;
}



function joinPath(root: string, child: string): string {
  const normalized = root.replace(/[\\/]+$/, "");
  return `${normalized}/${child}`;
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

function resetImportPreview() {
  importCandidates.value = [];
  selectedImportCandidateIds.value = [];
  importScanned.value = false;
}

function openImportDialog() {
  importDialogOpen.value = true;
  resetImportPreview();
}

function closeImportDialog() {
  importDialogOpen.value = false;
  resetImportPreview();
}

function setImportSourceMode(mode: ImportSourceMode) {
  importSourceMode.value = mode;
  resetImportPreview();
}

async function selectImportLocalPath() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      importLocalPath.value = selected;
      resetImportPreview();
    }
  } catch (cause) {
    emit("error", String(cause));
  }
}

async function scanImportSource() {
  if (!importSource.value) return;
  busy.value = true;
  try {
    const preview = await api.previewImportSkills(importSource.value);
    importCandidates.value = preview.candidates;
    selectedImportCandidateIds.value = preview.candidates
      .filter((candidate) => candidate.status === "ready")
      .map((candidate) => candidate.candidateId);
    importScanned.value = true;
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

function toggleImportCandidate(candidate: ImportSkillCandidate, checked: boolean) {
  if (candidate.status !== "ready") return;
  const next = new Set(selectedImportCandidateIds.value);
  if (checked) {
    next.add(candidate.candidateId);
  } else {
    next.delete(candidate.candidateId);
  }
  selectedImportCandidateIds.value = [...next];
}

function importCandidateChecked(candidateId: string): boolean {
  return selectedImportCandidateIds.value.includes(candidateId);
}

function toggleAllImportCandidates(checked: boolean) {
  selectedImportCandidateIds.value = checked
    ? importReadyCandidates.value.map((candidate) => candidate.candidateId)
    : [];
}

async function confirmImportSkills() {
  if (!importSource.value || !selectedImportCandidateIds.value.length) return;
  await run(() =>
    api.confirmImportSkills({
      source: importSource.value!,
      candidateIds: selectedImportCandidateIds.value,
    }),
  );
  closeImportDialog();
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

function openAddReferenceDialog() {
  pendingReferenceTarget.value = null;
  addReferenceDialogOpen.value = true;
}

function closeAddReferenceDialog() {
  addReferenceDialogOpen.value = false;
  pendingReferenceTarget.value = null;
  overwriteReferenceRequest.value = null;
}

function selectTargetProfile(profile: SkillTargetProfile) {
  overwriteReferenceRequest.value = null;
  pendingReferenceTarget.value = {
    targetName: profile.targetName,
    rootPath: profile.rootPath,
    scope: profile.scope,
  };
}

async function selectCustomReferenceRoot() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      overwriteReferenceRequest.value = null;
      pendingReferenceTarget.value = {
        targetName: "自定义目录",
        rootPath: selected,
        scope: "custom",
      };
    }
  } catch (cause) {
    emit("error", String(cause));
  }
}

async function confirmAddReference() {
  if (!selectedSkill.value || !pendingReferenceTarget.value) return;
  const request: AddSkillReferenceRequest = {
    skillId: selectedSkill.value.id,
    targetName: pendingReferenceTarget.value.targetName,
    rootPath: pendingReferenceTarget.value.rootPath,
    scope: pendingReferenceTarget.value.scope,
  };
  busy.value = true;
  try {
    emit("snapshot", await api.addSkillReference(request));
    closeAddReferenceDialog();
  } catch (cause) {
    if (!isRetargetedLinkError(cause)) {
      emit("error", String(cause));
      return;
    }
    overwriteReferenceRequest.value = request;
  } finally {
    busy.value = false;
  }
}

async function confirmOverwriteReference() {
  if (!overwriteReferenceRequest.value) return;
  busy.value = true;
  try {
    emit("snapshot", await api.addSkillReference({ ...overwriteReferenceRequest.value, overwrite: true }));
    closeAddReferenceDialog();
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

function cancelOverwriteReference() {
  overwriteReferenceRequest.value = null;
}

function isRetargetedLinkError(cause: unknown): boolean {
  return String(cause).includes("已指向其他位置");
}

function openDeleteReferenceDialog(reference: SkillReference) {
  referenceToDelete.value = reference;
  deleteReferenceDialogOpen.value = true;
}

function closeDeleteReferenceDialog() {
  deleteReferenceDialogOpen.value = false;
  referenceToDelete.value = null;
  removeReferenceConflictRequest.value = null;
}

async function confirmDeleteReference() {
  if (!referenceToDelete.value || !selectedSkill.value) return;
  const reference = referenceToDelete.value;
  busy.value = true;
  try {
    emit("snapshot", await api.removeSkillReference(reference.id));
    closeDeleteReferenceDialog();
  } catch (cause) {
    if (!isRetargetedLinkError(cause)) {
      emit("error", String(cause));
      return;
    }
    removeReferenceConflictRequest.value = {
      referenceId: reference.id,
      symlinkPath: reference.symlinkPath,
    };
  } finally {
    busy.value = false;
  }
}

async function confirmDeleteReferenceWithLink(removeExternalLink: boolean) {
  if (!removeReferenceConflictRequest.value) return;
  const { referenceId } = removeReferenceConflictRequest.value;
  busy.value = true;
  try {
    emit("snapshot", await api.removeSkillReference(referenceId, removeExternalLink));
    closeDeleteReferenceDialog();
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

function closeDeleteDialog() {
  deleteDialogOpen.value = false;
  deletePreview.value = null;
}

const skillMarkdown = ref("");
const isMarkdownLoading = ref(false);

async function loadSkillMarkdown() {
  if (!selectedSkill.value) {
    skillMarkdown.value = "";
    return;
  }
  isMarkdownLoading.value = true;
  try {
    const content = await api.readSkillFile(selectedSkill.value.id);
    skillMarkdown.value = content;
  } catch (err) {
    console.error("加载 SKILL.md 失败", err);
    skillMarkdown.value = "";
  } finally {
    isMarkdownLoading.value = false;
  }
}

watch([selectedSkill, activeDetailTab], async ([newSkill, newTab]) => {
  if (newSkill && newTab === "description") {
    await loadSkillMarkdown();
  }
}, { immediate: true });

function parseFrontMatter(content: string) {
  const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return { metadata: {} as Record<string, string>, body: content };
  const yamlStr = match[1];
  const body = content.slice(match[0].length).trim();
  const metadata: Record<string, string> = {};
  yamlStr.split(/\r?\n/).forEach(line => {
    const idx = line.indexOf(':');
    if (idx > -1) {
      const key = line.slice(0, idx).trim();
      const value = line.slice(idx + 1).trim().replace(/^['"]|['"]$/g, '');
      if (key) metadata[key] = value;
    }
  });
  return { metadata, body };
}

const parsedMarkdown = computed(() => {
  const { metadata, body } = parseFrontMatter(skillMarkdown.value);
  return { metadata, body };
});

const renderedMarkdown = computed(() => {
  if (!parsedMarkdown.value.body) return "";
  try {
    return marked.parse(parsedMarkdown.value.body) as string;
  } catch (err) {
    console.error("Markdown 解析失败:", err);
    return parsedMarkdown.value.body;
  }
});
</script>

<template>
  <div class="split-content">
    <section class="list-panel">
      <div class="list-panel-head">
        <div class="list-search-row" style="display: grid; grid-template-columns: auto minmax(0, 1fr) 30px; gap: 8px; align-items: center;">
          <span class="search-row-count" style="display: inline-flex; align-items: center; justify-content: center; width: 32px; height: 30px; font-family: ui-monospace, monospace; font-size: 12px; font-weight: 600; color: var(--text-secondary); background: var(--bg-input); border: 1px solid var(--border-default); border-radius: 6px; flex-shrink: 0;">{{ snapshot.state.skills.length }}</span>
          <input v-model="query" class="search-input" placeholder="搜索已安装 Skill" />
          <button class="icon-button" type="button" :disabled="busy" aria-label="新增 Skill" @click="openImportDialog">
            <Plus :size="18" />
          </button>
        </div>
      </div>

      <div v-if="skills.length" ref="listStackRef" class="list-stack" :class="{ 'list-stack--scrollable': listStackScrollable }">
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
                <h2>{{ selectedSkill.name }}</h2>
                <div class="extension-meta">
                  <code>{{ selectedSkill.id }}</code>
                  <span>{{ sourceLabel(selectedSkill) }}</span>
                </div>
              </div>
            </div>

            <div class="extension-command-panel">
              <div class="extension-actions">
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
            <button
              class="detail-tab"
              :class="{ active: activeDetailTab === 'description' }"
              type="button"
              @click="activeDetailTab = 'description'"
            >
              详情
            </button>
            <button
              class="detail-tab"
              :class="{ active: activeDetailTab === 'references' }"
              type="button"
              @click="activeDetailTab = 'references'"
            >
              引用
            </button>
          </nav>

          <section v-if="activeDetailTab === 'references'" class="reference-pane">
            <div class="reference-pane-header">
              <button
                class="icon-button icon-button--compact"
                type="button"
                :disabled="busy"
                aria-label="新增引用"
                title="新增引用"
                @click="openAddReferenceDialog"
              >
                <Plus :size="16" />
              </button>
              <div class="segmented-control segmented-control--compact" aria-label="引用视图切换">
                <button
                  type="button"
                  :class="{ active: referenceViewMode === 'list' }"
                  aria-label="列表视图"
                  title="列表视图"
                  @click="referenceViewMode = 'list'"
                >
                  <List :size="15" />
                </button>
                <button
                  type="button"
                  :class="{ active: referenceViewMode === 'graph' }"
                  aria-label="连线图"
                  title="连线图"
                  @click="referenceViewMode = 'graph'"
                >
                  <Network :size="15" />
                </button>
              </div>
            </div>

            <div v-if="selectedReferences.length && referenceViewMode === 'list'" class="reference-list">
              <article
                v-for="reference in selectedReferences"
                :key="reference.id"
                class="reference-row"
              >
                <div class="reference-row-main">
                  <div class="reference-row-top">
                    <div class="reference-title">
                      <span class="reference-app-icon" :title="reference.targetName" aria-hidden="true">
                        <AgentIcon :name="reference.targetName" :size="15" />
                      </span>
                      <strong>{{ reference.targetName }}</strong>
                    </div>
                    <div class="reference-actions">
                      <span class="status-tag" :class="`status-tag--${reference.status}`">
                        {{ referenceStatusLabels[reference.status] }}
                      </span>
                      <span class="status-tag">{{ scopeLabels[reference.scope] }}</span>
                      <button
                        v-if="reference.removable"
                        class="ghost-icon-button ghost-icon-button--danger"
                        type="button"
                        :disabled="busy"
                        aria-label="删除引用"
                        title="删除引用"
                        @click="openDeleteReferenceDialog(reference)"
                      >
                        <Trash2 :size="15" />
                      </button>
                    </div>
                  </div>
                  <code class="reference-path">{{ reference.symlinkPath }}</code>
                </div>
              </article>
            </div>

            <div v-else-if="selectedReferences.length" class="reference-graph" aria-label="Skill 引用连线图">
              <div class="reference-center-node">
                <span>当前 Skill</span>
                <strong>{{ selectedSkill.name }}</strong>
                <code>{{ selectedSkill.id }}</code>
              </div>
              <div class="reference-graph-stack">
                <article
                  v-for="reference in selectedReferences"
                  :key="reference.id"
                  class="reference-graph-item"
                >
                  <div class="reference-line" aria-hidden="true"></div>
                  <div class="reference-node">
                    <div>
                      <strong>{{ reference.targetName }}</strong>
                      <span>{{ referenceStatusLabels[reference.status] }}</span>
                    </div>
                    <small>{{ reference.symlinkPath }}</small>
                  </div>
                </article>
              </div>
            </div>

            <div v-else class="content-empty content-empty--compact">
              暂无引用。
            </div>
          </section>

          <section v-else class="description-pane">
            <div v-if="isMarkdownLoading" class="preview-loading">
              <span>加载中...</span>
            </div>
            <div v-else-if="skillMarkdown">
              <!-- Front matter metadata tags -->
              <div class="skill-meta-tags" v-if="Object.keys(parsedMarkdown.metadata).length">
                <div v-for="(val, key) in parsedMarkdown.metadata" :key="key" class="skill-meta-tag">
                  <span class="skill-meta-tag-key">{{ key }}</span>:
                  <span class="skill-meta-tag-val">{{ val }}</span>
                </div>
              </div>

              <!-- Markdown Body -->
              <div class="markdown-body" v-html="renderedMarkdown"></div>
            </div>
            <p v-else class="description-empty">暂无描述。</p>
          </section>
        </div>
      </template>

      <div v-else class="content-empty">选择左侧 skill 查看详情。</div>
    </section>
  </div>

  <div v-if="importDialogOpen" class="modal-backdrop" @click.self="closeImportDialog">
    <section class="modal-card import-modal">
      <div class="modal-title-row">
        <div>
          <h2>新增 Skill</h2>
        </div>
        <button class="ghost-icon-button" type="button" aria-label="关闭" @click="closeImportDialog">
          <X :size="16" />
        </button>
      </div>

      <div class="segmented-control import-source-tabs" aria-label="导入来源">
        <button
          type="button"
          :class="{ active: importSourceMode === 'local' }"
          @click="setImportSourceMode('local')"
        >
          <Folder :size="15" />
          本地
        </button>
        <button
          type="button"
          :class="{ active: importSourceMode === 'github' }"
          @click="setImportSourceMode('github')"
        >
          <Github :size="15" />
          GitHub
        </button>
      </div>

      <div v-if="importSourceMode === 'local'" class="import-source-panel">
        <div class="import-path-row">
          <input class="search-input" :value="importLocalPath" readonly placeholder="选择文件夹" />
          <button class="secondary-button" type="button" :disabled="busy" @click="selectImportLocalPath">
            <Folder :size="16" />
            选择
          </button>
        </div>
      </div>

      <div v-else class="import-source-panel">
        <input
          v-model="importGithubUrl"
          class="search-input"
          type="url"
          placeholder="GitHub URL"
          @input="resetImportPreview"
        />
        <div class="import-github-grid">
          <input v-model="importGithubRef" class="search-input" placeholder="ref" @input="resetImportPreview" />
          <input v-model="importGithubSubdir" class="search-input" placeholder="subdir" @input="resetImportPreview" />
        </div>
      </div>

      <div class="button-row button-row--end">
        <button class="secondary-button" type="button" :disabled="!canScanImports" @click="scanImportSource">
          扫描
        </button>
      </div>

      <section v-if="importScanned" class="import-results">
        <div class="import-results-head">
          <label class="import-check-all">
            <input
              type="checkbox"
              :checked="importReadyCandidates.length > 0 && selectedImportCandidateIds.length === importReadyCandidates.length"
              :disabled="importReadyCandidates.length === 0 || busy"
              @change="toggleAllImportCandidates(($event.target as HTMLInputElement).checked)"
            />
            <span>{{ selectedImportCandidateIds.length }}/{{ importReadyCandidates.length }}</span>
          </label>
        </div>

        <div v-if="importCandidates.length" class="import-candidate-list">
          <label
            v-for="candidate in importCandidates"
            :key="candidate.candidateId"
            class="import-candidate-row"
            :class="{ disabled: candidate.status !== 'ready' }"
          >
            <input
              type="checkbox"
              :checked="importCandidateChecked(candidate.candidateId)"
              :disabled="candidate.status !== 'ready' || busy"
              @change="toggleImportCandidate(candidate, ($event.target as HTMLInputElement).checked)"
            />
            <span class="import-candidate-main">
              <span class="import-candidate-top">
                <strong>{{ candidate.name }}</strong>
                <span class="status-tag" :class="`status-tag--${candidate.status === 'ready' ? 'healthy' : candidate.status}`">
                  {{ importStatusLabels[candidate.status] }}
                </span>
              </span>
              <span class="import-candidate-meta">
                <code>{{ candidate.id }}</code>
                <span>{{ candidate.relativePath }}</span>
              </span>
              <small v-if="candidate.description">{{ candidate.description }}</small>
            </span>
          </label>
        </div>

        <div v-else class="content-empty content-empty--compact">没有找到 skill。</div>
      </section>

      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="closeImportDialog">取消</button>
        <button class="primary-button" :disabled="!canConfirmImports" @click="confirmImportSkills">
          <Plus :size="16" />
          导入选中
        </button>
      </div>
    </section>
  </div>

  <div v-if="deleteDialogOpen && deletePreview" class="modal-backdrop" @click.self="closeDeleteDialog">
    <section class="modal-card">
      <div class="detail-header">
        <div>
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

  <div v-if="addReferenceDialogOpen" class="modal-backdrop" @click.self="closeAddReferenceDialog">
    <section class="modal-card modal-card--compact">
      <div class="modal-title-row">
        <div>
          <h2>{{ addReferenceTitle }}</h2>
        </div>
        <button class="ghost-icon-button" type="button" aria-label="关闭" @click="closeAddReferenceDialog">
          <X :size="16" />
        </button>
      </div>

      <template v-if="overwriteReferenceRequest">
        <p class="modal-note">引用链接已存在，且指向其他位置。覆盖后会把它改为当前 skill 的托管链接。</p>

        <dl class="detail-kv detail-kv--wide">
          <div>
            <dt>目标路径</dt>
            <dd>
              <code class="reference-path">{{ pendingReferencePath }}</code>
            </dd>
          </div>
        </dl>

        <div class="button-row button-row--end">
          <button class="secondary-button" :disabled="busy" @click="cancelOverwriteReference">取消</button>
          <button class="primary-button" :disabled="busy" @click="confirmOverwriteReference">
            <Plus :size="16" />
            覆盖引用
          </button>
        </div>
      </template>

      <template v-else-if="!pendingReferenceTarget">
        <div class="target-grid">
          <button
            v-for="profile in targetProfiles"
            :key="profile.id"
            class="target-tile"
            type="button"
            :disabled="busy"
            @click="selectTargetProfile(profile)"
          >
            <span class="target-tile-icon" aria-hidden="true">
              <AgentIcon :name="profile.targetName" :size="22" />
            </span>
            <strong>{{ profile.targetName }}</strong>
          </button>
        </div>

        <button class="target-custom-button" type="button" :disabled="busy" @click="selectCustomReferenceRoot">
          <Folder :size="18" />
          选择 skills 目录
        </button>
      </template>

      <template v-else>
        <dl class="detail-kv detail-kv--wide">
          <div>
            <dt>目标路径</dt>
            <dd>
              <code class="reference-path">{{ pendingReferencePath }}</code>
            </dd>
          </div>
        </dl>

        <div class="button-row button-row--end">
          <button class="secondary-button" :disabled="busy" @click="pendingReferenceTarget = null">返回</button>
          <button class="primary-button" :disabled="busy" @click="confirmAddReference">
            <Plus :size="16" />
            新增引用
          </button>
        </div>
      </template>
    </section>
  </div>

  <div v-if="deleteReferenceDialogOpen && referenceToDelete" class="modal-backdrop" @click.self="closeDeleteReferenceDialog">
    <section class="modal-card modal-card--compact">
      <div class="modal-title-row">
        <div>
          <h2>删除引用</h2>
        </div>
        <button class="ghost-icon-button" type="button" aria-label="关闭" @click="closeDeleteReferenceDialog">
          <X :size="16" />
        </button>
      </div>

      <template v-if="removeReferenceConflictRequest">
        <p class="modal-note">托管链接已指向其他位置（或存在内容冲突）。是否同时删除该外部链接？</p>

        <dl class="detail-kv detail-kv--wide">
          <div>
            <dt>目标路径</dt>
            <dd>
              <code class="reference-path">{{ removeReferenceConflictRequest.symlinkPath }}</code>
            </dd>
          </div>
        </dl>

        <div class="button-row button-row--end">
          <button class="secondary-button" :disabled="busy" @click="closeDeleteReferenceDialog">取消</button>
          <button class="secondary-button" :disabled="busy" @click="confirmDeleteReferenceWithLink(false)">
            否（只移除记录）
          </button>
          <button class="danger-button" :disabled="busy" @click="confirmDeleteReferenceWithLink(true)">
            是（删除外部链接）
          </button>
        </div>
      </template>

      <template v-else>
        <dl class="detail-kv detail-kv--wide">
          <div>
            <dt>目标路径</dt>
            <dd>
              <code class="reference-path">{{ referenceToDelete.symlinkPath }}</code>
            </dd>
          </div>
        </dl>
        <p class="modal-note">只移除这个托管引用，不会删除技能库中的 skill。</p>

        <div class="button-row button-row--end">
          <button class="secondary-button" :disabled="busy" @click="closeDeleteReferenceDialog">取消</button>
          <button class="danger-button" :disabled="busy" @click="confirmDeleteReference">
            <Trash2 :size="16" />
            删除引用
          </button>
        </div>
      </template>
    </section>
  </div>
</template>

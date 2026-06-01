<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderOpen, Moon, RotateCcw, Sun, Wrench } from "lucide-vue-next";
import * as api from "../api";
import { openDirectory } from "../dialog";
import type { AppSnapshot, DiagnosticItem } from "../types";

type SettingsGroupId = "appearance" | "storage" | "codex" | "issues";
type ThemeMode = "dark" | "light";

const props = defineProps<{
  snapshot: AppSnapshot;
  themeMode: ThemeMode;
}>();

const emit = defineEmits<{
  snapshot: [value: AppSnapshot];
  error: [value: string];
  "update:theme-mode": [value: ThemeMode];
}>();

const busy = ref(false);
const selectedGroup = ref<SettingsGroupId>("appearance");

const canRebuild = computed(() => props.snapshot.stateLoad.phase === "rebuildRequired");

const issueItems = computed(() =>
  props.snapshot.diagnostics.filter((item) => item.code !== "library-migrated"),
);

const applyState = computed(() => {
  const issueCount = issueItems.value.filter((item) => item.level === "error").length;
  const pendingCount = props.snapshot.state.syncStatus.pendingActions.filter((item) => item.kind !== "inspect").length;

  if (issueCount) {
    return {
      tone: "danger",
      label: "需处理",
      message: `有 ${issueCount} 个问题需要先处理。`,
    };
  }

  if (pendingCount) {
    return {
      tone: "warning",
      label: "未完成",
      message: `有 ${pendingCount} 项链接操作未完成。`,
    };
  }

  return {
    tone: "success",
    label: "状态正常",
    message: "当前没有待处理的链接操作。",
  };
});

const settingsGroups = computed(() => [
  {
    id: "appearance" as const,
    title: "外观",
    description: "黑白主题",
    issueCount: 0,
  },
  {
    id: "storage" as const,
    title: "存储位置",
    description: "技能库与状态文件",
    issueCount: props.snapshot.diagnostics.filter((item) => item.code.includes("library")).length,
  },
  {
    id: "codex" as const,
    title: "Codex",
    description: "路径与链接",
    issueCount: props.snapshot.state.syncStatus.pendingActions.length,
  },
  {
    id: "issues" as const,
    title: "问题与修复",
    description: "低频处理入口",
    issueCount: issueItems.value.length + Number(canRebuild.value),
  },
]);

function issueCategory(item: DiagnosticItem): string {
  if (item.code.startsWith("sync-")) return "路径与链接";
  if (item.code === "codex-conflict" || item.code === "managed-link-mismatch") return "内容冲突";
  if (item.code.startsWith("state-")) return "恢复";
  if (item.code.includes("project") || item.code === "effective-state-error") return "规则";
  return "存储";
}

function issueTone(item: DiagnosticItem): "danger" | "warning" | "neutral" {
  if (item.level === "error") return "danger";
  if (item.level === "warning") return "warning";
  return "neutral";
}

function diagnosticGuide(item: DiagnosticItem) {
  if (item.code === "skill-library-missing") {
    return {
      action: "检查技能库目录；必要时迁移到新的位置。",
    };
  }
  if (item.code === "state-rebuild-required") {
    return {
      action: "执行状态重建，然后重新检查并应用。",
    };
  }
  if (item.code === "state-restored-from-backup") {
    return {
      action: "检查诊断项后重新应用，必要时再重建状态。",
    };
  }
  if (item.code === "codex-conflict") {
    return {
      action: "先处理冲突路径，再重新检查并应用。",
    };
  }
  if (item.code === "effective-state-error") {
    return {
      action: "检查项目规则与当前上下文记录，必要时重建状态。",
    };
  }
  return {
    action: "检查对应路径、项目或目标目录，然后重新诊断。",
  };
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

async function chooseCodexPath() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await run(() => api.setCodexPath(selected));
    }
  } catch (cause) {
    emit("error", String(cause));
  }
}

async function chooseLibraryTarget() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await run(() => api.migrateLibrary(selected));
    }
  } catch (cause) {
    emit("error", String(cause));
  }
}
</script>

<template>
  <div class="split-content">
    <section class="list-panel">
      <div class="list-stack">
        <button
          v-for="group in settingsGroups"
          :key="group.id"
          class="list-row"
          :class="{ active: selectedGroup === group.id }"
          @click="selectedGroup = group.id"
        >
          <div class="list-row-copy">
            <strong>{{ group.title }}</strong>
            <small>{{ group.description }}</small>
          </div>
          <div class="list-row-meta">
            <span v-if="group.issueCount" class="status-tag status-tag--warning">{{ group.issueCount }} 项</span>
          </div>
        </button>
      </div>
    </section>

    <section class="detail-panel">
      <template v-if="selectedGroup === 'appearance'">
        <div class="detail-header">
          <div>
            <h2>外观</h2>
          </div>
        </div>

        <section class="detail-section">
          <div class="segmented-control segmented-control--binary theme-toggle" aria-label="主题切换">
            <button
              type="button"
              :class="{ active: themeMode === 'dark' }"
              @click="emit('update:theme-mode', 'dark')"
            >
              <Moon :size="15" />
              黑色
            </button>
            <button
              type="button"
              :class="{ active: themeMode === 'light' }"
              @click="emit('update:theme-mode', 'light')"
            >
              <Sun :size="15" />
              白色
            </button>
          </div>
        </section>
      </template>

      <template v-else-if="selectedGroup === 'storage'">
        <div class="detail-header">
          <div>
            <h2>存储位置</h2>
          </div>
        </div>

        <section class="detail-section">
          <dl class="detail-kv detail-kv--wide">
            <div>
              <dt>技能库路径</dt>
              <dd>{{ snapshot.state.skillLibraryPath }}</dd>
            </div>
            <div>
              <dt>状态文件</dt>
              <dd>{{ snapshot.paths.stateFile }}</dd>
            </div>
            <div>
              <dt>备份文件</dt>
              <dd>{{ snapshot.paths.backupFile }}</dd>
            </div>
          </dl>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>迁移与维护</h3>
          </div>
          <div class="button-row">
            <button class="primary-button" :disabled="busy" @click="chooseLibraryTarget">
              <FolderOpen :size="16" />
              迁移技能库
            </button>
            <button v-if="canRebuild" class="secondary-button" :disabled="busy" @click="run(api.rebuildState)">
              <RotateCcw :size="16" />
              重建状态
            </button>
          </div>
          <div v-if="snapshot.state.migrationNotice" class="inline-panel">
            <strong>迁移结果</strong>
            <span>{{ snapshot.state.migrationNotice.message }}</span>
            <span>新目录：{{ snapshot.state.migrationNotice.newLibraryPath }}</span>
            <span>旧目录：{{ snapshot.state.migrationNotice.oldLibraryPath }}</span>
          </div>
        </section>
      </template>

      <template v-else-if="selectedGroup === 'codex'">
        <div class="detail-header">
          <div>
            <h2>路径与链接</h2>
          </div>
          <div class="tag-row">
            <span class="status-tag" :class="snapshot.state.codexSkillsPath ? 'status-tag--success' : 'status-tag--muted'">
              {{ snapshot.state.codexSkillsPath ? "已设置" : "未设置" }}
            </span>
          </div>
        </div>

        <section class="detail-section">
          <dl class="detail-kv detail-kv--wide">
            <div>
              <dt>Codex 目录</dt>
              <dd>{{ snapshot.state.codexSkillsPath || "未设置" }}</dd>
            </div>
            <div>
              <dt>当前状态</dt>
              <dd>{{ applyState.message }}</dd>
            </div>
          </dl>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>Codex 目录</h3>
          </div>
          <div class="button-row">
            <button class="primary-button" :disabled="busy" @click="chooseCodexPath">
              <FolderOpen :size="16" />
              选择目录
            </button>
          </div>
          <div class="inline-panel" :class="`inline-panel--${applyState.tone}`">
            <strong>{{ applyState.label }}</strong>
            <span>{{ snapshot.state.syncStatus.message || applyState.message }}</span>
          </div>
          <div v-if="snapshot.state.syncStatus.pendingActions.length" class="issue-list">
            <div
              v-for="item in snapshot.state.syncStatus.pendingActions"
              :key="`${item.skillId}-${item.target}-${item.kind}`"
              class="issue-card issue-card--warning"
            >
              <strong>{{ item.skillId }}</strong>
              <p>{{ item.kind === 'inspect' ? '需要处理后再应用' : '链接操作未完成' }}</p>
              <small>{{ item.message }} · {{ item.target }}</small>
            </div>
          </div>
        </section>
      </template>

      <template v-else>
        <div class="detail-header">
          <div>
            <h2>问题与修复</h2>
          </div>
        </div>

        <section class="detail-section">
          <div class="section-heading">
            <h3>当前问题</h3>
          </div>
          <div v-if="issueItems.length" class="issue-list">
            <div
              v-for="item in issueItems"
              :key="`${item.code}-${item.detail}`"
              class="issue-card"
              :class="`issue-card--${issueTone(item)}`"
            >
              <strong>{{ item.title }}</strong>
              <p>{{ item.detail }}</p>
              <small>{{ issueCategory(item) }} · 下一步：{{ diagnosticGuide(item).action }}</small>
            </div>
          </div>
          <div v-else class="content-empty content-empty--compact">当前没有需要处理的问题。</div>
        </section>

        <section class="detail-section">
          <dl class="detail-kv">
            <div>
              <dt>状态恢复</dt>
              <dd>{{ snapshot.stateLoad.message || "当前状态文件正常加载。" }}</dd>
            </div>
            <div>
              <dt>恢复入口</dt>
              <dd>{{ canRebuild ? "可重建状态文件" : "当前不需要重建" }}</dd>
            </div>
          </dl>
        </section>

        <section class="detail-section detail-section--danger">
          <div class="section-heading">
            <h3>恢复操作</h3>
          </div>
          <div class="button-row">
            <button class="secondary-button" :disabled="busy" @click="run(api.syncCodex)">
              <Wrench :size="16" />
              重新检查并应用
            </button>
            <button v-if="canRebuild" class="danger-button" :disabled="busy" @click="run(api.rebuildState)">
              <RotateCcw :size="16" />
              重建状态文件
            </button>
          </div>
        </section>
      </template>
    </section>
  </div>
</template>

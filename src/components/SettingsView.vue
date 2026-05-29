<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderOpen, RefreshCw, RotateCcw, Wrench } from "lucide-vue-next";
import * as api from "../api";
import { openDirectory } from "../dialog";
import type { AppSnapshot, DiagnosticItem } from "../types";

type SettingsGroupId = "library" | "codex" | "diagnostics" | "advanced";

const props = defineProps<{ snapshot: AppSnapshot }>();

const emit = defineEmits<{
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const busy = ref(false);
const selectedGroup = ref<SettingsGroupId>("library");

const canRebuild = computed(() => props.snapshot.stateLoad.phase === "rebuildRequired");

const groupedDiagnostics = computed(() => ({
  error: props.snapshot.diagnostics.filter((item) => item.level === "error"),
  warning: props.snapshot.diagnostics.filter((item) => item.level === "warning"),
  info: props.snapshot.diagnostics.filter((item) => item.level === "info"),
}));

const settingsGroups = computed(() => [
  {
    id: "library" as const,
    title: "技能库",
    description: "路径与迁移",
    issueCount: props.snapshot.diagnostics.filter((item) => item.code.includes("library")).length,
  },
  {
    id: "codex" as const,
    title: "Codex 连接",
    description: "连接与同步",
    issueCount:
      Number(!props.snapshot.codexConnected) + props.snapshot.state.syncStatus.pendingActions.length,
  },
  {
    id: "diagnostics" as const,
    title: "诊断中心",
    description: "错误与警告",
    issueCount: props.snapshot.diagnostics.length,
  },
  {
    id: "advanced" as const,
    title: "高级与恢复",
    description: "恢复操作",
    issueCount: Number(canRebuild.value),
  },
]);

function diagnosticGuide(item: DiagnosticItem) {
  if (item.code === "skill-library-missing") {
    return {
      action: "检查技能库目录；必要时迁移到新的位置。",
    };
  }
  if (item.code === "codex-not-connected") {
    return {
      action: "选择正确的 Codex skills 目录，然后重新同步。",
    };
  }
  if (item.code === "state-rebuild-required") {
    return {
      action: "执行状态重建，然后重新同步一次。",
    };
  }
  if (item.code === "state-restored-from-backup") {
    return {
      action: "检查诊断项后执行一次同步，必要时再重建状态。",
    };
  }
  if (item.code === "codex-conflict") {
    return {
      action: "先处理冲突路径，再手动同步并复查诊断结果。",
    };
  }
  if (item.code === "effective-state-error") {
    return {
      action: "检查项目规则与当前上下文记录，必要时重建状态。",
    };
  }
  return {
    action: "检查对应路径、项目或同步目标，然后重新诊断。",
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
      <div class="panel-header">
        <div>
          <p class="eyebrow">Settings</p>
          <h1 class="panel-title">设置与诊断</h1>
        </div>
      </div>

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
      <template v-if="selectedGroup === 'library'">
        <div class="detail-header">
          <div>
            <p class="eyebrow">Skill Library</p>
            <h2>技能库</h2>
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
            <p class="eyebrow">Codex Connection</p>
            <h2>Codex 连接</h2>
          </div>
          <div class="tag-row">
            <span class="status-tag" :class="snapshot.codexConnected ? 'status-tag--success' : 'status-tag--muted'">
              {{ snapshot.codexConnected ? "已连接" : "离线" }}
            </span>
          </div>
        </div>

        <section class="detail-section">
          <dl class="detail-kv detail-kv--wide">
            <div>
              <dt>连接状态</dt>
              <dd>{{ snapshot.codexConnected ? "已连接" : "未连接或目录不可用" }}</dd>
            </div>
            <div>
              <dt>Codex skills 目录</dt>
              <dd>{{ snapshot.state.codexSkillsPath || "未设置" }}</dd>
            </div>
            <div>
              <dt>上次同步结果</dt>
              <dd>{{ snapshot.state.syncStatus.message || "尚未执行同步" }}</dd>
            </div>
          </dl>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>目录与同步</h3>
          </div>
          <div class="button-row">
            <button class="primary-button" :disabled="busy" @click="chooseCodexPath">
              <FolderOpen :size="16" />
              选择目录
            </button>
            <button class="secondary-button" :disabled="busy" @click="run(api.syncCodex)">
              <RefreshCw :size="16" />
              手动同步
            </button>
          </div>
          <div v-if="snapshot.state.syncStatus.pendingActions.length" class="issue-list">
            <div
              v-for="item in snapshot.state.syncStatus.pendingActions"
              :key="`${item.skillId}-${item.target}-${item.kind}`"
              class="issue-card issue-card--warning"
            >
              <strong>{{ item.skillId }}</strong>
              <p>{{ item.message }} · {{ item.target }}</p>
            </div>
          </div>
        </section>
      </template>

      <template v-else-if="selectedGroup === 'diagnostics'">
        <div class="detail-header">
          <div>
            <p class="eyebrow">Diagnostics</p>
            <h2>诊断中心</h2>
          </div>
        </div>

        <section v-if="groupedDiagnostics.error.length" class="detail-section detail-section--danger">
          <div class="section-heading">
            <h3>错误</h3>
          </div>
          <div class="issue-list">
            <div v-for="item in groupedDiagnostics.error" :key="`${item.code}-${item.detail}`" class="issue-card">
              <strong>{{ item.title }}</strong>
              <p>{{ item.detail }}</p>
              <small>下一步：{{ diagnosticGuide(item).action }}</small>
            </div>
          </div>
        </section>

        <section v-if="groupedDiagnostics.warning.length" class="detail-section">
          <div class="section-heading">
            <h3>警告</h3>
          </div>
          <div class="issue-list">
            <div v-for="item in groupedDiagnostics.warning" :key="`${item.code}-${item.detail}`" class="issue-card issue-card--warning">
              <strong>{{ item.title }}</strong>
              <p>{{ item.detail }}</p>
              <small>下一步：{{ diagnosticGuide(item).action }}</small>
            </div>
          </div>
        </section>

        <section class="detail-section">
          <div class="section-heading">
            <h3>信息</h3>
          </div>
          <div v-if="groupedDiagnostics.info.length" class="issue-list">
            <div v-for="item in groupedDiagnostics.info" :key="`${item.code}-${item.detail}`" class="issue-card issue-card--neutral">
              <strong>{{ item.title }}</strong>
              <p>{{ item.detail }}</p>
              <small>下一步：{{ diagnosticGuide(item).action }}</small>
            </div>
          </div>
          <div v-else class="content-empty content-empty--compact">没有信息项。</div>
        </section>
      </template>

      <template v-else>
        <div class="detail-header">
          <div>
            <p class="eyebrow">Advanced</p>
            <h2>高级与恢复</h2>
          </div>
        </div>

        <section class="detail-section">
          <dl class="detail-kv">
            <div>
              <dt>状态加载</dt>
              <dd>{{ snapshot.stateLoad.message || "当前状态文件正常加载。" }}</dd>
            </div>
            <div>
              <dt>当前阶段</dt>
              <dd>{{ snapshot.stateLoad.phase }}</dd>
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
              重新诊断并同步
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

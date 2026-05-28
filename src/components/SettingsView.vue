<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderOpen, RefreshCw, RotateCcw, Wrench } from "lucide-vue-next";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import type { AppSnapshot } from "../types";

const props = defineProps<{ snapshot: AppSnapshot }>();

const emit = defineEmits<{
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const busy = ref(false);

const canRebuild = computed(() => props.snapshot.stateLoad.phase === "rebuildRequired");

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
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await run(() => api.setCodexPath(selected));
    }
  } catch (cause) {
    emit("error", String(cause));
  }
}

async function chooseLibraryTarget() {
  try {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await run(() => api.migrateLibrary(selected));
    }
  } catch (cause) {
    emit("error", String(cause));
  }
}
</script>

<template>
  <div class="settings-grid">
    <section class="settings-section panel-card">
      <h2>技能库</h2>
      <p class="panel-copy">管理本地技能库的位置，并在需要时迁移目录。</p>
      <dl class="meta-list">
        <dt>技能库路径</dt>
        <dd>{{ snapshot.state.skillLibraryPath }}</dd>
        <dt>状态文件</dt>
        <dd>{{ snapshot.paths.stateFile }}</dd>
        <dt>备份文件</dt>
        <dd>{{ snapshot.paths.backupFile }}</dd>
      </dl>
      <div class="button-row">
        <button class="primary-button" :disabled="busy" @click="chooseLibraryTarget">
          <FolderOpen :size="16" />
          迁移技能库
        </button>
        <button v-if="canRebuild" class="icon-button" :disabled="busy" @click="run(api.rebuildState)">
          <RotateCcw :size="16" />
          重建状态
        </button>
      </div>
      <div v-if="snapshot.state.migrationNotice" class="inline-note">
        <strong>迁移结果</strong>
        <span>{{ snapshot.state.migrationNotice.message }}</span>
        <span>新目录：{{ snapshot.state.migrationNotice.newLibraryPath }}</span>
        <span>旧目录：{{ snapshot.state.migrationNotice.oldLibraryPath }}</span>
      </div>
    </section>

    <section class="settings-section panel-card">
      <h2>Codex 连接</h2>
      <p class="panel-copy">设置 Codex 的 skills 目录并执行同步。</p>
      <dl class="meta-list">
        <dt>状态</dt>
        <dd>{{ snapshot.codexConnected ? "已连接" : "未连接" }}</dd>
        <dt>Codex skills</dt>
        <dd>{{ snapshot.state.codexSkillsPath || "未设置" }}</dd>
        <dt>同步状态</dt>
        <dd>{{ snapshot.state.syncStatus.message || "尚未执行同步" }}</dd>
      </dl>
      <div class="button-row">
        <button class="primary-button" :disabled="busy" @click="chooseCodexPath">
          <FolderOpen :size="16" />
          选择目录
        </button>
        <button class="primary-button" :disabled="busy" @click="run(api.syncCodex)">
          <RefreshCw :size="16" />
          同步 Codex
        </button>
      </div>
      <div v-if="snapshot.state.syncStatus.pendingActions.length" class="inline-note inline-note--warn">
        <strong>待修复操作</strong>
        <span v-for="item in snapshot.state.syncStatus.pendingActions" :key="`${item.skillId}-${item.target}-${item.kind}`">
          {{ item.skillId }} · {{ item.message }} · {{ item.target }}
        </span>
      </div>
    </section>

    <section class="settings-section panel-card">
      <h2>诊断中心</h2>
      <p class="panel-copy">统一查看连接、状态文件、托管链接和冲突问题。</p>
      <div class="diagnostic-list">
        <div
          v-for="item in snapshot.diagnostics"
          :key="`${item.code}-${item.detail}`"
          class="diagnostic-card"
          :class="`diagnostic-card--${item.level}`"
        >
          <div class="diagnostic-header">
            <strong>{{ item.title }}</strong>
            <span>{{ item.level }}</span>
          </div>
          <p>{{ item.detail }}</p>
        </div>
        <div v-if="!snapshot.diagnostics.length" class="content-empty content-empty--inline">
          当前没有需要处理的诊断问题。
        </div>
      </div>
      <div v-if="snapshot.stateLoad.message" class="inline-note">
        <strong>状态加载</strong>
        <span>{{ snapshot.stateLoad.message }}</span>
      </div>
      <div v-if="!snapshot.codexConnected" class="inline-note">
        <strong>首次接入提示</strong>
        <span>即使 Codex 未检测到，SkillMaster 仍可继续管理技能库；完成目录选择后再同步即可。</span>
      </div>
      <div class="button-row">
        <button class="icon-button" :disabled="busy" @click="run(api.syncCodex)">
          <Wrench :size="16" />
          重新诊断并同步
        </button>
      </div>
    </section>
  </div>
</template>

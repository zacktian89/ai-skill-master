<script setup lang="ts">
import { ref } from "vue";
import { FolderOpen, RefreshCw } from "lucide-vue-next";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import type { AppSnapshot } from "../types";

defineProps<{ snapshot: AppSnapshot }>();

const emit = defineEmits<{
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const busy = ref(false);

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
        <dt>位置</dt>
        <dd>{{ snapshot.state.skillLibraryPath }}</dd>
      </dl>
      <div class="button-row">
        <button class="primary-button" :disabled="busy" @click="chooseLibraryTarget">
          <FolderOpen :size="16" />
          迁移技能库
        </button>
      </div>
    </section>

    <section class="settings-section panel-card">
      <h2>Codex 连接</h2>
      <p class="panel-copy">设置 Codex 的 skills 目录并执行同步。</p>
      <dl class="meta-list">
        <dt>状态</dt>
        <dd>{{ snapshot.codexConnected ? "已连接" : "未连接" }}</dd>
        <dt>目录</dt>
        <dd>{{ snapshot.state.codexSkillsPath || "未设置" }}</dd>
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
    </section>

    <section class="settings-section panel-card">
      <h2>诊断</h2>
      <p class="panel-copy">这里列出当前需要处理的连接或配置问题。</p>
      <ul class="diagnostics">
        <li v-for="item in snapshot.diagnostics" :key="item">{{ item }}</li>
        <li v-if="!snapshot.diagnostics.length">没有需要处理的问题。</li>
      </ul>
    </section>
  </div>
</template>

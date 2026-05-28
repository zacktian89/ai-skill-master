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
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    await run(() => api.setCodexPath(selected));
  }
}
</script>

<template>
  <div class="settings-grid">
    <section class="settings-section">
      <h2>技能库</h2>
      <dl class="meta-list">
        <dt>位置</dt>
        <dd>{{ snapshot.state.skillLibraryPath }}</dd>
      </dl>
      <p class="muted">迁移技能库会在后续任务中接入，第一版先展示当前位置。</p>
    </section>

    <section class="settings-section">
      <h2>Codex 连接</h2>
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

    <section class="settings-section">
      <h2>诊断</h2>
      <ul class="diagnostics">
        <li v-for="item in snapshot.diagnostics" :key="item">{{ item }}</li>
        <li v-if="!snapshot.diagnostics.length">没有需要处理的问题。</li>
      </ul>
    </section>
  </div>
</template>

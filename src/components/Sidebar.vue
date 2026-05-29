<script setup lang="ts">
import { computed } from "vue";
import { FolderKanban, Library, Link2, Settings, ShieldAlert } from "lucide-vue-next";
import type { AppSnapshot } from "../types";

type Section = "skills" | "projects" | "settings";

const props = defineProps<{
  activeSection: Section;
  snapshot: AppSnapshot | null;
}>();

defineEmits<{
  "update:activeSection": [value: Section];
}>();

const healthState = computed(() => {
  const diagnostics = props.snapshot?.diagnostics ?? [];
  if (diagnostics.some((item) => item.level === "error")) {
    return {
      tone: "danger",
      title: `${diagnostics.length} 个问题待处理`,
    };
  }
  if (diagnostics.some((item) => item.level === "warning")) {
    return {
      tone: "warning",
      title: `${diagnostics.length} 条诊断提醒`,
    };
  }
  return {
    tone: "success",
    title: diagnostics.length ? `${diagnostics.length} 条信息` : "没有待处理问题",
  };
});
</script>

<template>
  <aside class="sidebar-rail">
    <div class="rail-brand" title="SkillMaster">
      <div class="rail-brand-mark">SM</div>
    </div>

    <nav class="rail-nav" aria-label="Primary">
      <button
        class="rail-nav-button"
        :class="{ active: activeSection === 'skills' }"
        title="Skills"
        @click="$emit('update:activeSection', 'skills')"
      >
        <Library :size="20" />
        <span class="sr-only">Skills</span>
      </button>

      <button
        class="rail-nav-button"
        :class="{ active: activeSection === 'projects' }"
        title="Projects"
        @click="$emit('update:activeSection', 'projects')"
      >
        <FolderKanban :size="20" />
        <span class="sr-only">Projects</span>
      </button>
    </nav>

    <div class="rail-footer">
      <div
        class="rail-status-card"
        :class="snapshot?.codexConnected ? 'tone-success' : 'tone-offline'"
        :title="snapshot?.codexConnected ? 'Codex 已连接' : 'Codex 离线或未设置目录'"
      >
        <Link2 :size="16" />
      </div>

      <div class="rail-status-card" :class="`tone-${healthState.tone}`" :title="healthState.title">
        <ShieldAlert :size="16" />
      </div>

      <button
        class="rail-nav-button"
        :class="{ active: activeSection === 'settings' }"
        title="Settings"
        @click="$emit('update:activeSection', 'settings')"
      >
        <Settings :size="20" />
        <span class="sr-only">Settings</span>
      </button>
    </div>
  </aside>
</template>

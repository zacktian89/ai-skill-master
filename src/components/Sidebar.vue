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
      label: "需要处理",
      detail: `${diagnostics.length} 个问题待处理`,
    };
  }
  if (diagnostics.some((item) => item.level === "warning")) {
    return {
      tone: "warning",
      label: "存在提醒",
      detail: `${diagnostics.length} 条诊断提醒`,
    };
  }
  return {
    tone: "success",
    label: "状态正常",
    detail: diagnostics.length ? `${diagnostics.length} 条信息` : "没有待处理问题",
  };
});
</script>

<template>
  <aside class="sidebar-rail">
    <div class="rail-brand">
      <div class="rail-brand-mark">SM</div>
      <div class="rail-brand-copy">
        <p class="eyebrow">Skill Workspace</p>
        <strong>SkillMaster</strong>
        <span>本地技能、项目规则与同步诊断</span>
      </div>
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
        <span v-if="snapshot?.state.skills.length" class="rail-count">{{ snapshot.state.skills.length }}</span>
      </button>

      <button
        class="rail-nav-button"
        :class="{ active: activeSection === 'projects' }"
        title="Projects"
        @click="$emit('update:activeSection', 'projects')"
      >
        <FolderKanban :size="20" />
        <span class="sr-only">Projects</span>
        <span v-if="snapshot?.state.projects.length" class="rail-count">{{ snapshot.state.projects.length }}</span>
      </button>

      <button
        class="rail-nav-button"
        :class="{ active: activeSection === 'settings' }"
        title="Settings"
        @click="$emit('update:activeSection', 'settings')"
      >
        <Settings :size="20" />
        <span class="sr-only">Settings</span>
      </button>
    </nav>

    <div class="rail-footer">
      <div class="rail-status-card" :class="snapshot?.codexConnected ? 'tone-success' : 'tone-offline'">
        <Link2 :size="16" />
        <div>
          <strong>Codex</strong>
          <span>{{ snapshot?.codexConnected ? "已连接" : "离线或未设置目录" }}</span>
        </div>
      </div>

      <div class="rail-status-card" :class="`tone-${healthState.tone}`">
        <ShieldAlert :size="16" />
        <div>
          <strong>{{ healthState.label }}</strong>
          <span>{{ healthState.detail }}</span>
        </div>
      </div>
    </div>
  </aside>
</template>

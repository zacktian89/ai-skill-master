<script setup lang="ts">
import { computed } from "vue";
import {
  FolderKanban,
  Library,
  PanelLeftClose,
  PanelLeftOpen,
  Settings,
  Sparkles,
} from "lucide-vue-next";
import type { AppSnapshot } from "../types";

type Section = "skills" | "projects" | "settings";

const props = defineProps<{
  activeSection: Section;
  collapsed: boolean;
  snapshot: AppSnapshot | null;
}>();

defineEmits<{
  "update:activeSection": [value: Section];
  "update:collapsed": [value: boolean];
}>();

const workspaceState = computed(() => {
  const diagnostics = props.snapshot?.diagnostics ?? [];
  const pendingActions = props.snapshot?.state.syncStatus.pendingActions ?? [];
  const blockingIssues = diagnostics.filter((item) => item.level === "error").length;
  const pendingChanges = pendingActions.filter((item) => item.kind !== "inspect").length;

  if (blockingIssues) {
    return {
      title: `${blockingIssues} 个问题待处理`,
    };
  }
  if (pendingChanges) {
    return {
      title: `${pendingChanges} 项未完成`,
    };
  }
  return {
    title: "链接状态正常",
  };
});

const navItems = computed(() => [
  {
    id: "skills" as const,
    title: "Skills",
    count: props.snapshot?.state.skills.length ?? 0,
    icon: Library,
  },
  {
    id: "projects" as const,
    title: "Projects",
    count: props.snapshot?.state.projects.length ?? 0,
    icon: FolderKanban,
  },
]);
</script>

<template>
  <aside class="sidebar-rail" :class="{ 'sidebar-rail--collapsed': collapsed }">
    <span class="sr-only">SkillMaster</span>

    <div class="rail-brand" title="SkillMaster">
      <div class="rail-brand-mark">
        <Sparkles :size="18" />
      </div>
      <div class="rail-brand-copy">
        <strong>ai-skill-master</strong>
        <small>SkillMaster workspace</small>
      </div>
    </div>

    <button
      class="rail-collapse-button"
      type="button"
      :aria-label="collapsed ? '展开侧边栏' : '折叠侧边栏'"
      :title="collapsed ? '展开侧边栏' : '折叠侧边栏'"
      @click="$emit('update:collapsed', !collapsed)"
    >
      <PanelLeftOpen v-if="collapsed" :size="17" />
      <PanelLeftClose v-else :size="17" />
    </button>

    <nav class="rail-nav" aria-label="Primary">
      <button
        v-for="item in navItems"
        :key="item.id"
        class="rail-nav-button"
        :class="{ active: activeSection === item.id }"
        :title="collapsed ? `${item.title} (${item.count})` : undefined"
        :aria-label="collapsed ? `${item.title} (${item.count})` : undefined"
        @click="$emit('update:activeSection', item.id)"
      >
        <component :is="item.icon" :size="18" />
        <span class="rail-nav-copy">
          <strong>{{ item.title }} ({{ item.count }})</strong>
        </span>
      </button>
    </nav>

    <div class="rail-footer">
      <button
        class="rail-nav-button rail-nav-button--footer"
        :class="{ active: activeSection === 'settings' }"
        :title="collapsed ? (workspaceState.title !== '链接状态正常' ? `Settings · ${workspaceState.title}` : 'Settings') : undefined"
        :aria-label="collapsed ? (workspaceState.title !== '链接状态正常' ? `Settings · ${workspaceState.title}` : 'Settings') : undefined"
        @click="$emit('update:activeSection', 'settings')"
      >
        <Settings :size="18" />
        <span class="rail-nav-copy">
          <strong>Settings</strong>
          <small v-if="workspaceState.title !== '链接状态正常'">{{ workspaceState.title }}</small>
        </span>
      </button>
    </div>
  </aside>
</template>

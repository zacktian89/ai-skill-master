<script setup lang="ts">
import { computed } from "vue";
import {
  FolderKanban,
  Library,
  PanelLeftOpen,
  Settings,
  Sparkles,
  Bot,
} from "lucide-vue-next";
import type { AppSnapshot } from "../types";

type Section = "skills" | "projects" | "agents" | "settings";

const props = defineProps<{
  activeSection: Section;
  collapsed: boolean;
  snapshot: AppSnapshot | null;
  sidebarWidth: number;
  isDragging: boolean;
}>();

const emit = defineEmits<{
  "update:activeSection": [value: Section];
  "update:collapsed": [value: boolean];
  "update:sidebarWidth": [value: number];
  "update:isDragging": [value: boolean];
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
  {
    id: "agents" as const,
    title: "Agents",
    count: props.snapshot?.state.agents?.length ?? 0,
    icon: Bot,
  },
]);

const startResize = (e: MouseEvent) => {
  e.preventDefault();
  emit("update:isDragging", true);
  const startX = e.clientX;
  const startWidth = props.collapsed ? 54 : props.sidebarWidth;

  const onMouseMove = (moveEvent: MouseEvent) => {
    const deltaX = moveEvent.clientX - startX;
    const nextWidth = startWidth + deltaX;

    if (props.collapsed) {
      if (nextWidth > 100) {
        emit("update:collapsed", false);
        emit("update:sidebarWidth", Math.max(150, nextWidth));
      }
    } else {
      if (nextWidth < 120) {
        emit("update:collapsed", true);
      } else {
        emit("update:sidebarWidth", Math.min(300, Math.max(150, nextWidth)));
      }
    }
  };

  const onMouseUp = () => {
    emit("update:isDragging", false);
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  };

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
};
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

    <nav class="rail-nav" aria-label="Primary">
      <button
        v-for="item in navItems"
        :key="item.id"
        class="rail-nav-button"
        :class="{ active: activeSection === item.id }"
        :title="collapsed ? `${item.title} (${item.count})` : undefined"
        :aria-label="collapsed ? `${item.title} (${item.count})` : undefined"
        @click="emit('update:activeSection', item.id)"
      >
        <component :is="item.icon" :size="18" />
        <span class="rail-nav-copy">
          <strong>{{ item.title }} ({{ item.count }})</strong>
        </span>
      </button>
    </nav>

    <div class="rail-footer">
      <button
        v-if="collapsed"
        class="rail-nav-button rail-expand-button-narrow"
        type="button"
        aria-label="展开侧边栏"
        title="展开侧边栏"
        @click="emit('update:collapsed', false)"
      >
        <PanelLeftOpen :size="18" />
      </button>

      <button
        class="rail-nav-button rail-nav-button--footer"
        :class="{ active: activeSection === 'settings' }"
        :title="collapsed ? (workspaceState.title !== '链接状态正常' ? `Settings · ${workspaceState.title}` : 'Settings') : undefined"
        :aria-label="collapsed ? (workspaceState.title !== '链接状态正常' ? `Settings · ${workspaceState.title}` : 'Settings') : undefined"
        @click="emit('update:activeSection', 'settings')"
      >
        <Settings :size="18" />
        <span class="rail-nav-copy">
          <strong>Settings</strong>
          <small v-if="workspaceState.title !== '链接状态正常'">{{ workspaceState.title }}</small>
        </span>
      </button>
    </div>

    <!-- Resize Handle -->
    <div class="sidebar-resize-handle" @mousedown="startResize"></div>
  </aside>
</template>

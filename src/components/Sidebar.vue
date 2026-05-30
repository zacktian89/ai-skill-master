<script setup lang="ts">
import { computed } from "vue";
import {
  FolderKanban,
  Library,
  Link2,
  PanelLeftClose,
  PanelLeftOpen,
  Settings,
  ShieldAlert,
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
      tone: "danger",
      title: `${blockingIssues} 个问题待处理`,
      short: "需处理",
    };
  }
  if (pendingChanges) {
    return {
      tone: "warning",
      title: `${pendingChanges} 项待应用`,
      short: "待应用",
    };
  }
  return {
    tone: "success",
    title: "链接状态正常",
    short: "正常",
  };
});

const codexPathState = computed(() =>
  props.snapshot?.state.codexSkillsPath
    ? { tone: "success", label: "已设置" }
    : { tone: "muted", label: "未设置" },
);

const currentProject = computed(() => {
  const projects = props.snapshot?.state.projects ?? [];
  const currentId = props.snapshot?.state.currentProjectId;
  return projects.find((project) => project.id === currentId) ?? null;
});

const navItems = computed(() => [
  {
    id: "skills" as const,
    title: "Skills",
    subtitle: `${props.snapshot?.state.skills.length ?? 0} 个 skill`,
    icon: Library,
  },
  {
    id: "projects" as const,
    title: "Projects",
    subtitle: `${props.snapshot?.state.projects.length ?? 0} 个项目`,
    icon: FolderKanban,
  },
]);
</script>

<template>
  <aside class="sidebar-rail" :class="{ 'sidebar-rail--collapsed': collapsed }">
    <div class="rail-brand" title="SkillMaster">
      <div class="rail-brand-mark">
        <Sparkles :size="18" />
      </div>
      <div class="rail-brand-copy">
        <strong>ai-skill-master</strong>
        <small>SkillMaster workspace</small>
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
    </div>

    <nav class="rail-nav" aria-label="Primary">
      <button
        v-for="item in navItems"
        :key="item.id"
        class="rail-nav-button"
        :class="{ active: activeSection === item.id }"
        :title="collapsed ? `${item.title} · ${item.subtitle}` : undefined"
        :aria-label="collapsed ? `${item.title} · ${item.subtitle}` : undefined"
        @click="$emit('update:activeSection', item.id)"
      >
        <component :is="item.icon" :size="18" />
        <span class="rail-nav-copy">
          <strong>{{ item.title }}</strong>
          <small>{{ item.subtitle }}</small>
        </span>
      </button>
    </nav>

    <section class="sidebar-section sidebar-section--project">
      <p class="sidebar-section-title">当前项目</p>
      <button
        class="project-button"
        :class="{ active: activeSection === 'projects' }"
        :title="collapsed ? currentProject?.name ?? '全局默认' : undefined"
        :aria-label="collapsed ? currentProject?.name ?? '全局默认' : undefined"
        @click="$emit('update:activeSection', 'projects')"
      >
        <FolderKanban :size="18" />
        <span class="project-button-copy">
          <strong>{{ currentProject?.name ?? "全局默认" }}</strong>
          <small>{{ currentProject?.path ?? "当前未选择项目" }}</small>
        </span>
      </button>
    </section>

    <section class="sidebar-section">
      <p class="sidebar-section-title">状态</p>
      <div class="status-list">
        <div class="status-row">
          <div class="status-row-copy">
            <Link2 :size="15" />
            <span>目录</span>
          </div>
          <span class="status-value" :class="`is-${codexPathState.tone}`">
            {{ codexPathState.label }}
          </span>
        </div>

        <div class="status-row">
          <div class="status-row-copy">
            <ShieldAlert :size="15" />
            <span>应用</span>
          </div>
          <span class="status-value" :class="`is-${workspaceState.tone}`">
            {{ workspaceState.short }}
          </span>
        </div>
      </div>
    </section>

    <div class="rail-footer">
      <button
        class="rail-nav-button rail-nav-button--footer"
        :class="{ active: activeSection === 'settings' }"
        :title="collapsed ? `Settings · ${workspaceState.title}` : undefined"
        :aria-label="collapsed ? `Settings · ${workspaceState.title}` : undefined"
        @click="$emit('update:activeSection', 'settings')"
      >
        <Settings :size="18" />
        <span class="rail-nav-copy">
          <strong>Settings</strong>
          <small>{{ workspaceState.title }}</small>
        </span>
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  AlertCircle,
  ArrowLeft,
  ArrowRight,
  LayoutPanelTop,
  PanelRightOpen,
  Search,
  SquareTerminal,
} from "lucide-vue-next";
import * as api from "./api";
import Sidebar from "./components/Sidebar.vue";
import ProjectsView from "./components/ProjectsView.vue";
import SettingsView from "./components/SettingsView.vue";
import SkillsView from "./components/SkillsView.vue";
import type { AppSnapshot } from "./types";

type Section = "skills" | "projects" | "settings";

const activeSection = ref<Section>("skills");
const snapshot = ref<AppSnapshot | null>(null);
const selectedSkillId = ref<string | null>(null);
const selectedProjectId = ref<string | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

const sectionMeta: Record<Section, { eyebrow: string; title: string; label: string }> = {
  skills: {
    eyebrow: "Skill Library",
    title: "技能库",
    label: "Skills",
  },
  projects: {
    eyebrow: "Project Rules",
    title: "项目规则",
    label: "Projects",
  },
  settings: {
    eyebrow: "Settings",
    title: "设置与诊断",
    label: "Settings",
  },
};

const activeMeta = computed(() => sectionMeta[activeSection.value]);

const activeSummary = computed(() => {
  if (!snapshot.value) return "";

  if (activeSection.value === "skills") {
    const skills = snapshot.value.state.skills;
    const pendingSkillIds = new Set(snapshot.value.state.syncStatus.pendingActions.map((item) => item.skillId));
    const conflictCount = skills.filter((skill) => Boolean(skill.conflict)).length;
    const needsAttention = skills.filter(
      (skill) => Boolean(skill.conflict) || !skill.managedLinks.codex || pendingSkillIds.has(skill.id),
    ).length;

    const parts = [`${skills.length} 个 skill`];
    if (conflictCount) {
      parts.push(`${conflictCount} 个冲突`);
    } else if (needsAttention) {
      parts.push(`${needsAttention} 个待处理`);
    } else {
      parts.push("全部已同步");
    }
    return parts.join(" · ");
  }

  if (activeSection.value === "projects") {
    const projects = snapshot.value.state.projects;
    const currentProject = projects.find((project) => project.id === snapshot.value?.state.currentProjectId);
    const overrideCount = projects.reduce((total, project) => total + Object.keys(project.rules).length, 0);
    const parts = [`${projects.length} 个项目`];
    if (currentProject) parts.push(`当前 ${currentProject.name}`);
    parts.push(`${overrideCount} 条覆盖`);
    return parts.join(" · ");
  }

  const diagnostics = snapshot.value.diagnostics;
  const errorCount = diagnostics.filter((item) => item.level === "error").length;
  const warningCount = diagnostics.filter((item) => item.level === "warning").length;
  const parts = [snapshot.value.codexConnected ? "Codex 已连接" : "Codex 未连接"];
  if (errorCount) {
    parts.push(`${errorCount} 个错误`);
  } else if (warningCount) {
    parts.push(`${warningCount} 个警告`);
  } else if (diagnostics.length) {
    parts.push(`${diagnostics.length} 条信息`);
  } else {
    parts.push("没有待处理问题");
  }
  return parts.join(" · ");
});

const currentProjectName = computed(() => {
  const projects = snapshot.value?.state.projects ?? [];
  const currentId = snapshot.value?.state.currentProjectId;
  return projects.find((project) => project.id === currentId)?.name ?? "ai-skill-master";
});

async function refresh() {
  loading.value = true;
  error.value = null;
  try {
    const next = await api.getSnapshot();
    snapshot.value = next;
    selectedSkillId.value = next.state.skills[0]?.id ?? null;
    selectedProjectId.value = next.state.currentProjectId ?? next.state.projects[0]?.id ?? null;
  } catch (cause) {
    error.value = String(cause);
  } finally {
    loading.value = false;
  }
}

function applySnapshot(next: AppSnapshot) {
  snapshot.value = next;
  if (!selectedSkillId.value || !next.state.skills.some((skill) => skill.id === selectedSkillId.value)) {
    selectedSkillId.value = next.state.skills[0]?.id ?? null;
  }
  if (!selectedProjectId.value || !next.state.projects.some((project) => project.id === selectedProjectId.value)) {
    selectedProjectId.value = next.state.currentProjectId ?? next.state.projects[0]?.id ?? null;
  }
}

onMounted(refresh);
</script>

<template>
  <div class="app-shell">
    <Sidebar v-model:active-section="activeSection" :snapshot="snapshot" />

    <div class="workspace-shell">
      <header class="workspace-chrome">
        <div class="chrome-cluster">
          <div class="window-dots" aria-hidden="true">
            <span class="window-dot window-dot--red"></span>
            <span class="window-dot window-dot--amber"></span>
            <span class="window-dot window-dot--green"></span>
          </div>
          <button class="chrome-icon-button" type="button" aria-label="Back">
            <ArrowLeft :size="16" />
          </button>
          <button class="chrome-icon-button" type="button" aria-label="Forward">
            <ArrowRight :size="16" />
          </button>
        </div>

        <div class="chrome-title">
          <span>{{ currentProjectName }}</span>
        </div>

        <div class="chrome-cluster chrome-cluster--right">
          <button class="chrome-pill" type="button">
            <LayoutPanelTop :size="15" />
            <span>{{ activeMeta.label }}</span>
          </button>
          <button class="chrome-icon-button" type="button" aria-label="Search">
            <Search :size="16" />
          </button>
          <button class="chrome-icon-button" type="button" aria-label="Layout">
            <PanelRightOpen :size="16" />
          </button>
        </div>
      </header>

      <main class="workspace">
        <section class="workspace-hero">
          <p class="workspace-kicker">{{ activeMeta.eyebrow }}</p>
          <h1 class="workspace-hero-title">{{ activeMeta.title }}</h1>
          <p class="workspace-hero-summary">{{ activeSummary }}</p>
        </section>

        <section class="workspace-frame">
          <div v-if="error" class="notice notice--error">
            <AlertCircle :size="16" />
            <span>{{ error }}</span>
          </div>

          <section v-if="loading" class="workspace-empty">正在加载 ai-skill-master 工作区</section>

          <SkillsView
            v-else-if="activeSection === 'skills' && snapshot"
            :snapshot="snapshot"
            :selected-skill-id="selectedSkillId"
            @select-skill="selectedSkillId = $event"
            @snapshot="applySnapshot"
            @error="error = $event"
          />

          <ProjectsView
            v-else-if="activeSection === 'projects' && snapshot"
            :snapshot="snapshot"
            :selected-project-id="selectedProjectId"
            @select-project="selectedProjectId = $event"
            @snapshot="applySnapshot"
            @error="error = $event"
          />

          <SettingsView
            v-else-if="activeSection === 'settings' && snapshot"
            :snapshot="snapshot"
            @snapshot="applySnapshot"
            @error="error = $event"
          />
        </section>
      </main>

      <footer class="workspace-terminal">
        <div class="terminal-tabs">
          <div class="terminal-tab">
            <SquareTerminal :size="15" />
            <span>{{ currentProjectName }}</span>
          </div>
          <button class="terminal-add" type="button" aria-label="Add terminal tab">+</button>
        </div>

        <div class="terminal-body">
          <span class="terminal-prompt">fang@fangdeMacBook-Pro</span>
          <span class="terminal-command">{{ currentProjectName }} %</span>
        </div>
      </footer>
    </div>
  </div>
</template>

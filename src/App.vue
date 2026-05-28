<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { AlertCircle, FolderKanban, Library, Link2, ShieldAlert } from "lucide-vue-next";
import * as api from "./api";
import Sidebar from "./components/Sidebar.vue";
import SkillsView from "./components/SkillsView.vue";
import ProjectsView from "./components/ProjectsView.vue";
import SettingsView from "./components/SettingsView.vue";
import type { AppSnapshot } from "./types";

type Section = "skills" | "projects" | "settings";

const activeSection = ref<Section>("skills");
const snapshot = ref<AppSnapshot | null>(null);
const selectedSkillId = ref<string | null>(null);
const selectedProjectId = ref<string | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

const title = computed(() => {
  if (activeSection.value === "projects") return "Projects";
  if (activeSection.value === "settings") return "Settings";
  return "Skills";
});

const description = computed(() => {
  if (activeSection.value === "projects") return "按项目覆盖默认规则，确保技能只在需要的工作区里生效。";
  if (activeSection.value === "settings") return "管理技能库位置、Codex 连接状态和当前诊断信息。";
  return "集中维护本地技能库，控制默认启用状态，并同步到 Codex。";
});

const statusItems = computed(() => {
  if (!snapshot.value) return [];
  const currentSnapshot = snapshot.value;
  const currentProject =
    currentSnapshot.state.projects.find((project) => project.id === currentSnapshot.state.currentProjectId) ?? null;

  return [
    {
      key: "skills",
      icon: Library,
      label: `${currentSnapshot.state.skills.length} skills`,
      tone: "neutral",
    },
    {
      key: "projects",
      icon: FolderKanban,
      label: `${currentSnapshot.state.projects.length} projects`,
      tone: "neutral",
    },
    {
      key: "codex",
      icon: Link2,
      label: currentSnapshot.codexConnected ? "Codex connected" : "Codex offline",
      tone: currentSnapshot.codexConnected ? "success" : "warning",
    },
    {
      key: "current-project",
      icon: FolderKanban,
      label: currentProject ? `Current: ${currentProject.name}` : "No active project",
      tone: currentProject ? "neutral" : "muted",
    },
    {
      key: "diagnostics",
      icon: ShieldAlert,
      label: `${currentSnapshot.diagnostics.length} diagnostics`,
      tone: currentSnapshot.diagnostics.length ? "danger" : "success",
    },
  ];
});

async function refresh() {
  loading.value = true;
  error.value = null;
  try {
    snapshot.value = await api.getSnapshot();
    selectedProjectId.value =
      snapshot.value.state.currentProjectId ?? snapshot.value.state.projects[0]?.id ?? null;
  } catch (cause) {
    error.value = String(cause);
  } finally {
    loading.value = false;
  }
}

function applySnapshot(next: AppSnapshot) {
  snapshot.value = next;
}

onMounted(refresh);
</script>

<template>
  <div class="app-shell">
    <Sidebar v-model:active-section="activeSection" :snapshot="snapshot" />
    <main class="main-pane">
      <header class="topbar">
        <div class="topbar-copy">
          <p class="eyebrow">Local Skill Workspace</p>
          <h1>{{ title }}</h1>
          <p class="topbar-note">{{ description }}</p>
        </div>
        <div v-if="statusItems.length" class="status-strip">
          <div
            v-for="item in statusItems"
            :key="item.key"
            class="status-pill"
            :class="`status-pill--${item.tone}`"
          >
            <component :is="item.icon" :size="14" />
            <span>{{ item.label }}</span>
          </div>
        </div>
      </header>

      <div v-if="error" class="notice error">
        <AlertCircle :size="16" />
        <span>{{ error }}</span>
      </div>

      <section class="content-stage">
        <section v-if="loading" class="content-empty content-empty--hero">正在加载 SkillMaster</section>

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
  </div>
</template>

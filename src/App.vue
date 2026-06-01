<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { AlertCircle } from "lucide-vue-next";
import * as api from "./api";
import Sidebar from "./components/Sidebar.vue";
import ProjectsView from "./components/ProjectsView.vue";
import SettingsView from "./components/SettingsView.vue";
import SkillsView from "./components/SkillsView.vue";
import AgentsView from "./components/AgentsView.vue";
import type { AppSnapshot } from "./types";

type Section = "skills" | "projects" | "agents" | "settings";
type ThemeMode = "dark" | "light";

const themeStorageKey = "skillmaster-theme";

function readThemeMode(): ThemeMode {
  if (typeof localStorage === "undefined") return "dark";
  return localStorage.getItem(themeStorageKey) === "light" ? "light" : "dark";
}

const activeSection = ref<Section>("skills");
const snapshot = ref<AppSnapshot | null>(null);
const selectedSkillId = ref<string | null>(null);
const selectedProjectId = ref<string | null>(null);
const selectedAgentId = ref<string | null>(null);
const sidebarCollapsed = ref(false);
const sidebarWidth = ref(200);
const isDragging = ref(false);
const themeMode = ref<ThemeMode>(readThemeMode());
const loading = ref(true);
const error = ref<string | null>(null);

async function refresh() {
  loading.value = true;
  error.value = null;
  try {
    const next = await api.getSnapshot();
    snapshot.value = next;
    selectedSkillId.value = next.state.skills[0]?.id ?? null;
    selectedProjectId.value = next.state.currentProjectId ?? next.state.projects[0]?.id ?? null;
    selectedAgentId.value = next.state.agents?.[0]?.id ?? null;
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
  if (!selectedAgentId.value || !next.state.agents?.some((agent) => agent.id === selectedAgentId.value)) {
    selectedAgentId.value = next.state.agents?.[0]?.id ?? null;
  }
}

function setThemeMode(next: ThemeMode) {
  themeMode.value = next;
  localStorage.setItem(themeStorageKey, next);
}

let wasSmall = false;
const checkWidth = () => {
  const isSmall = window.innerWidth <= 1200;
  if (isSmall && !wasSmall) {
    sidebarCollapsed.value = true;
  }
  wasSmall = isSmall;
};

onMounted(() => {
  refresh();
  checkWidth();
  window.addEventListener("resize", checkWidth);
});

onUnmounted(() => {
  window.removeEventListener("resize", checkWidth);
});
</script>

<template>
  <div
    class="app-shell"
    :data-theme="themeMode"
    :class="{
      'app-shell--sidebar-collapsed': sidebarCollapsed,
      'app-shell--dragging': isDragging
    }"
    :style="{ '--sidebar-width': `${sidebarCollapsed ? 54 : sidebarWidth}px` }"
  >
    <Sidebar
      v-model:active-section="activeSection"
      v-model:collapsed="sidebarCollapsed"
      v-model:sidebar-width="sidebarWidth"
      v-model:is-dragging="isDragging"
      :snapshot="snapshot"
    />

    <div class="workspace-shell">
      <main class="workspace">
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

          <AgentsView
            v-else-if="activeSection === 'agents' && snapshot"
            :snapshot="snapshot"
            :selected-agent-id="selectedAgentId"
            @select-agent="selectedAgentId = $event"
            @snapshot="applySnapshot"
            @error="error = $event"
          />

          <SettingsView
            v-else-if="activeSection === 'settings' && snapshot"
            :snapshot="snapshot"
            :theme-mode="themeMode"
            @snapshot="applySnapshot"
            @error="error = $event"
            @update:theme-mode="setThemeMode"
          />
        </section>
      </main>
    </div>
  </div>
</template>

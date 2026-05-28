<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { AlertCircle } from "lucide-vue-next";
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
        <div>
          <h1>{{ title }}</h1>
          <p v-if="snapshot && !snapshot.codexConnected" class="topbar-note">
            Codex 未连接，可在 Settings 中设置目录。
          </p>
        </div>
      </header>

      <div v-if="error" class="notice error">
        <AlertCircle :size="16" />
        <span>{{ error }}</span>
      </div>

      <section v-if="loading" class="content-empty">正在加载 SkillMaster</section>

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
    </main>
  </div>
</template>

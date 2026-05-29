<script setup lang="ts">
import { onMounted, ref } from "vue";
import { AlertCircle } from "lucide-vue-next";
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

    <main class="workspace">
      <div v-if="error" class="notice notice--error">
        <AlertCircle :size="16" />
        <span>{{ error }}</span>
      </div>

      <section v-if="loading" class="workspace-empty">正在加载 SkillMaster 工作区</section>

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

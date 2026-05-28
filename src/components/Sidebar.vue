<script setup lang="ts">
import { FolderKanban, Library, Link2, Settings } from "lucide-vue-next";
import type { AppSnapshot } from "../types";

type Section = "skills" | "projects" | "settings";

defineProps<{
  activeSection: Section;
  snapshot: AppSnapshot | null;
}>();

defineEmits<{
  "update:activeSection": [value: Section];
}>();
</script>

<template>
  <aside class="sidebar">
    <div class="brand-block">
      <p class="eyebrow">Skill Control Center</p>
      <div class="brand-row">
        <div class="brand">SkillMaster</div>
        <span class="connection-badge" :class="{ connected: snapshot?.codexConnected }">
          <Link2 :size="13" />
          <span>{{ snapshot?.codexConnected ? "Connected" : "Offline" }}</span>
        </span>
      </div>
    </div>
    <nav class="nav-list">
      <button :class="{ active: activeSection === 'skills' }" @click="$emit('update:activeSection', 'skills')">
        <Library :size="18" />
        <span>Skills</span>
        <small v-if="snapshot">{{ snapshot.state.skills.length }}</small>
      </button>
      <button :class="{ active: activeSection === 'projects' }" @click="$emit('update:activeSection', 'projects')">
        <FolderKanban :size="18" />
        <span>Projects</span>
        <small v-if="snapshot">{{ snapshot.state.projects.length }}</small>
      </button>
    </nav>
    <button class="settings-button" :class="{ active: activeSection === 'settings' }" @click="$emit('update:activeSection', 'settings')">
      <Settings :size="18" />
      <span>Settings</span>
    </button>
  </aside>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { AlertCircle } from "lucide-vue-next";
import Sidebar from "./components/Sidebar.vue";
import { createAppStore } from "./stores/useAppStore";
import { createSelectionStore } from "./stores/useSelectionStore";

type Section = "skills" | "projects" | "agents" | "plugins" | "settings";

const router = useRouter();
const route = useRoute();

const activeSection = ref<Section>("skills");
const sidebarCollapsed = ref(false);
const sidebarWidth = ref(200);
const isDragging = ref(false);

// Initialize Stores
const appStore = createAppStore();
const selectionStore = createSelectionStore(appStore.snapshot);

const selectedPluginId = ref<string | null>(null);

watch(
  () => appStore.snapshot.value,
  (next) => {
    if (!next) return;
    if (!selectedPluginId.value || !next.state.plugins?.some((p) => p.id === selectedPluginId.value)) {
      selectedPluginId.value = next.state.plugins?.[0]?.id ?? null;
    }
  },
  { immediate: true }
);

watch(
  () => appStore.themeMode.value,
  (theme) => {
    document.body.setAttribute("data-theme", theme);
  },
  { immediate: true }
);

// Watch current route to sync activeSection
watch(
  () => route.name,
  (name) => {
    if (name) {
      activeSection.value = name as Section;
    }
  }
);

// Navigate when section is changed from sidebar
function handleSectionChange(section: Section) {
  router.push({ name: section });
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
  appStore.refresh();
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
    :data-theme="appStore.themeMode.value"
    :class="{
      'app-shell--sidebar-collapsed': sidebarCollapsed,
      'app-shell--dragging': isDragging
    }"
    :style="{ '--sidebar-width': `${sidebarCollapsed ? 54 : sidebarWidth}px` }"
  >
    <Sidebar
      :active-section="activeSection"
      @update:active-section="handleSectionChange"
      v-model:collapsed="sidebarCollapsed"
      v-model:sidebar-width="sidebarWidth"
      v-model:is-dragging="isDragging"
      :snapshot="appStore.snapshot.value"
    />

    <div class="workspace-shell">
      <main class="workspace">
        <section class="workspace-frame">
          <div v-if="appStore.error.value" class="notice notice--error">
            <AlertCircle :size="16" />
            <span>{{ appStore.error.value }}</span>
          </div>

          <section v-if="appStore.loading.value" class="workspace-empty">正在加载 SkillMaster 工作区</section>

          <router-view v-else v-slot="{ Component }">
            <component
              :is="Component"
              v-if="appStore.snapshot.value"
              :snapshot="appStore.snapshot.value"
              :selected-skill-id="selectionStore.selectedSkillId.value"
              :selected-project-id="selectionStore.selectedProjectId.value"
              :selected-agent-id="selectionStore.selectedAgentId.value"
              :selected-plugin-id="selectedPluginId"
              :theme-mode="appStore.themeMode.value"
              @select-skill="selectionStore.setSelectedSkillId"
              @select-project="selectionStore.setSelectedProjectId"
              @select-agent="selectionStore.setSelectedAgentId"
              @select-plugin="(id) => selectedPluginId = id"
              @snapshot="appStore.applySnapshot"
              @error="appStore.setError"
              @update:theme-mode="appStore.setThemeMode"
            />
          </router-view>
        </section>
      </main>
    </div>
  </div>
</template>

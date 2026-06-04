<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import Sidebar from "./components/Sidebar.vue";
import ToastContainer from "./components/ToastContainer.vue";
import ModalDialog from "./components/ModalDialog.vue";
import { checkForAppUpdate } from "./api/updater";
import { createAppStore } from "./stores/useAppStore";
import { createSelectionStore } from "./stores/useSelectionStore";

type Section = "skills" | "store" | "projects" | "agents" | "plugins" | "settings";

const router = useRouter();
const route = useRoute();

const activeSection = ref<Section>("skills");
const sidebarCollapsed = ref(false);
const sidebarWidth = ref(200);
const isDragging = ref(false);
const updatePrompt = ref<{ version: string; resolve: (accepted: boolean) => void } | null>(null);
const updateInstallBusy = ref(false);

// Initialize Stores
const appStore = createAppStore();
const selectionStore = createSelectionStore(appStore.snapshot);

const selectedPluginId = ref<string | null>(null);

function handlePluginSelection(id: string | null) {
  selectedPluginId.value = id;
}

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

function requestUpdateInstall(version: string): Promise<boolean> {
  return new Promise((resolve) => {
    updateInstallBusy.value = false;
    updatePrompt.value = { version, resolve };
  });
}

function cancelUpdateInstall() {
  if (!updatePrompt.value || updateInstallBusy.value) return;
  updatePrompt.value.resolve(false);
  updatePrompt.value = null;
}

function confirmUpdateInstall() {
  if (!updatePrompt.value || updateInstallBusy.value) return;
  updateInstallBusy.value = true;
  updatePrompt.value.resolve(true);
}

async function runUpdateCheck() {
  try {
    await checkForAppUpdate({
      notify: appStore.addToast,
      confirmInstall: requestUpdateInstall,
    });
  } finally {
    updatePrompt.value = null;
    updateInstallBusy.value = false;
  }
}

onMounted(() => {
  appStore.refresh();
  runUpdateCheck();
  checkWidth();
  window.addEventListener("resize", checkWidth);
});

onUnmounted(() => {
  if (updatePrompt.value) {
    updatePrompt.value.resolve(false);
  }
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
              @select-plugin="handlePluginSelection"
              @snapshot="appStore.applySnapshot"
              @error="appStore.setError"
              @update:theme-mode="appStore.setThemeMode"
            />
          </router-view>
        </section>
      </main>
    </div>

    <ToastContainer />

    <ModalDialog
      v-if="updatePrompt"
      title="发现新版本"
      :show-close="!updateInstallBusy"
      @close="cancelUpdateInstall"
    >
      <div class="form-stack">
        <p class="muted-text">SkillMaster {{ updatePrompt.version }} 已可安装。</p>
        <p class="muted-text">安装完成后应用会自动重启。</p>
      </div>

      <template #footer>
        <div class="button-row button-row--end dialog-footer-row">
          <button class="secondary-button" :disabled="updateInstallBusy" @click="cancelUpdateInstall">稍后</button>
          <button class="primary-button" :disabled="updateInstallBusy" @click="confirmUpdateInstall">
            {{ updateInstallBusy ? "正在安装..." : "立即安装" }}
          </button>
        </div>
      </template>
    </ModalDialog>
  </div>
</template>

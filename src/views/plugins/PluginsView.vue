<script setup lang="ts">
import { computed, ref, watch, inject, nextTick, onBeforeUnmount } from "vue";
import { useRouter } from "vue-router";
import { Puzzle, Terminal, Cpu, ArrowRight, Folder, User, Tag, Code, MoreHorizontal, Trash2, FolderOpen } from "lucide-vue-next";
import { openPath } from "@tauri-apps/plugin-opener";
import SplitPane from "../../components/SplitPane.vue";
import ListPanel from "../../components/ListPanel.vue";
import SearchInput from "../../components/SearchInput.vue";
import SwitchToggle from "../../components/SwitchToggle.vue";
import ModalDialog from "../../components/ModalDialog.vue";
import { useI18n } from "../../composables/useI18n";
import type { AppSnapshot, Plugin } from "../../types";
import { SelectionStoreKey } from "../../stores/useSelectionStore";
import { AppStoreKey } from "../../stores/useAppStore";

const { t } = useI18n();
const router = useRouter();
const selectionStore = inject(SelectionStoreKey, null);
const appStore = inject(AppStoreKey, null);

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedPluginId: string | null;
}>();

const emit = defineEmits<{
  "select-plugin": [value: string | null];
  "select-skill": [value: string | null];
}>();

const query = ref("");
const selectedAgentFilter = ref<string>("all");

const availableAgents = computed(() => {
  const agents = new Set<string>();
  const list = props.snapshot.state.plugins ?? [];
  for (const p of list) {
    if (p.agentTargets) {
      for (const target of p.agentTargets) {
        agents.add(target);
      }
    }
  }
  return Array.from(agents).sort((a, b) => a.localeCompare(b));
});

// More Actions Menu State
const moreMenuOpen = ref<{ x: number; y: number } | null>(null);
const moreMenuRef = ref<HTMLElement | null>(null);
let moreMenuCloseTimer: number | null = null;
const menuMargin = 8;
const fallbackMenuWidth = 148;

// Dialog State
const deleteDialogOpen = ref(false);

// Clean up listener on unmount
onBeforeUnmount(closeMoreMenu);

// Computed active plugin list based on search query and agent filter
const filteredPlugins = computed(() => {
  const q = query.value.trim().toLowerCase();
  let list = props.snapshot.state.plugins ?? [];

  if (selectedAgentFilter.value !== "all") {
    list = list.filter((p) => p.agentTargets.includes(selectedAgentFilter.value as any));
  }

  if (q) {
    list = list.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        p.description.toLowerCase().includes(q) ||
        p.id.toLowerCase().includes(q)
    );
  }

  return list;
});

// Currently selected plugin object
const selectedPlugin = computed(() => {
  const currentId = props.selectedPluginId;
  return filteredPlugins.value.find((p) => p.id === currentId) || filteredPlugins.value[0] || null;
});

const activeDetailTab = ref<"description" | "skills" | "mcp">("skills");

// Reset tab when active plugin changes
watch(
  () => props.selectedPluginId,
  () => {
    if (selectedPlugin.value?.type === "standard") {
      activeDetailTab.value = "skills";
    } else if (selectedPlugin.value?.type === "mcp") {
      activeDetailTab.value = "mcp";
    } else {
      activeDetailTab.value = "description";
    }
  },
  { immediate: true }
);

function handleSelectPlugin(id: string) {
  emit("select-plugin", id);
}

// Redirect and select a skill in the SkillsView
function navigateToSkill(skillId: string) {
  if (selectionStore) {
    selectionStore.setSelectedSkillId(skillId);
  } else {
    emit("select-skill", skillId);
  }
  router.push("/skills");
}

function getAgentBadgeClass(agent: string): string {
  if (agent === "Codex") return "badge-agent--codex";
  if (agent === "Claude Code") return "badge-agent--claude";
  return "badge-agent--default";
}

// Enable/Disable toggle logic
function handleTogglePlugin() {
  if (!selectedPlugin.value || !appStore?.snapshot.value) return;
  const cloned = JSON.parse(JSON.stringify(appStore.snapshot.value)) as AppSnapshot;
  const plugin = cloned.state.plugins?.find((p) => p.id === selectedPlugin.value?.id);
  if (plugin) {
    plugin.enabled = !plugin.enabled;
    appStore.applySnapshot(cloned);
  }
}

// Delete logic
function confirmDeletePlugin() {
  if (!selectedPlugin.value || !appStore?.snapshot.value) return;
  const cloned = JSON.parse(JSON.stringify(appStore.snapshot.value)) as AppSnapshot;
  if (cloned.state.plugins) {
    cloned.state.plugins = cloned.state.plugins.filter((p) => p.id !== selectedPlugin.value?.id);
    appStore.applySnapshot(cloned);
    const nextSelect = cloned.state.plugins[0]?.id ?? null;
    emit("select-plugin", nextSelect);
  }
  deleteDialogOpen.value = false;
}

// Context Menu Operations
function closeMoreMenu() {
  moreMenuOpen.value = null;
  if (moreMenuCloseTimer !== null) {
    window.clearTimeout(moreMenuCloseTimer);
    moreMenuCloseTimer = null;
  }
  document.removeEventListener("click", closeMoreMenu);
  document.removeEventListener("keydown", moreMenuOnEscape);
}

function moreMenuOnEscape(event: KeyboardEvent) {
  if (event.key === "Escape") closeMoreMenu();
}

function clampMenuPosition(x: number, y: number, width: number, height: number) {
  const maxX = Math.max(menuMargin, window.innerWidth - width - menuMargin);
  const maxY = Math.max(menuMargin, window.innerHeight - height - menuMargin);
  return {
    x: Math.min(Math.max(menuMargin, x), maxX),
    y: Math.min(Math.max(menuMargin, y), maxY),
  };
}

async function openMoreMenu(event: MouseEvent) {
  closeMoreMenu();
  const initialPosition = clampMenuPosition(event.clientX - fallbackMenuWidth, event.clientY, fallbackMenuWidth, 0);
  moreMenuOpen.value = initialPosition;
  await nextTick();
  const menuRect = moreMenuRef.value?.getBoundingClientRect();
  if (moreMenuOpen.value && menuRect) {
    const menuWidth = menuRect.width || fallbackMenuWidth;
    const position = clampMenuPosition(event.clientX - menuWidth, event.clientY, menuWidth, menuRect.height);
    moreMenuOpen.value = position;
  }
  moreMenuCloseTimer = window.setTimeout(() => {
    moreMenuCloseTimer = null;
    document.addEventListener("click", closeMoreMenu);
    document.addEventListener("keydown", moreMenuOnEscape);
  });
}

function runMoreMenuAction(action: () => void) {
  action();
  closeMoreMenu();
}

async function openPluginDirectory() {
  if (!selectedPlugin.value) return;
  try {
    await openPath(selectedPlugin.value.path);
  } catch (cause) {
    console.error(cause);
  }
}
</script>

<template>
  <SplitPane class="plugins-view">
    <template #left>
      <ListPanel :items="filteredPlugins" :has-search="true" :empty-text="t('plugins.empty')">
        <template #search-row>
          <div class="list-search-row">
            <SearchInput v-model="query" :placeholder="t('plugins.searchPlaceholder')" />
            <select v-model="selectedAgentFilter" class="agent-filter-select" :aria-label="t('plugins.filterAgent')">
              <option value="all">{{ t("plugins.typeAll") }}</option>
              <option v-for="agent in availableAgents" :key="agent" :value="agent">
                {{ agent }}
              </option>
            </select>
          </div>
        </template>

        <div
          v-for="plugin in filteredPlugins"
          :key="plugin.id"
          class="list-item plugin-item"
          :class="{ active: selectedPlugin?.id === plugin.id, disabled: !plugin.enabled }"
          role="button"
          tabindex="0"
          @click="handleSelectPlugin(plugin.id)"
          @keydown.enter="handleSelectPlugin(plugin.id)"
        >
          <div class="plugin-item-icon-wrapper" :class="`icon-type--${plugin.type}`">
            <component :is="plugin.type === 'mcp' ? Terminal : Puzzle" :size="16" />
          </div>
          <div class="plugin-item-content">
            <div class="plugin-item-title-row">
              <span class="plugin-item-name">{{ plugin.name }}</span>
              <span v-if="plugin.version" class="plugin-item-version">v{{ plugin.version }}</span>
            </div>
            <div class="plugin-item-description" :title="plugin.description">
              {{ plugin.description }}
            </div>
            <div class="plugin-item-footer">
              <span class="plugin-type-badge" :class="`badge-type--${plugin.type}`">
                {{ plugin.type === 'mcp' ? 'MCP' : 'Standard' }}
              </span>
              <span class="plugin-item-agent-tag" v-for="agent in plugin.agentTargets" :key="agent">
                {{ agent }}
              </span>
              <span v-if="!plugin.enabled" class="plugin-item-disabled-tag">
                {{ t('agents.disabledBadge') }}
              </span>
            </div>
          </div>
        </div>
      </ListPanel>
    </template>

    <template #right>
      <div v-if="selectedPlugin" class="extension-detail plugin-detail-page">
        <!-- Unified Header Styling -->
        <header class="extension-header">
          <div class="extension-identity">
            <div class="extension-icon" :title="selectedPlugin.name">
              <component :is="selectedPlugin.type === 'mcp' ? Cpu : Puzzle" :size="28" />
            </div>
            <div class="extension-title-group">
              <h2>{{ selectedPlugin.name }}</h2>
              <div class="extension-meta">
                <code>{{ selectedPlugin.id }}</code>
                <span class="plugin-detail-type-tag" :class="`badge-type--${selectedPlugin.type}`">
                  {{ selectedPlugin.type === 'mcp' ? t('plugins.typeMcp') : t('plugins.typeStandard') }}
                </span>
                <span v-if="selectedPlugin.version">v{{ selectedPlugin.version }}</span>
              </div>
            </div>
          </div>

          <div class="extension-command-panel">
            <div class="extension-actions" style="align-items: center; gap: 12px; display: flex;">
              <!-- Switch Status Label -->
              <span v-if="!selectedPlugin.enabled" class="toggle-status-text" style="font-size: var(--font-size-xs); color: var(--text-muted); font-weight: var(--font-weight-medium);">
                {{ t('agents.disabledBadge') }}
              </span>
              <!-- Toggle Switch Component -->
              <SwitchToggle
                :checked="selectedPlugin.enabled"
                :title="selectedPlugin.enabled ? '停用插件' : '启用插件'"
                @change="handleTogglePlugin"
              />
              <!-- More Actions Menu Trigger -->
              <button
                class="ghost-icon-button"
                type="button"
                :aria-label="t('skills.moreActions')"
                :title="t('skills.moreActions')"
                @click.stop="openMoreMenu"
              >
                <MoreHorizontal :size="16" />
              </button>
            </div>
          </div>

          <!-- Actions Teleport Context Menu -->
          <Teleport to="body">
            <div
              v-if="moreMenuOpen"
              ref="moreMenuRef"
              class="global-context-menu"
              :style="{ left: `${moreMenuOpen.x}px`, top: `${moreMenuOpen.y}px` }"
              role="menu"
              @click.stop
            >
              <button
                type="button"
                role="menuitem"
                class="global-context-menu-item delete-action-btn"
                @click="runMoreMenuAction(() => deleteDialogOpen = true)"
              >
                <Trash2 :size="15" />
                <span>{{ t('dialog.delete') }}</span>
              </button>

              <button
                type="button"
                role="menuitem"
                class="global-context-menu-item"
                @click="runMoreMenuAction(openPluginDirectory)"
              >
                <FolderOpen :size="15" />
                <span>{{ t('skills.openDirectory') }}</span>
              </button>
            </div>
          </Teleport>
        </header>

        <!-- Dynamic Header Description -->
        <div class="plugin-subheader-desc">
          <p class="plugin-detail-description">
            {{ selectedPlugin.description }}
          </p>
        </div>

        <!-- Unified Tabs Nav -->
        <nav class="detail-tabs" aria-label="Plugin detail tabs" style="margin-top: var(--spacing-sm);">
          <button
            v-if="selectedPlugin.type === 'standard'"
            class="detail-tab"
            :class="{ active: activeDetailTab === 'skills' }"
            type="button"
            @click="activeDetailTab = 'skills'"
          >
            {{ t('plugins.associatedSkills') }}
          </button>
          <button
            v-if="selectedPlugin.type === 'mcp'"
            class="detail-tab"
            :class="{ active: activeDetailTab === 'mcp' }"
            type="button"
            @click="activeDetailTab = 'mcp'"
          >
            配置
          </button>
          <button
            class="detail-tab"
            :class="{ active: activeDetailTab === 'description' }"
            type="button"
            @click="activeDetailTab = 'description'"
          >
            {{ t('skills.tabDetail') }}
          </button>
        </nav>

        <!-- Tab Panels -->
        <!-- Tab 1: Description Panel (Standard Meta Grid) -->
        <div v-if="activeDetailTab === 'description'" class="plugin-tab-pane description-pane" style="padding: var(--spacing-lg); overflow-y: auto; flex: 1;">
          <div class="detail-section">
            <div class="meta-grid">
              <div class="meta-grid-item" v-if="selectedPlugin.version">
                <span class="meta-label">
                  <Tag :size="14" /> {{ t('plugins.versionLabel') }}
                </span>
                <span class="meta-val">{{ selectedPlugin.version }}</span>
              </div>
              <div class="meta-grid-item" v-if="selectedPlugin.author">
                <span class="meta-label">
                  <User :size="14" /> {{ t('plugins.authorLabel') }}
                </span>
                <span class="meta-val">{{ selectedPlugin.author }}</span>
              </div>
              <div class="meta-grid-item">
                <span class="meta-label">
                  <Cpu :size="14" /> {{ t('plugins.agentTargetsLabel') }}
                </span>
                <div class="meta-val-badges">
                  <span
                    v-for="agent in selectedPlugin.agentTargets"
                    :key="agent"
                    class="agent-pill"
                    :class="getAgentBadgeClass(agent)"
                  >
                    {{ agent }}
                  </span>
                </div>
              </div>
              <div class="meta-grid-item full-width">
                <span class="meta-label">
                  <Folder :size="14" /> {{ t('plugins.pathLabel') }}
                </span>
                <span class="meta-val path-val" :title="selectedPlugin.path">{{ selectedPlugin.path }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Tab 2: Associated Skills Panel (For Codex / Standard) -->
        <div v-else-if="activeDetailTab === 'skills' && selectedPlugin.type === 'standard'" class="plugin-tab-pane skills-pane" style="padding: var(--spacing-lg); overflow-y: auto; flex: 1;">
          <div class="detail-section">
            <div v-if="selectedPlugin.skills && selectedPlugin.skills.length > 0" class="associated-skills-list">
              <div
                v-for="skill in selectedPlugin.skills"
                :key="skill.id"
                class="associated-skill-card"
              >
                <div class="project-skill-main">
                  <div class="project-skill-header">
                    <div class="project-skill-title-row">
                      <Folder class="project-skill-title-icon" :size="15" />
                      <strong>{{ skill.name }}</strong>
                    </div>
                  </div>
                  <div class="project-skill-meta">
                    <code>{{ skill.id }}</code>
                  </div>
                  <div v-if="skill.description" class="project-skill-description">
                    {{ skill.description }}
                  </div>
                </div>
              </div>
            </div>
            <div v-else class="empty-embedded">
              <p>{{ t('plugins.noSkills') }}</p>
            </div>
          </div>
        </div>

        <!-- Tab 3: MCP Config Code Block Panel (For Claude Code / MCP) -->
        <div v-else-if="activeDetailTab === 'mcp' && selectedPlugin.type === 'mcp'" class="plugin-tab-pane mcp-pane" style="padding: var(--spacing-lg); overflow-y: auto; flex: 1;">
          <div class="detail-section">
            <div class="mcp-config-container">
              <div class="mcp-config-header">
                <span class="mcp-server-name">{{ selectedPlugin.mcpServers?.[0] || selectedPlugin.name }}</span>
                <span class="mcp-status-badge">Active</span>
              </div>
              <div class="mcp-config-details" v-if="selectedPlugin.mcpConfig">
                <div class="config-block" v-if="selectedPlugin.mcpConfig.command">
                  <span class="config-title">{{ t('plugins.mcpCommand') }}</span>
                  <code>{{ selectedPlugin.mcpConfig.command }}</code>
                </div>
                <div class="config-block" v-if="selectedPlugin.mcpConfig.args">
                  <span class="config-title">{{ t('plugins.mcpArgs') }}</span>
                  <pre class="args-pre"><code>{{ JSON.stringify(selectedPlugin.mcpConfig.args, null, 2) }}</code></pre>
                </div>
                <div class="config-block" v-if="selectedPlugin.mcpConfig.env">
                  <span class="config-title">{{ t('plugins.mcpEnv') }}</span>
                  <pre class="env-pre"><code>{{ JSON.stringify(selectedPlugin.mcpConfig.env, null, 2) }}</code></pre>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div v-else class="content-empty">
        {{ t("plugins.selectDetail") }}
      </div>
    </template>
  </SplitPane>

  <!-- Delete Plugin Confirmation Dialog -->
  <ModalDialog
    v-if="deleteDialogOpen && selectedPlugin"
    :show="deleteDialogOpen"
    :title="`确认删除插件`"
    @close="deleteDialogOpen = false"
  >
    <div style="padding: var(--spacing-md) 0;">
      <p style="margin: 0; font-size: var(--font-size-sm); color: var(--text-secondary); line-height: 1.5;">
        确定要删除插件 <strong>{{ selectedPlugin.name }}</strong> 吗？
      </p>
      <p style="margin: 8px 0 0 0; font-size: var(--font-size-xs); color: var(--text-muted); line-height: 1.4; border-top: 1px solid var(--border-muted); padding-top: 8px; font-family: monospace; word-break: break-all;">
        配置路径: {{ selectedPlugin.path }}
      </p>
    </div>
    <template #footer>
      <div class="modal-footer-row" style="display: flex; justify-content: flex-end; gap: 8px; margin-top: var(--spacing-md);">
        <button class="button button--ghost" type="button" @click="deleteDialogOpen = false">
          {{ t('dialog.cancel') }}
        </button>
        <button class="button button--danger" type="button" @click="confirmDeletePlugin">
          {{ t('dialog.delete') }}
        </button>
      </div>
    </template>
  </ModalDialog>
</template>

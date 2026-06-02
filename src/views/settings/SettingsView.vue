<script setup lang="ts">
import { computed, inject, ref, onMounted } from "vue";
import { Moon, Sun, FolderOpen } from "lucide-vue-next";
import { openPath } from "@tauri-apps/plugin-opener";
import * as api from "../../api";
import { openDirectory } from "../../utils/dialog";
import { AppStoreKey } from "../../stores/useAppStore";
import { useAsyncAction } from "../../composables/useAsyncAction";
import { useI18n } from "../../composables/useI18n";
import type { AppSnapshot } from "../../types";

type ThemeMode = "dark" | "light";

const appStore = inject(AppStoreKey, null);
const { t, locale } = useI18n();

const props = defineProps<{
  snapshot: AppSnapshot;
  themeMode: ThemeMode;
}>();

const emit = defineEmits<{
  snapshot: [value: AppSnapshot];
  error: [value: string];
  "update:theme-mode": [value: ThemeMode];
}>();

const snapshot = computed(() => appStore?.snapshot.value ?? props.snapshot);
const themeMode = computed({
  get: () => appStore?.themeMode.value ?? props.themeMode,
  set: (val) => {
    if (appStore) appStore.setThemeMode(val);
    else emit("update:theme-mode", val);
  }
});

const { busy, run: executeAsync } = useAsyncAction({
  onError: (err) => {
    if (appStore) appStore.setError(String(err));
  }
});

const storageRootDir = computed(() => {
  const path = snapshot.value.paths.stateFile;
  const index = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  if (index !== -1) {
    return path.substring(0, index);
  }
  return path;
});

async function chooseLibraryTarget() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await executeAsync(
        () => api.migrateLibrary(selected),
        (next) => handleSnapshotSuccess(next)
      );
    }
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
  }
}

async function openStorageDir() {
  try {
    await openPath(storageRootDir.value);
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
  }
}

const appVersion = ref("0.1.0");

onMounted(async () => {
  try {
    if ((window as any).__TAURI_INTERNALS__) {
      const { getVersion } = await import("@tauri-apps/api/app");
      appVersion.value = await getVersion();
    }
  } catch (err) {
    // fallback
  }
});

function handleSnapshotSuccess(nextSnapshot: AppSnapshot) {
  if (appStore) appStore.applySnapshot(nextSnapshot);
  else emit("snapshot", nextSnapshot);
}
</script>

<template>
  <div class="detail-panel settings-page">
    <div class="settings-container">
      <div class="settings-header">
        <h1>{{ t('settings.title') }}</h1>
      </div>

      <!-- Card 1: Appearance -->
      <div class="settings-card">
        <div class="card-title">{{ t('settings.appearanceCard') }}</div>
        
        <!-- Theme selection -->
        <div class="settings-row">
          <div class="setting-info">
            <div class="setting-title">{{ t('settings.theme') }}</div>
          </div>
          <div class="setting-control">
            <div class="segmented-control segmented-control--binary theme-toggle" :aria-label="t('settings.theme')">
              <button
                type="button"
                :class="{ active: themeMode === 'dark' }"
                @click="themeMode = 'dark'"
              >
                <Moon :size="14" />
                {{ t('settings.themeDark') }}
              </button>
              <button
                type="button"
                :class="{ active: themeMode === 'light' }"
                @click="themeMode = 'light'"
              >
                <Sun :size="14" />
                {{ t('settings.themeLight') }}
              </button>
            </div>
          </div>
        </div>

        <!-- Language selection -->
        <div class="settings-row">
          <div class="setting-info">
            <div class="setting-title">{{ t('settings.language') }}</div>
          </div>
          <div class="setting-control">
            <div class="segmented-control segmented-control--binary language-toggle" :aria-label="t('settings.language')">
              <button
                type="button"
                :class="{ active: locale === 'zh' }"
                @click="locale = 'zh'"
              >
                {{ t('settings.langZh') }}
              </button>
              <button
                type="button"
                :class="{ active: locale === 'en' }"
                @click="locale = 'en'"
              >
                {{ t('settings.langEn') }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Card 2: Storage -->
      <div class="settings-card">
        <div class="card-title">{{ t('settings.storageCard') }}</div>
        
        <div class="settings-row">
          <div class="setting-info">
            <div class="setting-title">{{ t('settings.storageRootDir') }}</div>
            <div class="path-container">
              <code class="path-display" :title="storageRootDir">{{ storageRootDir }}</code>
            </div>
          </div>
          <div class="setting-control">
            <button class="secondary-button compact-btn" type="button" @click="openStorageDir">
              <FolderOpen :size="14" />
              <span>{{ t('settings.openButton') }}</span>
            </button>
          </div>
        </div>

        <div class="settings-row">
          <div class="setting-info">
            <div class="setting-title">{{ t('settings.migration') }}</div>
          </div>
          <div class="setting-control">
            <button class="primary-button compact-btn" :disabled="busy" @click="chooseLibraryTarget">
              <FolderOpen :size="14" />
              <span>{{ t('settings.migrationButton') }}</span>
            </button>
          </div>
        </div>

        <!-- Migration Notice -->
        <div v-if="snapshot.state.migrationNotice" class="migration-notice-card">
          <div class="notice-header">
            <strong>{{ t('settings.migrationSuccess') }}</strong>
          </div>
          <div class="notice-body">
            <div><span>{{ t('settings.migrationResult') }}</span>{{ snapshot.state.migrationNotice.message }}</div>
            <div><span>{{ t('settings.migrationNewDir') }}</span><code>{{ snapshot.state.migrationNotice.newLibraryPath }}</code></div>
            <div><span>{{ t('settings.migrationOldDir') }}</span><code>{{ snapshot.state.migrationNotice.oldLibraryPath }}</code></div>
          </div>
        </div>
      </div>

      <div class="settings-footer">
        <span>{{ t('settings.version', { version: appVersion }) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  flex: 1;
  overflow-y: auto;
}

.settings-container {
  max-width: 760px;
  margin: 0 auto;
  padding: var(--spacing-7xl) var(--spacing-4xl);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-5xl);
}

.settings-header {
  margin-bottom: var(--spacing-xs);
}

.settings-header h1 {
  margin: 0;
  font-size: var(--font-size-4xl);
  font-weight: var(--font-weight-bold);
  color: var(--text-primary);
  line-height: 1.2;
}

.settings-card {
  background: var(--bg-panel-muted);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-2xl);
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.card-title {
  padding: var(--spacing-xl) var(--spacing-3xl) var(--spacing-sm);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-bold);
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--text-tertiary);
  border-bottom: 1px solid var(--border-default);
  background: rgba(255, 255, 255, 0.015);
}

.settings-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-2xl) var(--spacing-3xl);
  gap: var(--spacing-4xl);
}

.settings-row:not(:last-child) {
  border-bottom: 1px solid var(--border-default);
}

.setting-info {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.setting-title {
  font-size: var(--font-size-xl);
  font-weight: var(--font-weight-medium);
  color: var(--text-primary);
}

.setting-control {
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.path-container {
  margin-top: var(--spacing-xs);
  width: 100%;
}

.path-display {
  display: inline-block;
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--bg-main-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, Monaco, Consolas, monospace;
  font-size: var(--font-size-md);
  color: var(--text-primary);
  max-width: 100%;
  box-sizing: border-box;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  user-select: all;
}

.compact-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: 6px var(--spacing-lg);
  height: 32px;
  font-size: var(--font-size-md);
}

/* Migration Notice styles */
.migration-notice-card {
  margin: 0 var(--spacing-3xl) var(--spacing-3xl);
  padding: var(--spacing-xl);
  background: var(--success-bg);
  border: 1px solid var(--success-border);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.notice-header {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--success-text);
}

.notice-body {
  font-size: var(--font-size-md);
  line-height: 1.5;
  color: var(--text-primary);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2xs);
}

.notice-body span {
  font-weight: var(--font-weight-medium);
  color: var(--success-text);
}

.notice-body code {
  font-family: monospace;
  background: rgba(0, 0, 0, 0.15);
  padding: 2px var(--spacing-sm);
  border-radius: var(--radius-xs);
}

.settings-footer {
  margin-top: var(--spacing-xl);
  text-align: center;
  font-size: var(--font-size-sm);
  color: var(--text-muted);
}
</style>

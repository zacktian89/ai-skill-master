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
            <div class="segmented-control theme-toggle segmented-control--compact" :aria-label="t('settings.theme')">
              <button
                type="button"
                :class="{ active: themeMode === 'dark' }"
                @click="themeMode = 'dark'"
              >
                <Moon :size="16" />
              </button>
              <button
                type="button"
                :class="{ active: themeMode === 'light' }"
                @click="themeMode = 'light'"
              >
                <Sun :size="16" />
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



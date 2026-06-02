<script setup lang="ts">
import { computed, ref, inject } from "vue";
import SplitPane from "../../components/SplitPane.vue";
import ListPanel from "../../components/ListPanel.vue";
import StatusTag from "../../components/StatusTag.vue";

import AppearancePanel from "./components/AppearancePanel.vue";
import StoragePanel from "./components/StoragePanel.vue";
import IssuesPanel from "./components/IssuesPanel.vue";

import type { AppSnapshot } from "../../types";
import { AppStoreKey } from "../../stores/useAppStore";

type SettingsGroupId = "appearance" | "storage" | "issues";
type ThemeMode = "dark" | "light";

const appStore = inject(AppStoreKey, null);

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

const selectedGroup = ref<SettingsGroupId>("appearance");



const canRebuild = computed(() => snapshot.value.stateLoad.phase === "rebuildRequired");

const issueItems = computed(() =>
  snapshot.value.diagnostics.filter((item) => item.code !== "library-migrated"),
);



const settingsGroups = computed(() => [
  {
    id: "appearance" as const,
    title: "外观",
    description: "黑白主题",
    issueCount: 0,
  },
  {
    id: "storage" as const,
    title: "存储位置",
    description: "技能库与状态文件",
    issueCount: snapshot.value.diagnostics.filter((item) => item.code.includes("library")).length,
  },
  {
    id: "issues" as const,
    title: "问题与修复",
    description: "低频处理入口",
    issueCount: issueItems.value.length + Number(canRebuild.value),
  },
]);

function handleSnapshotSuccess(nextSnapshot: AppSnapshot) {
  if (appStore) appStore.applySnapshot(nextSnapshot);
  else emit("snapshot", nextSnapshot);
}
</script>

<template>
  <SplitPane>
    <template #left>
      <ListPanel :items="settingsGroups">
        <button
          v-for="group in settingsGroups"
          :key="group.id"
          class="list-row"
          :class="{ active: selectedGroup === group.id }"
          @click="selectedGroup = group.id"
        >
          <div class="list-row-copy">
            <strong>{{ group.title }}</strong>
            <small>{{ group.description }}</small>
          </div>
          <div class="list-row-meta">
            <StatusTag v-if="group.issueCount" type="warning">{{ group.issueCount }} 项</StatusTag>
          </div>
        </button>
      </ListPanel>
    </template>

    <template #right>
      <AppearancePanel
        v-if="selectedGroup === 'appearance'"
        v-model:theme-mode="themeMode"
      />

      <StoragePanel
        v-else-if="selectedGroup === 'storage'"
        :snapshot="snapshot"
        :can-rebuild="canRebuild"
        @success="handleSnapshotSuccess"
      />



      <IssuesPanel
        v-else-if="selectedGroup === 'issues'"
        :snapshot="snapshot"
        :issue-items="issueItems"
        :can-rebuild="canRebuild"
        @success="handleSnapshotSuccess"
      />
    </template>
  </SplitPane>
</template>

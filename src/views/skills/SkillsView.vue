<script setup lang="ts">
import { computed, ref, watch, inject } from "vue";
import { Plus } from "lucide-vue-next";
import SplitPane from "../../components/SplitPane.vue";
import ListPanel from "../../components/ListPanel.vue";
import SearchInput from "../../components/SearchInput.vue";
import { useI18n } from "../../composables/useI18n";

import SkillListItem from "./components/SkillListItem.vue";
import SkillDetail from "./components/SkillDetail.vue";
import ImportSkillDialog from "./components/ImportSkillDialog.vue";
import DeleteSkillDialog from "./components/DeleteSkillDialog.vue";
import ReferenceDialogs from "./components/ReferenceDialogs.vue";

import type {
  AppSnapshot,
  PendingSyncAction,
  Skill,
  SkillTargetProfile,
  SkillReferenceDetail,
} from "../../types";
import { AppStoreKey } from "../../stores/useAppStore";
import { SelectionStoreKey } from "../../stores/useSelectionStore";
import { useAsyncAction } from "../../composables/useAsyncAction";
import { useSkillMarkdown } from "../../composables/useSkillMarkdown";

const { t } = useI18n();

type DetailTab = "references" | "description";

const appStore = inject(AppStoreKey, null);
const selectionStore = inject(SelectionStoreKey, null);

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedSkillId: string | null;
}>();

const emit = defineEmits<{
  "select-skill": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const snapshot = computed(() => appStore?.snapshot.value ?? props.snapshot);
const selectedSkillId = computed({
  get: () => selectionStore?.selectedSkillId.value ?? props.selectedSkillId,
  set: (val) => {
    if (selectionStore) {
      selectionStore.setSelectedSkillId(val);
    } else {
      emit("select-skill", val);
    }
  }
});

const query = ref("");

const { busy } = useAsyncAction({
  onError: (err) => {
    if (appStore) appStore.setError(String(err));
    else emit("error", String(err));
  }
});

const deleteDialogOpen = ref(false);
const importDialogOpen = ref(false);
const referenceDialogOpen = ref(false);
const referenceDialogMode = ref<"add" | "delete">("add");
const referenceToDelete = ref<SkillReferenceDetail | null>(null);

const activeDetailTab = ref<DetailTab>("description");

// Composable for Markdown
const {
  skillMarkdown,
  isMarkdownLoading,
  parsedMarkdown,
  renderedMarkdown
} = useSkillMarkdown(
  () => selectedSkillId.value,
  () => activeDetailTab.value
);

function actionsForSkill(skillId: string): PendingSyncAction[] {
  return snapshot.value.state.syncStatus.pendingActions.filter((item) => item.skillId === skillId);
}

function isReferenced(skill: Skill): boolean {
  if ((skill.references?.length ?? 0) > 0) return true;
  return snapshot.value.state.projects.some((project) => project.rules[skill.id] === "enable");
}

const skills = computed(() => {
  const normalized = query.value.trim().toLowerCase();
  return [...snapshot.value.state.skills]
    .filter((skill) => {
      if (!normalized) return true;
      return `${skill.name} ${skill.description} ${skill.id}`.toLowerCase().includes(normalized);
    })
    .sort((left, right) => left.name.localeCompare(right.name, "zh-CN"));
});

const selectedSkill = computed(
  () => skills.value.find((skill) => skill.id === selectedSkillId.value) ?? skills.value[0] ?? null,
);

const selectedIssues = computed(() => {
  if (!selectedSkill.value) return [];
  return actionsForSkill(selectedSkill.value.id)
    .filter((item) => item.kind === "inspect")
    .map((item) => ({
      key: `${item.kind}-${item.target}-${item.message}`,
      title: t("skills.needHandle"),
      detail: `${item.message} · ${item.target}`,
    }));
});

const selectedReferences = computed(() => {
  if (!selectedSkill.value) return [];
  return referencesForSkill(selectedSkill.value);
});

const targetProfiles = computed<SkillTargetProfile[]>(() => snapshot.value.targetProfiles ?? []);

function referencesForSkill(skill: Skill): SkillReferenceDetail[] {
  return (skill.references ?? []).map((reference) => ({
    id: reference.id,
    targetName: reference.targetName,
    symlinkPath: reference.targetPath,
    scope: reference.scope,
    status: reference.status,
    removable: true,
    legacyCodex: false,
  }));
}

function handleSnapshotSuccess(nextSnapshot: AppSnapshot) {
  if (appStore) appStore.applySnapshot(nextSnapshot);
  else emit("snapshot", nextSnapshot);
}

function openAddReferenceDialog() {
  referenceDialogMode.value = "add";
  referenceToDelete.value = null;
  referenceDialogOpen.value = true;
}

function openDeleteReferenceDialog(reference: SkillReferenceDetail) {
  referenceDialogMode.value = "delete";
  referenceToDelete.value = reference;
  referenceDialogOpen.value = true;
}

// Watch selectedSkill to reset activeTab/reset detail description markdown loading
watch(selectedSkill, () => {
  // Let the useSkillMarkdown hook load markdown automatically
});
</script>

<template>
  <SplitPane>
    <template #left>
      <ListPanel :items="skills" :has-search="true" :empty-text="t('skills.empty')">
        <template #search-row>
          <div class="list-search-row">
            <SearchInput v-model="query" :placeholder="t('skills.searchPlaceholder')" />
            <button
              class="icon-button"
              type="button"
              :disabled="busy"
              :aria-label="t('skills.addSkill')"
              @click="importDialogOpen = true"
            >
              <Plus :size="18" />
            </button>
          </div>
        </template>

        <SkillListItem
          v-for="skill in skills"
          :key="skill.id"
          :skill="skill"
          :is-active="selectedSkill?.id === skill.id"
          :is-referenced="isReferenced(skill)"
          @select="selectedSkillId = skill.id"
        />
      </ListPanel>
    </template>

    <template #right>
      <SkillDetail
        v-if="selectedSkill"
        :selected-skill="selectedSkill"
        :selected-references="selectedReferences"
        :selected-issues="selectedIssues"
        :is-markdown-loading="isMarkdownLoading"
        :skill-markdown="skillMarkdown"
        :parsed-markdown="parsedMarkdown"
        :rendered-markdown="renderedMarkdown"
        :active-detail-tab="activeDetailTab"
        :busy="busy"
        @update:active-detail-tab="activeDetailTab = $event"
        @delete-click="deleteDialogOpen = true"
        @open-add-reference="openAddReferenceDialog"
        @open-delete-reference="openDeleteReferenceDialog"
      />

      <div v-else class="content-empty">{{ t("skills.selectDetail") }}</div>
    </template>
  </SplitPane>

  <!-- Dialogs -->
  <ImportSkillDialog
    :show="importDialogOpen"
    :busy="busy"
    @close="importDialogOpen = false"
    @success="handleSnapshotSuccess"
  />

  <DeleteSkillDialog
    v-if="deleteDialogOpen && selectedSkill"
    :show="deleteDialogOpen"
    :skill-id="selectedSkill.id"
    :skill-name="selectedSkill.name"
    @close="deleteDialogOpen = false"
    @success="handleSnapshotSuccess"
  />

  <ReferenceDialogs
    v-if="referenceDialogOpen && selectedSkill"
    :mode="referenceDialogMode"
    :skill="selectedSkill"
    :reference-to-delete="referenceToDelete"
    :target-profiles="targetProfiles"
    @close="referenceDialogOpen = false"
    @success="handleSnapshotSuccess"
  />
</template>

<style scoped>
.list-search-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 30px;
  gap: 8px;
  align-items: center;
}
</style>

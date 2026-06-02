<script setup lang="ts">
import { computed, ref, watch, inject } from "vue";
import { Plus, Folder, Github, FolderGit, ChevronRight } from "lucide-vue-next";
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

const groupByGitHub = ref(false);

function getGitHubRepo(url: string | null | undefined): string | null {
  if (!url) return null;
  const match = url.match(/(?:github\.com[/:])([^/]+)\/([^/.]+)(?:\.git)?/i);
  if (match) {
    return `${match[1]}/${match[2]}`;
  }
  return null;
}

interface GroupedSkills {
  repoName: string;
  isGitHub: boolean;
  skills: Skill[];
}

const groupedSkills = computed<GroupedSkills[]>(() => {
  if (!groupByGitHub.value) return [];
  const groupsMap: Record<string, Skill[]> = {};
  const localSkills: Skill[] = [];

  for (const skill of skills.value) {
    const gitHubRepo = getGitHubRepo(skill.source?.url);
    if (skill.source?.kind === "github" && gitHubRepo) {
      if (!groupsMap[gitHubRepo]) {
        groupsMap[gitHubRepo] = [];
      }
      groupsMap[gitHubRepo].push(skill);
    } else {
      localSkills.push(skill);
    }
  }

  const result: GroupedSkills[] = [];

  // Sort GitHub groups alphabetically
  const gitHubRepos = Object.keys(groupsMap).sort((a, b) => a.localeCompare(b, "en"));
  for (const repo of gitHubRepos) {
    result.push({
      repoName: repo,
      isGitHub: true,
      skills: groupsMap[repo],
    });
  }

  // Add local skills if any
  if (localSkills.length > 0) {
    result.push({
      repoName: t("skills.localGroup"),
      isGitHub: false,
      skills: localSkills,
    });
  }

  return result;
});

const collapsedGroups = ref<Record<string, boolean>>({});

function toggleGroup(repoName: string) {
  collapsedGroups.value[repoName] = !collapsedGroups.value[repoName];
}

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
            <button
              class="ghost-icon-button"
              :class="{ active: groupByGitHub }"
              type="button"
              :title="t('skills.groupByGithub')"
              :aria-label="t('skills.groupByGithub')"
              @click="groupByGitHub = !groupByGitHub"
            >
              <FolderGit :size="18" />
            </button>
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

        <template v-if="groupByGitHub">
          <div v-for="group in groupedSkills" :key="group.repoName" class="skill-group">
            <div
              class="skill-group-header"
              role="button"
              tabindex="0"
              @click="toggleGroup(group.repoName)"
              @keydown.enter="toggleGroup(group.repoName)"
              @keydown.space.prevent="toggleGroup(group.repoName)"
            >
              <span class="skill-group-chevron" :class="{ collapsed: collapsedGroups[group.repoName] }">
                <ChevronRight :size="14" />
              </span>
              <span class="skill-group-icon">
                <component :is="group.isGitHub ? Github : Folder" :size="14" />
              </span>
              <span class="skill-group-title" :title="group.repoName">{{ group.repoName }}</span>
              <span class="skill-group-count">{{ group.skills.length }}</span>
            </div>
            <div v-if="!collapsedGroups[group.repoName]" class="skill-group-items">
              <SkillListItem
                v-for="skill in group.skills"
                :key="skill.id"
                :skill="skill"
                :is-active="selectedSkill?.id === skill.id"
                :is-referenced="isReferenced(skill)"
                @select="selectedSkillId = skill.id"
              />
            </div>
          </div>
        </template>
        <template v-else>
          <SkillListItem
            v-for="skill in skills"
            :key="skill.id"
            :skill="skill"
            :is-active="selectedSkill?.id === skill.id"
            :is-referenced="isReferenced(skill)"
            @select="selectedSkillId = skill.id"
          />
        </template>
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
  grid-template-columns: 34px minmax(0, 1fr) 30px;
  gap: 8px;
  align-items: center;
}

.ghost-icon-button.active {
  background: var(--brand-100);
  border-color: var(--brand-500);
  color: var(--text-primary);
}

.skill-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: var(--spacing-sm);
}

.skill-group-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: 6px 10px;
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  font-weight: var(--font-weight-medium);
  letter-spacing: 0.05em;
  text-transform: uppercase;
  user-select: none;
  cursor: pointer;
  border-radius: 4px;
}

.skill-group-header:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.skill-group-chevron {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  transform: rotate(90deg);
  transition: transform 0.15s ease;
}

.skill-group-chevron.collapsed {
  transform: rotate(0deg);
}

.skill-group-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.skill-group-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.skill-group-count {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  background: var(--bg-panel-muted);
  padding: 1px 6px;
  border-radius: 10px;
}

.skill-group-items {
  display: grid;
  gap: 2px;
  padding-left: var(--spacing-xs);
  border-left: 1px solid var(--border-default);
  margin-left: 16px;
}
</style>

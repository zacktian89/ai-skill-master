<script setup lang="ts">
import { computed, ref, watch, inject, nextTick, onBeforeUnmount } from "vue";
import {
  FolderPlus,
  Plus,
  Trash2,
  X,
  FolderOpen,
  RefreshCw,
  AlertTriangle,
  MoreHorizontal,
} from "lucide-vue-next";
import { openPath } from "@tauri-apps/plugin-opener";
import * as api from "../../api";
import { openDirectory } from "../../utils/dialog";
import SplitPane from "../../components/SplitPane.vue";
import ListPanel from "../../components/ListPanel.vue";
import SearchInput from "../../components/SearchInput.vue";
import ModalDialog from "../../components/ModalDialog.vue";
import ScannedSkillList from "../../components/ScannedSkillList.vue";
import SkillPreviewPanel from "../../components/SkillPreviewPanel.vue";
import type { AppSnapshot, Project, ProjectRule, ScannedCategory, ScannedSkill, ReferenceScope } from "../../types";
import { AppStoreKey } from "../../stores/useAppStore";
import { SelectionStoreKey } from "../../stores/useSelectionStore";
import { useAsyncAction } from "../../composables/useAsyncAction";
import { useSkillScanner } from "../../composables/useSkillScanner";
import { useSkillPicker } from "../../composables/useSkillPicker";
import { useSkillMarkdown } from "../../composables/useSkillMarkdown";
import { useI18n } from "../../composables/useI18n";
import { getRelPathForTarget, getNameMap, DEFAULT_SKILLS_DIR } from "../../config/agents";

const { t } = useI18n();

const appStore = inject(AppStoreKey, null);
const selectionStore = inject(SelectionStoreKey, null);

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedProjectId: string | null;
}>();

const emit = defineEmits<{
  "select-project": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const snapshot = computed(() => appStore?.snapshot.value ?? props.snapshot);
const selectedProjectId = computed({
  get: () => selectionStore?.selectedProjectId.value ?? props.selectedProjectId,
  set: (val) => {
    if (selectionStore) {
      selectionStore.setSelectedProjectId(val);
    } else {
      emit("select-project", val);
    }
  }
});

const projectQuery = ref("");
const skillQuery = ref("");
const addSkillDialogOpen = ref(false);
const selectedAddDir = ref("");
const selectedAddScope = ref<ReferenceScope>("project");
const selectedAddTargetName = ref("");
const deleteProjectDialogOpen = ref(false);
const pendingReferenceRemoval = ref<{ skillId: string; skillPath: string } | null>(null);
const pendingUnmanagedSkillDeletion = ref<{ skillId: string; skillName: string; skillPath: string } | null>(null);
const previewSkill = ref<ScannedSkill | null>(null);
const listSectionRef = ref<HTMLElement | null>(null);
const lastListScrollTop = ref(0);

// Composable for Async Actions
const { busy, run: executeAsync } = useAsyncAction({
  onError: (err) => {
    if (appStore) appStore.setError(String(err));
    else emit("error", String(err));
  }
});

function projectSkillCount(project: Project): number {
  return Object.values(project.rules).filter((rule) => rule === "enable" || rule === "disable").length;
}

const projects = computed(() => {
  const normalized = projectQuery.value.trim().toLowerCase();
  return [...snapshot.value.state.projects]
    .filter((project) => {
      if (!normalized) return true;
      return `${project.name} ${project.path}`.toLowerCase().includes(normalized);
    })
    .sort((left, right) => {
      const skillBias = projectSkillCount(right) - projectSkillCount(left);
      return skillBias || left.name.localeCompare(right.name, "zh-CN");
    });
});

const selectedProject = computed(
  () => projects.value.find((project) => project.id === selectedProjectId.value) ?? projects.value[0] ?? null
);

const projectProfiles = computed(() => {
  if (!selectedProject.value) return [];
  const projectRoot = selectedProject.value.path;
  return (snapshot.value.targetProfiles || []).map((profile) => {
    const relPath = getRelPathForTarget(profile.targetName, profile.rootPath);
    const fullPath = `${projectRoot}/${relPath}`.replace(/[\\/]+/g, "/");
    return {
      ...profile,
      rootPath: fullPath,
      scope: "project" as const,
    };
  });
});

async function run(action: () => Promise<AppSnapshot>) {
  await executeAsync(action, (next) => {
    if (appStore) appStore.applySnapshot(next);
    else emit("snapshot", next);
  });
}

async function addProject() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      const parts = selected.split(/[\\/]/).filter(Boolean);
      const name = parts[parts.length - 1] ?? selected;
      await run(() => api.addProject({ name, path: selected }));
    }
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
    else emit("error", String(cause));
  }
}

// Composable for Skill Scanner
const {
  scannedCategories,
  scanning,
  scannedSkillsCount,
  loadScan,
  refreshScan
} = useSkillScanner(
  () => selectedProject.value?.path,
  api.scanProjectSkills,
  {
    onError: (err) => {
      if (appStore) appStore.setError(String(err));
      else emit("error", String(err));
    }
  }
);

// Composable for Skill Picker
const {
  addSkillQuery,
  selectedSkillIds,
  filteredLibrarySkills,
  toggleAllLibrarySkills,
  resetPicker
} = useSkillPicker(() => snapshot.value.state.skills || []);

function openAddSkillDialog() {
  resetPicker();
  if (!selectedProject.value) return;

  const projectPath = selectedProject.value.path;
  const rootPath = `${projectPath}/.agents/skills`.replace(/[\\/]+/g, "/");
  selectedAddDir.value = rootPath;

  const matchingProfile = projectProfiles.value.find(
    (profile) => profile.rootPath.replace(/[\\/]+/g, "/").toLowerCase() === rootPath.toLowerCase()
  );

  if (matchingProfile) {
    selectedAddScope.value = matchingProfile.scope;
    selectedAddTargetName.value = matchingProfile.targetName;
  } else {
    selectedAddScope.value = "project";
    const nameMap = getNameMap();
    const mappedName = nameMap[".agents"];
    if (mappedName) {
      selectedAddTargetName.value = mappedName;
    } else {
      selectedAddTargetName.value = "Codex";
    }
  }

  addSkillDialogOpen.value = true;
}

function openAddSkillDialogForCategory(category: ScannedCategory) {
  const rootPath = `${category.path}/${DEFAULT_SKILLS_DIR}`.replace(/[\\/]+/g, "/");
  selectedAddDir.value = rootPath;
  resetPicker();

  const matchingProfile = projectProfiles.value.find(
    (profile) => profile.rootPath.replace(/[\\/]+/g, "/").toLowerCase() === rootPath.toLowerCase()
  );

  if (matchingProfile) {
    selectedAddScope.value = matchingProfile.scope;
    selectedAddTargetName.value = matchingProfile.targetName;
  } else {
    const projectPath = selectedProject.value?.path || "";
    const isInsideProject = rootPath.toLowerCase().startsWith(projectPath.replace(/[\\/]+/g, "/").toLowerCase());
    if (isInsideProject) {
      selectedAddScope.value = "project";
      const nameMap = getNameMap();
      const mappedName = nameMap[category.name];
      if (mappedName) {
        selectedAddTargetName.value = mappedName;
      } else {
        selectedAddTargetName.value = category.name === "." ? (selectedProject.value?.name || t("projects.projectDirectory")) : category.name;
      }
    } else {
      selectedAddScope.value = "custom";
      selectedAddTargetName.value = t("projects.customDirectory");
    }
  }

  addSkillDialogOpen.value = true;
}

function closeAddSkillDialog() {
  addSkillDialogOpen.value = false;
  selectedAddDir.value = "";
  selectedAddScope.value = "project";
  selectedAddTargetName.value = "";
  resetPicker();
}

function openDeleteProjectDialog() {
  if (!selectedProject.value) return;
  deleteProjectDialogOpen.value = true;
}

async function confirmDeleteProject() {
  if (!selectedProject.value) return;
  await executeAsync(
    () => api.deleteProject(selectedProject.value!.id),
    (next) => {
      if (appStore) appStore.applySnapshot(next);
      else emit("snapshot", next);
      selectedProjectId.value = next.state.projects[0]?.id ?? null;
      deleteProjectDialogOpen.value = false;
    }
  );
}

async function selectCustomAddDir() {
  try {
    let defaultPath: string | undefined = undefined;
    if (selectedAddDir.value) {
      const normalizedPath = selectedAddDir.value.replace(/[\\/]+/g, "/");
      const agentsIndex = normalizedPath.lastIndexOf("/.agents");
      if (agentsIndex !== -1) {
        defaultPath = normalizedPath.substring(0, agentsIndex);
      } else {
        const lastSlash = normalizedPath.lastIndexOf("/");
        if (lastSlash !== -1) {
          defaultPath = normalizedPath.substring(0, lastSlash);
        }
      }
    } else if (selectedProject.value) {
      defaultPath = selectedProject.value.path;
    }

    const selected = await openDirectory({
      directory: true,
      multiple: false,
      defaultPath,
    });
    if (typeof selected === "string") {
      selectedAddDir.value = selected;
      const projectPath = selectedProject.value?.path || "";
      const isInsideProject = selected.replace(/[\\/]+/g, "/").toLowerCase().startsWith(projectPath.replace(/[\\/]+/g, "/").toLowerCase());
      if (isInsideProject) {
        selectedAddScope.value = "project";
        selectedAddTargetName.value = selectedProject.value?.name || t("projects.projectDirectory");
      } else {
        selectedAddScope.value = "custom";
        selectedAddTargetName.value = t("projects.customDirectory");
      }
    }
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
    else emit("error", String(cause));
  }
}

async function confirmAddSkillReferences() {
  if (!selectedAddDir.value || selectedSkillIds.value.length === 0) return;
  await executeAsync(
    async () => {
      let currentSnapshot = snapshot.value;
      for (const skillId of selectedSkillIds.value) {
        const request = {
          skillId,
          targetName: selectedAddTargetName.value,
          rootPath: selectedAddDir.value,
          scope: selectedAddScope.value,
          overwrite: true,
        };
        currentSnapshot = await api.addSkillReference(request);
      }
      if (appStore) appStore.applySnapshot(currentSnapshot);
      else emit("snapshot", currentSnapshot);
      closeAddSkillDialog();
      await refreshScan();
    }
  );
}

function setSkillRule(skillId: string, rule: ProjectRule) {
  if (!selectedProject.value) return;
  return run(() => api.setProjectRule({ projectId: selectedProject.value!.id, skillId, rule }));
}

async function toggleSkillRule(skillId: string) {
  if (!selectedProject.value) return;
  const isCurrentlyDisabled = selectedProject.value.rules[skillId] === "disable";
  const newRule = isCurrentlyDisabled ? "enable" : "disable";
  await setSkillRule(skillId, newRule);
}

const conflictState = ref<{ skillId: string; libraryName: string; projectName: string; skillPath: string } | null>(null);

const filteredScannedCategories = computed(() => {
  const normalized = skillQuery.value.trim().toLowerCase();
  if (!normalized) return scannedCategories.value;
  return scannedCategories.value
    .map((category) => {
      const skills = category.skills.filter((skill) => {
        return (
          skill.name.toLowerCase().includes(normalized) ||
          skill.id.toLowerCase().includes(normalized) ||
          (skill.description && skill.description.toLowerCase().includes(normalized)) ||
          skill.path.toLowerCase().includes(normalized)
        );
      });
      return { ...category, skills };
    })
    .filter((category) => category.skills.length > 0);
});

const previewLibrarySkill = computed(
  () => snapshot.value.state.skills.find((skill) => skill.id === previewSkill.value?.id) ?? null
);

const {
  skillMarkdown,
  isMarkdownLoading,
  parsedMarkdown,
  renderedMarkdown,
} = useSkillMarkdown(
  () => previewLibrarySkill.value?.id ?? null,
  undefined,
  () => (previewLibrarySkill.value ? null : previewSkill.value?.path ?? null)
);

function findDetailScrollContainer() {
  return listSectionRef.value?.closest(".detail-panel") as HTMLElement | null;
}

function openSkillPreview(skill: ScannedSkill) {
  lastListScrollTop.value = findDetailScrollContainer()?.scrollTop ?? 0;
  previewSkill.value = skill;
}

async function closeSkillPreview() {
  previewSkill.value = null;
  await nextTick();
  const container = findDetailScrollContainer();
  if (container) {
    container.scrollTop = lastListScrollTop.value;
  }
}

function findReferenceIdForScannedSkill(skillId: string, skillPath: string): string | null {
  const skill = snapshot.value.state.skills.find(s => s.id === skillId);
  if (!skill || !skill.references) return null;
  return skill.references.find(r => r.targetPath.replace(/[\\/]+/g, "/").toLowerCase() === skillPath.replace(/[\\/]+/g, "/").toLowerCase())?.id ?? null;
}

async function removeManagedSkillReference(skillId: string, skillPath: string) {
  pendingReferenceRemoval.value = { skillId, skillPath };
}

async function confirmRemoveManagedSkillReference() {
  if (!pendingReferenceRemoval.value) return;
  const { skillId, skillPath } = pendingReferenceRemoval.value;
  const refId = findReferenceIdForScannedSkill(skillId, skillPath);
  if (!refId) {
    if (appStore) appStore.setError(t("reference.pathConflictError"));
    else emit("error", t("reference.pathConflictError"));
    pendingReferenceRemoval.value = null;
    return;
  }
  await run(() => api.removeSkillReference(refId, true));
  pendingReferenceRemoval.value = null;
  await refreshScan();
}

function deleteUnmanagedSkill(skillId: string, skillName: string, skillPath: string) {
  pendingUnmanagedSkillDeletion.value = { skillId, skillName, skillPath };
}

async function confirmDeleteUnmanagedSkill() {
  if (!pendingUnmanagedSkillDeletion.value) return;
  await run(() => api.deleteUnmanagedSkill(pendingUnmanagedSkillDeletion.value!.skillPath));
  pendingUnmanagedSkillDeletion.value = null;
  await refreshScan();
}

watch(
  () => selectedProject.value?.id,
  (newId, oldId) => {
    if (newId !== oldId) {
      scannedCategories.value = [];
      previewSkill.value = null;
    }
    loadScan();
  },
  { immediate: true }
);

async function handleImportSkill(skillPath: string, strategy?: "overwrite" | "keep_existing") {
  if (!selectedProject.value) return;
  await executeAsync(
    () => api.importProjectSkill(
      selectedProject.value!.name,
      skillPath,
      strategy
    ),
    async (result) => {
      if (result.type === "success") {
        if (appStore) appStore.applySnapshot(result.snapshot);
        else emit("snapshot", result.snapshot);
        conflictState.value = null;
        await refreshScan();
      } else if (result.type === "conflict") {
        conflictState.value = {
          skillId: result.skillId,
          libraryName: result.libraryName,
          projectName: result.projectName,
          skillPath,
        };
      }
    }
  );
}

const headerMenuOpen = ref<{ x: number; y: number } | null>(null);
const headerMenuRef = ref<HTMLElement | null>(null);
let headerMenuCloseTimer: number | null = null;
const menuMargin = 8;
const fallbackMenuWidth = 148;

function closeHeaderMenu() {
  headerMenuOpen.value = null;
  if (headerMenuCloseTimer !== null) {
    window.clearTimeout(headerMenuCloseTimer);
    headerMenuCloseTimer = null;
  }
  document.removeEventListener("click", closeHeaderMenu);
  document.removeEventListener("keydown", headerMenuOnEscape);
}

function headerMenuOnEscape(event: KeyboardEvent) {
  if (event.key === "Escape") closeHeaderMenu();
}

function clampMenuPosition(x: number, y: number, width: number, height: number) {
  const maxX = Math.max(menuMargin, window.innerWidth - width - menuMargin);
  const maxY = Math.max(menuMargin, window.innerHeight - height - menuMargin);
  return {
    x: Math.min(Math.max(menuMargin, x), maxX),
    y: Math.min(Math.max(menuMargin, y), maxY),
  };
}

async function openHeaderMenu(event: MouseEvent) {
  closeHeaderMenu();
  const initialPosition = clampMenuPosition(event.clientX - fallbackMenuWidth, event.clientY, fallbackMenuWidth, 0);
  headerMenuOpen.value = initialPosition;
  await nextTick();
  const menuRect = headerMenuRef.value?.getBoundingClientRect();
  if (headerMenuOpen.value && menuRect) {
    const menuWidth = menuRect.width || fallbackMenuWidth;
    const position = clampMenuPosition(event.clientX - menuWidth, event.clientY, menuWidth, menuRect.height);
    headerMenuOpen.value = position;
  }
  headerMenuCloseTimer = window.setTimeout(() => {
    headerMenuCloseTimer = null;
    document.addEventListener("click", closeHeaderMenu);
    document.addEventListener("keydown", headerMenuOnEscape);
  });
}

function runHeaderMenuAction(action: () => void) {
  action();
  closeHeaderMenu();
}

async function openProjectDirectory(path?: string) {
  const targetPath = path || selectedProject.value?.path;
  if (!targetPath) return;
  try {
    await openPath(targetPath);
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
    else emit("error", String(cause));
  }
}

const contextMenuOpen = ref<{ x: number; y: number; project: Project } | null>(null);
const contextMenuRef = ref<HTMLElement | null>(null);
let contextMenuCloseTimer: number | null = null;

function closeContextMenu() {
  contextMenuOpen.value = null;
  if (contextMenuCloseTimer !== null) {
    window.clearTimeout(contextMenuCloseTimer);
    contextMenuCloseTimer = null;
  }
  document.removeEventListener("click", closeContextMenu);
  document.removeEventListener("keydown", contextMenuOnEscape);
}

function contextMenuOnEscape(event: KeyboardEvent) {
  if (event.key === "Escape") closeContextMenu();
}

async function handleProjectContextMenu(event: MouseEvent, project: Project) {
  event.preventDefault();
  closeContextMenu();
  
  // Select the project first when right-clicking it
  selectedProjectId.value = project.id;
  
  const initialPosition = clampMenuPosition(event.clientX, event.clientY, fallbackMenuWidth, 0);
  contextMenuOpen.value = { ...initialPosition, project };
  
  await nextTick();
  const menuRect = contextMenuRef.value?.getBoundingClientRect();
  if (contextMenuOpen.value && menuRect) {
    const menuWidth = menuRect.width || fallbackMenuWidth;
    const position = clampMenuPosition(event.clientX, event.clientY, menuWidth, menuRect.height);
    contextMenuOpen.value = { ...position, project };
  }
  
  contextMenuCloseTimer = window.setTimeout(() => {
    contextMenuCloseTimer = null;
    document.addEventListener("click", closeContextMenu);
    document.addEventListener("keydown", contextMenuOnEscape);
  });
}

function runContextMenuAction(action: () => void) {
  action();
  closeContextMenu();
}

onBeforeUnmount(closeHeaderMenu);
onBeforeUnmount(closeContextMenu);
</script>

<template>
  <SplitPane class="projects-view">
    <template #left>
      <ListPanel :items="projects" :has-search="true" :empty-text="t('projects.empty')">
        <template #search-row>
          <div class="list-search-row">
            <SearchInput v-model="projectQuery" :placeholder="t('projects.searchPlaceholder')" />
            <button class="icon-button" type="button" :disabled="busy" :aria-label="t('projects.addProject')" @click="addProject">
              <FolderPlus :size="18" />
            </button>
          </div>
        </template>

        <button
          v-for="project in projects"
          :key="project.id"
          class="list-row"
          :class="{ active: selectedProject?.id === project.id }"
          @click="emit('select-project', project.id)"
          @contextmenu.prevent="handleProjectContextMenu($event, project)"
        >
          <div class="list-row-copy">
            <strong>{{ project.name }}</strong>
            <small>{{ project.path }}</small>
          </div>
        </button>
      </ListPanel>
    </template>

    <template #right>
      <template v-if="selectedProject">
        <SkillPreviewPanel
          v-if="previewSkill"
          :skill="previewSkill"
          :library-skill="previewLibrarySkill"
          :rule="selectedProject.rules[previewSkill.id]"
          :busy="busy"
          :show-category-title="true"
          :is-markdown-loading="isMarkdownLoading"
          :skill-markdown="skillMarkdown"
          :parsed-markdown="parsedMarkdown"
          :rendered-markdown="renderedMarkdown"
          @back="closeSkillPreview"
          @toggle-rule="toggleSkillRule"
          @remove-reference="removeManagedSkillReference"
          @import-skill="handleImportSkill"
          @delete-unmanaged-skill="deleteUnmanagedSkill"
        />
        <template v-else>
          <div class="detail-header">
            <div>
              <h2>{{ selectedProject.name }}</h2>
              <p>{{ selectedProject.path }}</p>
            </div>
            <div class="detail-actions">
              <button
                class="ghost-icon-button"
                type="button"
                :disabled="busy"
                :aria-label="t('projects.moreActions')"
                :title="t('projects.moreActions')"
                @click.stop="openHeaderMenu"
              >
                <MoreHorizontal :size="16" />
              </button>
            </div>
          </div>

          <Teleport to="body">
            <div
              v-if="headerMenuOpen"
              ref="headerMenuRef"
              class="global-context-menu"
              :style="{ left: `${headerMenuOpen.x}px`, top: `${headerMenuOpen.y}px` }"
              role="menu"
              @click.stop
            >
              <button
                type="button"
                role="menuitem"
                class="global-context-menu-item"
                :disabled="busy"
                @click="runHeaderMenuAction(openAddSkillDialog)"
              >
                <Plus :size="15" />
                <span>{{ t('projects.addSkill') }}</span>
              </button>

              <button
                type="button"
                role="menuitem"
                class="global-context-menu-item"
                :disabled="busy"
                @click="runHeaderMenuAction(openDeleteProjectDialog)"
              >
                <Trash2 :size="15" />
                <span>{{ t('projects.cancelManage') }}</span>
              </button>

              <button
                type="button"
                role="menuitem"
                class="global-context-menu-item"
                :disabled="busy"
                @click="runHeaderMenuAction(openProjectDirectory)"
              >
                <FolderOpen :size="15" />
                <span>{{ t('projects.openProjectDir') }}</span>
              </button>
            </div>
          </Teleport>

          <section ref="listSectionRef" class="detail-section">
            <div class="project-skill-toolbar">
              <span class="search-row-count">{{ scannedSkillsCount }}</span>
              <SearchInput v-model="skillQuery" :placeholder="t('projects.searchSkillPlaceholder')" class="detail-search-input" />
              <button class="ghost-icon-button" type="button" :disabled="busy || scanning" :aria-label="t('projects.rescan')" :title="t('projects.rescan')" @click="refreshScan">
                <RefreshCw :size="14" :class="{ 'spin-animation': scanning }" />
              </button>
            </div>

            <ScannedSkillList
              v-if="filteredScannedCategories.length"
              :categories="filteredScannedCategories"
              :rules="selectedProject.rules"
              :busy="busy"
              :show-category-title="true"
              :show-add-button="true"
              @preview-skill="openSkillPreview"
              @add-skill-click="openAddSkillDialogForCategory"
              @toggle-rule="toggleSkillRule"
              @remove-reference="removeManagedSkillReference"
              @import-skill="handleImportSkill"
              @delete-unmanaged-skill="deleteUnmanagedSkill"
            />
          </section>
        </template>
      </template>

      <div v-else class="content-empty">{{ t('projects.selectProjectDetail') }}</div>
    </template>
  </SplitPane>

  <ModalDialog
    v-if="addSkillDialogOpen && selectedProject"
    :title="t('projects.addSkill')"
    card-class="modal-card--compact"
    @close="closeAddSkillDialog"
  >
    <div class="modal-step-section">
      <!-- Header displaying chosen path -->
      <div class="chosen-path-box">
        <div class="chosen-path-copy">
          <span class="chosen-path-label">{{ t('projects.targetReferenceDir') }}</span>
          <code class="chosen-path-value">
            {{ selectedAddDir }}
          </code>
        </div>
        <button
          class="secondary-button chosen-path-edit-btn"
          type="button"
          :disabled="busy"
          @click="selectCustomAddDir"
        >
          {{ t('projects.modifyDir') }}
        </button>
      </div>

      <!-- Search box inside dialog -->
      <SearchInput v-model="addSkillQuery" :placeholder="t('projects.searchSkillPlaceholder')" class="dialog-search-input" />

      <!-- Check all / Selected count -->
      <div class="dialog-checklist-header">
        <label class="check-all-label">
          <input
            type="checkbox"
            :checked="filteredLibrarySkills.length > 0 && selectedSkillIds.length === filteredLibrarySkills.length"
            :disabled="filteredLibrarySkills.length === 0 || busy"
            @change="toggleAllLibrarySkills(($event.target as HTMLInputElement).checked)"
          />
          <span>{{ t('projects.checkAll') }}</span>
        </label>
        <span>{{ t('projects.selectedSkillsCount', { selected: selectedSkillIds.length, total: filteredLibrarySkills.length }) }}</span>
      </div>

      <!-- Skill list with checkboxes -->
      <div v-if="filteredLibrarySkills.length" class="project-skill-picker">
        <label
          v-for="skill in filteredLibrarySkills"
          :key="skill.id"
          class="dialog-skill-pick-row"
          :class="{ disabled: busy }"
        >
          <input
            type="checkbox"
            :value="skill.id"
            v-model="selectedSkillIds"
            :disabled="busy"
          />
          <span class="dialog-skill-info">
            <strong>{{ skill.name }}</strong>
            <small>
              <code>{{ skill.id }}</code>
              <span v-if="skill.description"> · {{ skill.description }}</span>
            </small>
          </span>
        </label>
      </div>
      <div v-else class="content-empty content-empty--compact">{{ t('projects.noMatchingSkills') }}</div>
    </div>

    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="closeAddSkillDialog">{{ t('dialog.cancel') }}</button>
        <button
          v-if="selectedAddDir"
          class="primary-button"
          :disabled="busy || selectedSkillIds.length === 0"
          @click="confirmAddSkillReferences"
        >
          {{ t('dialog.confirm') }}
        </button>
      </div>
    </template>
  </ModalDialog>

  <ModalDialog
    v-if="pendingUnmanagedSkillDeletion"
    :title="t('deleteUnmanaged.title')"
    @close="pendingUnmanagedSkillDeletion = null"
  >
    <p class="modal-note">
      {{ t('deleteUnmanaged.note', { name: pendingUnmanagedSkillDeletion.skillName }) }}
    </p>
    <dl class="modal-summary">
      <dt>{{ t('deleteUnmanaged.path') }}</dt>
      <dd>{{ pendingUnmanagedSkillDeletion.skillPath }}</dd>
    </dl>
    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="pendingUnmanagedSkillDeletion = null">{{ t('dialog.cancel') }}</button>
        <button class="danger-button" :disabled="busy" @click="confirmDeleteUnmanagedSkill">
          {{ t('deleteUnmanaged.deleteFolder') }}
        </button>
      </div>
    </template>
  </ModalDialog>

  <ModalDialog
    v-if="deleteProjectDialogOpen && selectedProject"
    :title="t('deleteProject.title')"
    @close="deleteProjectDialogOpen = false"
  >
    <p class="modal-note">
      {{ t('deleteProject.note', { name: selectedProject.name }) }}
    </p>
    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="deleteProjectDialogOpen = false">{{ t('dialog.cancel') }}</button>
        <button class="danger-button" :disabled="busy" @click="confirmDeleteProject">
          {{ t('deleteProject.deleteProjectButton') }}
        </button>
      </div>
    </template>
  </ModalDialog>

  <ModalDialog
    v-if="pendingReferenceRemoval"
    :title="t('deleteRefProj.title')"
    @close="pendingReferenceRemoval = null"
  >
    <p class="modal-note">
      {{ t('deleteRefProj.note') }}
    </p>
    <template #footer>
      <div class="button-row button-row--end dialog-footer-row">
        <button class="secondary-button" :disabled="busy" @click="pendingReferenceRemoval = null">{{ t('dialog.cancel') }}</button>
        <button class="danger-button" :disabled="busy" @click="confirmRemoveManagedSkillReference">
          {{ t('deleteRefProj.removeReferenceButton') }}
        </button>
      </div>
    </template>
  </ModalDialog>

  <ModalDialog
    v-if="conflictState"
    :is-conflict="true"
    @close="conflictState = null"
  >
    <template #header>
      <div class="modal-title-row header-warning-title">
        <div class="conflict-title-container">
          <AlertTriangle :size="20" />
          <h2 class="conflict-modal-title">{{ t('projects.importConflictTitle') }}</h2>
        </div>
        <button class="ghost-icon-button" type="button" :aria-label="t('dialog.close')" @click="conflictState = null">
          <X :size="16" />
        </button>
      </div>
    </template>

    <div class="conflict-modal-body">
      <p>{{ t('projects.importConflictDesc', { id: conflictState.skillId }) }}</p>
      
      <div class="conflict-compare-box">
        <div class="conflict-compare-item">
          <span class="conflict-compare-label">{{ t('projects.projectLocalVersion') }}</span>
          <span class="conflict-compare-value">{{ conflictState.projectName }}</span>
        </div>
        <div class="conflict-compare-item">
          <span class="conflict-compare-label">{{ t('projects.libraryExistingVersion') }}</span>
          <span class="conflict-compare-value">{{ conflictState.libraryName }}</span>
        </div>
      </div>
      
      <p class="conflict-explanation">
        {{ t('projects.conflictExplain') }}
      </p>
    </div>

    <template #footer>
      <div class="conflict-modal-footer">
        <button class="ghost-button" :disabled="busy" @click="conflictState = null">
          {{ t('dialog.cancel') }}
        </button>
        <button class="primary-button" :disabled="busy" @click="handleImportSkill(conflictState.skillPath, 'keep_existing')">
          {{ t('projects.keepExisting') }}
        </button>
        <button class="primary-button danger-override-btn" :disabled="busy" @click="handleImportSkill(conflictState.skillPath, 'overwrite')">
          {{ t('projects.overwriteExisting') }}
        </button>
      </div>
    </template>
  </ModalDialog>

  <Teleport to="body">
    <div
      v-if="contextMenuOpen"
      ref="contextMenuRef"
      class="global-context-menu"
      :style="{ left: `${contextMenuOpen.x}px`, top: `${contextMenuOpen.y}px` }"
      role="menu"
      @click.stop
    >
      <button
        type="button"
        role="menuitem"
        class="global-context-menu-item"
        :disabled="busy"
        @click="runContextMenuAction(openAddSkillDialog)"
      >
        <Plus :size="15" />
        <span>{{ t('projects.addSkill') }}</span>
      </button>

      <button
        type="button"
        role="menuitem"
        class="global-context-menu-item"
        :disabled="busy"
        @click="runContextMenuAction(openDeleteProjectDialog)"
      >
        <Trash2 :size="15" />
        <span>{{ t('projects.cancelManage') }}</span>
      </button>

      <button
        type="button"
        role="menuitem"
        class="global-context-menu-item"
        :disabled="busy"
        @click="runContextMenuAction(() => openProjectDirectory(contextMenuOpen?.project?.path))"
      >
        <FolderOpen :size="15" />
        <span>{{ t('projects.openProjectDir') }}</span>
      </button>
    </div>
  </Teleport>
</template>


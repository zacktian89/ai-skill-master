<script setup lang="ts">
import { computed, ref, inject } from "vue";
import { Folder, Plus, Trash2 } from "lucide-vue-next";
import * as api from "../../../api";
import { openDirectory } from "../../../utils/dialog";
import AgentIcon from "../../../components/icons/AgentIcon.vue";
import ModalDialog from "../../../components/ModalDialog.vue";
import SearchInput from "../../../components/SearchInput.vue";
import { AppStoreKey } from "../../../stores/useAppStore";
import { useAsyncAction } from "../../../composables/useAsyncAction";
import { useI18n } from "../../../composables/useI18n";
import type {
  AppSnapshot,
  Skill,
  SkillTargetProfile,
  ReferenceScope,
  AddSkillReferenceRequest,
  SkillReferenceDetail,
} from "../../../types";

type DialogMode = "add" | "delete";

interface PendingReferenceTarget {
  targetName: string;
  rootPath: string;
  scope: ReferenceScope;
}

const props = defineProps<{
  mode: DialogMode;
  skill: Skill;
  referenceToDelete: SkillReferenceDetail | null;
  targetProfiles: SkillTargetProfile[];
}>();

const emit = defineEmits<{
  close: [];
  success: [nextSnapshot: AppSnapshot];
}>();

const appStore = inject(AppStoreKey, null);
const { t } = useI18n();

const { busy, run: executeAsync } = useAsyncAction({
  onError: (err) => {
    if (appStore) appStore.setError(String(err));
  }
});

// Add Reference States
const pendingReferenceTarget = ref<PendingReferenceTarget | null>(null);
const overwriteReferenceRequest = ref<AddSkillReferenceRequest | null>(null);
const profileQuery = ref("");

const filteredTargetProfiles = computed(() => {
  const normalized = profileQuery.value.trim().toLowerCase();
  if (!normalized) return props.targetProfiles;
  return props.targetProfiles.filter((p) =>
    p.targetName.toLowerCase().includes(normalized)
  );
});

// Delete Reference States
const removeReferenceConflictRequest = ref<{ referenceId: string; symlinkPath: string } | null>(null);

function joinPath(root: string, child: string): string {
  const normalized = root.replace(/[\\/]+$/, "");
  return `${normalized}/${child}`;
}

const pendingReferencePath = computed(() => {
  if (!props.skill || !pendingReferenceTarget.value) return "";
  return joinPath(pendingReferenceTarget.value.rootPath, props.skill.id);
});

const addReferenceTitle = computed(() => {
  if (overwriteReferenceRequest.value) return t("reference.overwriteTitle");
  return pendingReferenceTarget.value ? t("reference.confirmAddTitle") : t("reference.addTitle");
});

function isRetargetedLinkError(cause: unknown): boolean {
  return String(cause).includes("已指向其他位置");
}

// Add Reference Actions
function selectTargetProfile(profile: SkillTargetProfile) {
  overwriteReferenceRequest.value = null;
  pendingReferenceTarget.value = {
    targetName: profile.targetName,
    rootPath: profile.rootPath,
    scope: profile.scope,
  };
}

async function selectCustomReferenceRoot() {
  try {
    const selected = await openDirectory({ directory: true, multiple: false });
    if (typeof selected === "string") {
      overwriteReferenceRequest.value = null;
      pendingReferenceTarget.value = {
        targetName: t("reference.customDirectory"),
        rootPath: selected,
        scope: "custom",
      };
    }
  } catch (cause) {
    if (appStore) appStore.setError(String(cause));
  }
}

async function confirmAddReference() {
  if (!props.skill || !pendingReferenceTarget.value) return;
  const request: AddSkillReferenceRequest = {
    skillId: props.skill.id,
    targetName: pendingReferenceTarget.value.targetName,
    rootPath: pendingReferenceTarget.value.rootPath,
    scope: pendingReferenceTarget.value.scope,
  };
  await executeAsync(
    () => api.addSkillReference(request),
    (nextSnapshot) => {
      emit("success", nextSnapshot);
      emit("close");
    },
    (cause) => {
      if (!isRetargetedLinkError(cause)) {
        if (appStore) appStore.setError(String(cause));
        return;
      }
      overwriteReferenceRequest.value = request;
    }
  );
}

async function confirmOverwriteReference() {
  if (!overwriteReferenceRequest.value) return;
  await executeAsync(
    () => api.addSkillReference({ ...overwriteReferenceRequest.value!, overwrite: true }),
    (nextSnapshot) => {
      emit("success", nextSnapshot);
      emit("close");
    }
  );
}

function cancelOverwriteReference() {
  overwriteReferenceRequest.value = null;
}

// Delete Reference Actions
async function confirmDeleteReference() {
  if (!props.referenceToDelete || !props.skill) return;
  const reference = props.referenceToDelete;
  await executeAsync(
    () => api.removeSkillReference(reference.id),
    (nextSnapshot) => {
      emit("success", nextSnapshot);
      emit("close");
    },
    (cause) => {
      if (!isRetargetedLinkError(cause)) {
        if (appStore) appStore.setError(String(cause));
        return;
      }
      removeReferenceConflictRequest.value = {
        referenceId: reference.id,
        symlinkPath: reference.symlinkPath,
      };
    }
  );
}

async function confirmDeleteReferenceWithLink(removeExternalLink: boolean) {
  if (!removeReferenceConflictRequest.value) return;
  const { referenceId } = removeReferenceConflictRequest.value;
  await executeAsync(
    () => api.removeSkillReference(referenceId, removeExternalLink),
    (nextSnapshot) => {
      emit("success", nextSnapshot);
      emit("close");
    }
  );
}
</script>

<template>
  <!-- ADD REFERENCE DIALOG -->
  <ModalDialog
    v-if="mode === 'add'"
    :title="addReferenceTitle"
    card-class="modal-card--compact"
    @close="$emit('close')"
  >
    <template v-if="overwriteReferenceRequest">
      <p class="modal-note">{{ t('reference.overwriteNote') }}</p>

      <dl class="detail-kv detail-kv--wide">
        <div>
          <dt>{{ t('reference.targetPath') }}</dt>
          <dd>
            <code class="reference-path">{{ pendingReferencePath }}</code>
          </dd>
        </div>
      </dl>

      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="cancelOverwriteReference">{{ t('dialog.cancel') }}</button>
        <button class="primary-button" :disabled="busy" @click="confirmOverwriteReference">
          <Plus :size="16" />
          {{ t('reference.overwriteReference') }}
        </button>
      </div>
    </template>

    <template v-else-if="!pendingReferenceTarget">
      <!-- Search Input for target profiles -->
      <div class="preset-search-row" style="margin-bottom: 12px;">
        <SearchInput v-model="profileQuery" :placeholder="t('agents.searchPresetPlaceholder') || '搜索目标...'" />
      </div>

      <div class="target-profiles-scroll-wrapper" style="max-height: 260px; overflow-y: auto; margin-bottom: 12px; border: 1px solid var(--border-default); border-radius: 8px; padding: 8px; background: var(--bg-panel);">
        <div class="target-grid">
          <button
            v-for="profile in filteredTargetProfiles"
            :key="profile.id"
            class="target-tile"
            type="button"
            :disabled="busy"
            @click="selectTargetProfile(profile)"
          >
            <span class="target-tile-icon" aria-hidden="true">
              <AgentIcon :name="profile.targetName" :size="22" />
            </span>
            <strong>{{ profile.targetName }}</strong>
          </button>
        </div>
      </div>

      <button class="target-custom-button" type="button" :disabled="busy" @click="selectCustomReferenceRoot" style="width: 100%;">
        <Folder :size="18" />
        {{ t('reference.selectSkillsDirButton') }}
      </button>
    </template>

    <template v-else>
      <dl class="detail-kv detail-kv--wide">
        <div>
          <dt>{{ t('reference.targetPath') }}</dt>
          <dd>
            <code class="reference-path">{{ pendingReferencePath }}</code>
          </dd>
        </div>
      </dl>

      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="pendingReferenceTarget = null">{{ t('reference.back') }}</button>
        <button class="primary-button" :disabled="busy" @click="confirmAddReference">
          <Plus :size="16" />
          {{ t('reference.addTitle') }}
        </button>
      </div>
    </template>
  </ModalDialog>

  <!-- DELETE REFERENCE DIALOG -->
  <ModalDialog
    v-if="mode === 'delete' && referenceToDelete"
    :title="t('reference.deleteTitle')"
    card-class="modal-card--compact"
    @close="$emit('close')"
  >
    <template v-if="removeReferenceConflictRequest">
      <p class="modal-note">{{ t('reference.deleteConflictNote') }}</p>

      <dl class="detail-kv detail-kv--wide">
        <div>
          <dt>{{ t('reference.targetPath') }}</dt>
          <dd>
            <code class="reference-path">{{ removeReferenceConflictRequest.symlinkPath }}</code>
          </dd>
        </div>
      </dl>

      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="$emit('close')">{{ t('dialog.cancel') }}</button>
        <button class="secondary-button" :disabled="busy" @click="confirmDeleteReferenceWithLink(false)">
          {{ t('reference.deleteConflictNo') }}
        </button>
        <button class="danger-button" :disabled="busy" @click="confirmDeleteReferenceWithLink(true)">
          {{ t('reference.deleteConflictYes') }}
        </button>
      </div>
    </template>

    <template v-else>
      <dl class="detail-kv detail-kv--wide">
        <div>
          <dt>{{ t('reference.targetPath') }}</dt>
          <dd>
            <code class="reference-path">{{ referenceToDelete.symlinkPath }}</code>
          </dd>
        </div>
      </dl>
      <p class="modal-note">{{ t('reference.deleteNote') }}</p>

      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="$emit('close')">{{ t('dialog.cancel') }}</button>
        <button class="danger-button" :disabled="busy" @click="confirmDeleteReference">
          <Trash2 :size="16" />
          {{ t('reference.deleteButton') }}
        </button>
      </div>
    </template>
  </ModalDialog>
</template>

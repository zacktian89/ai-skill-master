<script setup lang="ts">
import { computed, ref, inject } from "vue";
import { Folder, Plus, Trash2 } from "lucide-vue-next";
import * as api from "../../../api";
import { openDirectory } from "../../../utils/dialog";
import AgentIcon from "../../../components/icons/AgentIcon.vue";
import ModalDialog from "../../../components/ModalDialog.vue";
import { AppStoreKey } from "../../../stores/useAppStore";
import { useAsyncAction } from "../../../composables/useAsyncAction";
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

const { busy, run: executeAsync } = useAsyncAction({
  onError: (err) => {
    if (appStore) appStore.setError(String(err));
  }
});

// Add Reference States
const pendingReferenceTarget = ref<PendingReferenceTarget | null>(null);
const overwriteReferenceRequest = ref<AddSkillReferenceRequest | null>(null);

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
  if (overwriteReferenceRequest.value) return "覆盖引用链接";
  return pendingReferenceTarget.value ? "确认新增引用" : "新增引用";
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
        targetName: "自定义目录",
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
      <p class="modal-note">引用链接已存在，且指向其他位置。覆盖后会把它改为当前 skill 的托管链接。</p>

      <dl class="detail-kv detail-kv--wide">
        <div>
          <dt>目标路径</dt>
          <dd>
            <code class="reference-path">{{ pendingReferencePath }}</code>
          </dd>
        </div>
      </dl>

      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="cancelOverwriteReference">取消</button>
        <button class="primary-button" :disabled="busy" @click="confirmOverwriteReference">
          <Plus :size="16" />
          覆盖引用
        </button>
      </div>
    </template>

    <template v-else-if="!pendingReferenceTarget">
      <div class="target-grid">
        <button
          v-for="profile in targetProfiles"
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

      <button class="target-custom-button" type="button" :disabled="busy" @click="selectCustomReferenceRoot">
        <Folder :size="18" />
        选择 skills 目录
      </button>
    </template>

    <template v-else>
      <dl class="detail-kv detail-kv--wide">
        <div>
          <dt>目标路径</dt>
          <dd>
            <code class="reference-path">{{ pendingReferencePath }}</code>
          </dd>
        </div>
      </dl>

      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="pendingReferenceTarget = null">返回</button>
        <button class="primary-button" :disabled="busy" @click="confirmAddReference">
          <Plus :size="16" />
          新增引用
        </button>
      </div>
    </template>
  </ModalDialog>

  <!-- DELETE REFERENCE DIALOG -->
  <ModalDialog
    v-if="mode === 'delete' && referenceToDelete"
    title="删除引用"
    card-class="modal-card--compact"
    @close="$emit('close')"
  >
    <template v-if="removeReferenceConflictRequest">
      <p class="modal-note">托管链接已指向其他位置（或存在内容冲突）。是否同时删除该外部链接？</p>

      <dl class="detail-kv detail-kv--wide">
        <div>
          <dt>目标路径</dt>
          <dd>
            <code class="reference-path">{{ removeReferenceConflictRequest.symlinkPath }}</code>
          </dd>
        </div>
      </dl>

      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="$emit('close')">取消</button>
        <button class="secondary-button" :disabled="busy" @click="confirmDeleteReferenceWithLink(false)">
          否（只移除记录）
        </button>
        <button class="danger-button" :disabled="busy" @click="confirmDeleteReferenceWithLink(true)">
          是（删除外部链接）
        </button>
      </div>
    </template>

    <template v-else>
      <dl class="detail-kv detail-kv--wide">
        <div>
          <dt>目标路径</dt>
          <dd>
            <code class="reference-path">{{ referenceToDelete.symlinkPath }}</code>
          </dd>
        </div>
      </dl>
      <p class="modal-note">只移除这个托管引用，不会删除技能库中的 skill。</p>

      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="$emit('close')">取消</button>
        <button class="danger-button" :disabled="busy" @click="confirmDeleteReference">
          <Trash2 :size="16" />
          删除引用
        </button>
      </div>
    </template>
  </ModalDialog>
</template>

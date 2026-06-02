<script setup lang="ts">
import { onMounted, ref, watch, inject } from "vue";
import { Trash2 } from "lucide-vue-next";
import * as api from "../../../api";
import ModalDialog from "../../../components/ModalDialog.vue";
import { AppStoreKey } from "../../../stores/useAppStore";
import { useAsyncAction } from "../../../composables/useAsyncAction";
import type { AppSnapshot, DeleteSkillPreview } from "../../../types";
import { useI18n } from "../../../composables/useI18n";

const props = defineProps<{
  show: boolean;
  skillId: string;
  skillName: string;
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

const deletePreview = ref<DeleteSkillPreview | null>(null);

async function loadPreview() {
  if (!props.skillId) return;
  await executeAsync(
    () => api.previewDeleteSkill(props.skillId),
    (preview) => {
      deletePreview.value = preview;
    }
  );
}

async function confirmDelete() {
  if (!props.skillId) return;
  await executeAsync(
    () => api.deleteSkill(props.skillId),
    (nextSnapshot) => {
      emit("success", nextSnapshot);
      emit("close");
    }
  );
}

onMounted(() => {
  loadPreview();
});

watch(() => props.skillId, () => {
  loadPreview();
});
</script>

<template>
  <ModalDialog
    v-if="show && deletePreview"
    :show-close="false"
    @close="$emit('close')"
  >
    <template #header>
      <div class="detail-header">
        <div>
          <h2>{{ t('deleteSkill.title', { name: deletePreview.skillName }) }}</h2>
          <p>{{ t('deleteSkill.note') }}</p>
        </div>
      </div>
    </template>

    <dl class="detail-kv detail-kv--wide">
      <div>
        <dt>{{ t('deleteSkill.libraryPath') }}</dt>
        <dd>{{ deletePreview.libraryPath }}</dd>
      </div>
      <div>
        <dt>{{ t('deleteSkill.managedLinkTargets') }}</dt>
        <dd>
          <template v-if="deletePreview.managedLinkTargets.length">
            <div class="meta-stack">
              <span v-for="target in deletePreview.managedLinkTargets" :key="target">{{ target }}</span>
            </div>
          </template>
          <template v-else>{{ t('deleteSkill.none') }}</template>
        </dd>
      </div>
      <div>
        <dt>{{ t('deleteSkill.affectedProjects') }}</dt>
        <dd>
          <template v-if="deletePreview.affectedProjects.length">
            <div class="meta-stack">
              <span v-for="project in deletePreview.affectedProjects" :key="project.projectId">
                {{ project.projectName }} · {{ project.projectPath }}
              </span>
            </div>
          </template>
          <template v-else>{{ t('deleteSkill.none') }}</template>
        </dd>
      </div>
    </dl>

    <template #footer>
      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="$emit('close')">{{ t('dialog.cancel') }}</button>
        <button class="danger-button" :disabled="busy" @click="confirmDelete">
          <Trash2 :size="16" />
          {{ t('deleteSkill.confirmDelete') }}
        </button>
      </div>
    </template>
  </ModalDialog>
</template>

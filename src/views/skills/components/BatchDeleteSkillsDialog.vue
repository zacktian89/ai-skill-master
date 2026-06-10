<script setup lang="ts">
import { onMounted, ref, watch, inject } from "vue";
import { Trash2 } from "lucide-vue-next";
import * as api from "../../../api";
import ModalDialog from "../../../components/ModalDialog.vue";
import { AppStoreKey } from "../../../stores/useAppStore";
import { useAsyncAction } from "../../../composables/useAsyncAction";
import type { AppSnapshot, DeleteSkillsPreview } from "../../../types";
import { useI18n } from "../../../composables/useI18n";

const props = defineProps<{
  show: boolean;
  skillIds: string[];
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

const deletePreview = ref<DeleteSkillsPreview | null>(null);

async function loadPreview() {
  if (!props.skillIds.length) return;
  await executeAsync(
    () => api.previewDeleteSkills(props.skillIds),
    (preview) => {
      deletePreview.value = preview;
    }
  );
}

async function confirmDelete() {
  if (!props.skillIds.length) return;
  await executeAsync(
    () => api.deleteSkills(props.skillIds),
    (nextSnapshot) => {
      emit("success", nextSnapshot);
      emit("close");
    }
  );
}

onMounted(() => {
  loadPreview();
});

watch(() => props.skillIds.join("\n"), () => {
  deletePreview.value = null;
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
          <h2>{{ t('deleteSkill.batchTitle') }}</h2>
          <p>{{ t('deleteSkill.batchNote', { count: deletePreview.items.length }) }}</p>
        </div>
      </div>
    </template>

    <div class="batch-delete-summary">
      <span>{{ t('deleteSkill.batchManagedLinks', { count: deletePreview.totalManagedLinkTargets }) }}</span>
      <span>{{ t('deleteSkill.batchAffectedProjects', { count: deletePreview.totalAffectedProjects }) }}</span>
    </div>

    <div class="batch-delete-list">
      <details
        v-for="item in deletePreview.items"
        :key="item.skillId"
        class="batch-delete-item"
      >
        <summary>
          <strong>{{ item.skillName }}</strong>
          <code>{{ item.skillId }}</code>
        </summary>
        <dl class="detail-kv detail-kv--wide">
          <div>
            <dt>{{ t('deleteSkill.libraryPath') }}</dt>
            <dd>{{ item.libraryPath }}</dd>
          </div>
          <div>
            <dt>{{ t('deleteSkill.managedLinkTargets') }}</dt>
            <dd>
              <template v-if="item.managedLinkTargets.length">
                <div class="meta-stack">
                  <span v-for="target in item.managedLinkTargets" :key="target">{{ target }}</span>
                </div>
              </template>
              <template v-else>{{ t('deleteSkill.none') }}</template>
            </dd>
          </div>
          <div>
            <dt>{{ t('deleteSkill.affectedProjects') }}</dt>
            <dd>
              <template v-if="item.affectedProjects.length">
                <div class="meta-stack">
                  <span v-for="project in item.affectedProjects" :key="project.projectId">
                    {{ project.projectName }} · {{ project.projectPath }}
                  </span>
                </div>
              </template>
              <template v-else>{{ t('deleteSkill.none') }}</template>
            </dd>
          </div>
        </dl>
      </details>
    </div>

    <template #footer>
      <div class="button-row button-row--end">
        <button class="secondary-button" :disabled="busy" @click="$emit('close')">{{ t('dialog.cancel') }}</button>
        <button class="danger-button" :disabled="busy" @click="confirmDelete">
          <Trash2 :size="16" />
          {{ t('deleteSkill.confirmBatchDelete', { count: deletePreview.items.length }) }}
        </button>
      </div>
    </template>
  </ModalDialog>
</template>

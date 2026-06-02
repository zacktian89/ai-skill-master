<script setup lang="ts">
import { computed } from "vue";
import { Trash2 } from "lucide-vue-next";
import AgentIcon from "../../../components/icons/AgentIcon.vue";
import StatusTag from "../../../components/StatusTag.vue";
import type { SkillReferenceDetail } from "../../../types";
import { useI18n } from "../../../composables/useI18n";

defineProps<{
  selectedReferences: SkillReferenceDetail[];
  busy: boolean;
}>();

defineEmits<{
  "open-delete-reference": [reference: SkillReferenceDetail];
}>();

const { t } = useI18n();

const scopeLabels = computed<Record<string, string>>(() => ({
  user: t("skills.refScopeUser"),
  project: t("skills.refScopeProject"),
  custom: t("skills.refScopeCustom"),
}));

const referenceStatusLabels = computed<Record<string, string>>(() => ({
  healthy: t("skills.refStatusHealthy"),
  missing: t("skills.refStatusMissing"),
  conflict: t("skills.refStatusConflict"),
  stale: t("skills.refStatusStale"),
}));
</script>

<template>
  <section class="reference-pane">
    <div v-if="selectedReferences.length" class="reference-list">
      <article
        v-for="reference in selectedReferences"
        :key="reference.id"
        class="reference-row"
      >
        <div class="reference-row-main">
          <div class="reference-row-top">
            <div class="reference-title">
              <span class="reference-app-icon" :title="reference.targetName" aria-hidden="true">
                <AgentIcon :name="reference.targetName" :size="15" />
              </span>
              <strong>{{ reference.targetName }}</strong>
            </div>
            <div class="reference-actions">
              <StatusTag :type="reference.status">
                {{ referenceStatusLabels[reference.status] || reference.status }}
              </StatusTag>
              <StatusTag>{{ scopeLabels[reference.scope] || reference.scope }}</StatusTag>
              <button
                v-if="reference.removable"
                class="ghost-icon-button ghost-icon-button--danger"
                type="button"
                :disabled="busy"
                :aria-label="t('skills.deleteReference')"
                :title="t('skills.deleteReference')"
                @click="$emit('open-delete-reference', reference)"
              >
                <Trash2 :size="15" />
              </button>
            </div>
          </div>
          <code class="reference-path">{{ reference.symlinkPath }}</code>
        </div>
      </article>
    </div>

    <!-- Empty View -->
    <div v-else class="content-empty content-empty--compact">
      {{ t('skills.noReferences') }}
    </div>
  </section>
</template>

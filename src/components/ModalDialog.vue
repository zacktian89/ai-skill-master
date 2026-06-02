<script setup lang="ts">
import { X } from "lucide-vue-next";
import { useI18n } from "../composables/useI18n";

interface Props {
  title?: string;
  showClose?: boolean;
  cardClass?: string;
  isConflict?: boolean;
}

withDefaults(defineProps<Props>(), {
  title: "",
  showClose: true,
  cardClass: "",
  isConflict: false,
});

defineEmits<{
  close: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="modal-backdrop" @click.self="$emit('close')">
    <section :class="isConflict ? 'conflict-modal-card' : ['modal-card', cardClass]">
      <slot name="header">
        <div class="modal-title-row">
          <div>
            <h2>{{ title }}</h2>
          </div>
          <button
            v-if="showClose"
            class="ghost-icon-button"
            type="button"
            :aria-label="t('dialog.close')"
            @click="$emit('close')"
          >
            <X :size="16" />
          </button>
        </div>
      </slot>
      <slot></slot>
      <slot name="footer"></slot>
    </section>
  </div>
</template>

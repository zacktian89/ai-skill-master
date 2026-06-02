<script setup lang="ts">
defineProps<{
  checked?: boolean;
  disabled?: boolean;
  title?: string;
}>();

defineEmits<{
  change: [event: Event];
}>();
</script>

<template>
  <label class="switch-toggle" :title="title">
    <input
      type="checkbox"
      :checked="checked"
      :disabled="disabled"
      @change="$emit('change', $event)"
    />
    <span class="switch-slider"></span>
  </label>
</template>



<script setup lang="ts">
const model = defineModel<string>({ default: "" });

defineProps<{
  placeholder?: string;
  type?: string;
  readonly?: boolean;
}>();
</script>

<template>
  <input
    v-model="model"
    class="search-input"
    :type="type || 'text'"
    :placeholder="placeholder"
    :readonly="readonly"
  />
</template>

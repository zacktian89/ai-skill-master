<script setup lang="ts">
defineProps<{
  type?: string;
}>();
</script>

<template>
  <span class="status-tag" :class="type ? `status-tag--${type}` : ''">
    <slot></slot>
  </span>
</template>

<script setup lang="ts">
import { Moon, Sun } from "lucide-vue-next";

type ThemeMode = "dark" | "light";

defineProps<{
  themeMode: ThemeMode;
}>();

defineEmits<{
  "update:theme-mode": [value: ThemeMode];
}>();
</script>

<template>
  <div class="detail-header">
    <div>
      <h2>外观</h2>
    </div>
  </div>

  <section class="detail-section">
    <div class="segmented-control segmented-control--binary theme-toggle" aria-label="主题切换">
      <button
        type="button"
        :class="{ active: themeMode === 'dark' }"
        @click="$emit('update:theme-mode', 'dark')"
      >
        <Moon :size="15" />
        黑色
      </button>
      <button
        type="button"
        :class="{ active: themeMode === 'light' }"
        @click="$emit('update:theme-mode', 'light')"
      >
        <Sun :size="15" />
        白色
      </button>
    </div>
  </section>
</template>

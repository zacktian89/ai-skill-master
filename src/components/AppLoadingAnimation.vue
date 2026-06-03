<script setup lang="ts">
withDefaults(
  defineProps<{
    rows?: number;
    variant?: "panel" | "inline";
  }>(),
  {
    rows: 6,
    variant: "inline",
  }
);
</script>

<template>
  <div
    class="app-loading-animation"
    :class="[`app-loading-animation--${variant}`]"
    data-testid="app-loading-animation"
  >
    <div class="app-loading-animation__list" aria-hidden="true">
      <div
        v-for="rowIndex in rows"
        :key="rowIndex"
        class="app-loading-animation__row"
        data-testid="loading-skeleton-row"
      >
        <div class="app-loading-animation__row-top">
          <span
            class="app-loading-animation__bar app-loading-animation__row-title"
            :class="`app-loading-animation__row-title--${rowIndex % 3}`"
          ></span>
          <span class="app-loading-animation__bar app-loading-animation__row-badge"></span>
        </div>
        <div class="app-loading-animation__row-bottom">
          <span
            class="app-loading-animation__bar app-loading-animation__row-meta"
            :class="`app-loading-animation__row-meta--${rowIndex % 4}`"
          ></span>
          <span class="app-loading-animation__bar app-loading-animation__row-count"></span>
        </div>
      </div>
    </div>
  </div>
</template>

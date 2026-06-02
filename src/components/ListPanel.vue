<script setup lang="ts">
import { useScrollableList } from "../composables/useScrollableList";

interface Props {
  items?: any[];
  hasSearch?: boolean;
  emptyText?: string;
}

const props = withDefaults(defineProps<Props>(), {
  items: () => [],
  hasSearch: false,
  emptyText: "暂无数据。",
});

const { listStackRef, listStackScrollable } = useScrollableList(() => props.items);
</script>

<template>
  <div v-if="hasSearch" class="list-panel-head">
    <slot name="search-row"></slot>
  </div>

  <div
    v-if="items.length"
    ref="listStackRef"
    class="list-stack"
    :class="{ 'list-stack--scrollable': listStackScrollable }"
  >
    <slot></slot>
  </div>
  <div v-else class="content-empty">{{ emptyText }}</div>
</template>

<script setup lang="ts">
import { useScrollableList } from "../composables/useScrollableList";

import { useI18n } from "../composables/useI18n";

interface Props {
  items?: any[];
  hasSearch?: boolean;
  emptyText?: string;
}

const props = withDefaults(defineProps<Props>(), {
  items: () => [],
  hasSearch: false,
});

const { t } = useI18n();
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
  <slot v-else name="empty">
    <div class="content-empty">{{ emptyText || t('skills.noReferences') }}</div>
  </slot>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { CircleHelp, Folder, Github, ShoppingBag } from "lucide-vue-next";
import type { Skill, SkillSourceKind } from "../../../types";

const props = defineProps<{
  skill: Skill;
  isActive: boolean;
  isReferenced: boolean;
  selectable?: boolean;
  selected?: boolean;
}>();

defineEmits<{
  select: [];
  "toggle-selected": [];
  contextmenu: [event: MouseEvent];
}>();

const sourceIcons = {
  local: Folder,
  github: Github,
  openclawMarket: ShoppingBag,
  unknown: CircleHelp,
} as const;

const sourceLabels = {
  local: "本地",
  github: "GitHub",
  openclawMarket: "OpenClaw Market",
  unknown: "未知来源",
} as const;

const sourceKind = computed<SkillSourceKind>(() => props.skill.source?.kind ?? "local");

const label = computed(() => props.skill.source?.label || sourceLabels[sourceKind.value]);
const icon = computed(() => sourceIcons[sourceKind.value]);
</script>

<template>
  <div
    class="list-row"
    :class="{ active: isActive, 'list-row--selectable': selectable, selected }"
    role="button"
    tabindex="0"
    @click="$emit('select')"
    @keydown.enter="$emit('select')"
    @keydown.space.prevent="$emit('select')"
    @contextmenu.prevent="$emit('contextmenu', $event)"
  >
    <label
      v-if="selectable"
      class="list-row-check"
      :aria-label="selected ? '取消选择' : '选择'"
      @click.stop
    >
      <input
        type="checkbox"
        :checked="selected"
        @change="$emit('toggle-selected')"
      />
    </label>
    <div class="list-row-main">
      <div class="list-row-top">
        <strong>{{ skill.name }}</strong>
        <span
          class="reference-dot"
          :class="isReferenced ? 'reference-dot--active' : 'reference-dot--idle'"
          :aria-label="isReferenced ? '已引用' : '未引用'"
          :title="isReferenced ? '已引用' : '未引用'"
          role="img"
        ></span>
      </div>
      <div class="list-row-bottom">
        <span class="source-icon" :title="label" :aria-label="label" role="img">
          <component :is="icon" :size="13" />
        </span>
        <code class="skill-id-badge">{{ skill.id }}</code>
      </div>
    </div>
  </div>
</template>

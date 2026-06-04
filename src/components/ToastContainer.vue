<script setup lang="ts">
import { computed } from "vue";
import { AlertCircle, CheckCircle, AlertTriangle, Info, X } from "lucide-vue-next";
import { useAppStore } from "../stores/useAppStore";

const appStore = useAppStore();
const toasts = computed(() => appStore.toasts.value);

const getIcon = (type: string) => {
  switch (type) {
    case "error":
      return AlertCircle;
    case "success":
      return CheckCircle;
    case "warning":
      return AlertTriangle;
    default:
      return Info;
  }
};
</script>

<template>
  <Teleport to="body">
    <div class="toast-container">
      <TransitionGroup name="toast">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="toast"
          :class="`toast--${toast.type}`"
          role="alert"
        >
          <component :is="getIcon(toast.type)" :size="18" class="toast-icon" />
          <div class="toast-content">
            {{ toast.message }}
          </div>
          <button
            type="button"
            class="toast-close"
            @click="appStore.removeToast(toast.id)"
          >
            <X :size="14" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

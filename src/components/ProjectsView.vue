<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderPlus } from "lucide-vue-next";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import { ruleLabel, type AppSnapshot, type ProjectRule } from "../types";

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedProjectId: string | null;
}>();

const emit = defineEmits<{
  "select-project": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const busy = ref(false);

const selectedProject = computed(() =>
  props.snapshot.state.projects.find((project) => project.id === props.selectedProjectId) ??
  props.snapshot.state.projects[0] ??
  null,
);

async function run(action: () => Promise<AppSnapshot>) {
  busy.value = true;
  try {
    const next = await action();
    emit("snapshot", next);
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

async function addProject() {
  try {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      const parts = selected.split(/[\\/]/).filter(Boolean);
      const name = parts[parts.length - 1] ?? selected;
      await run(() => api.addProject({ name, path: selected }));
    }
  } catch (cause) {
    emit("error", String(cause));
  }
}

function setRule(skillId: string, rule: ProjectRule) {
  if (!selectedProject.value) return;
  return run(() => api.setProjectRule({ projectId: selectedProject.value!.id, skillId, rule }));
}
</script>

<template>
  <div class="split-content">
    <section class="list-panel panel-card">
      <div class="panel-header">
        <div>
          <p class="eyebrow">Workspace</p>
          <h2>项目列表</h2>
          <p class="panel-copy">为每个项目覆盖技能规则，避免全局开关过重。</p>
        </div>
        <span class="panel-count">{{ snapshot.state.projects.length }}</span>
      </div>

      <div class="toolbar">
        <button class="primary-button" :disabled="busy" @click="addProject">
          <FolderPlus :size="16" />
          添加项目
        </button>
      </div>

      <div v-if="snapshot.state.projects.length" class="list-stack">
        <button
          v-for="project in snapshot.state.projects"
          :key="project.id"
          class="row-item"
          :class="{ active: selectedProject?.id === project.id }"
          @click="emit('select-project', project.id)"
        >
          <span>
            <strong>{{ project.name }}</strong>
            <small>{{ project.path }}</small>
          </span>
        </button>
      </div>

      <div v-else class="content-empty content-empty--inline">还没有项目。</div>
    </section>

    <section class="detail-panel panel-card">
      <template v-if="selectedProject">
        <div class="detail-header">
          <div>
            <h2>{{ selectedProject.name }}</h2>
            <p>{{ selectedProject.path }}</p>
          </div>
          <button class="primary-button" :disabled="busy" @click="run(() => api.setCurrentProject(selectedProject!.id))">
            设为当前项目
          </button>
        </div>

        <div class="rule-list">
          <div v-for="skill in snapshot.state.skills" :key="skill.id" class="rule-row">
            <div>
              <strong>{{ skill.name }}</strong>
              <small>{{ ruleLabel(selectedProject.rules[skill.id]) }}</small>
            </div>
            <select
              :value="selectedProject.rules[skill.id] ?? 'inherit'"
              :disabled="busy"
              @change="setRule(skill.id, ($event.target as HTMLSelectElement).value as ProjectRule)"
            >
              <option value="inherit">跟随默认</option>
              <option value="enable">在此项目启用</option>
              <option value="disable">在此项目停用</option>
            </select>
          </div>
        </div>
      </template>
      <div v-else class="content-empty content-empty--inline">添加或选择一个项目。</div>
    </section>
  </div>
</template>

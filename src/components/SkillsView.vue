<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderPlus, RefreshCw, Trash2 } from "lucide-vue-next";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import type { AppSnapshot, DeleteSkillPreview } from "../types";

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedSkillId: string | null;
}>();

const emit = defineEmits<{
  "select-skill": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const query = ref("");
const busy = ref(false);
const deletePreview = ref<DeleteSkillPreview | null>(null);
const deleteDialogOpen = ref(false);

const skills = computed(() => {
  const normalized = query.value.trim().toLowerCase();
  if (!normalized) return props.snapshot.state.skills;
  return props.snapshot.state.skills.filter((skill) =>
    `${skill.name} ${skill.description} ${skill.id}`.toLowerCase().includes(normalized),
  );
});

const selectedSkill = computed(() =>
  props.snapshot.state.skills.find((skill) => skill.id === props.selectedSkillId) ?? props.snapshot.state.skills[0] ?? null,
);

async function run(action: () => Promise<AppSnapshot>) {
  busy.value = true;
  try {
    emit("snapshot", await action());
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

async function importSkill() {
  try {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await run(() => api.importSkill(selected));
    }
  } catch (cause) {
    emit("error", String(cause));
  }
}

async function openDeleteDialog() {
  if (!selectedSkill.value) return;
  busy.value = true;
  try {
    deletePreview.value = await api.previewDeleteSkill(selectedSkill.value.id);
    deleteDialogOpen.value = true;
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

async function confirmDelete() {
  if (!deletePreview.value) return;
  await run(() => api.deleteSkill(deletePreview.value!.skillId));
  deleteDialogOpen.value = false;
  deletePreview.value = null;
}

function closeDeleteDialog() {
  deleteDialogOpen.value = false;
  deletePreview.value = null;
}
</script>

<template>
  <div class="split-content">
    <section class="list-panel panel-card">
      <div class="panel-header">
        <div>
          <p class="eyebrow">Library</p>
          <h2>技能列表</h2>
          <p class="panel-copy">搜索、导入并同步你当前的 skill 库。</p>
        </div>
        <span class="panel-count">{{ skills.length }}</span>
      </div>

      <div class="toolbar">
        <input v-model="query" class="search-input" placeholder="搜索 skills" />
        <button class="icon-button" title="导入 skill" :disabled="busy" @click="importSkill">
          <FolderPlus :size="18" />
        </button>
        <button class="icon-button" title="同步 Codex" :disabled="busy" @click="run(api.syncCodex)">
          <RefreshCw :size="18" />
        </button>
      </div>

      <div v-if="skills.length" class="list-stack">
        <button
          v-for="skill in skills"
          :key="skill.id"
          class="row-item"
          :class="{ active: selectedSkill?.id === skill.id }"
          @click="emit('select-skill', skill.id)"
        >
          <span>
            <strong>{{ skill.name }}</strong>
            <small>{{ skill.description || skill.id }}</small>
          </span>
          <span class="status-dot" :class="{ on: skill.defaultEnabled, conflict: skill.conflict }"></span>
        </button>
      </div>

      <div v-else class="content-empty content-empty--inline">技能库里还没有 skill。</div>
    </section>

    <section class="detail-panel panel-card">
      <template v-if="selectedSkill">
        <div class="detail-header">
          <div>
            <h2>{{ selectedSkill.name }}</h2>
            <p>{{ selectedSkill.description || "没有描述" }}</p>
          </div>
          <label class="switch-row">
            <span>默认启用</span>
            <input
              type="checkbox"
              :checked="selectedSkill.defaultEnabled"
              :disabled="busy"
              @change="run(() => api.setDefaultEnabled(selectedSkill!.id, ($event.target as HTMLInputElement).checked))"
            />
          </label>
        </div>

        <dl class="meta-list">
          <dt>技能库路径</dt>
          <dd>{{ selectedSkill.libraryPath }}</dd>
          <dt>Codex</dt>
          <dd>{{ selectedSkill.managedLinks.codex || "未同步" }}</dd>
          <dt v-if="selectedSkill.conflict">冲突</dt>
          <dd v-if="selectedSkill.conflict" class="danger">
            {{ selectedSkill.conflict.message }}：{{ selectedSkill.conflict.path }}
          </dd>
        </dl>

        <button class="danger-button" :disabled="busy" @click="openDeleteDialog">
          <Trash2 :size="16" />
          删除 skill
        </button>
      </template>
      <div v-else class="content-empty content-empty--inline">选择或导入一个 skill。</div>
    </section>
  </div>

  <div v-if="deleteDialogOpen && deletePreview" class="modal-backdrop" @click.self="closeDeleteDialog">
    <section class="modal-card">
      <div class="panel-header">
        <div>
          <p class="eyebrow">Confirm Delete</p>
          <h2>删除 {{ deletePreview.skillName }}</h2>
          <p class="panel-copy">取消后不会修改任何文件或状态。</p>
        </div>
      </div>

      <dl class="meta-list">
        <dt>技能库目录</dt>
        <dd>{{ deletePreview.libraryPath }}</dd>
        <dt>托管链接</dt>
        <dd>
          <template v-if="deletePreview.managedLinkTargets.length">
            {{ deletePreview.managedLinkTargets.length }} 个
            <div class="meta-stack">
              <span v-for="target in deletePreview.managedLinkTargets" :key="target">{{ target }}</span>
            </div>
          </template>
          <template v-else>无</template>
        </dd>
        <dt>项目规则影响</dt>
        <dd>
          <template v-if="deletePreview.affectedProjects.length">
            {{ deletePreview.affectedProjects.length }} 个项目
            <div class="meta-stack">
              <span v-for="project in deletePreview.affectedProjects" :key="project.projectId">
                {{ project.projectName }} · {{ project.projectPath }}
              </span>
            </div>
          </template>
          <template v-else>无</template>
        </dd>
      </dl>

      <div class="button-row button-row--end">
        <button class="icon-button" :disabled="busy" @click="closeDeleteDialog">取消</button>
        <button class="danger-button" :disabled="busy" @click="confirmDelete">
          <Trash2 :size="16" />
          确认删除
        </button>
      </div>
    </section>
  </div>
</template>

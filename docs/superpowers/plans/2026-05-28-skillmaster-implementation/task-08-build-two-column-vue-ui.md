# Task 8: Build Two-Column Vue UI

**Files:**
- Modify: `src/App.vue`
- Create: `src/components/Sidebar.vue`
- Create: `src/components/SkillsView.vue`
- Create: `src/components/ProjectsView.vue`
- Create: `src/components/SettingsView.vue`
- Create: `src/styles.css`

- [ ] **Step 1: Replace App shell**

Replace `src/App.vue` with:

```vue
<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { AlertCircle } from "lucide-vue-next";
import * as api from "./api";
import Sidebar from "./components/Sidebar.vue";
import SkillsView from "./components/SkillsView.vue";
import ProjectsView from "./components/ProjectsView.vue";
import SettingsView from "./components/SettingsView.vue";
import type { AppSnapshot } from "./types";

type Section = "skills" | "projects" | "settings";

const activeSection = ref<Section>("skills");
const snapshot = ref<AppSnapshot | null>(null);
const selectedSkillId = ref<string | null>(null);
const selectedProjectId = ref<string | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

const title = computed(() => {
  if (activeSection.value === "projects") return "Projects";
  if (activeSection.value === "settings") return "Settings";
  return "Skills";
});

async function refresh() {
  loading.value = true;
  error.value = null;
  try {
    snapshot.value = await api.getSnapshot();
    selectedProjectId.value =
      snapshot.value.state.currentProjectId ?? snapshot.value.state.projects[0]?.id ?? null;
  } catch (cause) {
    error.value = String(cause);
  } finally {
    loading.value = false;
  }
}

function applySnapshot(next: AppSnapshot) {
  snapshot.value = next;
}

onMounted(refresh);
</script>

<template>
  <div class="app-shell">
    <Sidebar v-model:active-section="activeSection" :snapshot="snapshot" />
    <main class="main-pane">
      <header class="topbar">
        <div>
          <h1>{{ title }}</h1>
          <p v-if="snapshot && !snapshot.codexConnected" class="topbar-note">
            Codex 未连接，可在 Settings 中设置目录。
          </p>
        </div>
      </header>

      <div v-if="error" class="notice error">
        <AlertCircle :size="16" />
        <span>{{ error }}</span>
      </div>

      <section v-if="loading" class="content-empty">正在加载 SkillMaster</section>

      <SkillsView
        v-else-if="activeSection === 'skills' && snapshot"
        :snapshot="snapshot"
        :selected-skill-id="selectedSkillId"
        @select-skill="selectedSkillId = $event"
        @snapshot="applySnapshot"
        @error="error = $event"
      />

      <ProjectsView
        v-else-if="activeSection === 'projects' && snapshot"
        :snapshot="snapshot"
        :selected-project-id="selectedProjectId"
        @select-project="selectedProjectId = $event"
        @snapshot="applySnapshot"
        @error="error = $event"
      />

      <SettingsView
        v-else-if="activeSection === 'settings' && snapshot"
        :snapshot="snapshot"
        @snapshot="applySnapshot"
        @error="error = $event"
      />
    </main>
  </div>
</template>
```

- [ ] **Step 2: Add Sidebar**

Create `src/components/Sidebar.vue`:

```vue
<script setup lang="ts">
import { FolderKanban, Library, Settings } from "lucide-vue-next";
import type { AppSnapshot } from "../types";

type Section = "skills" | "projects" | "settings";

defineProps<{
  activeSection: Section;
  snapshot: AppSnapshot | null;
}>();

defineEmits<{
  "update:activeSection": [value: Section];
}>();
</script>

<template>
  <aside class="sidebar">
    <div class="brand">SkillMaster</div>
    <nav class="nav-list">
      <button :class="{ active: activeSection === 'skills' }" @click="$emit('update:activeSection', 'skills')">
        <Library :size="18" />
        <span>Skills</span>
        <small v-if="snapshot">{{ snapshot.state.skills.length }}</small>
      </button>
      <button :class="{ active: activeSection === 'projects' }" @click="$emit('update:activeSection', 'projects')">
        <FolderKanban :size="18" />
        <span>Projects</span>
        <small v-if="snapshot">{{ snapshot.state.projects.length }}</small>
      </button>
    </nav>
    <button class="settings-button" :class="{ active: activeSection === 'settings' }" @click="$emit('update:activeSection', 'settings')">
      <Settings :size="18" />
      <span>Settings</span>
    </button>
  </aside>
</template>
```

- [ ] **Step 3: Add Skills view**

Create `src/components/SkillsView.vue`:

```vue
<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderPlus, RefreshCw, Trash2 } from "lucide-vue-next";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import type { AppSnapshot } from "../types";

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
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    await run(() => api.importSkill(selected));
  }
}
</script>

<template>
  <div class="split-content">
    <section class="list-panel">
      <div class="toolbar">
        <input v-model="query" class="search-input" placeholder="搜索 skills" />
        <button class="icon-button" title="导入 skill" :disabled="busy" @click="importSkill">
          <FolderPlus :size="18" />
        </button>
        <button class="icon-button" title="同步 Codex" :disabled="busy" @click="run(api.syncCodex)">
          <RefreshCw :size="18" />
        </button>
      </div>

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

      <div v-if="!skills.length" class="content-empty">技能库里还没有 skill。</div>
    </section>

    <section class="detail-panel">
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
          <dd v-if="selectedSkill.conflict" class="danger">{{ selectedSkill.conflict.message }}：{{ selectedSkill.conflict.path }}</dd>
        </dl>

        <button class="danger-button" :disabled="busy" @click="run(() => api.deleteSkill(selectedSkill!.id))">
          <Trash2 :size="16" />
          删除 skill
        </button>
      </template>
      <div v-else class="content-empty">选择或导入一个 skill。</div>
    </section>
  </div>
</template>
```

- [ ] **Step 4: Add Projects view**

Create `src/components/ProjectsView.vue`:

```vue
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
  props.snapshot.state.projects.find((project) => project.id === props.selectedProjectId) ?? props.snapshot.state.projects[0] ?? null,
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
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    const name = selected.split(/[\\/]/).filter(Boolean).at(-1) ?? selected;
    await run(() => api.addProject({ name, path: selected }));
  }
}

function setRule(skillId: string, rule: ProjectRule) {
  if (!selectedProject.value) return;
  return run(() => api.setProjectRule({ projectId: selectedProject.value!.id, skillId, rule }));
}
</script>

<template>
  <div class="split-content">
    <section class="list-panel">
      <div class="toolbar">
        <button class="primary-button" :disabled="busy" @click="addProject">
          <FolderPlus :size="16" />
          添加项目
        </button>
      </div>

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

      <div v-if="!snapshot.state.projects.length" class="content-empty">还没有项目。</div>
    </section>

    <section class="detail-panel">
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
      <div v-else class="content-empty">添加或选择一个项目。</div>
    </section>
  </div>
</template>
```

- [ ] **Step 5: Add Settings view**

Create `src/components/SettingsView.vue`:

```vue
<script setup lang="ts">
import { ref } from "vue";
import { FolderOpen, RefreshCw } from "lucide-vue-next";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import type { AppSnapshot } from "../types";

defineProps<{ snapshot: AppSnapshot }>();

const emit = defineEmits<{
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const busy = ref(false);

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

async function chooseCodexPath() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    await run(() => api.setCodexPath(selected));
  }
}
</script>

<template>
  <div class="settings-grid">
    <section class="settings-section">
      <h2>技能库</h2>
      <dl class="meta-list">
        <dt>位置</dt>
        <dd>{{ snapshot.state.skillLibraryPath }}</dd>
      </dl>
      <p class="muted">迁移技能库会在后续任务中接入，第一版先展示当前位置。</p>
    </section>

    <section class="settings-section">
      <h2>Codex 连接</h2>
      <dl class="meta-list">
        <dt>状态</dt>
        <dd>{{ snapshot.codexConnected ? "已连接" : "未连接" }}</dd>
        <dt>目录</dt>
        <dd>{{ snapshot.state.codexSkillsPath || "未设置" }}</dd>
      </dl>
      <div class="button-row">
        <button class="primary-button" :disabled="busy" @click="chooseCodexPath">
          <FolderOpen :size="16" />
          选择目录
        </button>
        <button class="primary-button" :disabled="busy" @click="run(api.syncCodex)">
          <RefreshCw :size="16" />
          同步 Codex
        </button>
      </div>
    </section>

    <section class="settings-section">
      <h2>诊断</h2>
      <ul class="diagnostics">
        <li v-for="item in snapshot.diagnostics" :key="item">{{ item }}</li>
        <li v-if="!snapshot.diagnostics.length">没有需要处理的问题。</li>
      </ul>
    </section>
  </div>
</template>
```

- [ ] **Step 6: Add styles**

Create `src/styles.css`:

```css
:root {
  font-family: Inter, "Segoe UI", system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
  color: #202426;
  background: #f6f7f7;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  min-width: 960px;
  min-height: 100vh;
}

button,
input,
select {
  font: inherit;
}

.app-shell {
  display: grid;
  grid-template-columns: 260px 1fr;
  min-height: 100vh;
  background: #f8faf9;
}

.sidebar {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 22px 14px;
  background: #eaf5f7;
  border-right: 1px solid #dbe5e8;
}

.brand {
  padding: 0 12px;
  font-weight: 700;
  font-size: 18px;
}

.nav-list {
  display: grid;
  gap: 6px;
}

.nav-list button,
.settings-button {
  display: grid;
  grid-template-columns: 22px 1fr auto;
  align-items: center;
  gap: 10px;
  width: 100%;
  min-height: 38px;
  padding: 8px 12px;
  border: 0;
  border-radius: 8px;
  color: #536166;
  background: transparent;
  text-align: left;
  cursor: pointer;
}

.nav-list button.active,
.settings-button.active {
  color: #1e2a2e;
  background: rgba(255, 255, 255, 0.72);
}

.settings-button {
  margin-top: auto;
}

.main-pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: #ffffff;
}

.topbar {
  display: flex;
  align-items: center;
  min-height: 72px;
  padding: 18px 28px;
  border-bottom: 1px solid #eceff0;
}

.topbar h1,
.detail-header h2,
.settings-section h2 {
  margin: 0;
  font-size: 20px;
  line-height: 1.25;
}

.topbar-note,
.muted,
.detail-header p,
.row-item small,
.rule-row small {
  margin: 4px 0 0;
  color: #7a8589;
  font-size: 13px;
}

.notice {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 16px 28px 0;
  padding: 10px 12px;
  border-radius: 8px;
  background: #fff7e6;
  color: #7a4a00;
}

.notice.error {
  background: #fff1f0;
  color: #a4262c;
}

.split-content {
  display: grid;
  grid-template-columns: minmax(320px, 40%) 1fr;
  min-height: 0;
  flex: 1;
}

.list-panel {
  padding: 20px;
  border-right: 1px solid #eceff0;
  overflow: auto;
}

.detail-panel {
  padding: 24px 28px;
  overflow: auto;
}

.toolbar,
.button-row {
  display: flex;
  gap: 8px;
  margin-bottom: 14px;
}

.search-input {
  width: 100%;
  height: 38px;
  padding: 0 12px;
  border: 1px solid #d7dde0;
  border-radius: 8px;
  outline: none;
}

.search-input:focus {
  border-color: #6aa6b8;
}

.icon-button,
.primary-button,
.danger-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 38px;
  padding: 0 12px;
  border: 1px solid #d7dde0;
  border-radius: 8px;
  background: #ffffff;
  color: #243033;
  cursor: pointer;
}

.primary-button {
  background: #1f6f82;
  border-color: #1f6f82;
  color: #ffffff;
}

.danger-button {
  margin-top: 22px;
  color: #9d1c24;
}

button:disabled {
  opacity: 0.55;
  cursor: default;
}

.row-item {
  display: grid;
  grid-template-columns: 1fr auto;
  align-items: center;
  gap: 12px;
  width: 100%;
  min-height: 58px;
  padding: 10px 12px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  text-align: left;
  cursor: pointer;
}

.row-item.active {
  background: #edf6f7;
}

.row-item strong,
.rule-row strong {
  display: block;
  font-size: 14px;
}

.status-dot {
  width: 9px;
  height: 9px;
  border-radius: 999px;
  background: #b8c1c4;
}

.status-dot.on {
  background: #238a54;
}

.status-dot.conflict {
  background: #d83b01;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  margin-bottom: 24px;
}

.switch-row {
  display: inline-flex;
  align-items: center;
  gap: 10px;
}

.meta-list {
  display: grid;
  grid-template-columns: 112px minmax(0, 1fr);
  gap: 10px 14px;
  margin: 0;
}

.meta-list dt {
  color: #7a8589;
}

.meta-list dd {
  margin: 0;
  overflow-wrap: anywhere;
}

.danger {
  color: #a4262c;
}

.rule-list {
  display: grid;
  gap: 10px;
}

.rule-row {
  display: grid;
  grid-template-columns: 1fr 180px;
  align-items: center;
  gap: 16px;
  min-height: 54px;
  padding: 10px 0;
  border-bottom: 1px solid #edf0f1;
}

.rule-row select {
  height: 36px;
  border: 1px solid #d7dde0;
  border-radius: 8px;
  background: #fff;
}

.settings-grid {
  display: grid;
  gap: 18px;
  padding: 24px 28px;
}

.settings-section {
  padding-bottom: 18px;
  border-bottom: 1px solid #edf0f1;
}

.diagnostics {
  margin: 12px 0 0;
  padding-left: 20px;
  color: #536166;
}

.content-empty {
  padding: 32px;
  color: #7a8589;
  text-align: center;
}
```

- [ ] **Step 7: Build frontend**

Run:

```powershell
npm run build
```

Expected: Vue typecheck and Vite build pass.

- [ ] **Step 8: Commit**

Run:

```powershell
git add src/App.vue src/components src/styles.css
git commit -m "feat: add SkillMaster desktop UI"
```

Expected: commit succeeds.

---

Parent plan: [2026-05-28-skillmaster-implementation.md](../2026-05-28-skillmaster-implementation.md)

# Task 7: Add Frontend Types, API Wrapper, and Store State

**Files:**
- Create: `src/types.ts`
- Create: `src/api.ts`
- Modify: `src/main.ts`
- Test: `src/__tests__/effectiveState.test.ts`

- [ ] **Step 1: Add frontend types**

Create `src/types.ts`:

```ts
export type ProjectRule = "inherit" | "enable" | "disable";

export interface ManagedLinks {
  codex?: string | null;
}

export interface SkillConflict {
  target: string;
  path: string;
  message: string;
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  libraryPath: string;
  defaultEnabled: boolean;
  managedLinks: ManagedLinks;
  conflict?: SkillConflict | null;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  rules: Record<string, ProjectRule>;
}

export interface AppState {
  schemaVersion: number;
  skillLibraryPath: string;
  codexSkillsPath?: string | null;
  currentProjectId?: string | null;
  skills: Skill[];
  projects: Project[];
}

export interface AppSnapshot {
  state: AppState;
  codexConnected: boolean;
  diagnostics: string[];
}

export interface AddProjectRequest {
  name: string;
  path: string;
}

export interface SetProjectRuleRequest {
  projectId: string;
  skillId: string;
  rule: ProjectRule;
}

export function ruleLabel(rule: ProjectRule | undefined): string {
  if (rule === "enable") return "在此项目启用";
  if (rule === "disable") return "在此项目停用";
  return "跟随默认";
}
```

- [ ] **Step 2: Write frontend test**

Create `src/__tests__/effectiveState.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { ruleLabel } from "../types";

describe("ruleLabel", () => {
  it("uses friendly project rule labels", () => {
    expect(ruleLabel(undefined)).toBe("跟随默认");
    expect(ruleLabel("inherit")).toBe("跟随默认");
    expect(ruleLabel("enable")).toBe("在此项目启用");
    expect(ruleLabel("disable")).toBe("在此项目停用");
  });
});
```

- [ ] **Step 3: Run frontend test**

Run:

```powershell
npm test -- src/__tests__/effectiveState.test.ts
```

Expected: PASS for `ruleLabel`.

- [ ] **Step 4: Add Tauri API wrapper**

Create `src/api.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { AddProjectRequest, AppSnapshot, ProjectRule, SetProjectRuleRequest } from "./types";

export function getSnapshot(): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("get_snapshot");
}

export function importSkill(source: string): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("import_skill", { source });
}

export function deleteSkill(skillId: string): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("delete_skill", { skillId });
}

export function setDefaultEnabled(skillId: string, enabled: boolean): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("set_default_enabled", { skillId, enabled });
}

export function addProject(request: AddProjectRequest): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("add_project", { request });
}

export function setProjectRule(request: SetProjectRuleRequest): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("set_project_rule", { request });
}

export function setCurrentProject(projectId: string | null): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("set_current_project", { projectId });
}

export function setCodexPath(path: string): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("set_codex_path", { path });
}

export function syncCodex(): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("sync_codex");
}

export type { ProjectRule };
```

- [ ] **Step 5: Ensure Vue entry loads global CSS**

Modify `src/main.ts`:

```ts
import { createApp } from "vue";
import App from "./App.vue";
import "./styles.css";

createApp(App).mount("#app");
```

- [ ] **Step 6: Run frontend checks**

Run:

```powershell
npm test
npm run build
```

Expected: Vitest passes and TypeScript build succeeds.

- [ ] **Step 7: Commit**

Run:

```powershell
git add src/types.ts src/api.ts src/main.ts src/__tests__/effectiveState.test.ts package.json package-lock.json
git commit -m "feat: add frontend command API"
```

Expected: commit succeeds.

---

Parent plan: [2026-05-28-skillmaster-implementation.md](../2026-05-28-skillmaster-implementation.md)

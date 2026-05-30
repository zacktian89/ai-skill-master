import type { AddProjectRequest, AppSnapshot, DeleteSkillPreview, SetProjectRuleRequest } from "./types";
import * as mockApi from "./mockApi";

type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

let invokePromise: Promise<TauriInvoke | null> | null = null;

function hasTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function resolveInvoke(): Promise<TauriInvoke | null> {
  if (!invokePromise) {
    if (!hasTauriRuntime()) {
      invokePromise = Promise.resolve(null);
    } else {
      invokePromise = import("@tauri-apps/api/core")
        .then((module) => (typeof module.invoke === "function" ? module.invoke : null))
        .catch(() => null);
    }
  }
  return invokePromise;
}

export async function getSnapshot(): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("get_snapshot") : mockApi.getSnapshot();
}

export async function importSkill(source: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("import_skill", { source }) : mockApi.importSkill(source);
}

export async function deleteSkill(skillId: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("delete_skill", { skillId }) : mockApi.deleteSkill(skillId);
}

export async function previewDeleteSkill(skillId: string): Promise<DeleteSkillPreview> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<DeleteSkillPreview>("preview_delete_skill", { skillId }) : mockApi.previewDeleteSkill(skillId);
}

export async function setSkillLinkEnabled(skillId: string, enabled: boolean): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke
    ? invoke<AppSnapshot>("set_skill_link_enabled", { skillId, enabled })
    : mockApi.setSkillLinkEnabled(skillId, enabled);
}

export async function addProject(request: AddProjectRequest): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("add_project", { request }) : mockApi.addProject(request);
}

export async function setProjectRule(request: SetProjectRuleRequest): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("set_project_rule", { request }) : mockApi.setProjectRule(request);
}

export async function setCurrentProject(projectId: string | null): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("set_current_project", { projectId }) : mockApi.setCurrentProject(projectId);
}

export async function resetProjectRules(projectId: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("reset_project_rules", { projectId }) : mockApi.resetProjectRules(projectId);
}

export async function deleteProject(projectId: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("delete_project", { projectId }) : mockApi.deleteProject(projectId);
}

export async function setCodexPath(path: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("set_codex_path", { path }) : mockApi.setCodexPath(path);
}

export async function migrateLibrary(target: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("migrate_library", { target }) : mockApi.migrateLibrary(target);
}

export async function rebuildState(): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("rebuild_state") : mockApi.rebuildState();
}

export async function syncCodex(): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("sync_codex") : mockApi.syncCodex();
}

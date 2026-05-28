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

export function migrateLibrary(target: string): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("migrate_library", { target });
}

export function syncCodex(): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("sync_codex");
}

export type { ProjectRule };

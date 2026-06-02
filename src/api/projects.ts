import type {
  AppSnapshot,
  AddProjectRequest,
  SetProjectRuleRequest,
  ScannedCategory,
  ImportProjectSkillResult,
} from "../types";
import { resolveInvoke, autoSync } from "./client";
import * as mockProjects from "./mock/projects";
import { getSnapshot } from "./settings";

export async function addProject(request: AddProjectRequest): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("add_project", { request }) : mockProjects.addProject(request);
}

export async function setProjectRule(request: SetProjectRuleRequest): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("set_project_rule", { request }) : await mockProjects.setProjectRule(request);
  return autoSync(next);
}

export async function setCurrentProject(projectId: string | null): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("set_current_project", { projectId }) : await mockProjects.setCurrentProject(projectId);
  return autoSync(next);
}

export async function resetProjectRules(projectId: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("reset_project_rules", { projectId }) : await mockProjects.resetProjectRules(projectId);
  return autoSync(next);
}

export async function deleteProject(projectId: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("delete_project", { projectId }) : await mockProjects.deleteProject(projectId);
  return autoSync(next);
}

export async function scanProjectSkills(projectPath: string): Promise<ScannedCategory[]> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<ScannedCategory[]>("scan_project_skills", { projectPath }) : [];
}

export async function importProjectSkill(
  projectName: string,
  skillPath: string,
  strategy?: "overwrite" | "keep_existing"
): Promise<ImportProjectSkillResult> {
  const invoke = await resolveInvoke();
  return invoke
    ? invoke<ImportProjectSkillResult>("import_project_skill", {
        projectName,
        skillPath,
        strategy: strategy || null,
      })
    : { type: "success", snapshot: await getSnapshot() };
}

export async function deleteUnmanagedSkill(skillPath: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke
    ? await invoke<AppSnapshot>("delete_unmanaged_skill", { skillPath })
    : await mockProjects.deleteUnmanagedSkill(skillPath);
  return autoSync(next);
}

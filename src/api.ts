import type {
  AddProjectRequest,
  AddSkillReferenceRequest,
  AppSnapshot,
  ConfirmImportSkillsRequest,
  DeleteSkillPreview,
  ImportSkillPreview,
  ImportSkillSource,
  SetProjectRuleRequest,
} from "./types";
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
  const next = invoke ? await invoke<AppSnapshot>("import_skill", { source }) : await mockApi.importSkill(source);
  return autoSync(next);
}

export async function previewImportSkills(source: ImportSkillSource): Promise<ImportSkillPreview> {
  const invoke = await resolveInvoke();
  return invoke
    ? invoke<ImportSkillPreview>("preview_import_skills", { source })
    : mockApi.previewImportSkills(source);
}

export async function confirmImportSkills(request: ConfirmImportSkillsRequest): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke
    ? await invoke<AppSnapshot>("confirm_import_skills", { request })
    : await mockApi.confirmImportSkills(request);
  return autoSync(next);
}

export async function deleteSkill(skillId: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("delete_skill", { skillId }) : await mockApi.deleteSkill(skillId);
  return autoSync(next);
}

export async function previewDeleteSkill(skillId: string): Promise<DeleteSkillPreview> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<DeleteSkillPreview>("preview_delete_skill", { skillId }) : mockApi.previewDeleteSkill(skillId);
}

export async function addSkillReference(request: AddSkillReferenceRequest): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("add_skill_reference", { request }) : mockApi.addSkillReference(request);
}

export async function removeSkillReference(referenceId: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke
    ? invoke<AppSnapshot>("remove_skill_reference", { referenceId })
    : mockApi.removeSkillReference(referenceId);
}

export async function addProject(request: AddProjectRequest): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("add_project", { request }) : mockApi.addProject(request);
}

export async function setProjectRule(request: SetProjectRuleRequest): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("set_project_rule", { request }) : await mockApi.setProjectRule(request);
  return autoSync(next);
}

export async function setCurrentProject(projectId: string | null): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("set_current_project", { projectId }) : await mockApi.setCurrentProject(projectId);
  return autoSync(next);
}

export async function resetProjectRules(projectId: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("reset_project_rules", { projectId }) : await mockApi.resetProjectRules(projectId);
  return autoSync(next);
}

export async function deleteProject(projectId: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("delete_project", { projectId }) : await mockApi.deleteProject(projectId);
  return autoSync(next);
}

export async function setCodexPath(path: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("set_codex_path", { path }) : await mockApi.setCodexPath(path);
  return autoSync(next);
}

export async function migrateLibrary(target: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("migrate_library", { target }) : await mockApi.migrateLibrary(target);
  return autoSync(next);
}

export async function rebuildState(): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("rebuild_state") : await mockApi.rebuildState();
  return autoSync(next);
}

export async function syncCodex(): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("sync_codex") : mockApi.syncCodex();
}

async function autoSync(snapshot: AppSnapshot): Promise<AppSnapshot> {
  if (!snapshot.state.codexSkillsPath) return snapshot;
  return syncCodex();
}

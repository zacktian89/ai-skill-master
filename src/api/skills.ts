import type {
  AppSnapshot,
  ImportSkillSource,
  ImportSkillPreview,
  ConfirmImportSkillsRequest,
  DeleteSkillPreview,
  AddSkillReferenceRequest,
} from "../types";
import { resolveInvoke, autoSync } from "./client";
import * as mockSkills from "./mock/skills";

export async function importSkill(source: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("import_skill", { source }) : await mockSkills.importSkill(source);
  return autoSync(next);
}

export async function previewImportSkills(source: ImportSkillSource): Promise<ImportSkillPreview> {
  const invoke = await resolveInvoke();
  return invoke
    ? invoke<ImportSkillPreview>("preview_import_skills", { source })
    : mockSkills.previewImportSkills(source);
}

export async function confirmImportSkills(request: ConfirmImportSkillsRequest): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke
    ? await invoke<AppSnapshot>("confirm_import_skills", { request })
    : await mockSkills.confirmImportSkills(request);
  return autoSync(next);
}

export async function deleteSkill(skillId: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("delete_skill", { skillId }) : await mockSkills.deleteSkill(skillId);
  return autoSync(next);
}

export async function previewDeleteSkill(skillId: string): Promise<DeleteSkillPreview> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<DeleteSkillPreview>("preview_delete_skill", { skillId }) : mockSkills.previewDeleteSkill(skillId);
}

export async function addSkillReference(request: AddSkillReferenceRequest): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("add_skill_reference", { request }) : mockSkills.addSkillReference(request);
}

export async function removeSkillReference(referenceId: string, removeExternalLink?: boolean): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke
    ? invoke<AppSnapshot>("remove_skill_reference", { referenceId, removeExternalLink })
    : mockSkills.removeSkillReference(referenceId, removeExternalLink);
}

export async function readSkillFile(skillId: string): Promise<string> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<string>("read_skill_file", { skillId }) : mockSkills.readSkillFile(skillId);
}

export async function readSkillFileAtPath(skillPath: string): Promise<string> {
  const invoke = await resolveInvoke();
  return invoke
    ? invoke<string>("read_skill_file_at_path", { skillPath })
    : mockSkills.readSkillFile(skillPath.split(/[\\/]/).filter(Boolean).pop() || skillPath);
}

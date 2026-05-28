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

export type ProjectRule = "inherit" | "enable" | "disable";
export type DiagnosticLevel = "info" | "warning" | "error";
export type SyncPhase = "idle" | "healthy" | "repairRequired";
export type PendingSyncActionKind = "create" | "remove" | "inspect";
export type StateLoadPhase = "clean" | "restoredFromBackup" | "rebuildRequired";

export interface ManagedLinks {
  codex?: string | null;
}

export type SkillSourceKind = "local" | "github" | "openclawMarket" | "unknown";

export interface SkillSource {
  kind: SkillSourceKind;
  label?: string | null;
  url?: string | null;
  path?: string | null;
  ref?: string | null;
  commit?: string | null;
  subdir?: string | null;
}

export interface SkillConflict {
  target: string;
  path: string;
  message: string;
}

export interface PendingSyncAction {
  kind: PendingSyncActionKind;
  skillId: string;
  target: string;
  source?: string | null;
  message: string;
}

export interface SyncStatus {
  phase: SyncPhase;
  message?: string | null;
  pendingActions: PendingSyncAction[];
}

export interface MigrationNotice {
  oldLibraryPath: string;
  newLibraryPath: string;
  message: string;
  requiresCodexResync: boolean;
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  libraryPath: string;
  source?: SkillSource | null;
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
  syncStatus: SyncStatus;
  migrationNotice?: MigrationNotice | null;
  skills: Skill[];
  projects: Project[];
}

export interface DiagnosticItem {
  level: DiagnosticLevel;
  code: string;
  title: string;
  detail: string;
}

export interface SnapshotPaths {
  stateFile: string;
  backupFile: string;
}

export interface StateLoadInfo {
  phase: StateLoadPhase;
  message?: string | null;
}

export interface AppSnapshot {
  state: AppState;
  codexConnected: boolean;
  diagnostics: DiagnosticItem[];
  paths: SnapshotPaths;
  stateLoad: StateLoadInfo;
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

export interface ProjectImpact {
  projectId: string;
  projectName: string;
  projectPath: string;
}

export interface DeleteSkillPreview {
  skillId: string;
  skillName: string;
  libraryPath: string;
  managedLinkTargets: string[];
  affectedProjects: ProjectImpact[];
}

export function ruleLabel(rule: ProjectRule | undefined): string {
  if (rule === "enable") return "在此项目启用";
  if (rule === "disable") return "在此项目停用";
  return "跟随默认";
}

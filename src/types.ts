export type ProjectRule = "inherit" | "enable" | "disable";
export type DiagnosticLevel = "info" | "warning" | "error";
export type SyncPhase = "idle" | "healthy" | "repairRequired";
export type PendingSyncActionKind = "create" | "remove" | "inspect";
export type StateLoadPhase = "clean" | "restoredFromBackup" | "rebuildRequired";
export type ReferenceScope = "user" | "project" | "custom";
export type ReferenceStatus = "healthy" | "missing" | "conflict" | "stale";

export interface ManagedLinks {
  codex?: string | null;
}

export interface SkillReference {
  id: string;
  targetName: string;
  targetPath: string;
  scope: ReferenceScope;
  status: ReferenceStatus;
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
  references?: SkillReference[];
  managedLinks: ManagedLinks;
  conflict?: SkillConflict | null;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  rules: Record<string, ProjectRule>;
}

export interface Agent {
  id: string;
  name: string;
  path: string;
  rules: Record<string, ProjectRule>;
}

export interface SetAgentRuleRequest {
  agentId: string;
  skillId: string;
  rule: ProjectRule;
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
  agents: Agent[];
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
  targetProfiles?: SkillTargetProfile[];
  diagnostics: DiagnosticItem[];
  paths: SnapshotPaths;
  stateLoad: StateLoadInfo;
}

export interface SkillTargetProfile {
  id: string;
  targetName: string;
  rootPath: string;
  scope: ReferenceScope;
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

export interface AddSkillReferenceRequest {
  skillId: string;
  targetName: string;
  rootPath: string;
  scope: ReferenceScope;
  overwrite?: boolean;
}

export type ImportSkillSource =
  | {
      kind: "local";
      path: string;
    }
  | {
      kind: "github";
      url: string;
      ref?: string | null;
      subdir?: string | null;
    };

export type ImportSkillCandidateStatus = "ready" | "duplicate" | "conflict" | "invalid";

export interface ImportSkillCandidate {
  candidateId: string;
  id: string;
  name: string;
  description: string;
  relativePath: string;
  status: ImportSkillCandidateStatus;
  message?: string | null;
}

export interface ImportSkillPreview {
  candidates: ImportSkillCandidate[];
}

export interface ConfirmImportSkillsRequest {
  source: ImportSkillSource;
  candidateIds: string[];
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

export interface ScannedSkill {
  id: string;
  name: string;
  description: string;
  path: string;
  isManaged: boolean;
}

export interface ScannedCategory {
  name: string;
  path: string;
  skills: ScannedSkill[];
}

export type ImportProjectSkillResult =
  | { type: "success"; snapshot: AppSnapshot }
  | {
      type: "conflict";
      skillId: string;
      libraryName: string;
      projectName: string;
    };


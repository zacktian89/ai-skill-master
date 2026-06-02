export type ThemeMode = "dark" | "light";
export type DiagnosticLevel = "info" | "warning" | "error";
export type SyncPhase = "idle" | "healthy" | "repairRequired";
export type PendingSyncActionKind = "create" | "remove" | "inspect";
export type StateLoadPhase = "clean" | "restoredFromBackup" | "rebuildRequired";
export type ReferenceScope = "user" | "project" | "custom";
export type ReferenceStatus = "healthy" | "missing" | "conflict" | "stale";

import type { Skill } from "./skill";
import type { Project } from "./project";
import type { Agent } from "./agent";

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

export interface SkillTargetProfile {
  id: string;
  targetName: string;
  rootPath: string;
  scope: ReferenceScope;
}

export interface AppSnapshot {
  state: AppState;
  targetProfiles?: SkillTargetProfile[];
  diagnostics: DiagnosticItem[];
  paths: SnapshotPaths;
  stateLoad: StateLoadInfo;
}

export type ImportProjectSkillResult =
  | { type: "success"; snapshot: AppSnapshot }
  | {
      type: "conflict";
      skillId: string;
      libraryName: string;
      projectName: string;
    };

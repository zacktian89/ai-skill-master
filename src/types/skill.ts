import type { ReferenceScope, ReferenceStatus } from "./app";

export interface ManagedLinks {}

export interface SkillReference {
  id: string;
  targetName: string;
  targetPath: string;
  scope: ReferenceScope;
  status: ReferenceStatus;
}

export interface SkillReferenceDetail {
  id: string;
  targetName: string;
  symlinkPath: string;
  scope: ReferenceScope;
  status: ReferenceStatus;
  removable: boolean;
  legacyCodex: boolean;
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

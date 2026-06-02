export type ProjectRule = "inherit" | "enable" | "disable";

export interface Project {
  id: string;
  name: string;
  path: string;
  rules: Record<string, ProjectRule>;
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

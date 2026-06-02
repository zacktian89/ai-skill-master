import type { ProjectRule } from "./project";

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

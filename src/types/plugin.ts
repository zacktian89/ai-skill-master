import type { Skill } from "./skill";

export interface Plugin {
  id: string;
  name: string;
  description: string;
  path: string;
  version?: string;
  author?: string;
  agentTargets: ("Codex" | "Claude Code")[];
  skills: Skill[];
  configKey?: string | null;
  disabledSkillIds?: string[];
  mcpServers?: string[];
  mcpConfig?: Record<string, unknown>; // 存放 Claude Code MCP server 的原始配置
  type: "standard" | "mcp";
  enabled: boolean;
}

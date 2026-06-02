import type {
  AppSnapshot,
  SetAgentRuleRequest,
  ScannedCategory,
} from "../types";
import { resolveInvoke } from "./client";
import * as mockAgents from "./mock/agents";

export async function addAgent(name: string, path: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("add_agent", { name, path }) : mockAgents.addAgent(name, path);
}

export async function deleteAgent(agentId: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("delete_agent", { agentId }) : mockAgents.deleteAgent(agentId);
}

export async function setAgentRule(request: SetAgentRuleRequest): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("set_agent_rule", { request }) : mockAgents.setAgentRule(request);
}

export async function scanAgentSkills(agentPath: string): Promise<ScannedCategory[]> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<ScannedCategory[]>("scan_agent_skills", { agentPath }) : mockAgents.scanAgentSkills(agentPath);
}

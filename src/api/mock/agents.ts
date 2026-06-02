import type {
  AppSnapshot,
  SetAgentRuleRequest,
  ScannedCategory,
} from "../../types";
import { mockSnapshot, snapshot } from "./data";

export function addAgent(name: string, path: string): Promise<AppSnapshot> {
  const id = name.toLowerCase().replace(/[^a-z0-9-]+/g, "-") + "-agent";
  if (!mockSnapshot.state.agents) {
    mockSnapshot.state.agents = [];
  }
  if (!mockSnapshot.state.agents.some((agent) => agent.id === id)) {
    mockSnapshot.state.agents.push({
      id,
      name,
      path,
      rules: {},
    });
  }
  return snapshot();
}

export function deleteAgent(agentId: string): Promise<AppSnapshot> {
  if (mockSnapshot.state.agents) {
    mockSnapshot.state.agents = mockSnapshot.state.agents.filter((agent) => agent.id !== agentId);
  }
  return snapshot();
}

export function setAgentRule(request: SetAgentRuleRequest): Promise<AppSnapshot> {
  if (mockSnapshot.state.agents) {
    const agent = mockSnapshot.state.agents.find((item) => item.id === request.agentId);
    if (agent) {
      if (request.rule === "inherit") {
        delete agent.rules[request.skillId];
      } else {
        agent.rules[request.skillId] = request.rule;
      }
    }
  }
  return snapshot();
}

export function scanAgentSkills(agentPath: string): Promise<ScannedCategory[]> {
  const skills = mockSnapshot.state.skills.map((skill, index) => {
    const targetPath = `${agentPath}/${skill.id}`.replace(/[\\/]+/g, "/");
    const hasReference = skill.references?.some(
      (reference) => reference.targetPath.replace(/[\\/]+/g, "/") === targetPath
    );
    const isManaged = Boolean(hasReference) || index < 3;
    return {
      id: skill.id,
      name: skill.name,
      description: skill.description,
      path: `${agentPath}/${skill.id}`,
      isManaged,
    };
  });
  return Promise.resolve([
    {
      name: ".",
      path: agentPath,
      skills,
    },
  ]);
}

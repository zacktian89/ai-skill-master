import type {
  AppSnapshot,
  AddProjectRequest,
  SetProjectRuleRequest,
  ImportProjectSkillResult,
} from "../../types";
import { mockSnapshot, snapshot } from "./data";

export function addProject(request: AddProjectRequest): Promise<AppSnapshot> {
  const id = request.name.toLowerCase().replace(/[^a-z0-9-]+/g, "-");
  if (!mockSnapshot.state.projects.some((project) => project.id === id)) {
    mockSnapshot.state.projects.push({
      id,
      name: request.name,
      path: request.path,
      rules: {},
    });
  }
  return snapshot();
}

export function setProjectRule(request: SetProjectRuleRequest): Promise<AppSnapshot> {
  const project = mockSnapshot.state.projects.find((item) => item.id === request.projectId);
  if (project) {
    if (request.rule === "inherit") {
      delete project.rules[request.skillId];
    } else {
      project.rules[request.skillId] = request.rule;
    }
  }
  return snapshot();
}

export function setCurrentProject(projectId: string | null): Promise<AppSnapshot> {
  mockSnapshot.state.currentProjectId = projectId;
  return snapshot();
}

export function resetProjectRules(projectId: string): Promise<AppSnapshot> {
  const project = mockSnapshot.state.projects.find((item) => item.id === projectId);
  if (project) {
    project.rules = {};
  }
  return snapshot();
}

export function deleteProject(projectId: string): Promise<AppSnapshot> {
  mockSnapshot.state.projects = mockSnapshot.state.projects.filter((project) => project.id !== projectId);
  if (mockSnapshot.state.currentProjectId === projectId) {
    mockSnapshot.state.currentProjectId = null;
  }
  return snapshot();
}

export function deleteUnmanagedSkill(_skillPath: string): Promise<AppSnapshot> {
  return snapshot();
}

function referenceId(path: string): string {
  return `ref-${path.replace(/[^a-zA-Z0-9]+/g, "-")}`;
}

export async function importProjectSkill(
  projectName: string,
  skillPath: string,
  _strategy?: "overwrite" | "keep_existing"
): Promise<ImportProjectSkillResult> {
  const normalizedPath = skillPath.replace(/[\\/]+/g, "/");
  const skillId = normalizedPath.split("/").filter(Boolean).pop();
  const skill = mockSnapshot.state.skills.find((item) => item.id === skillId);
  if (skill && skillId) {
    const rootPath = normalizedPath.slice(0, -skillId.length).replace(/\/$/, "");
    skill.references ??= [];
    if (!skill.references.some((reference) => reference.targetPath.replace(/[\\/]+/g, "/") === normalizedPath)) {
      skill.references.push({
        id: referenceId(normalizedPath),
        targetName: projectName,
        targetPath: skillPath,
        scope: "project",
        status: "healthy",
      });
    }
    for (const agent of mockSnapshot.state.agents || []) {
      if (agent.path.replace(/[\\/]+/g, "/") === rootPath) {
        agent.rules[skill.id] = "enable";
      }
    }
  }
  return { type: "success", snapshot: await snapshot() };
}

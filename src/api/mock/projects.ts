import type {
  AppSnapshot,
  AddProjectRequest,
  SetProjectRuleRequest,
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

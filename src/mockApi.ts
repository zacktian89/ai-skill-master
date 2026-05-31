import type {
  AddProjectRequest,
  AddSkillReferenceRequest,
  AppSnapshot,
  ConfirmImportSkillsRequest,
  DeleteSkillPreview,
  ImportSkillCandidate,
  ImportSkillPreview,
  ImportSkillSource,
  ProjectRule,
  SetProjectRuleRequest,
  Skill,
} from "./types";

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

const mockSnapshot: AppSnapshot = {
  state: {
    schemaVersion: 1,
    skillLibraryPath: "/Users/demo/.skillmaster/skills",
    codexSkillsPath: "/Users/demo/.codex/skills",
    currentProjectId: "acme-web",
    syncStatus: {
      phase: "healthy",
      message: "Mock 环境已同步到浏览器预览状态。",
      pendingActions: [],
    },
    migrationNotice: null,
    skills: [
      {
        id: "writer-pro",
        name: "Writer Pro",
        description: "长文写作与风格控制",
        libraryPath: "/Users/demo/.skillmaster/skills/writer-pro",
        references: [
          {
            id: "ref-claude-writer-pro",
            targetName: "Claude Code",
            targetPath: "/Users/demo/.claude/skills/writer-pro",
            scope: "user",
            status: "healthy",
          },
        ],
        managedLinks: {
          codex: "/Users/demo/.codex/skills/writer-pro",
        },
        conflict: null,
      },
      {
        id: "ui-auditor",
        name: "UI Auditor",
        description: "检查界面层级、配色和布局一致性",
        libraryPath: "/Users/demo/.skillmaster/skills/ui-auditor",
        references: [],
        managedLinks: {
          codex: "/Users/demo/.codex/skills/ui-auditor",
        },
        conflict: null,
      },
      {
        id: "deploy-guard",
        name: "Deploy Guard",
        description: "发布前校验与回滚预案",
        libraryPath: "/Users/demo/.skillmaster/skills/deploy-guard",
        references: [],
        managedLinks: {
          codex: null,
        },
        conflict: null,
      },
      {
        id: "legacy-review",
        name: "Legacy Review",
        description: "旧系统审计与兼容性分析",
        libraryPath: "/Users/demo/.skillmaster/skills/legacy-review",
        references: [],
        managedLinks: {
          codex: "/Users/demo/.codex/skills/legacy-review",
        },
        conflict: {
          target: "codex",
          path: "/Users/demo/.codex/skills/legacy-review",
          message: "目标目录包含非托管内容",
        },
      },
    ],
    projects: [
      {
        id: "acme-web",
        name: "Acme Web",
        path: "/work/acme/web",
        rules: {
          "deploy-guard": "enable",
          "legacy-review": "disable",
        },
      },
      {
        id: "mobile-shell",
        name: "Mobile Shell",
        path: "/work/acme/mobile-shell",
        rules: {
          "writer-pro": "disable",
        },
      },
      {
        id: "ops-kit",
        name: "Ops Kit",
        path: "/work/internal/ops-kit",
        rules: {},
      },
    ],
  },
  targetProfiles: [
    {
      id: "codex-user",
      targetName: "Codex",
      rootPath: "/Users/demo/.agents/skills",
      scope: "user",
    },
    {
      id: "claude-user",
      targetName: "Claude Code",
      rootPath: "/Users/demo/.claude/skills",
      scope: "user",
    },
    {
      id: "copilot-user",
      targetName: "GitHub Copilot",
      rootPath: "/Users/demo/.copilot/skills",
      scope: "user",
    },
    {
      id: "cursor-user",
      targetName: "Cursor",
      rootPath: "/Users/demo/.cursor/skills",
      scope: "user",
    },
    {
      id: "windsurf-user",
      targetName: "Windsurf",
      rootPath: "/Users/demo/.codeium/windsurf/skills",
      scope: "user",
    },
    {
      id: "kiro-user",
      targetName: "Kiro",
      rootPath: "/Users/demo/.kiro/skills",
      scope: "user",
    },
    {
      id: "opencode-user",
      targetName: "OpenCode",
      rootPath: "/Users/demo/.config/opencode/skill",
      scope: "user",
    },
  ],
  diagnostics: [
    {
      level: "warning",
      code: "codex-conflict",
      title: "Skill 冲突：legacy-review",
      detail: "目标目录包含非托管内容，需要人工确认后再同步。",
    },
  ],
  paths: {
    stateFile: "/Users/demo/.skillmaster/state.json",
    backupFile: "/Users/demo/.skillmaster/state.json.bak",
  },
  stateLoad: {
    phase: "clean",
    message: null,
  },
};

function skillName(skillId: string): string {
  return mockSnapshot.state.skills.find((skill) => skill.id === skillId)?.name ?? skillId;
}

function effectiveEnabled(skill: Skill): boolean {
  const currentProject = mockSnapshot.state.projects.find((project) => project.id === mockSnapshot.state.currentProjectId);
  const projectRule = currentProject?.rules[skill.id];
  if (projectRule === "enable") return true;
  if (projectRule === "disable") return false;
  return Boolean(skill.managedLinks.codex);
}

function snapshot(): Promise<AppSnapshot> {
  return Promise.resolve(clone(mockSnapshot));
}

function syncMockCodex() {
  for (const skill of mockSnapshot.state.skills) {
    if (effectiveEnabled(skill)) {
      skill.managedLinks.codex = `${mockSnapshot.state.codexSkillsPath}/${skill.id}`;
    } else {
      skill.managedLinks.codex = null;
    }
  }
  mockSnapshot.state.syncStatus = {
    phase: "healthy",
    message: "Mock 环境同步完成。",
    pendingActions: [],
  };
}

export function getSnapshot(): Promise<AppSnapshot> {
  return snapshot();
}

export function importSkill(source: string): Promise<AppSnapshot> {
  const name = source.split(/[\\/]/).filter(Boolean).pop() ?? "new-skill";
  const id = name.toLowerCase().replace(/[^a-z0-9-]+/g, "-");
  mockSnapshot.state.skills.unshift({
    id,
    name,
    description: "浏览器 mock 导入的示例 skill",
    libraryPath: `${mockSnapshot.state.skillLibraryPath}/${id}`,
    references: [],
    managedLinks: {
      codex: null,
    },
    conflict: null,
  });
  mockSnapshot.state.syncStatus.pendingActions = [
    {
      kind: "create",
      skillId: id,
      target: `${mockSnapshot.state.codexSkillsPath}/${id}`,
      source: `${mockSnapshot.state.skillLibraryPath}/${id}`,
      message: "新导入 skill 待同步到 Codex。",
    },
  ];
  mockSnapshot.state.syncStatus.phase = "repairRequired";
  mockSnapshot.state.syncStatus.message = "存在待同步项。";
  return snapshot();
}

function mockImportCandidates(source: ImportSkillSource): ImportSkillCandidate[] {
  const base =
    source.kind === "local"
      ? source.path.split(/[\\/]/).filter(Boolean).pop() || "local-pack"
      : source.url.split("/").filter(Boolean).pop()?.replace(/\.git$/, "") || "github-pack";
  const ids = [`${base}-writer`, `${base}-reviewer`].map((value) =>
    value.toLowerCase().replace(/[^a-z0-9-]+/g, "-"),
  );
  return ids.map((id, index) => {
    const duplicate = mockSnapshot.state.skills.some((skill) => skill.id === id);
    return {
      candidateId: index === 0 ? "writer" : "reviewer",
      id,
      name: id
        .split("-")
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(" "),
      description: index === 0 ? "Mock 扫描到的写作 skill" : "Mock 扫描到的审阅 skill",
      relativePath: index === 0 ? "writer" : "reviewer",
      status: duplicate ? "duplicate" : "ready",
      message: duplicate ? "已存在" : null,
    };
  });
}

export function previewImportSkills(source: ImportSkillSource): Promise<ImportSkillPreview> {
  return Promise.resolve({
    candidates: mockImportCandidates(source),
  });
}

export function confirmImportSkills(request: ConfirmImportSkillsRequest): Promise<AppSnapshot> {
  const candidates = mockImportCandidates(request.source);
  const selected = new Set(request.candidateIds);
  for (const candidate of candidates) {
    if (!selected.has(candidate.candidateId) || candidate.status !== "ready") continue;
    mockSnapshot.state.skills.unshift({
      id: candidate.id,
      name: candidate.name,
      description: candidate.description,
      libraryPath: `${mockSnapshot.state.skillLibraryPath}/${candidate.id}`,
      source:
        request.source.kind === "github"
          ? {
              kind: "github",
              label: "GitHub",
              url: request.source.url,
              ref: request.source.ref ?? null,
              commit: "mock-commit",
              subdir: candidate.relativePath,
            }
          : {
              kind: "local",
              label: "本地",
              path: `${request.source.path}/${candidate.relativePath}`,
            },
      references: [],
      managedLinks: {
        codex: null,
      },
      conflict: null,
    });
  }
  return snapshot();
}

export function deleteSkill(skillId: string): Promise<AppSnapshot> {
  mockSnapshot.state.skills = mockSnapshot.state.skills.filter((skill) => skill.id !== skillId);
  for (const project of mockSnapshot.state.projects) {
    delete project.rules[skillId];
  }
  return snapshot();
}

export function previewDeleteSkill(skillId: string): Promise<DeleteSkillPreview> {
  const skill = mockSnapshot.state.skills.find((item) => item.id === skillId);
  if (!skill) {
    return Promise.reject(new Error(`找不到 skill：${skillId}`));
  }
  return Promise.resolve({
    skillId,
    skillName: skill.name,
    libraryPath: skill.libraryPath,
    managedLinkTargets: [
      ...(skill.managedLinks.codex ? [skill.managedLinks.codex] : []),
      ...(skill.references?.map((reference) => reference.targetPath) ?? []),
    ],
    affectedProjects: mockSnapshot.state.projects
      .filter((project) => project.rules[skillId])
      .map((project) => ({
        projectId: project.id,
        projectName: project.name,
        projectPath: project.path,
      })),
  });
}

function referenceId(path: string): string {
  return `ref-${path.replace(/[^a-zA-Z0-9]+/g, "-")}`;
}

export function setSkillLinkEnabled(skillId: string, enabled: boolean): Promise<AppSnapshot> {
  const skill = mockSnapshot.state.skills.find((item) => item.id === skillId);
  if (skill) {
    skill.managedLinks.codex = enabled ? `${mockSnapshot.state.codexSkillsPath}/${skill.id}` : null;
    mockSnapshot.state.syncStatus = {
      phase: "healthy",
      message: enabled ? "Mock 托管链接已创建。" : "Mock 托管链接已移除。",
      pendingActions: [],
    };
  }
  return snapshot();
}

export function addSkillReference(request: AddSkillReferenceRequest): Promise<AppSnapshot> {
  const skill = mockSnapshot.state.skills.find((item) => item.id === request.skillId);
  if (skill) {
    const targetPath = `${request.rootPath.replace(/[\\/]$/, "")}/${skill.id}`;
    skill.references ??= [];
    if (!skill.references.some((reference) => reference.targetPath === targetPath)) {
      skill.references.push({
        id: referenceId(targetPath),
        targetName: request.targetName,
        targetPath,
        scope: request.scope,
        status: "healthy",
      });
    }
  }
  return snapshot();
}

export function removeSkillReference(referenceId: string): Promise<AppSnapshot> {
  for (const skill of mockSnapshot.state.skills) {
    skill.references = skill.references?.filter((reference) => reference.id !== referenceId) ?? [];
  }
  return snapshot();
}

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

export function setCodexPath(path: string): Promise<AppSnapshot> {
  mockSnapshot.state.codexSkillsPath = path;
  return snapshot();
}

export function migrateLibrary(target: string): Promise<AppSnapshot> {
  const previous = mockSnapshot.state.skillLibraryPath;
  mockSnapshot.state.skillLibraryPath = target;
  for (const skill of mockSnapshot.state.skills) {
    skill.libraryPath = `${target}/${skill.id}`;
  }
  mockSnapshot.state.migrationNotice = {
    oldLibraryPath: previous,
    newLibraryPath: target,
    message: "浏览器 mock 中已更新技能库路径。",
    requiresCodexResync: true,
  };
  return snapshot();
}

export function rebuildState(): Promise<AppSnapshot> {
  mockSnapshot.stateLoad = {
    phase: "clean",
    message: "浏览器 mock 状态已重建。",
  };
  return snapshot();
}

export function syncCodex(): Promise<AppSnapshot> {
  syncMockCodex();
  mockSnapshot.diagnostics = mockSnapshot.diagnostics.map((item) =>
    item.code === "codex-conflict"
      ? {
          ...item,
          detail: `请继续检查 ${skillName("legacy-review")} 的目标目录；其余 mock skill 已完成同步。`,
        }
      : item,
  );
  return snapshot();
}

export type { ProjectRule };

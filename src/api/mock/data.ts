import type { AppSnapshot, Skill } from "../../types";

export function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

export const mockSnapshot: AppSnapshot = {
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
    agents: [
      {
        id: "codex-agent",
        name: "Codex",
        path: "/Users/demo/.agents/skills",
        rules: {
          "writer-pro": "enable",
          "legacy-review": "disable",
        },
      },
      {
        id: "claude-agent",
        name: "Claude Code",
        path: "/Users/demo/.claude/skills",
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
      id: "gemini-user",
      targetName: "Gemini CLI",
      rootPath: "/Users/demo/.gemini/config/skills",
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
      id: "workbuddy-user",
      targetName: "WorkBuddy",
      rootPath: "/Users/demo/.workbuddy/skills",
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

export function skillName(skillId: string): string {
  return mockSnapshot.state.skills.find((skill) => skill.id === skillId)?.name ?? skillId;
}

export function effectiveEnabled(skill: Skill): boolean {
  const currentProject = mockSnapshot.state.projects.find((project) => project.id === mockSnapshot.state.currentProjectId);
  const projectRule = currentProject?.rules[skill.id];
  if (projectRule === "enable") return true;
  if (projectRule === "disable") return false;
  return Boolean(skill.managedLinks.codex);
}

export function snapshot(): Promise<AppSnapshot> {
  return Promise.resolve(clone(mockSnapshot));
}

export function syncMockCodex() {
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

import type {
  AppSnapshot,
  ImportSkillSource,
  ImportSkillPreview,
  ImportSkillCandidate,
  ConfirmImportSkillsRequest,
  DeleteSkillPreview,
  AddSkillReferenceRequest,
  StoreLeaderboardType,
  StoreSkill,
} from "../../types";
import { mockSnapshot, snapshot } from "./data";

export function importSkill(source: string): Promise<AppSnapshot> {
  const name = source.split(/[\\/]/).filter(Boolean).pop() ?? "new-skill";
  const id = name.toLowerCase().replace(/[^a-z0-9-]+/g, "-");
  mockSnapshot.state.skills.unshift({
    id,
    name,
    description: "浏览器 mock 导入的示例 skill",
    libraryPath: `${mockSnapshot.state.skillLibraryPath}/${id}`,
    references: [],
    managedLinks: {},
    conflict: null,
  });
  mockSnapshot.state.syncStatus.pendingActions = [];
  mockSnapshot.state.syncStatus.phase = "healthy";
  mockSnapshot.state.syncStatus.message = "Mock 环境已同步到浏览器预览状态。";
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

const mockStoreSkills: StoreSkill[] = [
  {
    id: "openai/skills/playwright",
    skillId: "playwright",
    name: "Playwright",
    source: "openai/skills",
    installs: 4200,
  },
  {
    id: "acme/skills/installed-skill",
    skillId: "installed-skill",
    name: "Installed Skill",
    source: "acme/skills",
    installs: 1200,
  },
  {
    id: "anthropic/skills/researcher",
    skillId: "researcher",
    name: "Researcher",
    source: "anthropic/skills",
    installs: 1800,
  },
];

export function fetchStoreLeaderboard(_board: StoreLeaderboardType): Promise<StoreSkill[]> {
  return Promise.resolve(mockStoreSkills);
}

export function searchStoreSkills(query: string, _limit = 60): Promise<StoreSkill[]> {
  const normalized = query.trim().toLowerCase();
  return Promise.resolve(
    mockStoreSkills.filter((skill) =>
      `${skill.name} ${skill.skillId} ${skill.source}`.toLowerCase().includes(normalized)
    )
  );
}

export function confirmImportSkills(request: ConfirmImportSkillsRequest): Promise<AppSnapshot> {
  const candidates = mockImportCandidates(request.source);
  const selected = new Set(request.candidateIds);
  for (const candidate of candidates) {
    if (!selected.has(candidate.candidateId)) continue;
    if (candidate.status !== "ready" && !(request.overwrite && candidate.status === "duplicate")) continue;
    if (request.overwrite) {
      mockSnapshot.state.skills = mockSnapshot.state.skills.filter((skill) => skill.id !== candidate.id);
    }
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
      managedLinks: {},
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
    managedLinkTargets: skill.references?.map((reference) => reference.targetPath) ?? [],
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

export function removeSkillReference(referenceId: string, _removeExternalLink?: boolean): Promise<AppSnapshot> {
  for (const skill of mockSnapshot.state.skills) {
    skill.references = skill.references?.filter((reference) => reference.id !== referenceId) ?? [];
  }
  return snapshot();
}

export function readSkillFile(skillId: string): Promise<string> {
  const skill = mockSnapshot.state.skills.find((s) => s.id === skillId);
  const name = skill?.name || skillId;
  const desc = skill?.description || "暂无描述";

  return Promise.resolve(`---
name: "${name}"
description: "${desc}"
author: "SkillMaster Mock"
version: "1.0.0"
---

# ${name}

这是 ${name} 的详细技能描述文件，包含使用规则与清单。

## 适用场景
- 自动化工作流校验与辅助开发。
- 当用户要求评估或执行 ${name} 相关任务时。

## 执行清单
- [ ] 校验项目上下文。
- [ ] 根据定义的原则对任务进行评估与修正。
- [ ] 导出检查报告。

## 示例代码
\`\`\`javascript
// 示例代码
function run() {
  console.log("Running ${name}");
}
\`\`\`
`);
}

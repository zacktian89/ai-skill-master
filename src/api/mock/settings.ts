import type { AppSnapshot } from "../../types";
import { mockSnapshot, snapshot, syncMockCodex, skillName } from "./data";

export function getSnapshot(): Promise<AppSnapshot> {
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

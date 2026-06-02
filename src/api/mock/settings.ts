import type { AppSnapshot } from "../../types";
import { mockSnapshot, snapshot } from "./data";

export function getSnapshot(): Promise<AppSnapshot> {
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
  };
  return snapshot();
}

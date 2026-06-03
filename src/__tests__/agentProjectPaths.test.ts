import { describe, expect, it } from "vitest";
import { AGENT_PRESETS, getNameMap, getRelPathForTarget } from "../config/agents";

function skillsParent(relPath: string): string {
  return relPath.replace(/[\\/]+skills$/, "");
}

describe("agent project skill paths", () => {
  it("uses project-local paths that project scanning can recognize", () => {
    const nameMap = getNameMap();

    for (const preset of AGENT_PRESETS) {
      const relPath = getRelPathForTarget(preset.targetName, preset.defaultPath);
      expect(relPath, preset.targetName).toMatch(/[\\/]skills$/);
      expect(nameMap[skillsParent(relPath)], preset.targetName).toBe(preset.targetName);
    }
  });

  it("keeps user-level paths separate from project-level paths for shared agent folders", () => {
    expect(getRelPathForTarget("Codex", "~/.codex/skills")).toBe(".agents/skills");
    expect(getRelPathForTarget("Gemini CLI", "~/.gemini/config/skills")).toBe(".gemini/skills");
    expect(getRelPathForTarget("OpenCode", "~/.config/opencode/skill")).toBe(".opencode/skills");
    expect(getRelPathForTarget("Antigravity", "~/.gemini/antigravity/skills")).toBe(".agent/skills");
    expect(getRelPathForTarget("Cline", "~/.agents/skills")).toBe(".cline/skills");
    expect(getRelPathForTarget("Deep Agents", "~/.deepagents/agent/skills")).toBe(".deepagents/skills");
    expect(getRelPathForTarget("Kimi Code CLI", "~/.config/agents/skills")).toBe(".kimi/skills");
    expect(getRelPathForTarget("Replit", "~/.config/agents/skills")).toBe(".replit/skills");
    expect(getRelPathForTarget("Warp", "~/.agents/skills")).toBe(".warp/skills");
  });
});

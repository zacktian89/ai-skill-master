export interface AgentPreset {
  name: string;
  targetName: string;
  defaultPath: string;
  relPath: string;
  detectedDirNames?: string[];
}

export const AGENT_PRESETS: AgentPreset[] = [
  {
    name: "Codex",
    targetName: "Codex",
    defaultPath: "~/.codex/skills",
    relPath: ".codex/skills",
    detectedDirNames: [".codex", ".agent", ".agents"],
  },
  {
    name: "Claude Code",
    targetName: "Claude Code",
    defaultPath: "~/.claude/skills",
    relPath: ".claude/skills",
    detectedDirNames: [".claude"],
  },
  {
    name: "Gemini CLI",
    targetName: "Gemini CLI",
    defaultPath: "~/.gemini/config/skills",
    relPath: ".gemini/config/skills",
    detectedDirNames: [".gemini"],
  },
  {
    name: "GitHub Copilot",
    targetName: "GitHub Copilot",
    defaultPath: "~/.copilot/skills",
    relPath: ".copilot/skills",
    detectedDirNames: [".copilot"],
  },
  {
    name: "Cursor",
    targetName: "Cursor",
    defaultPath: "~/.cursor/skills",
    relPath: ".cursor/skills",
    detectedDirNames: [".cursor"],
  },
  {
    name: "WorkBuddy",
    targetName: "WorkBuddy",
    defaultPath: "~/.workbuddy/skills",
    relPath: ".workbuddy/skills",
    detectedDirNames: [".workbuddy"],
  },
  {
    name: "Windsurf",
    targetName: "Windsurf",
    defaultPath: "~/.codeium/windsurf/skills",
    relPath: ".codeium/windsurf/skills",
    detectedDirNames: [".codeium/windsurf"],
  },
  {
    name: "Kiro",
    targetName: "Kiro",
    defaultPath: "~/.kiro/skills",
    relPath: ".kiro/skills",
    detectedDirNames: [".kiro"],
  },
  {
    name: "OpenCode",
    targetName: "OpenCode",
    defaultPath: "~/.config/opencode/skill",
    relPath: ".config/opencode/skill",
    detectedDirNames: [".opencode"],
  },
  {
    name: "CodeBuddy",
    targetName: "CodeBuddy",
    defaultPath: "~/.codebuddy/skills",
    relPath: ".codebuddy/skills",
    detectedDirNames: [".codebuddy"],
  },
];

export const DEFAULT_SKILLS_DIR = "skills";

/**
 * Get the relative path of skills inside a project for a specific agent target.
 */
export function getRelPathForTarget(targetName: string, rootPath?: string): string {
  const preset = AGENT_PRESETS.find((p) => p.targetName === targetName);
  if (preset) {
    return preset.relPath;
  }
  if (rootPath) {
    const match = rootPath.match(/[\\/](\.[^\\/]+[\\/].*)$/);
    if (match) {
      return match[1];
    }
  }
  return DEFAULT_SKILLS_DIR;
}

/**
 * Get name mapping for auto-detecting the target agent from directory names.
 */
export function getNameMap(): Record<string, string> {
  const map: Record<string, string> = {};
  for (const preset of AGENT_PRESETS) {
    if (preset.detectedDirNames) {
      for (const dirName of preset.detectedDirNames) {
        map[dirName] = preset.targetName;
      }
    }
  }
  return map;
}

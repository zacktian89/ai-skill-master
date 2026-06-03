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
  {
    name: "Antigravity",
    targetName: "Antigravity",
    defaultPath: "~/.gemini/antigravity/skills",
    relPath: ".gemini/antigravity/skills",
    detectedDirNames: [".gemini/antigravity"],
  },
  {
    name: "Amp",
    targetName: "Amp",
    defaultPath: "~/.config/agents/skills",
    relPath: ".config/agents/skills",
    detectedDirNames: [".config/agents"],
  },
  {
    name: "Kilo Code",
    targetName: "Kilo Code",
    defaultPath: "~/.kilocode/skills",
    relPath: ".kilocode/skills",
    detectedDirNames: [".kilocode"],
  },
  {
    name: "Roo Code",
    targetName: "Roo Code",
    defaultPath: "~/.roo/skills",
    relPath: ".roo/skills",
    detectedDirNames: [".roo"],
  },
  {
    name: "Goose",
    targetName: "Goose",
    defaultPath: "~/.config/goose/skills",
    relPath: ".config/goose/skills",
    detectedDirNames: [".config/goose"],
  },
  {
    name: "OpenClaw",
    targetName: "OpenClaw",
    defaultPath: "~/.openclaw/skills",
    relPath: ".openclaw/skills",
    detectedDirNames: [".openclaw"],
  },
  {
    name: "Droid",
    targetName: "Droid",
    defaultPath: "~/.factory/skills",
    relPath: ".factory/skills",
    detectedDirNames: [".factory"],
  },
  {
    name: "TRAE IDE",
    targetName: "TRAE IDE",
    defaultPath: "~/.trae/skills",
    relPath: ".trae/skills",
    detectedDirNames: [".trae"],
  },
  {
    name: "Cline",
    targetName: "Cline",
    defaultPath: "~/.agents/skills",
    relPath: ".agents/skills",
    detectedDirNames: [".cline"],
  },
  {
    name: "Deep Agents",
    targetName: "Deep Agents",
    defaultPath: "~/.deepagents/agent/skills",
    relPath: ".deepagents/agent/skills",
    detectedDirNames: [".deepagents"],
  },
  {
    name: "Firebender",
    targetName: "Firebender",
    defaultPath: "~/.firebender/skills",
    relPath: ".firebender/skills",
    detectedDirNames: [".firebender"],
  },
  {
    name: "Kimi Code CLI",
    targetName: "Kimi Code CLI",
    defaultPath: "~/.config/agents/skills",
    relPath: ".config/agents/skills",
    detectedDirNames: [".kimi"],
  },
  {
    name: "Replit",
    targetName: "Replit",
    defaultPath: "~/.config/agents/skills",
    relPath: ".config/agents/skills",
    detectedDirNames: [".replit"],
  },
  {
    name: "Warp",
    targetName: "Warp",
    defaultPath: "~/.agents/skills",
    relPath: ".agents/skills",
    detectedDirNames: [".warp"],
  },
  {
    name: "Augment",
    targetName: "Augment",
    defaultPath: "~/.augment/skills",
    relPath: ".augment/skills",
    detectedDirNames: [".augment"],
  },
  {
    name: "IBM Bob",
    targetName: "IBM Bob",
    defaultPath: "~/.bob/skills",
    relPath: ".bob/skills",
    detectedDirNames: [".bob"],
  },
  {
    name: "Command Code",
    targetName: "Command Code",
    defaultPath: "~/.commandcode/skills",
    relPath: ".commandcode/skills",
    detectedDirNames: [".commandcode"],
  },
  {
    name: "Continue",
    targetName: "Continue",
    defaultPath: "~/.continue/skills",
    relPath: ".continue/skills",
    detectedDirNames: [".continue"],
  },
  {
    name: "Cortex Code",
    targetName: "Cortex Code",
    defaultPath: "~/.snowflake/cortex/skills",
    relPath: ".snowflake/cortex/skills",
    detectedDirNames: [".snowflake/cortex"],
  },
  {
    name: "Crush",
    targetName: "Crush",
    defaultPath: "~/.config/crush/skills",
    relPath: ".config/crush/skills",
    detectedDirNames: [".config/crush"],
  },
  {
    name: "iFlow CLI",
    targetName: "iFlow CLI",
    defaultPath: "~/.iflow/skills",
    relPath: ".iflow/skills",
    detectedDirNames: [".iflow"],
  },
  {
    name: "Junie",
    targetName: "Junie",
    defaultPath: "~/.junie/skills",
    relPath: ".junie/skills",
    detectedDirNames: [".junie"],
  },
  {
    name: "Kode",
    targetName: "Kode",
    defaultPath: "~/.kode/skills",
    relPath: ".kode/skills",
    detectedDirNames: [".kode"],
  },
  {
    name: "MCPJam",
    targetName: "MCPJam",
    defaultPath: "~/.mcpjam/skills",
    relPath: ".mcpjam/skills",
    detectedDirNames: [".mcpjam"],
  },
  {
    name: "Mistral Vibe",
    targetName: "Mistral Vibe",
    defaultPath: "~/.vibe/skills",
    relPath: ".vibe/skills",
    detectedDirNames: [".vibe"],
  },
  {
    name: "Mux",
    targetName: "Mux",
    defaultPath: "~/.mux/skills",
    relPath: ".mux/skills",
    detectedDirNames: [".mux"],
  },
  {
    name: "Neovate",
    targetName: "Neovate",
    defaultPath: "~/.neovate/skills",
    relPath: ".neovate/skills",
    detectedDirNames: [".neovate"],
  },
  {
    name: "OpenHands",
    targetName: "OpenHands",
    defaultPath: "~/.openhands/skills",
    relPath: ".openhands/skills",
    detectedDirNames: [".openhands"],
  },
  {
    name: "Pi",
    targetName: "Pi",
    defaultPath: "~/.pi/agent/skills",
    relPath: ".pi/agent/skills",
    detectedDirNames: [".pi/agent"],
  },
  {
    name: "Pochi",
    targetName: "Pochi",
    defaultPath: "~/.pochi/skills",
    relPath: ".pochi/skills",
    detectedDirNames: [".pochi"],
  },
  {
    name: "Qoder",
    targetName: "Qoder",
    defaultPath: "~/.qoder/skills",
    relPath: ".qoder/skills",
    detectedDirNames: [".qoder"],
  },
  {
    name: "Qwen Code",
    targetName: "Qwen Code",
    defaultPath: "~/.qwen/skills",
    relPath: ".qwen/skills",
    detectedDirNames: [".qwen"],
  },
  {
    name: "TRAE CN",
    targetName: "TRAE CN",
    defaultPath: "~/.trae-cn/skills",
    relPath: ".trae-cn/skills",
    detectedDirNames: [".trae-cn"],
  },
  {
    name: "Zencoder",
    targetName: "Zencoder",
    defaultPath: "~/.zencoder/skills",
    relPath: ".zencoder/skills",
    detectedDirNames: [".zencoder"],
  },
  {
    name: "AdaL",
    targetName: "AdaL",
    defaultPath: "~/.adal/skills",
    relPath: ".adal/skills",
    detectedDirNames: [".adal"],
  },
  {
    name: "Hermes Agent",
    targetName: "Hermes Agent",
    defaultPath: "~/.hermes/skills",
    relPath: ".hermes/skills",
    detectedDirNames: [".hermes"],
  },
  {
    name: "QClaw",
    targetName: "QClaw",
    defaultPath: "~/.qclaw/skills",
    relPath: ".qclaw/skills",
    detectedDirNames: [".qclaw"],
  },
  {
    name: "EasyClaw",
    targetName: "EasyClaw",
    defaultPath: "~/.easyclaw/skills",
    relPath: ".easyclaw/skills",
    detectedDirNames: [".easyclaw"],
  },
  {
    name: "AutoClaw",
    targetName: "AutoClaw",
    defaultPath: "~/.openclaw-autoclaw/skills",
    relPath: ".openclaw-autoclaw/skills",
    detectedDirNames: [".openclaw-autoclaw"],
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

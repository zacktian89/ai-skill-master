use crate::models::ReferenceScope;
use crate::path_utils::user_home_path;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillTargetProfile {
    pub id: String,
    pub target_name: String,
    pub root_path: PathBuf,
    pub scope: ReferenceScope,
}

pub fn built_in_target_profiles() -> Vec<SkillTargetProfile> {
    let home = user_home_path();
    let specs: [(&str, &str, &[&str]); 49] = [
        ("codex-user", "Codex", &[".codex", "skills"]),
        ("claude-user", "Claude Code", &[".claude", "skills"]),
        (
            "gemini-user",
            "Gemini CLI",
            &[".gemini", "config", "skills"],
        ),
        ("copilot-user", "GitHub Copilot", &[".copilot", "skills"]),
        ("cursor-user", "Cursor", &[".cursor", "skills"]),
        ("workbuddy-user", "WorkBuddy", &[".workbuddy", "skills"]),
        (
            "windsurf-user",
            "Windsurf",
            &[".codeium", "windsurf", "skills"],
        ),
        ("kiro-user", "Kiro", &[".kiro", "skills"]),
        (
            "opencode-user",
            "OpenCode",
            &[".config", "opencode", "skill"],
        ),
        ("codebuddy-user", "CodeBuddy", &[".codebuddy", "skills"]),
        (
            "antigravity-user",
            "Antigravity",
            &[".gemini", "antigravity", "skills"],
        ),
        ("amp-user", "Amp", &[".config", "agents", "skills"]),
        ("kilocode-user", "Kilo Code", &[".kilocode", "skills"]),
        ("roo-user", "Roo Code", &[".roo", "skills"]),
        ("goose-user", "Goose", &[".config", "goose", "skills"]),
        ("openclaw-user", "OpenClaw", &[".openclaw", "skills"]),
        ("droid-user", "Droid", &[".factory", "skills"]),
        ("trae-user", "TRAE IDE", &[".trae", "skills"]),
        ("cline-user", "Cline", &[".agents", "skills"]),
        (
            "deepagents-user",
            "Deep Agents",
            &[".deepagents", "agent", "skills"],
        ),
        ("firebender-user", "Firebender", &[".firebender", "skills"]),
        (
            "kimi-user",
            "Kimi Code CLI",
            &[".config", "agents", "skills"],
        ),
        ("replit-user", "Replit", &[".config", "agents", "skills"]),
        ("warp-user", "Warp", &[".agents", "skills"]),
        ("augment-user", "Augment", &[".augment", "skills"]),
        ("bob-user", "IBM Bob", &[".bob", "skills"]),
        (
            "command_code-user",
            "Command Code",
            &[".commandcode", "skills"],
        ),
        ("continue-user", "Continue", &[".continue", "skills"]),
        (
            "cortex-user",
            "Cortex Code",
            &[".snowflake", "cortex", "skills"],
        ),
        ("crush-user", "Crush", &[".config", "crush", "skills"]),
        ("iflow-user", "iFlow CLI", &[".iflow", "skills"]),
        ("junie-user", "Junie", &[".junie", "skills"]),
        ("kode-user", "Kode", &[".kode", "skills"]),
        ("mcpjam-user", "MCPJam", &[".mcpjam", "skills"]),
        ("mistral_vibe-user", "Mistral Vibe", &[".vibe", "skills"]),
        ("mux-user", "Mux", &[".mux", "skills"]),
        ("neovate-user", "Neovate", &[".neovate", "skills"]),
        ("openhands-user", "OpenHands", &[".openhands", "skills"]),
        ("pi-user", "Pi", &[".pi", "agent", "skills"]),
        ("pochi-user", "Pochi", &[".pochi", "skills"]),
        ("qoder-user", "Qoder", &[".qoder", "skills"]),
        ("qwen-user", "Qwen Code", &[".qwen", "skills"]),
        ("trae_cn-user", "TRAE CN", &[".trae-cn", "skills"]),
        ("zencoder-user", "Zencoder", &[".zencoder", "skills"]),
        ("adal-user", "AdaL", &[".adal", "skills"]),
        ("hermes-user", "Hermes Agent", &[".hermes", "skills"]),
        ("qclaw-user", "QClaw", &[".qclaw", "skills"]),
        ("easyclaw-user", "EasyClaw", &[".easyclaw", "skills"]),
        (
            "autoclaw-user",
            "AutoClaw",
            &[".openclaw-autoclaw", "skills"],
        ),
    ];

    specs
        .iter()
        .map(|(id, target_name, segments)| {
            let mut root_path = home.clone();
            for segment in *segments {
                root_path = root_path.join(segment);
            }
            SkillTargetProfile {
                id: (*id).to_string(),
                target_name: (*target_name).to_string(),
                root_path,
                scope: ReferenceScope::User,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_profiles_include_new_agent_skill_paths() {
        let profiles = built_in_target_profiles();

        assert!(profiles.iter().any(|profile| {
            profile.target_name == "Gemini CLI"
                && profile.root_path.ends_with(
                    std::path::Path::new(".gemini")
                        .join("config")
                        .join("skills"),
                )
        }));
        assert!(profiles.iter().any(|profile| {
            profile.target_name == "WorkBuddy"
                && profile
                    .root_path
                    .ends_with(std::path::Path::new(".workbuddy").join("skills"))
        }));
    }
}

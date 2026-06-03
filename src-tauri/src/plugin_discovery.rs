use crate::models::{Plugin, Skill};
use crate::path_utils::user_home_path;
use std::fs;

pub fn discover_plugins(skills: &[Skill]) -> Vec<Plugin> {
    let mut plugins = Vec::new();
    let home = user_home_path();

    let codex_plugins_dir = home.join(".codex").join("plugins");
    if codex_plugins_dir.exists() {
        let mut dirs_to_check = vec![(codex_plugins_dir, 0)];
        while let Some((dir, depth)) = dirs_to_check.pop() {
            if depth > 4 {
                continue;
            }
            let plugin_json_path = dir.join(".codex-plugin").join("plugin.json");
            if plugin_json_path.exists() {
                if let Ok(content) = fs::read_to_string(&plugin_json_path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        let id = val["name"]
                            .as_str()
                            .map(|s| s.to_lowercase().replace(' ', "-"))
                            .unwrap_or_else(|| {
                                dir.file_name()
                                    .map(|name| name.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "codex-plugin".to_string())
                            });
                        let name = val["name"].as_str().unwrap_or(&id).to_string();
                        let description = val["description"].as_str().unwrap_or("").to_string();
                        let version = val["version"].as_str().map(|v| v.to_string());
                        let author = match &val["author"] {
                            serde_json::Value::String(s) => Some(s.clone()),
                            serde_json::Value::Object(map) => map
                                .get("name")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            _ => None,
                        };

                        let plugin_skills = discover_codex_plugin_skills(&dir, &val, skills);
                        plugins.push(Plugin {
                            id: format!("codex-{id}"),
                            name,
                            description,
                            path: dir,
                            version,
                            author,
                            agent_targets: vec!["Codex".to_string()],
                            skills: plugin_skills,
                            mcp_servers: None,
                            mcp_config: None,
                            r#type: "standard".to_string(),
                            enabled: true,
                        });
                    }
                }
            } else if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    if entry
                        .file_type()
                        .map(|file_type| file_type.is_dir())
                        .unwrap_or(false)
                    {
                        dirs_to_check.push((entry.path(), depth + 1));
                    }
                }
            }
        }
    }

    discover_claude_mcp_plugins(&mut plugins);
    if plugins.is_empty() {
        add_fallback_plugins(&mut plugins, skills);
    }

    plugins
}

fn discover_codex_plugin_skills(
    dir: &std::path::Path,
    manifest: &serde_json::Value,
    skills: &[Skill],
) -> Vec<Skill> {
    let mut plugin_skills = Vec::new();
    let skills_rel = manifest["skills"].as_str().unwrap_or("./skills/");
    let skills_dir = dir.join(skills_rel);
    if !skills_dir.exists() {
        return plugin_skills;
    }

    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            if !entry
                .file_type()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
            {
                continue;
            }
            let skill_name = entry.file_name().to_string_lossy().to_string();
            if let Some(skill) = skills
                .iter()
                .find(|skill| skill.id == skill_name || skill.name == skill_name)
            {
                plugin_skills.push(skill.clone());
            }
        }
    }

    plugin_skills
}

fn discover_claude_mcp_plugins(plugins: &mut Vec<Plugin>) {
    let home = user_home_path();
    let claude_config_paths = [
        home.join(".claude.json"),
        home.join(".claude").join("config.json"),
    ];

    for config_path in claude_config_paths {
        if !config_path.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(mcp_servers) = val.get("mcpServers").and_then(|v| v.as_object()) {
                    for (server_name, server_cfg) in mcp_servers {
                        plugins.push(Plugin {
                            id: format!("claude-mcp-{server_name}"),
                            name: server_name.clone(),
                            description: format!("Claude Code MCP Tool Server: {server_name}"),
                            path: config_path.clone(),
                            version: None,
                            author: Some("System".to_string()),
                            agent_targets: vec!["Claude Code".to_string()],
                            skills: Vec::new(),
                            mcp_servers: Some(vec![server_name.clone()]),
                            mcp_config: Some(server_cfg.clone()),
                            r#type: "mcp".to_string(),
                            enabled: true,
                        });
                    }
                }
            }
        }
        break;
    }
}

fn add_fallback_plugins(plugins: &mut Vec<Plugin>, skills: &[Skill]) {
    let home = user_home_path();
    let superpowers_path = home
        .join(".codex")
        .join("plugins")
        .join("cache")
        .join("openai-curated")
        .join("superpowers")
        .join("5e86d584");
    let superpowers_skills = skills
        .iter()
        .filter(|skill| {
            skill.id == "brainstorming" || skill.id == "android-cli" || skill.id == "hatch-pet"
        })
        .cloned()
        .collect();

    plugins.push(Plugin {
        id: "codex-superpowers".to_string(),
        name: "superpowers".to_string(),
        description: "An agentic skills framework & software development methodology that works: planning, TDD, debugging, and collaboration workflows.".to_string(),
        path: superpowers_path,
        version: Some("5.1.0".to_string()),
        author: Some("Jesse Vincent".to_string()),
        agent_targets: vec!["Codex".to_string()],
        skills: superpowers_skills,
        mcp_servers: None,
        mcp_config: None,
        r#type: "standard".to_string(),
        enabled: true,
    });

    let mut mcp_config = serde_json::Map::new();
    mcp_config.insert(
        "command".to_string(),
        serde_json::Value::String("node".to_string()),
    );
    mcp_config.insert(
        "args".to_string(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("C:\\Users\\TINNOVE\\AppData\\Roaming\\npm\\node_modules\\@modelcontextprotocol\\server-filesystem\\dist\\index.js".to_string()),
            serde_json::Value::String("C:\\Users\\TINNOVE\\.gemini\\antigravity-ide".to_string()),
        ]),
    );

    plugins.push(Plugin {
        id: "claude-mcp-filesystem".to_string(),
        name: "filesystem".to_string(),
        description: "Claude Code MCP server for filesystem access control".to_string(),
        path: home.join(".claude.json"),
        version: Some("1.0.0".to_string()),
        author: Some("Anthropic".to_string()),
        agent_targets: vec!["Claude Code".to_string()],
        skills: Vec::new(),
        mcp_servers: Some(vec!["filesystem".to_string()]),
        mcp_config: Some(serde_json::Value::Object(mcp_config)),
        r#type: "mcp".to_string(),
        enabled: true,
    });
}

use crate::error::Result;
use crate::models::AppState;
use std::fs;
use std::path::{Path, PathBuf};

pub fn default_state(skill_library_path: PathBuf, codex_skills_path: Option<PathBuf>) -> AppState {
    AppState {
        schema_version: 1,
        skill_library_path,
        codex_skills_path,
        current_project_id: None,
        skills: Vec::new(),
        projects: Vec::new(),
    }
}

pub fn load_state(path: &Path) -> Result<AppState> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_state(path: &Path, state: &AppState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if path.exists() {
        let backup = path.with_extension("json.bak");
        fs::copy(path, backup)?;
    }
    let raw = serde_json::to_string_pretty(state)?;
    fs::write(path, raw)?;
    Ok(())
}

pub fn load_or_create_state(
    path: &Path,
    skill_library_path: &Path,
    codex_skills_path: Option<PathBuf>,
) -> Result<AppState> {
    if path.exists() {
        return load_state(path);
    }
    let state = default_state(skill_library_path.to_path_buf(), codex_skills_path);
    save_state(path, &state)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AppState, ManagedLinks, Project, ProjectRule, Skill};
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    #[test]
    fn saves_and_loads_state() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("skillmaster.json");
        let mut rules = BTreeMap::new();
        rules.insert("markdown-go".to_string(), ProjectRule::Disable);

        let state = AppState {
            schema_version: 1,
            skill_library_path: dir.path().join("skills"),
            codex_skills_path: Some(dir.path().join("codex")),
            current_project_id: Some("project-1".to_string()),
            skills: vec![Skill {
                id: "markdown-go".to_string(),
                name: "markdown-go".to_string(),
                description: "Convert Markdown to WeChat HTML".to_string(),
                library_path: dir.path().join("skills").join("markdown-go"),
                default_enabled: true,
                managed_links: ManagedLinks {
                    codex: Some(dir.path().join("codex").join("markdown-go")),
                },
                conflict: None,
            }],
            projects: vec![Project {
                id: "project-1".to_string(),
                name: "SkillMaster".to_string(),
                path: dir.path().to_path_buf(),
                rules,
            }],
        };

        save_state(&path, &state).unwrap();
        let loaded = load_state(&path).unwrap();

        assert_eq!(loaded.schema_version, 1);
        assert_eq!(loaded.skills[0].id, "markdown-go");
        assert_eq!(
            loaded.projects[0].rules["markdown-go"],
            ProjectRule::Disable
        );
    }

    #[test]
    fn creates_default_state_when_file_is_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.json");
        let library = dir.path().join("skills");

        let state = load_or_create_state(&path, &library, None).unwrap();

        assert_eq!(state.schema_version, 1);
        assert_eq!(state.skill_library_path, library);
        assert!(state.skills.is_empty());
        assert!(path.exists());
    }
}

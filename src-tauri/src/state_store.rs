use crate::error::Result;
use crate::models::AppState;
use crate::skill_library::scan_skill_library;
use std::fs;
use std::path::{Path, PathBuf};

pub fn default_state(skill_library_path: PathBuf, codex_skills_path: Option<PathBuf>) -> AppState {
    AppState {
        schema_version: 1,
        skill_library_path,
        codex_skills_path,
        current_project_id: None,
        sync_status: Default::default(),
        migration_notice: None,
        skills: Vec::new(),
        projects: Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedState {
    pub state: AppState,
    pub load_status: StateLoadStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateLoadStatus {
    Clean,
    RestoredFromBackup { message: String },
    RebuildRequired { message: String },
}

pub fn state_backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.bak")
}

pub fn load_state(path: &Path) -> Result<AppState> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_state(path: &Path, state: &AppState) -> Result<()> {
    write_state(path, state, true)
}

fn write_state(path: &Path, state: &AppState, refresh_backup: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if refresh_backup && path.exists() {
        let backup = state_backup_path(path);
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
) -> Result<LoadedState> {
    fs::create_dir_all(skill_library_path)?;
    if path.exists() {
        return load_state_with_recovery(path, skill_library_path, codex_skills_path);
    }
    let state = default_state(skill_library_path.to_path_buf(), codex_skills_path);
    save_state(path, &state)?;
    Ok(LoadedState {
        state,
        load_status: StateLoadStatus::Clean,
    })
}

pub fn rebuild_state_from_library(
    skill_library_path: &Path,
    codex_skills_path: Option<PathBuf>,
) -> Result<AppState> {
    fs::create_dir_all(skill_library_path)?;
    let mut state = default_state(skill_library_path.to_path_buf(), codex_skills_path);
    state.skills = scan_skill_library(skill_library_path)?;
    Ok(state)
}

fn load_state_with_recovery(
    path: &Path,
    skill_library_path: &Path,
    codex_skills_path: Option<PathBuf>,
) -> Result<LoadedState> {
    match load_state(path) {
        Ok(state) => Ok(LoadedState {
            state,
            load_status: StateLoadStatus::Clean,
        }),
        Err(main_error) => {
            let backup_path = state_backup_path(path);
            if backup_path.exists() {
                match load_state(&backup_path) {
                    Ok(state) => {
                        write_state(path, &state, false)?;
                        return Ok(LoadedState {
                            state,
                            load_status: StateLoadStatus::RestoredFromBackup {
                                message: format!(
                                    "主状态文件已损坏，已从备份恢复。主文件：{}；备份：{}",
                                    path.display(),
                                    backup_path.display()
                                ),
                            },
                        });
                    }
                    Err(backup_error) => {
                        let state =
                            rebuild_state_from_library(skill_library_path, codex_skills_path)?;
                        return Ok(LoadedState {
                            state,
                            load_status: StateLoadStatus::RebuildRequired {
                                message: format!(
                                    "主状态文件和备份都不可用。主文件错误：{main_error}；备份错误：{backup_error}"
                                ),
                            },
                        });
                    }
                }
            }

            let state = rebuild_state_from_library(skill_library_path, codex_skills_path)?;
            Ok(LoadedState {
                state,
                load_status: StateLoadStatus::RebuildRequired {
                    message: format!(
                        "主状态文件不可用且找不到备份，当前为待重建状态：{main_error}"
                    ),
                },
            })
        }
    }
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
            sync_status: Default::default(),
            migration_notice: None,
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

        assert_eq!(state.state.schema_version, 1);
        assert_eq!(state.state.skill_library_path, library);
        assert!(state.state.skills.is_empty());
        assert!(path.exists());
        assert!(library.exists());
    }

    #[test]
    fn restores_from_backup_when_primary_state_is_invalid() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("skillmaster.json");
        let backup = state_backup_path(&path);
        let library = dir.path().join("skills");

        fs::write(&path, "{ not-json").unwrap();
        let state = default_state(library.clone(), None);
        fs::write(&backup, serde_json::to_string(&state).unwrap()).unwrap();

        let loaded = load_or_create_state(&path, &library, None).unwrap();

        assert_eq!(
            loaded.load_status,
            StateLoadStatus::RestoredFromBackup {
                message: format!(
                    "主状态文件已损坏，已从备份恢复。主文件：{}；备份：{}",
                    path.display(),
                    backup.display()
                )
            }
        );
        assert_eq!(loaded.state.skill_library_path, library);
    }

    #[test]
    fn falls_back_to_rebuildable_state_when_primary_and_backup_are_invalid() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("skillmaster.json");
        let backup = state_backup_path(&path);
        let library = dir.path().join("skills");

        fs::write(&path, "{ invalid").unwrap();
        fs::write(&backup, "{ invalid").unwrap();

        let loaded = load_or_create_state(&path, &library, None).unwrap();

        assert!(matches!(
            loaded.load_status,
            StateLoadStatus::RebuildRequired { .. }
        ));
        assert_eq!(loaded.state.skill_library_path, library);
    }
}

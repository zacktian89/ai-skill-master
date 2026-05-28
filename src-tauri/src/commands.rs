use crate::app_paths::{detect_codex_skills_path, AppPaths};
use crate::codex_sync::{create_directory_link, plan_codex_sync, remove_managed_link};
use crate::effective_state::effective_skill_ids;
use crate::error::{Result, SkillMasterError};
use crate::models::{AppState, Project, ProjectRule, SkillConflict};
use crate::skill_library::{
    delete_skill as delete_skill_from_library, import_skill as import_skill_into_library,
    migrate_skill_library,
};
use crate::state_store::{load_or_create_state, save_state};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub state: AppState,
    pub codex_connected: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProjectRequest {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetProjectRuleRequest {
    pub project_id: String,
    pub skill_id: String,
    pub rule: ProjectRule,
}

pub fn build_snapshot(state: AppState) -> AppSnapshot {
    let codex_connected = state
        .codex_skills_path
        .as_ref()
        .map(|path| path.exists())
        .unwrap_or(false);
    let mut diagnostics = Vec::new();
    if !codex_connected {
        diagnostics.push("Codex skills 目录未连接".to_string());
    }
    AppSnapshot {
        state,
        codex_connected,
        diagnostics,
    }
}

fn command_paths(app: &AppHandle) -> Result<AppPaths> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| SkillMasterError::InvalidPath(error.to_string()))?;
    Ok(AppPaths::from_config_dir(&config_dir))
}

fn load_command_state(app: &AppHandle) -> Result<(AppPaths, AppState)> {
    let paths = command_paths(app)?;
    let home = app
        .path()
        .home_dir()
        .map_err(|error| SkillMasterError::InvalidPath(error.to_string()))?;
    let detected_codex = detect_codex_skills_path(&home);
    let codex = if detected_codex.exists() {
        Some(detected_codex)
    } else {
        None
    };
    let state = load_or_create_state(&paths.state_file, &paths.skill_library, codex)?;
    Ok((paths, state))
}

fn persist(paths: &AppPaths, state: &AppState) -> Result<()> {
    save_state(&paths.state_file, state)
}

fn project_id_from_path(path: &std::path::Path) -> String {
    let raw = path.to_string_lossy();
    format!("{:x}", md5_like_hash(raw.as_bytes()))
}

fn md5_like_hash(bytes: &[u8]) -> u64 {
    let mut hash = 1469598103934665603u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

#[tauri::command]
pub fn get_snapshot(app: AppHandle) -> std::result::Result<AppSnapshot, String> {
    load_command_state(&app)
        .map(|(_, state)| build_snapshot(state))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_skill(app: AppHandle, source: PathBuf) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    import_skill_into_library(&mut state, &source).map_err(|error| error.to_string())?;
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn delete_skill(app: AppHandle, skill_id: String) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    let links =
        delete_skill_from_library(&mut state, &skill_id).map_err(|error| error.to_string())?;
    for link in links {
        remove_managed_link(&link).map_err(|error| error.to_string())?;
    }
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn set_default_enabled(
    app: AppHandle,
    skill_id: String,
    enabled: bool,
) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    let skill = state
        .skills
        .iter_mut()
        .find(|skill| skill.id == skill_id)
        .ok_or_else(|| format!("找不到 skill：{skill_id}"))?;
    skill.default_enabled = enabled;
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn add_project(
    app: AppHandle,
    request: AddProjectRequest,
) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    let id = project_id_from_path(&request.path);
    if !state.projects.iter().any(|project| project.id == id) {
        state.projects.push(Project {
            id,
            name: request.name,
            path: request.path,
            rules: BTreeMap::new(),
        });
    }
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn set_project_rule(
    app: AppHandle,
    request: SetProjectRuleRequest,
) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    let project = state
        .projects
        .iter_mut()
        .find(|project| project.id == request.project_id)
        .ok_or_else(|| format!("找不到项目：{}", request.project_id))?;
    if request.rule == ProjectRule::Inherit {
        project.rules.remove(&request.skill_id);
    } else {
        project.rules.insert(request.skill_id, request.rule);
    }
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn set_current_project(
    app: AppHandle,
    project_id: Option<String>,
) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    state.current_project_id = project_id;
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn set_codex_path(app: AppHandle, path: PathBuf) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    state.codex_skills_path = Some(path);
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn migrate_library(
    app: AppHandle,
    target: PathBuf,
) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    migrate_skill_library(&mut state, &target).map_err(|error| error.to_string())?;
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn sync_codex(app: AppHandle) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    let codex_path = state
        .codex_skills_path
        .clone()
        .ok_or_else(|| "Codex skills 目录未设置".to_string())?;
    let active = effective_skill_ids(&state, state.current_project_id.as_deref())
        .map_err(|error| error.to_string())?;
    let report =
        plan_codex_sync(&state.skills, &active, &codex_path).map_err(|error| error.to_string())?;

    for skill in &mut state.skills {
        skill.conflict = None;
    }
    for conflict in report.conflicts {
        if let Some(skill) = state
            .skills
            .iter_mut()
            .find(|skill| skill.id == conflict.skill_id)
        {
            skill.conflict = Some(SkillConflict {
                target: "codex".to_string(),
                path: conflict.target,
                message: conflict.message,
            });
        }
    }
    for action in report.to_remove {
        remove_managed_link(&action.target).map_err(|error| error.to_string())?;
        if let Some(skill) = state
            .skills
            .iter_mut()
            .find(|skill| skill.id == action.skill_id)
        {
            skill.managed_links.codex = None;
        }
    }
    for action in report.to_create {
        create_directory_link(&action.source, &action.target).map_err(|error| error.to_string())?;
        if let Some(skill) = state
            .skills
            .iter_mut()
            .find(|skill| skill.id == action.skill_id)
        {
            skill.managed_links.codex = Some(action.target);
        }
    }
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ManagedLinks, Skill};
    use crate::state_store::default_state;
    use tempfile::tempdir;

    #[test]
    fn snapshot_marks_codex_connected_when_path_exists() {
        let dir = tempdir().unwrap();
        let codex = dir.path().join("codex");
        std::fs::create_dir_all(&codex).unwrap();
        let mut state = default_state(dir.path().join("skills"), Some(codex));
        state.skills.push(Skill {
            id: "writer".to_string(),
            name: "writer".to_string(),
            description: String::new(),
            library_path: dir.path().join("skills").join("writer"),
            default_enabled: true,
            managed_links: ManagedLinks::default(),
            conflict: None,
        });

        let snapshot = build_snapshot(state);

        assert!(snapshot.codex_connected);
        assert_eq!(snapshot.state.skills.len(), 1);
    }
}

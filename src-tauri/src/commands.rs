use crate::agent_state::{
    add_agent_to_state, delete_agent_from_state, scan_agent_skills as scan_agent_skills_from_state,
    set_agent_rule_in_state, SetAgentRuleRequest,
};
use crate::command_context::{load_command_state, persist};
use crate::project_state::{
    add_project_to_state, delete_project_from_state, reset_project_rules_in_state,
    set_current_project_in_state, set_project_rule_in_state, AddProjectRequest,
    SetProjectRuleRequest,
};
use crate::skill_files::{
    read_skill_file_at_path as read_skill_markdown_at_path, read_skill_file_from_state,
};
use crate::skill_library::{
    delete_skill as delete_skill_from_library, import_selected_skills,
    import_skill as import_skill_into_library, migrate_skill_library,
    preview_import_skills as preview_import_source, ImportSkillPreview, ImportSkillSource,
};
use crate::skill_references::{
    add_skill_reference_to_state, delete_preview_from_state, remove_skill_reference_from_state,
    AddSkillReferenceRequest, DeleteSkillPreview,
};
use crate::snapshot::{build_snapshot, AppSnapshot};
use crate::state_store::StateLoadStatus;
use crate::store_market::{self, LeaderboardType, StoreSkill};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::AppHandle;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmImportSkillsRequest {
    pub source: ImportSkillSource,
    pub candidate_ids: Vec<String>,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchStoreSkillsRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ImportProjectSkillResult {
    Success {
        snapshot: AppSnapshot,
    },
    Conflict {
        skill_id: String,
        library_name: String,
        project_name: String,
    },
}

fn snapshot_after_save(
    command_state: crate::command_context::CommandState,
) -> Result<AppSnapshot, String> {
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_snapshot(app: AppHandle) -> Result<AppSnapshot, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &command_state.load_status,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn fetch_store_leaderboard(board: String) -> Result<Vec<StoreSkill>, String> {
    store_market::fetch_leaderboard(LeaderboardType::from_str(&board))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn search_store_skills(request: SearchStoreSkillsRequest) -> Result<Vec<StoreSkill>, String> {
    let query = request.query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    store_market::search_skills(query, request.limit.unwrap_or(60))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_skill(app: AppHandle, source: PathBuf) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    import_skill_into_library(&mut command_state.state, &source)
        .map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn preview_import_skills(
    app: AppHandle,
    source: ImportSkillSource,
) -> Result<ImportSkillPreview, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    preview_import_source(&command_state.state, &source).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn confirm_import_skills(
    app: AppHandle,
    request: ConfirmImportSkillsRequest,
) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    import_selected_skills(
        &mut command_state.state,
        &request.source,
        &request.candidate_ids,
        request.overwrite,
    )
    .map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn preview_delete_skill(
    app: AppHandle,
    skill_id: String,
) -> Result<DeleteSkillPreview, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    delete_preview_from_state(&command_state.state, &skill_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_skill(app: AppHandle, skill_id: String) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    delete_skill_from_library(&mut command_state.state, &skill_id)
        .map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn add_skill_reference(
    app: AppHandle,
    request: AddSkillReferenceRequest,
) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    add_skill_reference_to_state(&mut command_state.state, request)
        .map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn remove_skill_reference(
    app: AppHandle,
    reference_id: String,
    remove_external_link: Option<bool>,
) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    remove_skill_reference_from_state(
        &mut command_state.state,
        &reference_id,
        remove_external_link,
    )
    .map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn add_project(app: AppHandle, request: AddProjectRequest) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    add_project_to_state(&mut command_state.state, request);
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn set_project_rule(
    app: AppHandle,
    request: SetProjectRuleRequest,
) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    set_project_rule_in_state(&mut command_state.state, request)
        .map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn set_current_project(
    app: AppHandle,
    project_id: Option<String>,
) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    set_current_project_in_state(&mut command_state.state, project_id)
        .map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn reset_project_rules(app: AppHandle, project_id: String) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    reset_project_rules_in_state(&mut command_state.state, &project_id)
        .map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn delete_project(app: AppHandle, project_id: String) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    delete_project_from_state(&mut command_state.state, &project_id)
        .map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn migrate_library(app: AppHandle, target: PathBuf) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    migrate_skill_library(&mut command_state.state, &target).map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn read_skill_file(app: AppHandle, skill_id: String) -> Result<String, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    read_skill_file_from_state(&command_state.state, &skill_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn read_skill_file_at_path(_app: AppHandle, skill_path: PathBuf) -> Result<String, String> {
    read_skill_markdown_at_path(skill_path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn scan_project_skills(
    app: AppHandle,
    project_path: PathBuf,
) -> Result<Vec<crate::project_scan::ScannedCategory>, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    crate::project_scan::scan_project_skills(&command_state.state, &project_path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_project_skill(
    app: AppHandle,
    project_name: String,
    skill_path: PathBuf,
    strategy: Option<String>,
) -> Result<ImportProjectSkillResult, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    let internal_result = crate::project_scan::import_project_skill(
        &mut command_state.state,
        &project_name,
        &skill_path,
        strategy.as_deref(),
    )
    .map_err(|error| error.to_string())?;

    match internal_result {
        crate::project_scan::InternalImportResult::Success => {
            let snapshot = snapshot_after_save(command_state)?;
            Ok(ImportProjectSkillResult::Success { snapshot })
        }
        crate::project_scan::InternalImportResult::Conflict {
            skill_id,
            library_name,
            project_name,
        } => Ok(ImportProjectSkillResult::Conflict {
            skill_id,
            library_name,
            project_name,
        }),
    }
}

#[tauri::command]
pub fn delete_unmanaged_skill(app: AppHandle, skill_path: PathBuf) -> Result<AppSnapshot, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    crate::project_scan::delete_unmanaged_skill_dir(&command_state.state, &skill_path)
        .map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn add_agent(app: AppHandle, name: String, path: PathBuf) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    add_agent_to_state(&mut command_state.state, name, path);
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn delete_agent(app: AppHandle, agent_id: String) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    delete_agent_from_state(&mut command_state.state, &agent_id);
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn set_agent_rule(app: AppHandle, request: SetAgentRuleRequest) -> Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    set_agent_rule_in_state(&mut command_state.state, request)
        .map_err(|error| error.to_string())?;
    snapshot_after_save(command_state)
}

#[tauri::command]
pub fn scan_agent_skills(
    app: AppHandle,
    agent_path: PathBuf,
) -> Result<Vec<crate::project_scan::ScannedCategory>, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    scan_agent_skills_from_state(&command_state.state, agent_path)
        .map_err(|error| error.to_string())
}

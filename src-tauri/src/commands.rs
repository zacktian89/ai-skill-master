use crate::app_paths::AppPaths;
use crate::managed_link::{
    create_directory_link, managed_link_issue_message, remove_managed_link,
    validate_managed_link, ManagedLinkValidation,
};
use crate::error::{Result, SkillMasterError};
use crate::models::{
    AppState, Project, ProjectRule, ReferenceScope,
    ReferenceStatus, Skill, SkillReference,
};
use crate::skill_library::{
    delete_skill as delete_skill_from_library, import_selected_skills,
    import_skill as import_skill_into_library, migrate_skill_library,
    preview_import_skills as preview_import_source, ImportSkillPreview, ImportSkillSource,
};
use crate::state_store::{
    load_or_create_state, save_state, state_backup_path, LoadedState,
    StateLoadStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub state: AppState,
    pub target_profiles: Vec<SkillTargetProfile>,
    pub diagnostics: Vec<DiagnosticItem>,
    pub paths: SnapshotPaths,
    pub state_load: StateLoadInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillTargetProfile {
    pub id: String,
    pub target_name: String,
    pub root_path: PathBuf,
    pub scope: ReferenceScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticItem {
    pub level: DiagnosticLevel,
    pub code: String,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotPaths {
    pub state_file: PathBuf,
    pub backup_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateLoadInfo {
    pub phase: StateLoadPhase,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StateLoadPhase {
    Clean,
    RestoredFromBackup,
    RebuildRequired,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSkillReferenceRequest {
    pub skill_id: String,
    pub target_name: String,
    pub root_path: PathBuf,
    pub scope: ReferenceScope,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmImportSkillsRequest {
    pub source: ImportSkillSource,
    pub candidate_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSkillPreview {
    pub skill_id: String,
    pub skill_name: String,
    pub library_path: PathBuf,
    pub managed_link_targets: Vec<PathBuf>,
    pub affected_projects: Vec<ProjectImpact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectImpact {
    pub project_id: String,
    pub project_name: String,
    pub project_path: PathBuf,
}

fn delete_project_from_state(state: &mut AppState, project_id: &str) -> Result<()> {
    let original_len = state.projects.len();
    state.projects.retain(|project| project.id != project_id);
    if state.projects.len() == original_len {
        return Err(SkillMasterError::ProjectNotFound(project_id.to_string()));
    }
    if state.current_project_id.as_deref() == Some(project_id) {
        state.current_project_id = None;
    }
    Ok(())
}

fn reset_project_rules_in_state(state: &mut AppState, project_id: &str) -> Result<()> {
    let project = state
        .projects
        .iter_mut()
        .find(|project| project.id == project_id)
        .ok_or_else(|| SkillMasterError::ProjectNotFound(project_id.to_string()))?;
    project.rules.clear();
    Ok(())
}

struct CommandState {
    paths: AppPaths,
    state: AppState,
    load_status: StateLoadStatus,
}

pub fn build_snapshot(
    paths: &AppPaths,
    mut state: AppState,
    load_status: &StateLoadStatus,
) -> Result<AppSnapshot> {
    let mut diagnostics = Vec::new();
    let mut seen = BTreeSet::new();

    for skill in &mut state.skills {
        skill.conflict = None;
        refresh_reference_statuses(skill)?;
    }

    if !state.skill_library_path.exists() {
        push_diagnostic(
            &mut diagnostics,
            &mut seen,
            DiagnosticItem {
                level: DiagnosticLevel::Error,
                code: "skill-library-missing".to_string(),
                title: "技能库目录缺失".to_string(),
                detail: format!(
                    "当前技能库目录不存在：{}",
                    state.skill_library_path.display()
                ),
            },
        );
    }

    match load_status {
        StateLoadStatus::Clean => {}
        StateLoadStatus::RestoredFromBackup { message } => push_diagnostic(
            &mut diagnostics,
            &mut seen,
            DiagnosticItem {
                level: DiagnosticLevel::Warning,
                code: "state-restored-from-backup".to_string(),
                title: "状态已从备份恢复".to_string(),
                detail: message.clone(),
            },
        ),
        StateLoadStatus::RebuildRequired { message } => push_diagnostic(
            &mut diagnostics,
            &mut seen,
            DiagnosticItem {
                level: DiagnosticLevel::Error,
                code: "state-rebuild-required".to_string(),
                title: "状态文件需要重建".to_string(),
                detail: message.clone(),
            },
        ),
    }

    if let Some(message) = state
        .migration_notice
        .as_ref()
        .map(|notice| notice.message.clone())
    {
        push_diagnostic(
            &mut diagnostics,
            &mut seen,
            DiagnosticItem {
                level: DiagnosticLevel::Info,
                code: "library-migrated".to_string(),
                title: "技能库已迁移".to_string(),
                detail: message,
            },
        );
    }

    if let Some(project_id) = state.current_project_id.as_deref() {
        if !state
            .projects
            .iter()
            .any(|project| project.id == project_id)
        {
            push_diagnostic(
                &mut diagnostics,
                &mut seen,
                DiagnosticItem {
                    level: DiagnosticLevel::Error,
                    code: "current-project-missing".to_string(),
                    title: "当前项目上下文无效".to_string(),
                    detail: format!("当前项目记录不存在：{project_id}"),
                },
            );
        }
    }


    Ok(AppSnapshot {
        state,
        target_profiles: built_in_target_profiles(),
        diagnostics,
        paths: SnapshotPaths {
            state_file: paths.state_file.clone(),
            backup_file: state_backup_path(&paths.state_file),
        },
        state_load: StateLoadInfo {
            phase: match load_status {
                StateLoadStatus::Clean => StateLoadPhase::Clean,
                StateLoadStatus::RestoredFromBackup { .. } => StateLoadPhase::RestoredFromBackup,
                StateLoadStatus::RebuildRequired { .. } => StateLoadPhase::RebuildRequired,
            },
            message: match load_status {
                StateLoadStatus::Clean => None,
                StateLoadStatus::RestoredFromBackup { message }
                | StateLoadStatus::RebuildRequired { message } => Some(message.clone()),
            },
        },
    })
}

fn built_in_target_profiles() -> Vec<SkillTargetProfile> {
    let home = user_home_path();
    vec![
        SkillTargetProfile {
            id: "codex-user".to_string(),
            target_name: "Codex".to_string(),
            root_path: home.join(".codex").join("skills"),
            scope: ReferenceScope::User,
        },
        SkillTargetProfile {
            id: "claude-user".to_string(),
            target_name: "Claude Code".to_string(),
            root_path: home.join(".claude").join("skills"),
            scope: ReferenceScope::User,
        },
        SkillTargetProfile {
            id: "gemini-user".to_string(),
            target_name: "Gemini CLI".to_string(),
            root_path: home.join(".gemini").join("config").join("skills"),
            scope: ReferenceScope::User,
        },
        SkillTargetProfile {
            id: "copilot-user".to_string(),
            target_name: "GitHub Copilot".to_string(),
            root_path: home.join(".copilot").join("skills"),
            scope: ReferenceScope::User,
        },
        SkillTargetProfile {
            id: "cursor-user".to_string(),
            target_name: "Cursor".to_string(),
            root_path: home.join(".cursor").join("skills"),
            scope: ReferenceScope::User,
        },
        SkillTargetProfile {
            id: "workbuddy-user".to_string(),
            target_name: "WorkBuddy".to_string(),
            root_path: home.join(".workbuddy").join("skills"),
            scope: ReferenceScope::User,
        },
        SkillTargetProfile {
            id: "windsurf-user".to_string(),
            target_name: "Windsurf".to_string(),
            root_path: home.join(".codeium").join("windsurf").join("skills"),
            scope: ReferenceScope::User,
        },
        SkillTargetProfile {
            id: "kiro-user".to_string(),
            target_name: "Kiro".to_string(),
            root_path: home.join(".kiro").join("skills"),
            scope: ReferenceScope::User,
        },
        SkillTargetProfile {
            id: "opencode-user".to_string(),
            target_name: "OpenCode".to_string(),
            root_path: home.join(".config").join("opencode").join("skill"),
            scope: ReferenceScope::User,
        },
        SkillTargetProfile {
            id: "codebuddy-user".to_string(),
            target_name: "CodeBuddy".to_string(),
            root_path: home.join(".codebuddy").join("skills"),
            scope: ReferenceScope::User,
        },
    ]
}

fn user_home_path() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn refresh_reference_statuses(skill: &mut Skill) -> Result<()> {
    let source = skill.library_path.clone();
    for reference in &mut skill.references {
        reference.status = match validate_managed_link(&source, &reference.target_path)? {
            ManagedLinkValidation::Valid => ReferenceStatus::Healthy,
            ManagedLinkValidation::Missing => ReferenceStatus::Missing,
            ManagedLinkValidation::MissingSource => ReferenceStatus::Stale,
            ManagedLinkValidation::WrongType | ManagedLinkValidation::WrongTarget { .. } => {
                ReferenceStatus::Conflict
            }
        };
    }
    Ok(())
}

fn command_paths(app: &AppHandle) -> Result<AppPaths> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| SkillMasterError::InvalidPath(error.to_string()))?;
    Ok(AppPaths::from_config_dir(&config_dir))
}

fn load_command_state(app: &AppHandle) -> Result<CommandState> {
    let paths = command_paths(app)?;
    let LoadedState { state, load_status } =
        load_or_create_state(&paths.state_file, &paths.skill_library)?;
    Ok(CommandState {
        paths,
        state,
        load_status,
    })
}

fn persist(paths: &AppPaths, state: &AppState) -> Result<()> {
    save_state(&paths.state_file, state)
}

fn project_id_from_path(path: &std::path::Path) -> String {
    let raw = path.to_string_lossy();
    format!("{:x}", md5_like_hash(raw.as_bytes()))
}

pub(crate) fn md5_like_hash(bytes: &[u8]) -> u64 {
    let mut hash = 1469598103934665603u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

fn push_diagnostic(
    diagnostics: &mut Vec<DiagnosticItem>,
    seen: &mut BTreeSet<String>,
    item: DiagnosticItem,
) {
    let key = format!("{}:{}:{}", item.code, item.title, item.detail);
    if seen.insert(key) {
        diagnostics.push(item);
    }
}

fn delete_preview_from_state(state: &AppState, skill_id: &str) -> Result<DeleteSkillPreview> {
    let skill = state
        .skills
        .iter()
        .find(|skill| skill.id == skill_id)
        .ok_or_else(|| SkillMasterError::SkillNotFound(skill_id.to_string()))?;
    let affected_projects = state
        .projects
        .iter()
        .filter(|project| project.rules.contains_key(skill_id))
        .map(|project| ProjectImpact {
            project_id: project.id.clone(),
            project_name: project.name.clone(),
            project_path: project.path.clone(),
        })
        .collect::<Vec<_>>();

    Ok(DeleteSkillPreview {
        skill_id: skill.id.clone(),
        skill_name: skill.name.clone(),
        library_path: skill.library_path.clone(),
        managed_link_targets: skill
            .references
            .iter()
            .map(|reference| reference.target_path.clone())
            .collect::<Vec<_>>(),
        affected_projects,
    })
}

fn reference_id(target_path: &std::path::Path) -> String {
    let raw = target_path.to_string_lossy();
    format!("ref-{:x}", md5_like_hash(raw.as_bytes()))
}

fn add_skill_reference_to_state(
    state: &mut AppState,
    request: AddSkillReferenceRequest,
) -> Result<()> {
    let skill = state
        .skills
        .iter_mut()
        .find(|skill| skill.id == request.skill_id)
        .ok_or_else(|| SkillMasterError::SkillNotFound(request.skill_id.clone()))?;
    let target_path = request.root_path.join(&skill.id);

    if skill
        .references
        .iter()
        .any(|reference| reference.target_path == target_path)
    {
        return Ok(());
    }

    match validate_managed_link(&skill.library_path, &target_path)? {
        ManagedLinkValidation::Valid => {}
        ManagedLinkValidation::Missing => create_directory_link(&skill.library_path, &target_path)?,
        ManagedLinkValidation::WrongTarget { .. } if request.overwrite => {
            remove_managed_link(&target_path)?;
            create_directory_link(&skill.library_path, &target_path)?;
        }
        ManagedLinkValidation::MissingSource => {
            return Err(SkillMasterError::MissingDirectory(
                skill.library_path.clone(),
            ));
        }
        validation => {
            return Err(SkillMasterError::InvalidPath(managed_link_issue_message(
                &target_path,
                &validation,
            )));
        }
    }

    skill.references.push(SkillReference {
        id: reference_id(&target_path),
        target_name: request.target_name,
        target_path,
        scope: request.scope,
        status: ReferenceStatus::Healthy,
    });
    Ok(())
}

fn remove_skill_reference_from_state(
    state: &mut AppState,
    reference_id: &str,
    remove_external_link: Option<bool>,
) -> Result<()> {
    for skill in &mut state.skills {
        let Some(index) = skill
            .references
            .iter()
            .position(|reference| reference.id == reference_id)
        else {
            continue;
        };
        let reference = skill.references[index].clone();
        for agent in &mut state.agents {
            let target_path = agent.path.join(&skill.id);
            if target_path == reference.target_path {
                agent.rules.remove(&skill.id);
            }
        }
        match validate_managed_link(&skill.library_path, &reference.target_path)? {
            ManagedLinkValidation::Valid => remove_managed_link(&reference.target_path)?,
            ManagedLinkValidation::Missing => {}
            validation => {
                match remove_external_link {
                    Some(true) => {
                        remove_managed_link(&reference.target_path)?;
                    }
                    Some(false) => {}
                    None => {
                        return Err(SkillMasterError::InvalidPath(managed_link_issue_message(
                            &reference.target_path,
                            &validation,
                        )));
                    }
                }
            }
        }
        skill.references.remove(index);
        return Ok(());
    }
    Err(SkillMasterError::InvalidPath(format!(
        "找不到引用：{reference_id}"
    )))
}

#[tauri::command]
pub fn get_snapshot(app: AppHandle) -> std::result::Result<AppSnapshot, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &command_state.load_status,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_skill(app: AppHandle, source: PathBuf) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    import_skill_into_library(&mut command_state.state, &source)
        .map_err(|error| error.to_string())?;
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn preview_import_skills(
    app: AppHandle,
    source: ImportSkillSource,
) -> std::result::Result<ImportSkillPreview, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    preview_import_source(&command_state.state, &source).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn confirm_import_skills(
    app: AppHandle,
    request: ConfirmImportSkillsRequest,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    import_selected_skills(
        &mut command_state.state,
        &request.source,
        &request.candidate_ids,
    )
    .map_err(|error| error.to_string())?;
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn preview_delete_skill(
    app: AppHandle,
    skill_id: String,
) -> std::result::Result<DeleteSkillPreview, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    delete_preview_from_state(&command_state.state, &skill_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_skill(app: AppHandle, skill_id: String) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    delete_skill_from_library(&mut command_state.state, &skill_id)
        .map_err(|error| error.to_string())?;
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}


#[tauri::command]
pub fn add_skill_reference(
    app: AppHandle,
    request: AddSkillReferenceRequest,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    add_skill_reference_to_state(&mut command_state.state, request)
        .map_err(|error| error.to_string())?;
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn remove_skill_reference(
    app: AppHandle,
    reference_id: String,
    remove_external_link: Option<bool>,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    remove_skill_reference_from_state(
        &mut command_state.state,
        &reference_id,
        remove_external_link,
    )
    .map_err(|error| error.to_string())?;
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn add_project(
    app: AppHandle,
    request: AddProjectRequest,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    let id = project_id_from_path(&request.path);
    if !command_state
        .state
        .projects
        .iter()
        .any(|project| project.id == id)
    {
        command_state.state.projects.push(Project {
            id,
            name: request.name,
            path: request.path,
            rules: BTreeMap::new(),
        });
    }
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_project_rule(
    app: AppHandle,
    request: SetProjectRuleRequest,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    let project = command_state
        .state
        .projects
        .iter_mut()
        .find(|project| project.id == request.project_id)
        .ok_or_else(|| format!("找不到项目：{}", request.project_id))?;
    if request.rule == ProjectRule::Inherit {
        project.rules.remove(&request.skill_id);
    } else {
        project.rules.insert(request.skill_id, request.rule);
    }
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_current_project(
    app: AppHandle,
    project_id: Option<String>,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    if let Some(project_id) = project_id.as_deref() {
        if !command_state
            .state
            .projects
            .iter()
            .any(|project| project.id == project_id)
        {
            return Err(format!("找不到项目：{project_id}"));
        }
    }
    command_state.state.current_project_id = project_id;
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn reset_project_rules(
    app: AppHandle,
    project_id: String,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    reset_project_rules_in_state(&mut command_state.state, &project_id)
        .map_err(|error| error.to_string())?;
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_project(
    app: AppHandle,
    project_id: String,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    delete_project_from_state(&mut command_state.state, &project_id)
        .map_err(|error| error.to_string())?;
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn migrate_library(
    app: AppHandle,
    target: PathBuf,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    migrate_skill_library(&mut command_state.state, &target).map_err(|error| error.to_string())?;
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn read_skill_file(
    app: AppHandle,
    skill_id: String,
) -> std::result::Result<String, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    let skill = command_state
        .state
        .skills
        .iter()
        .find(|skill| skill.id == skill_id)
        .ok_or_else(|| format!("找不到 skill：{}", skill_id))?;

    let target_path = skill.library_path.join("SKILL.md");
    if !target_path.exists() {
        return Err(format!("找不到 SKILL.md 文件：{}", target_path.display()));
    }

    let canonical_library = skill.library_path.canonicalize()
        .map_err(|e| format!("无法解析技能目录：{}", e))?;
    let canonical_target = target_path.canonicalize()
        .map_err(|e| format!("无法解析 target 文件：{}", e))?;

    if !canonical_target.starts_with(&canonical_library) {
        return Err("安全错误：非法路径访问".to_string());
    }

    let content = fs::read_to_string(&canonical_target)
        .map_err(|error| format!("读取文件失败：{}", error))?;
    Ok(content)
}

#[tauri::command]
pub fn read_skill_file_at_path(
    _app: AppHandle,
    skill_path: PathBuf,
) -> std::result::Result<String, String> {
    if !skill_path.is_dir() {
        return Err(format!("找不到 skill 目录：{}", skill_path.display()));
    }

    let target_path = skill_path.join("SKILL.md");
    if !target_path.exists() {
        return Err(format!("找不到 SKILL.md 文件：{}", target_path.display()));
    }

    let canonical_skill_dir = skill_path
        .canonicalize()
        .map_err(|e| format!("无法解析技能目录：{}", e))?;
    let canonical_target = target_path
        .canonicalize()
        .map_err(|e| format!("无法解析 target 文件：{}", e))?;

    if !canonical_target.starts_with(&canonical_skill_dir) {
        return Err("安全错误：非法路径访问".to_string());
    }

    fs::read_to_string(&canonical_target).map_err(|error| format!("读取文件失败：{}", error))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[tauri::command]
pub fn scan_project_skills(
    app: AppHandle,
    project_path: PathBuf,
) -> std::result::Result<Vec<crate::project_scan::ScannedCategory>, String> {
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
) -> std::result::Result<ImportProjectSkillResult, String> {
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
            persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
            let snapshot = build_snapshot(&command_state.paths, command_state.state, &StateLoadStatus::Clean)
                .map_err(|error| error.to_string())?;
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
pub fn delete_unmanaged_skill(
    app: AppHandle,
    skill_path: PathBuf,
) -> std::result::Result<AppSnapshot, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    crate::project_scan::delete_unmanaged_skill_dir(&command_state.state, &skill_path)
        .map_err(|error| error.to_string())?;
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

fn resolve_path_with_home(path: PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with('~') {
        let home = user_home_path();
        if path_str == "~" {
            home
        } else {
            let remainder = &path_str[1..];
            let remainder_clean = remainder.trim_start_matches('/').trim_start_matches('\\');
            home.join(remainder_clean)
        }
    } else {
        path
    }
}

#[tauri::command]
pub fn add_agent(
    app: AppHandle,
    name: String,
    path: PathBuf,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    let resolved_path = resolve_path_with_home(path);
    let id = project_id_from_path(&resolved_path);
    if !command_state
        .state
        .agents
        .iter()
        .any(|agent| agent.id == id)
    {
        command_state.state.agents.push(crate::models::Agent {
            id,
            name,
            path: resolved_path,
            rules: BTreeMap::new(),
        });
    }
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_agent(
    app: AppHandle,
    agent_id: String,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    command_state.state.agents.retain(|agent| agent.id != agent_id);
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAgentRuleRequest {
    pub agent_id: String,
    pub skill_id: String,
    pub rule: ProjectRule,
}

fn set_agent_rule_in_state(
    state: &mut AppState,
    request: SetAgentRuleRequest,
) -> Result<()> {
    let agent = state.agents.iter_mut()
        .find(|a| a.id == request.agent_id)
        .ok_or_else(|| {
            SkillMasterError::InvalidPath(format!("找不到 Agent：{}", request.agent_id))
        })?;
        
    let agent_path = agent.path.clone();
    let agent_name = agent.name.clone();
    let target_path = agent_path.join(&request.skill_id);

    if request.rule == ProjectRule::Inherit {
        agent.rules.remove(&request.skill_id);
    } else {
        agent.rules.insert(request.skill_id.clone(), request.rule);
    }

    let skill = state.skills.iter_mut()
        .find(|s| s.id == request.skill_id)
        .ok_or_else(|| SkillMasterError::SkillNotFound(request.skill_id.clone()))?;

    if request.rule == ProjectRule::Inherit {
        match validate_managed_link(&skill.library_path, &target_path)? {
            ManagedLinkValidation::Valid => remove_managed_link(&target_path)?,
            ManagedLinkValidation::Missing => {}
            validation => {
                return Err(SkillMasterError::InvalidPath(managed_link_issue_message(
                    &target_path,
                    &validation,
                )));
            }
        }
        skill.references.retain(|reference| reference.target_path != target_path);
        return Ok(());
    }

    match request.rule {
        ProjectRule::Disable => {
            if target_path.exists() {
                remove_managed_link(&target_path)?;
            }
        }
        ProjectRule::Enable => {
            match validate_managed_link(&skill.library_path, &target_path)? {
                ManagedLinkValidation::Valid => {}
                ManagedLinkValidation::Missing => {
                    create_directory_link(&skill.library_path, &target_path)?;
                }
                ManagedLinkValidation::WrongTarget { .. } => {
                    remove_managed_link(&target_path)?;
                    create_directory_link(&skill.library_path, &target_path)?;
                }
                validation => {
                    return Err(SkillMasterError::InvalidPath(managed_link_issue_message(
                        &target_path,
                        &validation,
                    )));
                }
            }
            let ref_id = reference_id(&target_path);
            if !skill.references.iter().any(|r| r.id == ref_id) {
                skill.references.push(SkillReference {
                    id: ref_id,
                    target_name: agent_name,
                    target_path,
                    scope: ReferenceScope::User,
                    status: ReferenceStatus::Healthy,
                });
            }
        }
        ProjectRule::Inherit => {}
    }

    Ok(())
}

#[tauri::command]
pub fn set_agent_rule(
    app: AppHandle,
    request: SetAgentRuleRequest,
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    set_agent_rule_in_state(&mut command_state.state, request)
        .map_err(|error| error.to_string())?;
    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn scan_agent_skills(
    app: AppHandle,
    agent_path: PathBuf,
) -> std::result::Result<Vec<crate::project_scan::ScannedCategory>, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    let resolved_path = resolve_path_with_home(agent_path);
    if !resolved_path.is_dir() {
        return Ok(Vec::new());
    }
    let scan_dir = if resolved_path.join("skills").is_dir() {
        resolved_path.join("skills")
    } else {
        resolved_path.clone()
    };
    
    let mut scanned_skills = Vec::new();
    let entries = fs::read_dir(&scan_dir).map_err(|error| error.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let skill_path = entry.path();
        if !skill_path.is_dir() {
            continue;
        }
        if skill_path.join("SKILL.md").exists() {
            if let Ok(metadata) = crate::skill_library::read_skill_metadata(&skill_path) {
                let mut is_managed = false;
                if let Some(matching_skill) = command_state.state.skills.iter().find(|s| s.id == metadata.id) {
                    if let Ok(validation) = validate_managed_link(&matching_skill.library_path, &skill_path) {
                        if validation == ManagedLinkValidation::Valid {
                            is_managed = true;
                        }
                    }
                }
                scanned_skills.push(crate::project_scan::ScannedSkill {
                    id: metadata.id,
                    name: metadata.name,
                    description: metadata.description,
                    path: skill_path,
                    is_managed,
                });
            }
        }
    }
    
    let agent_rules_and_refs: Vec<String> = command_state.state.skills.iter()
        .filter(|s| {
            let has_rule = command_state.state.agents.iter()
                .any(|a| a.path == resolved_path && a.rules.contains_key(&s.id));
            let has_ref = s.references.iter()
                .any(|r| r.target_path.parent() == Some(&scan_dir) || r.target_path.parent() == Some(&resolved_path));
            has_rule || has_ref
        })
        .map(|s| s.id.clone())
        .collect();

    for skill_id in agent_rules_and_refs {
        if !scanned_skills.iter().any(|s| s.id == skill_id) {
            if let Some(skill) = command_state.state.skills.iter().find(|s| s.id == skill_id) {
                let skill_path = scan_dir.join(&skill.id);
                scanned_skills.push(crate::project_scan::ScannedSkill {
                    id: skill.id.clone(),
                    name: skill.name.clone(),
                    description: skill.description.clone(),
                    path: skill_path,
                    is_managed: true,
                });
            }
        }
    }

    if scanned_skills.is_empty() {
        Ok(Vec::new())
    } else {
        scanned_skills.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(vec![crate::project_scan::ScannedCategory {
            name: ".".to_string(),
            path: scan_dir,
            skills: scanned_skills,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_store::default_state;
    use tempfile::tempdir;

    #[test]
    fn snapshot_includes_skills() {
        let dir = tempdir().unwrap();
        let paths = AppPaths::from_config_dir(dir.path());
        let mut state = default_state(dir.path().join("skills"));
        state.skills.push(Skill {
            id: "writer".to_string(),
            name: "writer".to_string(),
            description: String::new(),
            library_path: dir.path().join("skills").join("writer"),
            source: Default::default(),
            references: Vec::new(),
            managed_links: Default::default(),
            conflict: None,
        });

        let snapshot = build_snapshot(&paths, state, &StateLoadStatus::Clean).unwrap();

        assert_eq!(snapshot.state.skills.len(), 1);
    }

    #[test]
    fn delete_preview_includes_managed_links_and_project_impacts() {
        let dir = tempdir().unwrap();
        let mut state = default_state(dir.path().join("skills"));
        state.skills.push(Skill {
            id: "writer".to_string(),
            name: "Writer".to_string(),
            description: String::new(),
            library_path: dir.path().join("skills").join("writer"),
            source: Default::default(),
            references: vec![SkillReference {
                id: "ref-1".to_string(),
                target_name: "Claude".to_string(),
                target_path: dir.path().join("codex").join("writer"),
                scope: ReferenceScope::User,
                status: ReferenceStatus::Healthy,
            }],
            managed_links: crate::models::ManagedLinks {},
            conflict: None,
        });
        let mut rules = BTreeMap::new();
        rules.insert("writer".to_string(), ProjectRule::Disable);
        state.projects.push(Project {
            id: "p1".to_string(),
            name: "Demo".to_string(),
            path: dir.path().join("demo"),
            rules,
        });

        let preview = delete_preview_from_state(&state, "writer").unwrap();

        assert_eq!(preview.skill_name, "Writer");
        assert_eq!(preview.managed_link_targets.len(), 1);
        assert_eq!(preview.affected_projects.len(), 1);
    }

    #[test]
    fn built_in_profiles_include_new_agent_skill_paths() {
        let profiles = built_in_target_profiles();

        assert!(profiles.iter().any(|profile| {
            profile.target_name == "Gemini CLI"
                && profile
                    .root_path
                    .ends_with(std::path::Path::new(".gemini").join("config").join("skills"))
        }));
        assert!(profiles.iter().any(|profile| {
            profile.target_name == "WorkBuddy"
                && profile.root_path.ends_with(std::path::Path::new(".workbuddy").join("skills"))
        }));
    }

    #[test]
    fn deleting_current_project_clears_context() {
        let dir = tempdir().unwrap();
        let mut state = default_state(dir.path().join("skills"));
        state.current_project_id = Some("demo".to_string());
        state.projects.push(Project {
            id: "demo".to_string(),
            name: "Demo".to_string(),
            path: dir.path().join("demo"),
            rules: BTreeMap::new(),
        });

        delete_project_from_state(&mut state, "demo").unwrap();

        assert!(state.projects.is_empty());
        assert_eq!(state.current_project_id, None);
    }

    #[test]
    fn add_reference_keeps_retargeted_link_without_overwrite() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("claude").join("skills");
        let other = dir.path().join("other-html-go");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let target = root.join("html-go");
        create_directory_link(&other, &target).unwrap();

        let mut state = default_state(library.clone());
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: Vec::new(),
            managed_links: Default::default(),
            conflict: None,
        });

        let result = add_skill_reference_to_state(
            &mut state,
            AddSkillReferenceRequest {
                skill_id: "html-go".to_string(),
                target_name: "Claude".to_string(),
                root_path: root,
                scope: ReferenceScope::User,
                overwrite: false,
            },
        );

        assert!(result.unwrap_err().to_string().contains("已指向其他位置"));
        assert_eq!(std::fs::read_link(&target).unwrap(), other);
        assert!(state.skills[0].references.is_empty());
    }

    #[test]
    fn add_reference_replaces_retargeted_link_with_overwrite() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("claude").join("skills");
        let other = dir.path().join("other-html-go");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let target = root.join("html-go");
        create_directory_link(&other, &target).unwrap();

        let mut state = default_state(library.clone());
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: Vec::new(),
            managed_links: Default::default(),
            conflict: None,
        });

        add_skill_reference_to_state(
            &mut state,
            AddSkillReferenceRequest {
                skill_id: "html-go".to_string(),
                target_name: "Claude".to_string(),
                root_path: root,
                scope: ReferenceScope::User,
                overwrite: true,
            },
        )
        .unwrap();

        assert_eq!(
            std::fs::canonicalize(std::fs::read_link(&target).unwrap()).unwrap(),
            std::fs::canonicalize(library.join("html-go")).unwrap()
        );
        assert_eq!(state.skills[0].references.len(), 1);
        assert_eq!(state.skills[0].references[0].target_path, target);
    }

    #[test]
    fn resetting_project_rules_clears_only_selected_project() {
        let dir = tempdir().unwrap();
        let mut state = default_state(dir.path().join("skills"));

        let mut demo_rules = BTreeMap::new();
        demo_rules.insert("writer".to_string(), ProjectRule::Enable);
        let mut keep_rules = BTreeMap::new();
        keep_rules.insert("reviewer".to_string(), ProjectRule::Disable);

        state.projects.push(Project {
            id: "demo".to_string(),
            name: "Demo".to_string(),
            path: dir.path().join("demo"),
            rules: demo_rules,
        });
        state.projects.push(Project {
            id: "keep".to_string(),
            name: "Keep".to_string(),
            path: dir.path().join("keep"),
            rules: keep_rules,
        });

        reset_project_rules_in_state(&mut state, "demo").unwrap();

        assert!(state.projects[0].rules.is_empty());
        assert_eq!(state.projects[1].rules.len(), 1);
    }

    #[test]
    fn remove_reference_blocks_on_mismatch_without_force() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("claude").join("skills");
        let other = dir.path().join("other-html-go");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let target = root.join("html-go");
        create_directory_link(&other, &target).unwrap();

        let mut state = default_state(library.clone());
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: vec![SkillReference {
                id: "ref-id".to_string(),
                target_name: "Claude".to_string(),
                target_path: target.clone(),
                scope: ReferenceScope::User,
                status: ReferenceStatus::Conflict,
            }],
            managed_links: Default::default(),
            conflict: None,
        });

        let result = remove_skill_reference_from_state(&mut state, "ref-id", None);

        assert!(result.unwrap_err().to_string().contains("已指向其他位置"));
        assert_eq!(std::fs::read_link(&target).unwrap(), other);
        assert_eq!(state.skills[0].references.len(), 1);
    }

    #[test]
    fn remove_reference_handles_mismatch_with_force_true() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("claude").join("skills");
        let other = dir.path().join("other-html-go");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let target = root.join("html-go");
        create_directory_link(&other, &target).unwrap();

        let mut state = default_state(library.clone());
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: vec![SkillReference {
                id: "ref-id".to_string(),
                target_name: "Claude".to_string(),
                target_path: target.clone(),
                scope: ReferenceScope::User,
                status: ReferenceStatus::Conflict,
            }],
            managed_links: Default::default(),
            conflict: None,
        });

        remove_skill_reference_from_state(&mut state, "ref-id", Some(true)).unwrap();

        assert!(!target.exists());
        assert!(state.skills[0].references.is_empty());
    }

    #[test]
    fn remove_reference_handles_mismatch_with_force_false() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("claude").join("skills");
        let other = dir.path().join("other-html-go");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let target = root.join("html-go");
        create_directory_link(&other, &target).unwrap();

        let mut state = default_state(library.clone());
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: vec![SkillReference {
                id: "ref-id".to_string(),
                target_name: "Claude".to_string(),
                target_path: target.clone(),
                scope: ReferenceScope::User,
                status: ReferenceStatus::Conflict,
            }],
            managed_links: Default::default(),
            conflict: None,
        });

        remove_skill_reference_from_state(&mut state, "ref-id", Some(false)).unwrap();

        assert_eq!(std::fs::read_link(&target).unwrap(), other);
        assert!(state.skills[0].references.is_empty());
    }

    #[test]
    fn setting_agent_rule_to_inherit_removes_managed_link_and_reference() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("skills");
        let root = dir.path().join("windsurf").join("skills");
        std::fs::create_dir_all(library.join("html-go")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("html-go");
        create_directory_link(&library.join("html-go"), &target).unwrap();

        let mut state = default_state(library.clone());
        let mut rules = BTreeMap::new();
        rules.insert("html-go".to_string(), ProjectRule::Enable);
        state.agents.push(crate::models::Agent {
            id: "windsurf".to_string(),
            name: "Windsurf".to_string(),
            path: root.clone(),
            rules,
        });
        state.skills.push(Skill {
            id: "html-go".to_string(),
            name: "html-go".to_string(),
            description: String::new(),
            library_path: library.join("html-go"),
            source: Default::default(),
            references: vec![SkillReference {
                id: reference_id(&target),
                target_name: "Windsurf".to_string(),
                target_path: target.clone(),
                scope: ReferenceScope::User,
                status: ReferenceStatus::Healthy,
            }],
            managed_links: Default::default(),
            conflict: None,
        });

        set_agent_rule_in_state(
            &mut state,
            SetAgentRuleRequest {
                agent_id: "windsurf".to_string(),
                skill_id: "html-go".to_string(),
                rule: ProjectRule::Inherit,
            },
        )
        .unwrap();

        assert!(!target.exists());
        assert!(!state.agents[0].rules.contains_key("html-go"));
        assert!(state.skills[0].references.is_empty());
    }
}

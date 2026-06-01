use crate::app_paths::{detect_codex_skills_path, AppPaths};
use crate::codex_sync::{
    create_directory_link, managed_link_issue_message, plan_codex_sync, remove_managed_link,
    validate_managed_link, ManagedLinkValidation, SyncReport,
};
use crate::effective_state::effective_skill_ids;
use crate::error::{Result, SkillMasterError};
use crate::models::{
    AppState, PendingSyncAction, PendingSyncActionKind, Project, ProjectRule, ReferenceScope,
    ReferenceStatus, Skill, SkillConflict, SkillReference, SyncPhase, SyncStatus,
};
use crate::skill_library::{
    delete_skill as delete_skill_from_library, import_selected_skills,
    import_skill as import_skill_into_library, migrate_skill_library,
    preview_import_skills as preview_import_source, ImportSkillPreview, ImportSkillSource,
};
use crate::state_store::{
    load_or_create_state, rebuild_state_from_library, save_state, state_backup_path, LoadedState,
    StateLoadStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
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

    if let Some(codex_path) = state
        .codex_skills_path
        .as_ref()
        .filter(|path| path.exists())
    {
        match effective_skill_ids(&state, state.current_project_id.as_deref()) {
            Ok(active) => {
                let report = plan_codex_sync(&state.skills, &active, codex_path)?;
                for conflict in report.conflicts {
                    mark_skill_conflict(
                        &mut state.skills,
                        &conflict.skill_id,
                        &conflict.target,
                        &conflict.message,
                    );
                    push_diagnostic(
                        &mut diagnostics,
                        &mut seen,
                        DiagnosticItem {
                            level: if conflict.message.contains("托管链接")
                                || conflict.message.contains("源目录不存在")
                            {
                                DiagnosticLevel::Error
                            } else {
                                DiagnosticLevel::Warning
                            },
                            code: "codex-conflict".to_string(),
                            title: format!("Skill 冲突：{}", conflict.skill_id),
                            detail: conflict.message,
                        },
                    );
                }
            }
            Err(error) => push_diagnostic(
                &mut diagnostics,
                &mut seen,
                DiagnosticItem {
                    level: DiagnosticLevel::Error,
                    code: "effective-state-error".to_string(),
                    title: "无法计算当前项目生效状态".to_string(),
                    detail: error.to_string(),
                },
            ),
        }
    }

    for skill in &mut state.skills {
        if let Some(target) = skill.managed_links.codex.clone() {
            let validation = validate_managed_link(&skill.library_path, &target)?;
            if validation != ManagedLinkValidation::Valid {
                if skill.conflict.is_none() {
                    skill.conflict = Some(SkillConflict {
                        target: "codex".to_string(),
                        path: target.clone(),
                        message: managed_link_issue_message(&target, &validation),
                    });
                }
                push_diagnostic(
                    &mut diagnostics,
                    &mut seen,
                    DiagnosticItem {
                        level: DiagnosticLevel::Error,
                        code: "managed-link-mismatch".to_string(),
                        title: format!("托管链接异常：{}", skill.name),
                        detail: managed_link_issue_message(&target, &validation),
                    },
                );
            }
        }
    }

    if state.sync_status.phase == SyncPhase::RepairRequired {
        push_diagnostic(
            &mut diagnostics,
            &mut seen,
            DiagnosticItem {
                level: DiagnosticLevel::Error,
                code: "sync-repair-required".to_string(),
                title: "Codex 同步需要修复".to_string(),
                detail: state.sync_status.message.clone().unwrap_or_else(|| {
                    "上一轮同步未完成，请根据挂起操作重新同步或手动处理。".to_string()
                }),
            },
        );
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
            root_path: home.join(".agents").join("skills"),
            scope: ReferenceScope::User,
        },
        SkillTargetProfile {
            id: "claude-user".to_string(),
            target_name: "Claude Code".to_string(),
            root_path: home.join(".claude").join("skills"),
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
    let LoadedState { state, load_status } =
        load_or_create_state(&paths.state_file, &paths.skill_library, codex)?;
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

fn md5_like_hash(bytes: &[u8]) -> u64 {
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

fn mark_skill_conflict(
    skills: &mut [Skill],
    skill_id: &str,
    path: &std::path::Path,
    message: &str,
) {
    if let Some(skill) = skills.iter_mut().find(|skill| skill.id == skill_id) {
        skill.conflict = Some(SkillConflict {
            target: "codex".to_string(),
            path: path.to_path_buf(),
            message: message.to_string(),
        });
    }
}

fn build_pending_actions(report: &SyncReport) -> Vec<PendingSyncAction> {
    let mut actions = Vec::new();
    for action in &report.to_create {
        actions.push(PendingSyncAction {
            kind: PendingSyncActionKind::Create,
            skill_id: action.skill_id.clone(),
            target: action.target.clone(),
            source: Some(action.source.clone()),
            message: "需要重新创建托管链接".to_string(),
        });
    }
    for action in &report.to_remove {
        actions.push(PendingSyncAction {
            kind: PendingSyncActionKind::Remove,
            skill_id: action.skill_id.clone(),
            target: action.target.clone(),
            source: Some(action.source.clone()),
            message: "需要移除旧的托管链接".to_string(),
        });
    }
    for conflict in &report.conflicts {
        actions.push(PendingSyncAction {
            kind: PendingSyncActionKind::Inspect,
            skill_id: conflict.skill_id.clone(),
            target: conflict.target.clone(),
            source: None,
            message: conflict.message.clone(),
        });
    }
    actions
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
            .managed_links
            .codex
            .iter()
            .cloned()
            .chain(
                skill
                    .references
                    .iter()
                    .map(|reference| reference.target_path.clone()),
            )
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

fn remove_skill_reference_from_state(state: &mut AppState, reference_id: &str) -> Result<()> {
    for skill in &mut state.skills {
        let Some(index) = skill
            .references
            .iter()
            .position(|reference| reference.id == reference_id)
        else {
            continue;
        };
        let reference = skill.references[index].clone();
        match validate_managed_link(&skill.library_path, &reference.target_path)? {
            ManagedLinkValidation::Valid => remove_managed_link(&reference.target_path)?,
            ManagedLinkValidation::Missing => {}
            validation => {
                return Err(SkillMasterError::InvalidPath(managed_link_issue_message(
                    &reference.target_path,
                    &validation,
                )));
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
    let skill = command_state
        .state
        .skills
        .iter()
        .find(|skill| skill.id == skill_id)
        .cloned()
        .ok_or_else(|| format!("找不到 skill：{skill_id}"))?;

    let mut pending = Vec::new();
    let mut issues = Vec::new();

    if let Some(target) = &skill.managed_links.codex {
        let validation = validate_managed_link(&skill.library_path, target)
            .map_err(|error| error.to_string())?;
        if validation != ManagedLinkValidation::Valid {
            issues.push(managed_link_issue_message(target, &validation));
            pending.push(PendingSyncAction {
                kind: PendingSyncActionKind::Inspect,
                skill_id: skill.id.clone(),
                target: target.clone(),
                source: Some(skill.library_path.clone()),
                message: managed_link_issue_message(target, &validation),
            });
        }
    }

    if !issues.is_empty() {
        command_state.state.sync_status = SyncStatus {
            phase: SyncPhase::RepairRequired,
            message: Some(format!(
                "删除 skill 前发现托管链接异常，未执行删除：{}",
                issues.join("；")
            )),
            pending_actions: pending,
        };
        persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
        return build_snapshot(
            &command_state.paths,
            command_state.state,
            &StateLoadStatus::Clean,
        )
        .map_err(|error| error.to_string());
    }

    if let Some(target) = &skill.managed_links.codex {
        if let Err(error) = remove_managed_link(target) {
            command_state.state.sync_status = SyncStatus {
                phase: SyncPhase::RepairRequired,
                message: Some(format!(
                    "删除 skill 前移除托管链接失败：{} -> {}",
                    target.display(),
                    error
                )),
                pending_actions: vec![PendingSyncAction {
                    kind: PendingSyncActionKind::Remove,
                    skill_id: skill.id.clone(),
                    target: target.clone(),
                    source: Some(skill.library_path.clone()),
                    message: "需要先移除托管链接后才能删除 skill".to_string(),
                }],
            };
            persist(&command_state.paths, &command_state.state)
                .map_err(|persist_error| persist_error.to_string())?;
            return build_snapshot(
                &command_state.paths,
                command_state.state,
                &StateLoadStatus::Clean,
            )
            .map_err(|persist_error| persist_error.to_string());
        }
    }

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
) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    remove_skill_reference_from_state(&mut command_state.state, &reference_id)
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
pub fn set_codex_path(app: AppHandle, path: PathBuf) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    command_state.state.codex_skills_path = Some(path);
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
pub fn rebuild_state(app: AppHandle) -> std::result::Result<AppSnapshot, String> {
    let command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    let mut rebuilt = rebuild_state_from_library(
        &command_state.state.skill_library_path,
        command_state.state.codex_skills_path.clone(),
    )
    .map_err(|error| error.to_string())?;
    rebuilt.sync_status = SyncStatus {
        phase: SyncPhase::Idle,
        message: Some("状态文件已重建，可重新同步 Codex。".to_string()),
        pending_actions: Vec::new(),
    };
    persist(&command_state.paths, &rebuilt).map_err(|error| error.to_string())?;
    build_snapshot(&command_state.paths, rebuilt, &StateLoadStatus::Clean)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn sync_codex(app: AppHandle) -> std::result::Result<AppSnapshot, String> {
    let mut command_state = load_command_state(&app).map_err(|error| error.to_string())?;
    let codex_path = command_state
        .state
        .codex_skills_path
        .clone()
        .ok_or_else(|| "Codex skills 目录未设置".to_string())?;
    let active = effective_skill_ids(
        &command_state.state,
        command_state.state.current_project_id.as_deref(),
    )
    .map_err(|error| error.to_string())?;
    let report = plan_codex_sync(&command_state.state.skills, &active, &codex_path)
        .map_err(|error| error.to_string())?;

    let mut issues = Vec::new();

    for action in &report.to_remove {
        match validate_managed_link(&action.source, &action.target)
            .map_err(|error| error.to_string())?
        {
            ManagedLinkValidation::Valid => {
                if let Err(error) = remove_managed_link(&action.target) {
                    issues.push(format!(
                        "移除托管链接失败：{} -> {}",
                        action.target.display(),
                        error
                    ));
                } else if let Some(skill) = command_state
                    .state
                    .skills
                    .iter_mut()
                    .find(|skill| skill.id == action.skill_id)
                {
                    skill.managed_links.codex = None;
                }
            }
            validation => issues.push(managed_link_issue_message(&action.target, &validation)),
        }
    }

    for action in &report.to_create {
        if let Err(error) = create_directory_link(&action.source, &action.target) {
            issues.push(format!(
                "创建托管链接失败：{} -> {}，原因：{}",
                action.source.display(),
                action.target.display(),
                error
            ));
        } else if let Some(skill) = command_state
            .state
            .skills
            .iter_mut()
            .find(|skill| skill.id == action.skill_id)
        {
            skill.managed_links.codex = Some(action.target.clone());
        }
    }

    if issues.is_empty() {
        command_state.state.sync_status = SyncStatus {
            phase: SyncPhase::Healthy,
            message: Some(if report.conflicts.is_empty() {
                "Codex 同步已完成。".to_string()
            } else {
                format!(
                    "Codex 同步已完成，但仍有 {} 个冲突未覆盖。",
                    report.conflicts.len()
                )
            }),
            pending_actions: Vec::new(),
        };
    } else {
        let repair_report = plan_codex_sync(&command_state.state.skills, &active, &codex_path)
            .map_err(|error| error.to_string())?;
        command_state.state.sync_status = SyncStatus {
            phase: SyncPhase::RepairRequired,
            message: Some(format!("Codex 同步未完成：{}", issues.join("；"))),
            pending_actions: build_pending_actions(&repair_report),
        };
    }

    persist(&command_state.paths, &command_state.state).map_err(|error| error.to_string())?;
    build_snapshot(
        &command_state.paths,
        command_state.state,
        &StateLoadStatus::Clean,
    )
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_store::default_state;
    use tempfile::tempdir;

    #[test]
    fn snapshot_includes_skills_when_codex_path_exists() {
        let dir = tempdir().unwrap();
        let codex = dir.path().join("codex");
        std::fs::create_dir_all(&codex).unwrap();
        let paths = AppPaths::from_config_dir(dir.path());
        let mut state = default_state(dir.path().join("skills"), Some(codex));
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
        let mut state = default_state(dir.path().join("skills"), None);
        state.skills.push(Skill {
            id: "writer".to_string(),
            name: "Writer".to_string(),
            description: String::new(),
            library_path: dir.path().join("skills").join("writer"),
            source: Default::default(),
            references: Vec::new(),
            managed_links: crate::models::ManagedLinks {
                codex: Some(dir.path().join("codex").join("writer")),
            },
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
    fn deleting_current_project_clears_context() {
        let dir = tempdir().unwrap();
        let mut state = default_state(dir.path().join("skills"), None);
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

        let mut state = default_state(library.clone(), None);
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

        let mut state = default_state(library.clone(), None);
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
        let mut state = default_state(dir.path().join("skills"), None);

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
}

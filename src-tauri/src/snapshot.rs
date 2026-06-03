use crate::app_paths::AppPaths;
use crate::error::Result;
use crate::models::AppState;
use crate::plugin_discovery::discover_plugins;
use crate::skill_references::refresh_reference_statuses;
use crate::state_store::{state_backup_path, StateLoadStatus};
use crate::target_profiles::{built_in_target_profiles, SkillTargetProfile};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

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

    state.plugins = discover_plugins(&state.skills);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_store::default_state;
    use crate::{app_paths::AppPaths, models::Skill};
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
}

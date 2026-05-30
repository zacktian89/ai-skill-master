use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub schema_version: u32,
    pub skill_library_path: PathBuf,
    pub codex_skills_path: Option<PathBuf>,
    pub current_project_id: Option<String>,
    #[serde(default)]
    pub sync_status: SyncStatus,
    #[serde(default)]
    pub migration_notice: Option<MigrationNotice>,
    pub skills: Vec<Skill>,
    pub projects: Vec<Project>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub library_path: PathBuf,
    #[serde(default)]
    pub source: SkillSource,
    pub managed_links: ManagedLinks,
    pub conflict: Option<SkillConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillSource {
    pub kind: SkillSourceKind,
    pub label: Option<String>,
    pub url: Option<String>,
    pub path: Option<PathBuf>,
    #[serde(rename = "ref")]
    pub source_ref: Option<String>,
    pub commit: Option<String>,
    pub subdir: Option<String>,
}

impl SkillSource {
    pub fn local(path: Option<PathBuf>) -> Self {
        Self {
            kind: SkillSourceKind::Local,
            label: Some("本地".to_string()),
            url: None,
            path,
            source_ref: None,
            commit: None,
            subdir: None,
        }
    }
}

impl Default for SkillSource {
    fn default() -> Self {
        Self::local(None)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SkillSourceKind {
    Local,
    Github,
    OpenclawMarket,
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManagedLinks {
    pub codex: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillConflict {
    pub target: String,
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    #[serde(default)]
    pub phase: SyncPhase,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub pending_actions: Vec<PendingSyncAction>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SyncPhase {
    #[default]
    Idle,
    Healthy,
    RepairRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PendingSyncAction {
    pub kind: PendingSyncActionKind,
    pub skill_id: String,
    pub target: PathBuf,
    pub source: Option<PathBuf>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PendingSyncActionKind {
    Create,
    Remove,
    Inspect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MigrationNotice {
    pub old_library_path: PathBuf,
    pub new_library_path: PathBuf,
    pub message: String,
    pub requires_codex_resync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub rules: BTreeMap<String, ProjectRule>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectRule {
    Inherit,
    Enable,
    Disable,
}

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub schema_version: u32,
    pub skill_library_path: PathBuf,
    pub current_project_id: Option<String>,
    #[serde(default)]
    pub sync_status: SyncStatus,
    #[serde(default)]
    pub migration_notice: Option<MigrationNotice>,
    pub skills: Vec<Skill>,
    pub projects: Vec<Project>,
    #[serde(default)]
    pub agents: Vec<Agent>,
    #[serde(default)]
    pub plugins: Vec<Plugin>,
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
    #[serde(default)]
    pub references: Vec<SkillReference>,
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

    pub fn github(
        url: String,
        source_ref: Option<String>,
        commit: Option<String>,
        subdir: Option<String>,
    ) -> Self {
        Self {
            kind: SkillSourceKind::Github,
            label: Some("GitHub".to_string()),
            url: Some(url),
            path: None,
            source_ref,
            commit,
            subdir,
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
pub struct ManagedLinks {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillReference {
    pub id: String,
    pub target_name: String,
    pub target_path: PathBuf,
    #[serde(default = "default_reference_scope")]
    pub scope: ReferenceScope,
    #[serde(default = "default_reference_status")]
    pub status: ReferenceStatus,
}

fn default_reference_scope() -> ReferenceScope {
    ReferenceScope::User
}

fn default_reference_status() -> ReferenceStatus {
    ReferenceStatus::Healthy
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ReferenceScope {
    User,
    Project,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ReferenceStatus {
    Healthy,
    Missing,
    Conflict,
    Stale,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub rules: BTreeMap<String, ProjectRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub version: Option<String>,
    pub author: Option<String>,
    pub agent_targets: Vec<String>,
    pub skills: Vec<Skill>,
    pub mcp_servers: Option<Vec<String>>,
    pub mcp_config: Option<serde_json::Value>,
    pub r#type: String, // "standard" | "mcp"
    #[serde(default = "default_plugin_enabled")]
    pub enabled: bool,
}

fn default_plugin_enabled() -> bool {
    true
}

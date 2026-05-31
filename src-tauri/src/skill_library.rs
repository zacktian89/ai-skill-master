use crate::error::{Result, SkillMasterError};
use crate::models::{ManagedLinks, MigrationNotice, Skill, SkillSource};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ImportSkillSource {
    Local {
        path: PathBuf,
    },
    Github {
        url: String,
        #[serde(rename = "ref")]
        source_ref: Option<String>,
        subdir: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportSkillPreview {
    pub candidates: Vec<ImportSkillCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportSkillCandidate {
    pub candidate_id: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub relative_path: String,
    pub status: ImportSkillCandidateStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImportSkillCandidateStatus {
    Ready,
    Duplicate,
    Conflict,
    Invalid,
}

#[derive(Debug, Clone)]
struct ResolvedImportSource {
    root: PathBuf,
    original_url: Option<String>,
    source_ref: Option<String>,
    commit: Option<String>,
    subdir: Option<String>,
}

#[derive(Debug, Clone)]
struct SkillCandidateEntry {
    metadata: SkillMetadata,
    path: PathBuf,
    relative_path: String,
}

pub fn read_skill_metadata(skill_dir: &Path) -> Result<SkillMetadata> {
    let skill_md = skill_dir.join("SKILL.md");
    if !skill_md.exists() {
        return Err(SkillMasterError::MissingSkillMarkdown(
            skill_dir.to_path_buf(),
        ));
    }
    let raw = fs::read_to_string(&skill_md)?;
    let id = skill_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| SkillMasterError::InvalidPath(skill_dir.display().to_string()))?
        .to_string();

    let mut name = id.clone();
    let mut description = String::new();
    if raw.starts_with("---\n") {
        if let Some(end) = raw[4..].find("\n---") {
            let front_matter = &raw[4..4 + end];
            for line in front_matter.lines() {
                if let Some(value) = line.strip_prefix("name:") {
                    name = value.trim().trim_matches('"').to_string();
                }
                if let Some(value) = line.strip_prefix("description:") {
                    description = value.trim().trim_matches('"').to_string();
                }
            }
        }
    }

    Ok(SkillMetadata {
        id,
        name,
        description,
    })
}

pub fn import_skill(state: &mut crate::models::AppState, source: &Path) -> Result<()> {
    if !source.is_dir() {
        return Err(SkillMasterError::MissingDirectory(source.to_path_buf()));
    }
    let metadata = read_skill_metadata(source)?;
    import_skill_from_path(
        state,
        source,
        metadata,
        SkillSource::local(Some(source.to_path_buf())),
    )
}

pub fn preview_import_skills(
    state: &crate::models::AppState,
    source: &ImportSkillSource,
) -> Result<ImportSkillPreview> {
    with_import_source(source, |resolved| {
        let mut candidates = scan_skill_candidates(state, resolved.root())?;
        candidates.sort_by(|left, right| {
            left.relative_path
                .cmp(&right.relative_path)
                .then_with(|| left.name.cmp(&right.name))
        });
        Ok(ImportSkillPreview { candidates })
    })
}

pub fn import_selected_skills(
    state: &mut crate::models::AppState,
    source: &ImportSkillSource,
    candidate_ids: &[String],
) -> Result<()> {
    if candidate_ids.is_empty() {
        return Ok(());
    }

    with_import_source(source, |resolved| {
        let entries = collect_skill_entries(resolved.root())?;
        for candidate_id in candidate_ids {
            let entry = entries
                .iter()
                .find(|entry| &entry.relative_path == candidate_id)
                .ok_or_else(|| SkillMasterError::InvalidPath(candidate_id.clone()))?;
            if candidate_status(state, &entry.metadata) != ImportSkillCandidateStatus::Ready {
                return Err(SkillMasterError::DuplicateSkill(entry.metadata.id.clone()));
            }
            let source = source_for_candidate(resolved, entry);
            import_skill_from_path(state, &entry.path, entry.metadata.clone(), source)?;
        }
        Ok(())
    })
}

fn import_skill_from_path(
    state: &mut crate::models::AppState,
    source: &Path,
    metadata: SkillMetadata,
    skill_source: SkillSource,
) -> Result<()> {
    if state.skills.iter().any(|skill| skill.id == metadata.id) {
        return Err(SkillMasterError::DuplicateSkill(metadata.id));
    }
    fs::create_dir_all(&state.skill_library_path)?;
    let target = state.skill_library_path.join(&metadata.id);
    if target.exists() {
        return Err(SkillMasterError::DuplicateSkill(metadata.id));
    }
    copy_dir_all(source, &target)?;
    state.skills.push(Skill {
        id: metadata.id,
        name: metadata.name,
        description: metadata.description,
        library_path: target,
        source: skill_source,
        references: Vec::new(),
        managed_links: ManagedLinks::default(),
        conflict: None,
    });
    state.skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(())
}

pub fn scan_skill_library(root: &Path) -> Result<Vec<Skill>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let skill_dir = entry.path();
        let metadata = match read_skill_metadata(&skill_dir) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        skills.push(Skill {
            id: metadata.id,
            name: metadata.name,
            description: metadata.description,
            library_path: skill_dir,
            source: SkillSource::default(),
            references: Vec::new(),
            managed_links: ManagedLinks::default(),
            conflict: None,
        });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

pub fn delete_skill(state: &mut crate::models::AppState, skill_id: &str) -> Result<()> {
    let skill = state
        .skills
        .iter()
        .find(|skill| skill.id == skill_id)
        .cloned()
        .ok_or_else(|| SkillMasterError::SkillNotFound(skill_id.to_string()))?;

    if skill.library_path.exists() {
        fs::remove_dir_all(&skill.library_path)?;
    }
    state.skills.retain(|skill| skill.id != skill_id);
    for project in &mut state.projects {
        project.rules.remove(skill_id);
    }
    Ok(())
}

pub fn migrate_skill_library(
    state: &mut crate::models::AppState,
    target_root: &Path,
) -> Result<()> {
    let previous_root = state.skill_library_path.clone();
    fs::create_dir_all(target_root)?;
    for skill in &state.skills {
        let target = target_root.join(&skill.id);
        if target.exists() {
            return Err(SkillMasterError::DuplicateSkill(skill.id.clone()));
        }
        copy_dir_all(&skill.library_path, &target)?;
    }
    state.skill_library_path = target_root.to_path_buf();
    for skill in &mut state.skills {
        skill.library_path = target_root.join(&skill.id);
        skill.managed_links.codex = None;
    }
    state.migration_notice = Some(MigrationNotice {
        old_library_path: previous_root,
        new_library_path: target_root.to_path_buf(),
        message: "技能库迁移已完成，SkillMaster 已切换到新目录。旧目录不会自动删除；如需让 Codex 使用新技能库，请重新同步。"
            .to_string(),
        requires_codex_resync: true,
    });
    Ok(())
}

enum ImportSourceGuard {
    Local(ResolvedImportSource),
    Temp {
        resolved: ResolvedImportSource,
        _temp_dir: TempDir,
    },
}

impl ImportSourceGuard {
    fn root(&self) -> &Path {
        match self {
            ImportSourceGuard::Local(resolved) | ImportSourceGuard::Temp { resolved, .. } => {
                &resolved.root
            }
        }
    }

    fn resolved(&self) -> &ResolvedImportSource {
        match self {
            ImportSourceGuard::Local(resolved) | ImportSourceGuard::Temp { resolved, .. } => {
                resolved
            }
        }
    }
}

fn with_import_source<T>(
    source: &ImportSkillSource,
    operation: impl FnOnce(&ImportSourceGuard) -> Result<T>,
) -> Result<T> {
    let guard = match source {
        ImportSkillSource::Local { path } => {
            if !path.is_dir() {
                return Err(SkillMasterError::MissingDirectory(path.clone()));
            }
            ImportSourceGuard::Local(ResolvedImportSource {
                root: path.clone(),
                original_url: None,
                source_ref: None,
                commit: None,
                subdir: None,
            })
        }
        ImportSkillSource::Github {
            url,
            source_ref,
            subdir,
        } => {
            let parsed = parse_github_tree_url(url);
            let clone_url = parsed
                .as_ref()
                .map(|parsed| parsed.clone_url.clone())
                .unwrap_or_else(|| url.clone());
            let requested_ref = source_ref
                .clone()
                .or_else(|| parsed.as_ref().and_then(|parsed| parsed.source_ref.clone()));
            let requested_subdir = subdir
                .clone()
                .or_else(|| parsed.as_ref().and_then(|parsed| parsed.subdir.clone()))
                .filter(|value| !value.trim().is_empty());
            let temp_dir = TempDir::new()?;
            clone_github_source(&clone_url, requested_ref.as_deref(), temp_dir.path())?;
            let commit = git_commit(temp_dir.path()).ok();
            let root = if let Some(subdir) = requested_subdir.as_deref() {
                let root = temp_dir.path().join(subdir);
                if !root.is_dir() {
                    return Err(SkillMasterError::MissingDirectory(root));
                }
                root
            } else {
                temp_dir.path().to_path_buf()
            };
            ImportSourceGuard::Temp {
                resolved: ResolvedImportSource {
                    root,
                    original_url: Some(clone_url),
                    source_ref: requested_ref,
                    commit,
                    subdir: requested_subdir,
                },
                _temp_dir: temp_dir,
            }
        }
    };

    operation(&guard)
}

#[derive(Debug, Clone)]
struct ParsedGithubTreeUrl {
    clone_url: String,
    source_ref: Option<String>,
    subdir: Option<String>,
}

fn parse_github_tree_url(url: &str) -> Option<ParsedGithubTreeUrl> {
    let marker = "github.com/";
    let (_, after_host) = url.split_once(marker)?;
    let parts = after_host
        .trim_end_matches('/')
        .split('/')
        .collect::<Vec<_>>();
    if parts.len() < 2 {
        return None;
    }
    let owner = parts[0];
    let repo = parts[1].trim_end_matches(".git");
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    let clone_url = format!("https://github.com/{owner}/{repo}.git");
    let tree_index = parts.iter().position(|part| *part == "tree");
    let source_ref = tree_index
        .and_then(|index| parts.get(index + 1))
        .map(|value| (*value).to_string());
    let subdir = tree_index
        .map(|index| parts.iter().skip(index + 2).copied().collect::<Vec<_>>())
        .filter(|parts| !parts.is_empty())
        .map(|parts| parts.join("/"));
    Some(ParsedGithubTreeUrl {
        clone_url,
        source_ref,
        subdir,
    })
}

fn clone_github_source(url: &str, source_ref: Option<&str>, target: &Path) -> Result<()> {
    let mut command = Command::new("git");
    command.arg("clone").arg("--depth").arg("1");
    if let Some(source_ref) = source_ref.filter(|value| !value.trim().is_empty()) {
        command.arg("--branch").arg(source_ref);
    }
    command.arg(url).arg(target);
    run_command(command, "GitHub clone")
}

fn git_commit(repo: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("rev-parse")
        .arg("HEAD")
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(SkillMasterError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

fn run_command(mut command: Command, label: &str) -> Result<()> {
    let output = command.output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(SkillMasterError::CommandFailed(format!(
        "{label}：{}",
        if stderr.is_empty() {
            "未知错误".to_string()
        } else {
            stderr
        }
    )))
}

fn scan_skill_candidates(
    state: &crate::models::AppState,
    root: &Path,
) -> Result<Vec<ImportSkillCandidate>> {
    Ok(collect_skill_entries(root)?
        .into_iter()
        .map(|entry| {
            let status = candidate_status(state, &entry.metadata);
            ImportSkillCandidate {
                candidate_id: entry.relative_path.clone(),
                id: entry.metadata.id,
                name: entry.metadata.name,
                description: entry.metadata.description,
                relative_path: entry.relative_path,
                status,
                message: candidate_status_message(status),
            }
        })
        .collect())
}

fn collect_skill_entries(root: &Path) -> Result<Vec<SkillCandidateEntry>> {
    if !root.is_dir() {
        return Err(SkillMasterError::MissingDirectory(root.to_path_buf()));
    }
    let mut entries = Vec::new();
    collect_skill_entries_recursive(root, root, &mut entries)?;
    Ok(entries)
}

fn collect_skill_entries_recursive(
    root: &Path,
    current: &Path,
    entries: &mut Vec<SkillCandidateEntry>,
) -> Result<()> {
    if current.join("SKILL.md").exists() {
        let metadata = read_skill_metadata(current)?;
        entries.push(SkillCandidateEntry {
            metadata,
            path: current.to_path_buf(),
            relative_path: relative_candidate_path(root, current),
        });
    }

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        if should_skip_scan_dir(name.to_string_lossy().as_ref()) {
            continue;
        }
        collect_skill_entries_recursive(root, &entry.path(), entries)?;
    }
    Ok(())
}

fn should_skip_scan_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | ".next"
            | ".nuxt"
            | ".venv"
            | "venv"
            | "__pycache__"
    )
}

fn relative_candidate_path(root: &Path, current: &Path) -> String {
    let relative = current.strip_prefix(root).unwrap_or(current);
    if relative.as_os_str().is_empty() {
        ".".to_string()
    } else {
        relative.to_string_lossy().replace('\\', "/")
    }
}

fn candidate_status(
    state: &crate::models::AppState,
    metadata: &SkillMetadata,
) -> ImportSkillCandidateStatus {
    if state.skills.iter().any(|skill| skill.id == metadata.id) {
        ImportSkillCandidateStatus::Duplicate
    } else if state.skill_library_path.join(&metadata.id).exists() {
        ImportSkillCandidateStatus::Conflict
    } else {
        ImportSkillCandidateStatus::Ready
    }
}

fn candidate_status_message(status: ImportSkillCandidateStatus) -> Option<String> {
    match status {
        ImportSkillCandidateStatus::Ready => None,
        ImportSkillCandidateStatus::Duplicate => Some("已存在".to_string()),
        ImportSkillCandidateStatus::Conflict => Some("目标目录已存在".to_string()),
        ImportSkillCandidateStatus::Invalid => Some("无效".to_string()),
    }
}

fn source_for_candidate(resolved: &ImportSourceGuard, entry: &SkillCandidateEntry) -> SkillSource {
    let resolved_source = resolved.resolved();
    if let Some(url) = resolved_source.original_url.clone() {
        SkillSource::github(
            url,
            resolved_source.source_ref.clone(),
            resolved_source.commit.clone(),
            combined_subdir(resolved_source.subdir.as_deref(), &entry.relative_path),
        )
    } else {
        SkillSource::local(Some(entry.path.clone()))
    }
}

fn combined_subdir(base: Option<&str>, relative: &str) -> Option<String> {
    match (base.filter(|value| !value.is_empty()), relative) {
        (None, ".") => None,
        (None, relative) => Some(relative.to_string()),
        (Some(base), ".") => Some(base.to_string()),
        (Some(base), relative) => Some(format!("{}/{}", base.trim_end_matches('/'), relative)),
    }
}

fn copy_dir_all(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AppState, Project};
    use crate::state_store::default_state;
    use std::collections::BTreeMap;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parses_front_matter_name_and_description() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("markdown-go");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: markdown-go\ndescription: Convert Markdown to WeChat HTML\n---\n# Body\n",
        )
        .unwrap();

        let metadata = read_skill_metadata(&skill_dir).unwrap();

        assert_eq!(metadata.id, "markdown-go");
        assert_eq!(metadata.name, "markdown-go");
        assert_eq!(metadata.description, "Convert Markdown to WeChat HTML");
    }

    #[test]
    fn rejects_folder_without_skill_markdown() {
        let dir = tempdir().unwrap();
        let err = read_skill_metadata(dir.path()).unwrap_err().to_string();

        assert!(err.contains("SKILL.md"));
    }

    #[test]
    fn imports_skill_into_library() {
        let source_root = tempdir().unwrap();
        let library_root = tempdir().unwrap();
        let source = source_root.path().join("writer");
        fs::create_dir_all(&source).unwrap();
        fs::write(
            source.join("SKILL.md"),
            "---\nname: writer\ndescription: Write drafts\n---\n",
        )
        .unwrap();
        let mut state = default_state(library_root.path().to_path_buf(), None);

        import_skill(&mut state, &source).unwrap();

        assert_eq!(state.skills.len(), 1);
        assert_eq!(state.skills[0].id, "writer");
        assert_eq!(state.skills[0].source, SkillSource::local(Some(source)));
        assert!(library_root.path().join("writer").join("SKILL.md").exists());
    }

    #[test]
    fn previews_multiple_skills_from_collection() {
        let source_root = tempdir().unwrap();
        let library_root = tempdir().unwrap();
        let writer = source_root.path().join("skills").join("writer");
        let reviewer = source_root.path().join("skills").join("reviewer");
        fs::create_dir_all(&writer).unwrap();
        fs::create_dir_all(&reviewer).unwrap();
        fs::write(
            writer.join("SKILL.md"),
            "---\nname: writer\ndescription: Write drafts\n---\n",
        )
        .unwrap();
        fs::write(
            reviewer.join("SKILL.md"),
            "---\nname: reviewer\ndescription: Review drafts\n---\n",
        )
        .unwrap();
        let state = default_state(library_root.path().to_path_buf(), None);

        let preview = preview_import_skills(
            &state,
            &ImportSkillSource::Local {
                path: source_root.path().to_path_buf(),
            },
        )
        .unwrap();

        assert_eq!(preview.candidates.len(), 2);
        assert!(preview
            .candidates
            .iter()
            .all(|candidate| candidate.status == ImportSkillCandidateStatus::Ready));
        assert!(preview
            .candidates
            .iter()
            .any(|candidate| candidate.relative_path == "skills/writer"));
    }

    #[test]
    fn imports_selected_skills_from_collection() {
        let source_root = tempdir().unwrap();
        let library_root = tempdir().unwrap();
        let writer = source_root.path().join("writer");
        let reviewer = source_root.path().join("reviewer");
        fs::create_dir_all(&writer).unwrap();
        fs::create_dir_all(&reviewer).unwrap();
        fs::write(writer.join("SKILL.md"), "---\nname: writer\n---\n").unwrap();
        fs::write(reviewer.join("SKILL.md"), "---\nname: reviewer\n---\n").unwrap();
        let mut state = default_state(library_root.path().to_path_buf(), None);

        import_selected_skills(
            &mut state,
            &ImportSkillSource::Local {
                path: source_root.path().to_path_buf(),
            },
            &["writer".to_string()],
        )
        .unwrap();

        assert_eq!(state.skills.len(), 1);
        assert_eq!(state.skills[0].id, "writer");
        assert!(library_root.path().join("writer").join("SKILL.md").exists());
        assert!(!library_root.path().join("reviewer").exists());
    }

    #[test]
    fn scans_existing_skill_library() {
        let root = tempdir().unwrap();
        let skill_dir = root.path().join("writer");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: writer\n---\n").unwrap();

        let skills = scan_skill_library(root.path()).unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].id, "writer");
        assert_eq!(skills[0].source, SkillSource::default());
    }

    #[test]
    fn migrates_skill_library_and_updates_skill_paths() {
        let source_root = tempdir().unwrap();
        let old_root = tempdir().unwrap();
        let new_root = tempdir().unwrap();
        let source = source_root.path().join("writer");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("SKILL.md"), "---\nname: writer\n---\n").unwrap();
        let mut state = default_state(old_root.path().to_path_buf(), None);
        import_skill(&mut state, &source).unwrap();

        migrate_skill_library(&mut state, new_root.path()).unwrap();

        assert_eq!(state.skill_library_path, new_root.path());
        assert_eq!(state.skills[0].library_path, new_root.path().join("writer"));
        assert!(new_root.path().join("writer").join("SKILL.md").exists());
        assert!(state.migration_notice.is_some());
    }

    #[test]
    fn delete_skill_removes_project_rules() {
        let source_root = tempdir().unwrap();
        let library_root = tempdir().unwrap();
        let skill_dir = source_root.path().join("writer");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: writer\n---\n").unwrap();

        let mut state: AppState = default_state(library_root.path().to_path_buf(), None);
        import_skill(&mut state, &skill_dir).unwrap();
        let mut rules = BTreeMap::new();
        rules.insert("writer".to_string(), crate::models::ProjectRule::Disable);
        state.projects.push(Project {
            id: "p1".to_string(),
            name: "Project".to_string(),
            path: library_root.path().join("project"),
            rules,
        });

        delete_skill(&mut state, "writer").unwrap();

        assert!(state.skills.is_empty());
        assert!(!state.projects[0].rules.contains_key("writer"));
    }
}

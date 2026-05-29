use crate::error::Result;
use crate::models::Skill;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub to_create: Vec<LinkAction>,
    pub to_remove: Vec<LinkAction>,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkAction {
    pub skill_id: String,
    pub source: PathBuf,
    pub target: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncConflict {
    pub skill_id: String,
    pub target: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedLinkValidation {
    Valid,
    Missing,
    WrongType,
    WrongTarget { actual: PathBuf },
    MissingSource,
}

pub fn plan_codex_sync(
    skills: &[Skill],
    active_skill_ids: &[String],
    codex_skills_path: &Path,
) -> Result<SyncReport> {
    let active = active_skill_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut report = SyncReport {
        to_create: Vec::new(),
        to_remove: Vec::new(),
        conflicts: Vec::new(),
    };

    for skill in skills {
        let target = codex_skills_path.join(&skill.id);
        let should_be_active = active.contains(&skill.id);
        let managed_recorded = skill.managed_links.codex.as_ref() == Some(&target);

        if should_be_active {
            match validate_managed_link(&skill.library_path, &target)? {
                ManagedLinkValidation::Missing => report.to_create.push(LinkAction {
                    skill_id: skill.id.clone(),
                    source: skill.library_path.clone(),
                    target,
                }),
                ManagedLinkValidation::Valid => {}
                ManagedLinkValidation::WrongType => report.conflicts.push(SyncConflict {
                    skill_id: skill.id.clone(),
                    target,
                    message: if managed_recorded {
                        "托管链接记录与磁盘现场不一致：目标已被替换为普通目录或文件，SkillMaster 未覆盖它"
                            .to_string()
                    } else {
                        "Codex 目录中已有同名非托管 skill".to_string()
                    },
                }),
                ManagedLinkValidation::WrongTarget { actual } => {
                    report.conflicts.push(SyncConflict {
                        skill_id: skill.id.clone(),
                        target,
                        message: if managed_recorded {
                            format!(
                                "托管链接已改为指向其他位置，SkillMaster 未覆盖它：{}",
                                actual.display()
                            )
                        } else {
                            "Codex 目录中已有同名非托管 skill".to_string()
                        },
                    })
                }
                ManagedLinkValidation::MissingSource => report.conflicts.push(SyncConflict {
                    skill_id: skill.id.clone(),
                    target,
                    message: "技能库中的源目录不存在，无法同步到 Codex".to_string(),
                }),
            }
        } else if let Some(managed_target) = &skill.managed_links.codex {
            report.to_remove.push(LinkAction {
                skill_id: skill.id.clone(),
                source: skill.library_path.clone(),
                target: managed_target.clone(),
            });
        }
    }

    Ok(report)
}

pub fn create_directory_link(source: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)?;
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)?;
    }
    Ok(())
}

pub fn remove_managed_link(target: &Path) -> Result<()> {
    if fs::symlink_metadata(target).is_ok() {
        fs::remove_dir(target)?;
    }
    Ok(())
}

pub fn validate_managed_link(source: &Path, target: &Path) -> Result<ManagedLinkValidation> {
    if !source.exists() {
        return Ok(ManagedLinkValidation::MissingSource);
    }

    let metadata = match fs::symlink_metadata(target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ManagedLinkValidation::Missing);
        }
        Err(error) => return Err(error.into()),
    };

    if !metadata.file_type().is_symlink() {
        return Ok(ManagedLinkValidation::WrongType);
    }

    let actual = fs::read_link(target)?;
    if same_path(&actual, source)? {
        Ok(ManagedLinkValidation::Valid)
    } else {
        Ok(ManagedLinkValidation::WrongTarget { actual })
    }
}

pub fn managed_link_issue_message(target: &Path, validation: &ManagedLinkValidation) -> String {
    match validation {
        ManagedLinkValidation::Valid => "托管链接状态正常".to_string(),
        ManagedLinkValidation::Missing => format!(
            "托管链接记录仍在，但磁盘上的目标不存在：{}",
            target.display()
        ),
        ManagedLinkValidation::WrongType => format!(
            "托管链接记录仍在，但目标已不是 SkillMaster 创建的目录链接：{}",
            target.display()
        ),
        ManagedLinkValidation::WrongTarget { actual } => format!(
            "托管链接已指向其他位置，SkillMaster 未删除它：{} -> {}",
            target.display(),
            actual.display()
        ),
        ManagedLinkValidation::MissingSource => {
            "技能库中的源目录不存在，无法验证或重建托管链接".to_string()
        }
    }
}

fn same_path(actual: &Path, expected: &Path) -> Result<bool> {
    let actual = canonical_or_original(actual)?;
    let expected = canonical_or_original(expected)?;
    Ok(actual == expected)
}

fn canonical_or_original(path: &Path) -> Result<PathBuf> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(path.to_path_buf()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ManagedLinks, Skill, SkillSource};
    use std::fs;
    use tempfile::tempdir;

    fn skill(id: &str, library_root: &std::path::Path) -> Skill {
        let library_path = library_root.join(id);
        fs::create_dir_all(&library_path).unwrap();
        Skill {
            id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            library_path,
            source: SkillSource::default(),
            default_enabled: true,
            managed_links: ManagedLinks::default(),
            conflict: None,
        }
    }

    #[test]
    fn detects_existing_non_managed_folder_as_conflict() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("library");
        let codex = dir.path().join("codex");
        fs::create_dir_all(codex.join("writer")).unwrap();
        let skill = skill("writer", &library);

        let report = plan_codex_sync(&[skill], &["writer".to_string()], &codex).unwrap();

        assert_eq!(report.conflicts.len(), 1);
        assert_eq!(report.conflicts[0].skill_id, "writer");
        assert!(report.to_create.is_empty());
    }

    #[test]
    fn plans_create_for_active_missing_skill() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("library");
        let codex = dir.path().join("codex");
        fs::create_dir_all(&codex).unwrap();
        let skill = skill("writer", &library);

        let report = plan_codex_sync(&[skill], &["writer".to_string()], &codex).unwrap();

        assert_eq!(report.to_create.len(), 1);
        assert_eq!(report.to_create[0].skill_id, "writer");
        assert!(report.conflicts.is_empty());
    }

    #[test]
    fn treats_retargeted_managed_link_as_conflict() {
        let dir = tempdir().unwrap();
        let library = dir.path().join("library");
        let codex = dir.path().join("codex");
        let other = dir.path().join("other");
        fs::create_dir_all(&codex).unwrap();
        fs::create_dir_all(&other).unwrap();
        let mut skill = skill("writer", &library);
        let target = codex.join("writer");
        create_directory_link(&other, &target).unwrap();
        skill.managed_links.codex = Some(target);

        let report = plan_codex_sync(&[skill], &["writer".to_string()], &codex).unwrap();

        assert_eq!(report.conflicts.len(), 1);
        assert!(report.conflicts[0].message.contains("指向其他位置"));
    }

    #[test]
    fn validates_managed_link_against_expected_source() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source");
        let target_root = dir.path().join("target");
        let target = target_root.join("writer");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target_root).unwrap();
        create_directory_link(&source, &target).unwrap();

        let validation = validate_managed_link(&source, &target).unwrap();

        assert_eq!(validation, ManagedLinkValidation::Valid);
    }
}

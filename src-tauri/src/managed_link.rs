use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedLinkValidation {
    Valid,
    Missing,
    WrongType,
    WrongTarget { actual: PathBuf },
    MissingSource,
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
    use tempfile::tempdir;

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

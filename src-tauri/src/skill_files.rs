use crate::error::{Result, SkillMasterError};
use crate::models::AppState;
use std::fs;
use std::path::PathBuf;

pub fn read_skill_file_from_state(state: &AppState, skill_id: &str) -> Result<String> {
    let skill = state
        .skills
        .iter()
        .find(|skill| skill.id == skill_id)
        .ok_or_else(|| SkillMasterError::SkillNotFound(skill_id.to_string()))?;

    read_skill_file_at_path(skill.library_path.clone())
}

pub fn read_skill_file_at_path(skill_path: PathBuf) -> Result<String> {
    if !skill_path.is_dir() {
        return Err(SkillMasterError::MissingDirectory(skill_path));
    }

    let target_path = skill_path.join("SKILL.md");
    if !target_path.exists() {
        return Err(SkillMasterError::MissingSkillMarkdown(target_path));
    }

    let canonical_skill_dir = skill_path
        .canonicalize()
        .map_err(|error| SkillMasterError::InvalidPath(format!("无法解析技能目录：{error}")))?;
    let canonical_target = target_path
        .canonicalize()
        .map_err(|error| SkillMasterError::InvalidPath(format!("无法解析 target 文件：{error}")))?;

    if !canonical_target.starts_with(&canonical_skill_dir) {
        return Err(SkillMasterError::InvalidPath(
            "安全错误：非法路径访问".to_string(),
        ));
    }

    fs::read_to_string(&canonical_target)
        .map_err(|error| SkillMasterError::InvalidPath(format!("读取文件失败：{error}")))
}

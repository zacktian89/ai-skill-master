use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkillMasterError {
    #[error("文件系统操作失败：{0}")]
    Io(#[from] std::io::Error),
    #[error("状态文件格式无效：{0}")]
    Json(#[from] serde_json::Error),
    #[error("目录不存在：{0}")]
    MissingDirectory(PathBuf),
    #[error("目录缺少 SKILL.md：{0}")]
    MissingSkillMarkdown(PathBuf),
    #[error("skill 已存在：{0}")]
    DuplicateSkill(String),
    #[error("Codex 目录存在同名非托管 skill：{0}")]
    CodexConflict(String),
    #[error("找不到 skill：{0}")]
    SkillNotFound(String),
    #[error("找不到项目：{0}")]
    ProjectNotFound(String),
    #[error("路径无效：{0}")]
    InvalidPath(String),
}

pub type Result<T> = std::result::Result<T, SkillMasterError>;

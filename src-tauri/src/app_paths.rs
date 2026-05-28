use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub state_file: PathBuf,
    pub skill_library: PathBuf,
}

impl AppPaths {
    pub fn from_config_dir(config_dir: &Path) -> Self {
        Self {
            state_file: config_dir.join("skillmaster.json"),
            skill_library: config_dir.join("skills"),
        }
    }
}

pub fn detect_codex_skills_path(home_dir: &Path) -> PathBuf {
    home_dir.join(".codex").join("skills")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn derives_default_paths_from_config_dir() {
        let config = tempdir().unwrap();
        let paths = AppPaths::from_config_dir(config.path());

        assert_eq!(paths.state_file, config.path().join("skillmaster.json"));
        assert_eq!(paths.skill_library, config.path().join("skills"));
    }

    #[test]
    fn detects_codex_skills_under_home() {
        let home = tempdir().unwrap();
        let expected = home.path().join(".codex").join("skills");

        assert_eq!(detect_codex_skills_path(home.path()), expected);
    }
}

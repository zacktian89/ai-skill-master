# Task 3: Add App Paths and Skill Library Operations

**Files:**
- Create: `src-tauri/src/app_paths.rs`
- Create: `src-tauri/src/skill_library.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: Rust unit tests inside `app_paths.rs` and `skill_library.rs`

- [ ] **Step 1: Write failing app path tests**

Create `src-tauri/src/app_paths.rs` with tests:

```rust
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
```

- [ ] **Step 2: Write failing skill library tests**

Create `src-tauri/src/skill_library.rs` with tests:

```rust
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
        assert!(library_root.path().join("writer").join("SKILL.md").exists());
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
```

- [ ] **Step 3: Run tests to verify they fail**

Run:

```powershell
Set-Location src-tauri
cargo test app_paths
cargo test skill_library
Set-Location ..
```

Expected: FAIL because the functions and modules are missing.

- [ ] **Step 4: Implement app paths**

Replace non-test content in `src-tauri/src/app_paths.rs`:

```rust
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
```

- [ ] **Step 5: Implement skill metadata and copy logic**

Replace non-test content in `src-tauri/src/skill_library.rs`:

```rust
use crate::error::{Result, SkillMasterError};
use crate::models::{ManagedLinks, Skill};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
}

pub fn read_skill_metadata(skill_dir: &Path) -> Result<SkillMetadata> {
    let skill_md = skill_dir.join("SKILL.md");
    if !skill_md.exists() {
        return Err(SkillMasterError::MissingSkillMarkdown(skill_dir.to_path_buf()));
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
        default_enabled: false,
        managed_links: ManagedLinks::default(),
        conflict: None,
    });
    state.skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(())
}

pub fn delete_skill(state: &mut crate::models::AppState, skill_id: &str) -> Result<Vec<PathBuf>> {
    let skill = state
        .skills
        .iter()
        .find(|skill| skill.id == skill_id)
        .cloned()
        .ok_or_else(|| SkillMasterError::SkillNotFound(skill_id.to_string()))?;

    if skill.library_path.exists() {
        fs::remove_dir_all(&skill.library_path)?;
    }
    let removed_links = skill
        .managed_links
        .codex
        .iter()
        .cloned()
        .collect::<Vec<PathBuf>>();
    state.skills.retain(|skill| skill.id != skill_id);
    for project in &mut state.projects {
        project.rules.remove(skill_id);
    }
    Ok(removed_links)
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
```

- [ ] **Step 6: Register modules**

Modify `src-tauri/src/lib.rs` module list:

```rust
pub mod app_paths;
pub mod error;
pub mod models;
pub mod skill_library;
pub mod state_store;
```

- [ ] **Step 7: Run tests**

Run:

```powershell
Set-Location src-tauri
cargo test app_paths
cargo test skill_library
Set-Location ..
```

Expected: all `app_paths` and `skill_library` tests pass.

- [ ] **Step 8: Commit**

Run:

```powershell
git add src-tauri/src/app_paths.rs src-tauri/src/skill_library.rs src-tauri/src/lib.rs
git commit -m "feat: manage local skill library"
```

Expected: commit succeeds.

---

Parent plan: [2026-05-28-skillmaster-implementation.md](../2026-05-28-skillmaster-implementation.md)

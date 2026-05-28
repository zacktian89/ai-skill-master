# Task 2: Define Rust Models and State Store

**Files:**
- Create: `src-tauri/src/models.rs`
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/state_store.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: Rust unit tests inside `state_store.rs`

- [ ] **Step 1: Write failing state-store tests**

Create `src-tauri/src/state_store.rs` with these tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AppState, ManagedLinks, Project, ProjectRule, Skill};
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    #[test]
    fn saves_and_loads_state() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("skillmaster.json");
        let mut rules = BTreeMap::new();
        rules.insert("markdown-go".to_string(), ProjectRule::Disable);

        let state = AppState {
            schema_version: 1,
            skill_library_path: dir.path().join("skills"),
            codex_skills_path: Some(dir.path().join("codex")),
            current_project_id: Some("project-1".to_string()),
            skills: vec![Skill {
                id: "markdown-go".to_string(),
                name: "markdown-go".to_string(),
                description: "Convert Markdown to WeChat HTML".to_string(),
                library_path: dir.path().join("skills").join("markdown-go"),
                default_enabled: true,
                managed_links: ManagedLinks {
                    codex: Some(dir.path().join("codex").join("markdown-go")),
                },
                conflict: None,
            }],
            projects: vec![Project {
                id: "project-1".to_string(),
                name: "SkillMaster".to_string(),
                path: dir.path().to_path_buf(),
                rules,
            }],
        };

        save_state(&path, &state).unwrap();
        let loaded = load_state(&path).unwrap();

        assert_eq!(loaded.schema_version, 1);
        assert_eq!(loaded.skills[0].id, "markdown-go");
        assert_eq!(loaded.projects[0].rules["markdown-go"], ProjectRule::Disable);
    }

    #[test]
    fn creates_default_state_when_file_is_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.json");
        let library = dir.path().join("skills");

        let state = load_or_create_state(&path, &library, None).unwrap();

        assert_eq!(state.schema_version, 1);
        assert_eq!(state.skill_library_path, library);
        assert!(state.skills.is_empty());
        assert!(path.exists());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
Set-Location src-tauri
cargo test state_store
Set-Location ..
```

Expected: FAIL because `AppState`, `ProjectRule`, `save_state`, and `load_state` are undefined.

- [ ] **Step 3: Add models**

Create `src-tauri/src/models.rs`:

```rust
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
    pub default_enabled: bool,
    pub managed_links: ManagedLinks,
    pub conflict: Option<SkillConflict>,
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
```

- [ ] **Step 4: Add error type**

Create `src-tauri/src/error.rs`:

```rust
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
```

- [ ] **Step 5: Implement state store**

Replace the non-test part of `src-tauri/src/state_store.rs` with:

```rust
use crate::error::Result;
use crate::models::AppState;
use std::fs;
use std::path::{Path, PathBuf};

pub fn default_state(skill_library_path: PathBuf, codex_skills_path: Option<PathBuf>) -> AppState {
    AppState {
        schema_version: 1,
        skill_library_path,
        codex_skills_path,
        current_project_id: None,
        skills: Vec::new(),
        projects: Vec::new(),
    }
}

pub fn load_state(path: &Path) -> Result<AppState> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_state(path: &Path, state: &AppState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if path.exists() {
        let backup = path.with_extension("json.bak");
        fs::copy(path, backup)?;
    }
    let raw = serde_json::to_string_pretty(state)?;
    fs::write(path, raw)?;
    Ok(())
}

pub fn load_or_create_state(
    path: &Path,
    skill_library_path: &Path,
    codex_skills_path: Option<PathBuf>,
) -> Result<AppState> {
    if path.exists() {
        return load_state(path);
    }
    let state = default_state(skill_library_path.to_path_buf(), codex_skills_path);
    save_state(path, &state)?;
    Ok(state)
}
```

- [ ] **Step 6: Register modules**

Modify `src-tauri/src/lib.rs`:

```rust
pub mod error;
pub mod models;
pub mod state_store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to run SkillMaster");
}
```

- [ ] **Step 7: Run tests**

Run:

```powershell
Set-Location src-tauri
cargo test state_store
Set-Location ..
```

Expected: PASS for `saves_and_loads_state` and `creates_default_state_when_file_is_missing`.

- [ ] **Step 8: Commit**

Run:

```powershell
git add src-tauri/src/models.rs src-tauri/src/error.rs src-tauri/src/state_store.rs src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add persisted state model"
```

Expected: commit succeeds.

---

Parent plan: [2026-05-28-skillmaster-implementation.md](../2026-05-28-skillmaster-implementation.md)

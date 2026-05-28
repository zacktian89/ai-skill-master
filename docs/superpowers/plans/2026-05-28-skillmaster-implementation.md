# SkillMaster Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first installable SkillMaster desktop app for managing a local skill library, default skill activation, project rules, and Codex sync.

**Architecture:** Tauri owns all filesystem operations and exposes a small command API to Vue. Vue renders a quiet Codex-like two-column UI with only Skills, Projects, and Settings as top-level concepts. State is stored in one JSON file; skill folders live in one app-managed skill library.

**Tech Stack:** Tauri 2, Vue 3, TypeScript, Vite, Rust, serde, serde_json, thiserror, tempfile, Vitest, @vue/test-utils, lucide-vue-next.

---

## Scope Check

The spec is one product slice: a local desktop skill manager with one Codex sync target. It has backend filesystem logic and frontend UI, but those pieces are tightly coupled around one workflow. Keep it as one implementation plan.

## External References Checked

- Tauri official create-project docs, last updated 2026-04-02: `npm create tauri-app@latest`, Vue template support, and `npm run tauri dev`.
- create-tauri-app package docs: `vue-ts` template and `.` project name support.

## File Structure

Create or modify these files:

- Create: `package.json` from the Tauri Vue template, then modify scripts and dependencies.
- Create: `index.html` from the Tauri Vue template.
- Create: `vite.config.ts` from the Tauri Vue template.
- Create: `tsconfig.json` from the Tauri Vue template.
- Create: `src/main.ts` to mount Vue.
- Create: `src/App.vue` to own app-level navigation and async loading.
- Create: `src/types.ts` for frontend DTOs that mirror Rust command payloads.
- Create: `src/api.ts` for Tauri command wrappers.
- Create: `src/components/Sidebar.vue` for left navigation.
- Create: `src/components/SkillsView.vue` for skill search, list, default switches, and selected skill details.
- Create: `src/components/ProjectsView.vue` for project list, current context, and project rules.
- Create: `src/components/SettingsView.vue` for skill library path, Codex path, diagnostics, and migration.
- Create: `src/styles.css` for the Codex-like two-column visual system.
- Create: `src/__tests__/effectiveState.test.ts` for frontend rule-label behavior.
- Create: `src-tauri/Cargo.toml` from the Tauri template, then add Rust dependencies.
- Create: `src-tauri/tauri.conf.json` from the Tauri template.
- Create: `src-tauri/src/main.rs` as the Tauri app entrypoint.
- Create: `src-tauri/src/lib.rs` for command registration and module wiring.
- Create: `src-tauri/src/models.rs` for persisted state and DTOs.
- Create: `src-tauri/src/error.rs` for user-facing error types.
- Create: `src-tauri/src/app_paths.rs` for app config, state file, skill library, and Codex path detection.
- Create: `src-tauri/src/state_store.rs` for JSON load/save with backup behavior.
- Create: `src-tauri/src/skill_library.rs` for skill metadata parsing, import, delete, and migration.
- Create: `src-tauri/src/effective_state.rs` for default/project rule resolution.
- Create: `src-tauri/src/codex_sync.rs` for conflict detection and managed link sync.
- Create: `src-tauri/src/commands.rs` for Tauri commands consumed by Vue.
- Create: `src-tauri/capabilities/default.json` from the Tauri template, then include dialog capability if the plugin scaffold requires it.
- Modify: `docs/superpowers/specs/2026-05-28-skillmaster-design.md` only if implementation discovers a spec contradiction.

## Task 1: Bootstrap Tauri + Vue

**Files:**
- Create: `package.json`
- Create: `index.html`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `src/main.ts`
- Create: `src/App.vue`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/capabilities/default.json`

- [ ] **Step 1: Scaffold in a temporary sibling directory**

Run from `D:\code`:

```powershell
npm create tauri-app@latest SkillMaster-bootstrap -- --template vue-ts --manager npm
```

Expected: the generator creates `D:\code\SkillMaster-bootstrap` and prints commands that include `npm install` and `npm run tauri dev`.

- [ ] **Step 2: Copy scaffold files into the repository**

Run:

```powershell
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\package.json' -Destination 'D:\code\SkillMaster\package.json'
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\index.html' -Destination 'D:\code\SkillMaster\index.html'
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\vite.config.ts' -Destination 'D:\code\SkillMaster\vite.config.ts'
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\tsconfig.json' -Destination 'D:\code\SkillMaster\tsconfig.json'
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\src' -Destination 'D:\code\SkillMaster\src' -Recurse
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\src-tauri' -Destination 'D:\code\SkillMaster\src-tauri' -Recurse
```

Expected: `rg --files` lists Vue and Tauri files alongside `docs/superpowers`.

- [ ] **Step 3: Set product identity**

Modify `src-tauri/tauri.conf.json` so the visible app name and identifier are:

```json
{
  "productName": "SkillMaster",
  "identifier": "com.zacktian.skillmaster"
}
```

Keep the rest of the generated file structure intact.

- [ ] **Step 4: Install runtime and test dependencies**

Run from `D:\code\SkillMaster`:

```powershell
npm install
npm install lucide-vue-next @tauri-apps/plugin-dialog
npm install -D vitest @vue/test-utils jsdom
Set-Location src-tauri
cargo add serde --features derive
cargo add serde_json
cargo add thiserror
cargo add tauri-plugin-dialog
cargo add tempfile --dev
Set-Location ..
```

Expected: `package-lock.json` exists, `src-tauri/Cargo.lock` exists, and no install command exits with an error.

- [ ] **Step 5: Add test scripts**

Modify `package.json` scripts to include:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "tauri": "tauri"
  }
}
```

Keep generated scripts that already match these names.

- [ ] **Step 6: Verify the clean scaffold**

Run:

```powershell
npm run build
Set-Location src-tauri
cargo test
Set-Location ..
```

Expected: Vue build succeeds and Rust tests pass.

- [ ] **Step 7: Commit**

Run:

```powershell
git add package.json package-lock.json index.html vite.config.ts tsconfig.json src src-tauri
git commit -m "chore: scaffold Tauri Vue app"
```

Expected: commit succeeds with scaffold files.

## Task 2: Define Rust Models and State Store

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

## Task 3: Add App Paths and Skill Library Operations

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

## Task 4: Add Effective State and Project Rules

**Files:**
- Create: `src-tauri/src/effective_state.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: Rust unit tests inside `effective_state.rs`

- [ ] **Step 1: Write failing tests**

Create `src-tauri/src/effective_state.rs` with tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ManagedLinks, Project, ProjectRule, Skill};
    use crate::state_store::default_state;
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    fn skill(id: &str, default_enabled: bool) -> Skill {
        Skill {
            id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            library_path: tempdir().unwrap().path().join(id),
            default_enabled,
            managed_links: ManagedLinks::default(),
            conflict: None,
        }
    }

    #[test]
    fn no_project_uses_default_enabled_skills() {
        let dir = tempdir().unwrap();
        let mut state = default_state(dir.path().join("skills"), None);
        state.skills = vec![skill("writer", true), skill("imagegen", false)];

        let active = effective_skill_ids(&state, None).unwrap();

        assert_eq!(active, vec!["writer".to_string()]);
    }

    #[test]
    fn project_rule_overrides_default_state() {
        let dir = tempdir().unwrap();
        let mut state = default_state(dir.path().join("skills"), None);
        state.skills = vec![skill("writer", true), skill("imagegen", false)];
        let mut rules = BTreeMap::new();
        rules.insert("writer".to_string(), ProjectRule::Disable);
        rules.insert("imagegen".to_string(), ProjectRule::Enable);
        state.projects.push(Project {
            id: "p1".to_string(),
            name: "Project".to_string(),
            path: dir.path().join("project"),
            rules,
        });

        let active = effective_skill_ids(&state, Some("p1")).unwrap();

        assert_eq!(active, vec!["imagegen".to_string()]);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
Set-Location src-tauri
cargo test effective_state
Set-Location ..
```

Expected: FAIL because `effective_skill_ids` is undefined.

- [ ] **Step 3: Implement effective state**

Replace non-test content in `src-tauri/src/effective_state.rs`:

```rust
use crate::error::{Result, SkillMasterError};
use crate::models::{AppState, ProjectRule};

pub fn effective_skill_ids(state: &AppState, project_id: Option<&str>) -> Result<Vec<String>> {
    let project = match project_id {
        Some(id) => Some(
            state
                .projects
                .iter()
                .find(|project| project.id == id)
                .ok_or_else(|| SkillMasterError::ProjectNotFound(id.to_string()))?,
        ),
        None => None,
    };

    let mut active = Vec::new();
    for skill in &state.skills {
        let enabled = match project.and_then(|project| project.rules.get(&skill.id)) {
            Some(ProjectRule::Enable) => true,
            Some(ProjectRule::Disable) => false,
            Some(ProjectRule::Inherit) | None => skill.default_enabled,
        };
        if enabled {
            active.push(skill.id.clone());
        }
    }
    active.sort();
    Ok(active)
}
```

- [ ] **Step 4: Register module**

Modify `src-tauri/src/lib.rs`:

```rust
pub mod app_paths;
pub mod effective_state;
pub mod error;
pub mod models;
pub mod skill_library;
pub mod state_store;
```

- [ ] **Step 5: Run tests**

Run:

```powershell
Set-Location src-tauri
cargo test effective_state
Set-Location ..
```

Expected: PASS for both effective-state tests.

- [ ] **Step 6: Commit**

Run:

```powershell
git add src-tauri/src/effective_state.rs src-tauri/src/lib.rs
git commit -m "feat: resolve project skill rules"
```

Expected: commit succeeds.

## Task 5: Add Codex Sync Planning and Link Ownership

**Files:**
- Create: `src-tauri/src/codex_sync.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: Rust unit tests inside `codex_sync.rs`

- [ ] **Step 1: Write failing tests**

Create `src-tauri/src/codex_sync.rs` with tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ManagedLinks, Skill};
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
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
Set-Location src-tauri
cargo test codex_sync
Set-Location ..
```

Expected: FAIL because `plan_codex_sync` is undefined.

- [ ] **Step 3: Implement sync plan and link helpers**

Replace non-test content in `src-tauri/src/codex_sync.rs`:

```rust
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
        if should_be_active {
            if target.exists() && skill.managed_links.codex.as_ref() != Some(&target) {
                report.conflicts.push(SyncConflict {
                    skill_id: skill.id.clone(),
                    target,
                    message: "Codex 目录中已有同名非托管 skill".to_string(),
                });
            } else if !target.exists() {
                report.to_create.push(LinkAction {
                    skill_id: skill.id.clone(),
                    source: skill.library_path.clone(),
                    target,
                });
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
    if target.exists() {
        fs::remove_dir(target)?;
    }
    Ok(())
}
```

- [ ] **Step 4: Register module**

Modify `src-tauri/src/lib.rs`:

```rust
pub mod app_paths;
pub mod codex_sync;
pub mod effective_state;
pub mod error;
pub mod models;
pub mod skill_library;
pub mod state_store;
```

- [ ] **Step 5: Run tests**

Run:

```powershell
Set-Location src-tauri
cargo test codex_sync
Set-Location ..
```

Expected: PASS for conflict and create-plan tests.

- [ ] **Step 6: Commit**

Run:

```powershell
git add src-tauri/src/codex_sync.rs src-tauri/src/lib.rs
git commit -m "feat: plan Codex skill sync"
```

Expected: commit succeeds.

## Task 6: Add Tauri Command API

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `cargo test`

- [ ] **Step 1: Create command DTOs and tests**

Create `src-tauri/src/commands.rs` with this test module at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AppState, ManagedLinks, Skill};
    use crate::state_store::default_state;
    use tempfile::tempdir;

    #[test]
    fn snapshot_marks_codex_connected_when_path_exists() {
        let dir = tempdir().unwrap();
        let codex = dir.path().join("codex");
        std::fs::create_dir_all(&codex).unwrap();
        let mut state = default_state(dir.path().join("skills"), Some(codex));
        state.skills.push(Skill {
            id: "writer".to_string(),
            name: "writer".to_string(),
            description: String::new(),
            library_path: dir.path().join("skills").join("writer"),
            default_enabled: true,
            managed_links: ManagedLinks::default(),
            conflict: None,
        });

        let snapshot = build_snapshot(state);

        assert!(snapshot.codex_connected);
        assert_eq!(snapshot.skills.len(), 1);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
Set-Location src-tauri
cargo test commands
Set-Location ..
```

Expected: FAIL because `build_snapshot` and DTOs are undefined.

- [ ] **Step 3: Implement command DTOs and pure snapshot builder**

Add to `src-tauri/src/commands.rs`:

```rust
use crate::app_paths::{detect_codex_skills_path, AppPaths};
use crate::codex_sync::{create_directory_link, plan_codex_sync, remove_managed_link};
use crate::effective_state::effective_skill_ids;
use crate::error::{Result, SkillMasterError};
use crate::models::{AppState, Project, ProjectRule, SkillConflict};
use crate::skill_library::{delete_skill as delete_skill_from_library, import_skill as import_skill_into_library};
use crate::state_store::{load_or_create_state, save_state};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub state: AppState,
    pub codex_connected: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProjectRequest {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetProjectRuleRequest {
    pub project_id: String,
    pub skill_id: String,
    pub rule: ProjectRule,
}

pub fn build_snapshot(state: AppState) -> AppSnapshot {
    let codex_connected = state
        .codex_skills_path
        .as_ref()
        .map(|path| path.exists())
        .unwrap_or(false);
    let mut diagnostics = Vec::new();
    if !codex_connected {
        diagnostics.push("Codex skills 目录未连接".to_string());
    }
    AppSnapshot {
        state,
        codex_connected,
        diagnostics,
    }
}
```

- [ ] **Step 4: Implement state helpers for commands**

Add below the DTOs in `commands.rs`:

```rust
fn command_paths(app: &AppHandle) -> Result<AppPaths> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| SkillMasterError::InvalidPath(error.to_string()))?;
    Ok(AppPaths::from_config_dir(&config_dir))
}

fn load_command_state(app: &AppHandle) -> Result<(AppPaths, AppState)> {
    let paths = command_paths(app)?;
    let home = app
        .path()
        .home_dir()
        .map_err(|error| SkillMasterError::InvalidPath(error.to_string()))?;
    let detected_codex = detect_codex_skills_path(&home);
    let codex = if detected_codex.exists() {
        Some(detected_codex)
    } else {
        None
    };
    let state = load_or_create_state(&paths.state_file, &paths.skill_library, codex)?;
    Ok((paths, state))
}

fn persist(paths: &AppPaths, state: &AppState) -> Result<()> {
    save_state(&paths.state_file, state)
}

fn project_id_from_path(path: &std::path::Path) -> String {
    let raw = path.to_string_lossy();
    format!("{:x}", md5_like_hash(raw.as_bytes()))
}

fn md5_like_hash(bytes: &[u8]) -> u64 {
    let mut hash = 1469598103934665603u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}
```

- [ ] **Step 5: Implement Tauri commands**

Add these command functions in `commands.rs`:

```rust
#[tauri::command]
pub fn get_snapshot(app: AppHandle) -> std::result::Result<AppSnapshot, String> {
    load_command_state(&app)
        .map(|(_, state)| build_snapshot(state))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_skill(app: AppHandle, source: PathBuf) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    import_skill_into_library(&mut state, &source).map_err(|error| error.to_string())?;
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn delete_skill(app: AppHandle, skill_id: String) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    let links = delete_skill_from_library(&mut state, &skill_id).map_err(|error| error.to_string())?;
    for link in links {
        remove_managed_link(&link).map_err(|error| error.to_string())?;
    }
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn set_default_enabled(
    app: AppHandle,
    skill_id: String,
    enabled: bool,
) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    let skill = state
        .skills
        .iter_mut()
        .find(|skill| skill.id == skill_id)
        .ok_or_else(|| format!("找不到 skill：{skill_id}"))?;
    skill.default_enabled = enabled;
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn add_project(
    app: AppHandle,
    request: AddProjectRequest,
) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    let id = project_id_from_path(&request.path);
    if !state.projects.iter().any(|project| project.id == id) {
        state.projects.push(Project {
            id,
            name: request.name,
            path: request.path,
            rules: BTreeMap::new(),
        });
    }
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn set_project_rule(
    app: AppHandle,
    request: SetProjectRuleRequest,
) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    let project = state
        .projects
        .iter_mut()
        .find(|project| project.id == request.project_id)
        .ok_or_else(|| format!("找不到项目：{}", request.project_id))?;
    if request.rule == ProjectRule::Inherit {
        project.rules.remove(&request.skill_id);
    } else {
        project.rules.insert(request.skill_id, request.rule);
    }
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn set_current_project(
    app: AppHandle,
    project_id: Option<String>,
) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    state.current_project_id = project_id;
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn set_codex_path(
    app: AppHandle,
    path: PathBuf,
) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    state.codex_skills_path = Some(path);
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}

#[tauri::command]
pub fn sync_codex(app: AppHandle) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    let codex_path = state
        .codex_skills_path
        .clone()
        .ok_or_else(|| "Codex skills 目录未设置".to_string())?;
    let active = effective_skill_ids(&state, state.current_project_id.as_deref())
        .map_err(|error| error.to_string())?;
    let report = plan_codex_sync(&state.skills, &active, &codex_path)
        .map_err(|error| error.to_string())?;

    for skill in &mut state.skills {
        skill.conflict = None;
    }
    for conflict in report.conflicts {
        if let Some(skill) = state.skills.iter_mut().find(|skill| skill.id == conflict.skill_id) {
            skill.conflict = Some(SkillConflict {
                target: "codex".to_string(),
                path: conflict.target,
                message: conflict.message,
            });
        }
    }
    for action in report.to_remove {
        remove_managed_link(&action.target).map_err(|error| error.to_string())?;
        if let Some(skill) = state.skills.iter_mut().find(|skill| skill.id == action.skill_id) {
            skill.managed_links.codex = None;
        }
    }
    for action in report.to_create {
        create_directory_link(&action.source, &action.target).map_err(|error| error.to_string())?;
        if let Some(skill) = state.skills.iter_mut().find(|skill| skill.id == action.skill_id) {
            skill.managed_links.codex = Some(action.target);
        }
    }
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}
```

- [ ] **Step 6: Register commands and dialog plugin**

Modify `src-tauri/src/lib.rs`:

```rust
pub mod app_paths;
pub mod codex_sync;
pub mod commands;
pub mod effective_state;
pub mod error;
pub mod models;
pub mod skill_library;
pub mod state_store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::import_skill,
            commands::delete_skill,
            commands::set_default_enabled,
            commands::add_project,
            commands::set_project_rule,
            commands::set_current_project,
            commands::set_codex_path,
            commands::sync_codex
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SkillMaster");
}
```

- [ ] **Step 7: Run tests**

Run:

```powershell
Set-Location src-tauri
cargo test
Set-Location ..
```

Expected: all Rust tests pass.

- [ ] **Step 8: Commit**

Run:

```powershell
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: expose SkillMaster command API"
```

Expected: commit succeeds.

## Task 7: Add Frontend Types, API Wrapper, and Store State

**Files:**
- Create: `src/types.ts`
- Create: `src/api.ts`
- Modify: `src/main.ts`
- Test: `src/__tests__/effectiveState.test.ts`

- [ ] **Step 1: Add frontend types**

Create `src/types.ts`:

```ts
export type ProjectRule = "inherit" | "enable" | "disable";

export interface ManagedLinks {
  codex?: string | null;
}

export interface SkillConflict {
  target: string;
  path: string;
  message: string;
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  libraryPath: string;
  defaultEnabled: boolean;
  managedLinks: ManagedLinks;
  conflict?: SkillConflict | null;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  rules: Record<string, ProjectRule>;
}

export interface AppState {
  schemaVersion: number;
  skillLibraryPath: string;
  codexSkillsPath?: string | null;
  currentProjectId?: string | null;
  skills: Skill[];
  projects: Project[];
}

export interface AppSnapshot {
  state: AppState;
  codexConnected: boolean;
  diagnostics: string[];
}

export interface AddProjectRequest {
  name: string;
  path: string;
}

export interface SetProjectRuleRequest {
  projectId: string;
  skillId: string;
  rule: ProjectRule;
}

export function ruleLabel(rule: ProjectRule | undefined): string {
  if (rule === "enable") return "在此项目启用";
  if (rule === "disable") return "在此项目停用";
  return "跟随默认";
}
```

- [ ] **Step 2: Write frontend test**

Create `src/__tests__/effectiveState.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { ruleLabel } from "../types";

describe("ruleLabel", () => {
  it("uses friendly project rule labels", () => {
    expect(ruleLabel(undefined)).toBe("跟随默认");
    expect(ruleLabel("inherit")).toBe("跟随默认");
    expect(ruleLabel("enable")).toBe("在此项目启用");
    expect(ruleLabel("disable")).toBe("在此项目停用");
  });
});
```

- [ ] **Step 3: Run frontend test**

Run:

```powershell
npm test -- src/__tests__/effectiveState.test.ts
```

Expected: PASS for `ruleLabel`.

- [ ] **Step 4: Add Tauri API wrapper**

Create `src/api.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { AddProjectRequest, AppSnapshot, ProjectRule, SetProjectRuleRequest } from "./types";

export function getSnapshot(): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("get_snapshot");
}

export function importSkill(source: string): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("import_skill", { source });
}

export function deleteSkill(skillId: string): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("delete_skill", { skillId });
}

export function setDefaultEnabled(skillId: string, enabled: boolean): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("set_default_enabled", { skillId, enabled });
}

export function addProject(request: AddProjectRequest): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("add_project", { request });
}

export function setProjectRule(request: SetProjectRuleRequest): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("set_project_rule", { request });
}

export function setCurrentProject(projectId: string | null): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("set_current_project", { projectId });
}

export function setCodexPath(path: string): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("set_codex_path", { path });
}

export function syncCodex(): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("sync_codex");
}

export type { ProjectRule };
```

- [ ] **Step 5: Ensure Vue entry loads global CSS**

Modify `src/main.ts`:

```ts
import { createApp } from "vue";
import App from "./App.vue";
import "./styles.css";

createApp(App).mount("#app");
```

- [ ] **Step 6: Run frontend checks**

Run:

```powershell
npm test
npm run build
```

Expected: Vitest passes and TypeScript build succeeds.

- [ ] **Step 7: Commit**

Run:

```powershell
git add src/types.ts src/api.ts src/main.ts src/__tests__/effectiveState.test.ts package.json package-lock.json
git commit -m "feat: add frontend command API"
```

Expected: commit succeeds.

## Task 8: Build Two-Column Vue UI

**Files:**
- Modify: `src/App.vue`
- Create: `src/components/Sidebar.vue`
- Create: `src/components/SkillsView.vue`
- Create: `src/components/ProjectsView.vue`
- Create: `src/components/SettingsView.vue`
- Create: `src/styles.css`

- [ ] **Step 1: Replace App shell**

Replace `src/App.vue` with:

```vue
<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { AlertCircle } from "lucide-vue-next";
import * as api from "./api";
import Sidebar from "./components/Sidebar.vue";
import SkillsView from "./components/SkillsView.vue";
import ProjectsView from "./components/ProjectsView.vue";
import SettingsView from "./components/SettingsView.vue";
import type { AppSnapshot } from "./types";

type Section = "skills" | "projects" | "settings";

const activeSection = ref<Section>("skills");
const snapshot = ref<AppSnapshot | null>(null);
const selectedSkillId = ref<string | null>(null);
const selectedProjectId = ref<string | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

const title = computed(() => {
  if (activeSection.value === "projects") return "Projects";
  if (activeSection.value === "settings") return "Settings";
  return "Skills";
});

async function refresh() {
  loading.value = true;
  error.value = null;
  try {
    snapshot.value = await api.getSnapshot();
    selectedProjectId.value =
      snapshot.value.state.currentProjectId ?? snapshot.value.state.projects[0]?.id ?? null;
  } catch (cause) {
    error.value = String(cause);
  } finally {
    loading.value = false;
  }
}

function applySnapshot(next: AppSnapshot) {
  snapshot.value = next;
}

onMounted(refresh);
</script>

<template>
  <div class="app-shell">
    <Sidebar v-model:active-section="activeSection" :snapshot="snapshot" />
    <main class="main-pane">
      <header class="topbar">
        <div>
          <h1>{{ title }}</h1>
          <p v-if="snapshot && !snapshot.codexConnected" class="topbar-note">
            Codex 未连接，可在 Settings 中设置目录。
          </p>
        </div>
      </header>

      <div v-if="error" class="notice error">
        <AlertCircle :size="16" />
        <span>{{ error }}</span>
      </div>

      <section v-if="loading" class="content-empty">正在加载 SkillMaster</section>

      <SkillsView
        v-else-if="activeSection === 'skills' && snapshot"
        :snapshot="snapshot"
        :selected-skill-id="selectedSkillId"
        @select-skill="selectedSkillId = $event"
        @snapshot="applySnapshot"
        @error="error = $event"
      />

      <ProjectsView
        v-else-if="activeSection === 'projects' && snapshot"
        :snapshot="snapshot"
        :selected-project-id="selectedProjectId"
        @select-project="selectedProjectId = $event"
        @snapshot="applySnapshot"
        @error="error = $event"
      />

      <SettingsView
        v-else-if="activeSection === 'settings' && snapshot"
        :snapshot="snapshot"
        @snapshot="applySnapshot"
        @error="error = $event"
      />
    </main>
  </div>
</template>
```

- [ ] **Step 2: Add Sidebar**

Create `src/components/Sidebar.vue`:

```vue
<script setup lang="ts">
import { FolderKanban, Library, Settings } from "lucide-vue-next";
import type { AppSnapshot } from "../types";

type Section = "skills" | "projects" | "settings";

defineProps<{
  activeSection: Section;
  snapshot: AppSnapshot | null;
}>();

defineEmits<{
  "update:activeSection": [value: Section];
}>();
</script>

<template>
  <aside class="sidebar">
    <div class="brand">SkillMaster</div>
    <nav class="nav-list">
      <button :class="{ active: activeSection === 'skills' }" @click="$emit('update:activeSection', 'skills')">
        <Library :size="18" />
        <span>Skills</span>
        <small v-if="snapshot">{{ snapshot.state.skills.length }}</small>
      </button>
      <button :class="{ active: activeSection === 'projects' }" @click="$emit('update:activeSection', 'projects')">
        <FolderKanban :size="18" />
        <span>Projects</span>
        <small v-if="snapshot">{{ snapshot.state.projects.length }}</small>
      </button>
    </nav>
    <button class="settings-button" :class="{ active: activeSection === 'settings' }" @click="$emit('update:activeSection', 'settings')">
      <Settings :size="18" />
      <span>Settings</span>
    </button>
  </aside>
</template>
```

- [ ] **Step 3: Add Skills view**

Create `src/components/SkillsView.vue`:

```vue
<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderPlus, RefreshCw, Trash2 } from "lucide-vue-next";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import type { AppSnapshot } from "../types";

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedSkillId: string | null;
}>();

const emit = defineEmits<{
  "select-skill": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const query = ref("");
const busy = ref(false);

const skills = computed(() => {
  const normalized = query.value.trim().toLowerCase();
  if (!normalized) return props.snapshot.state.skills;
  return props.snapshot.state.skills.filter((skill) =>
    `${skill.name} ${skill.description} ${skill.id}`.toLowerCase().includes(normalized),
  );
});

const selectedSkill = computed(() =>
  props.snapshot.state.skills.find((skill) => skill.id === props.selectedSkillId) ?? props.snapshot.state.skills[0] ?? null,
);

async function run(action: () => Promise<AppSnapshot>) {
  busy.value = true;
  try {
    emit("snapshot", await action());
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

async function importSkill() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    await run(() => api.importSkill(selected));
  }
}
</script>

<template>
  <div class="split-content">
    <section class="list-panel">
      <div class="toolbar">
        <input v-model="query" class="search-input" placeholder="搜索 skills" />
        <button class="icon-button" title="导入 skill" :disabled="busy" @click="importSkill">
          <FolderPlus :size="18" />
        </button>
        <button class="icon-button" title="同步 Codex" :disabled="busy" @click="run(api.syncCodex)">
          <RefreshCw :size="18" />
        </button>
      </div>

      <button
        v-for="skill in skills"
        :key="skill.id"
        class="row-item"
        :class="{ active: selectedSkill?.id === skill.id }"
        @click="emit('select-skill', skill.id)"
      >
        <span>
          <strong>{{ skill.name }}</strong>
          <small>{{ skill.description || skill.id }}</small>
        </span>
        <span class="status-dot" :class="{ on: skill.defaultEnabled, conflict: skill.conflict }"></span>
      </button>

      <div v-if="!skills.length" class="content-empty">技能库里还没有 skill。</div>
    </section>

    <section class="detail-panel">
      <template v-if="selectedSkill">
        <div class="detail-header">
          <div>
            <h2>{{ selectedSkill.name }}</h2>
            <p>{{ selectedSkill.description || "没有描述" }}</p>
          </div>
          <label class="switch-row">
            <span>默认启用</span>
            <input
              type="checkbox"
              :checked="selectedSkill.defaultEnabled"
              :disabled="busy"
              @change="run(() => api.setDefaultEnabled(selectedSkill!.id, ($event.target as HTMLInputElement).checked))"
            />
          </label>
        </div>

        <dl class="meta-list">
          <dt>技能库路径</dt>
          <dd>{{ selectedSkill.libraryPath }}</dd>
          <dt>Codex</dt>
          <dd>{{ selectedSkill.managedLinks.codex || "未同步" }}</dd>
          <dt v-if="selectedSkill.conflict">冲突</dt>
          <dd v-if="selectedSkill.conflict" class="danger">{{ selectedSkill.conflict.message }}：{{ selectedSkill.conflict.path }}</dd>
        </dl>

        <button class="danger-button" :disabled="busy" @click="run(() => api.deleteSkill(selectedSkill!.id))">
          <Trash2 :size="16" />
          删除 skill
        </button>
      </template>
      <div v-else class="content-empty">选择或导入一个 skill。</div>
    </section>
  </div>
</template>
```

- [ ] **Step 4: Add Projects view**

Create `src/components/ProjectsView.vue`:

```vue
<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderPlus } from "lucide-vue-next";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import { ruleLabel, type AppSnapshot, type ProjectRule } from "../types";

const props = defineProps<{
  snapshot: AppSnapshot;
  selectedProjectId: string | null;
}>();

const emit = defineEmits<{
  "select-project": [value: string | null];
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const busy = ref(false);

const selectedProject = computed(() =>
  props.snapshot.state.projects.find((project) => project.id === props.selectedProjectId) ?? props.snapshot.state.projects[0] ?? null,
);

async function run(action: () => Promise<AppSnapshot>) {
  busy.value = true;
  try {
    const next = await action();
    emit("snapshot", next);
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

async function addProject() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    const name = selected.split(/[\\/]/).filter(Boolean).at(-1) ?? selected;
    await run(() => api.addProject({ name, path: selected }));
  }
}

function setRule(skillId: string, rule: ProjectRule) {
  if (!selectedProject.value) return;
  return run(() => api.setProjectRule({ projectId: selectedProject.value!.id, skillId, rule }));
}
</script>

<template>
  <div class="split-content">
    <section class="list-panel">
      <div class="toolbar">
        <button class="primary-button" :disabled="busy" @click="addProject">
          <FolderPlus :size="16" />
          添加项目
        </button>
      </div>

      <button
        v-for="project in snapshot.state.projects"
        :key="project.id"
        class="row-item"
        :class="{ active: selectedProject?.id === project.id }"
        @click="emit('select-project', project.id)"
      >
        <span>
          <strong>{{ project.name }}</strong>
          <small>{{ project.path }}</small>
        </span>
      </button>

      <div v-if="!snapshot.state.projects.length" class="content-empty">还没有项目。</div>
    </section>

    <section class="detail-panel">
      <template v-if="selectedProject">
        <div class="detail-header">
          <div>
            <h2>{{ selectedProject.name }}</h2>
            <p>{{ selectedProject.path }}</p>
          </div>
          <button class="primary-button" :disabled="busy" @click="run(() => api.setCurrentProject(selectedProject!.id))">
            设为当前项目
          </button>
        </div>

        <div class="rule-list">
          <div v-for="skill in snapshot.state.skills" :key="skill.id" class="rule-row">
            <div>
              <strong>{{ skill.name }}</strong>
              <small>{{ ruleLabel(selectedProject.rules[skill.id]) }}</small>
            </div>
            <select
              :value="selectedProject.rules[skill.id] ?? 'inherit'"
              :disabled="busy"
              @change="setRule(skill.id, ($event.target as HTMLSelectElement).value as ProjectRule)"
            >
              <option value="inherit">跟随默认</option>
              <option value="enable">在此项目启用</option>
              <option value="disable">在此项目停用</option>
            </select>
          </div>
        </div>
      </template>
      <div v-else class="content-empty">添加或选择一个项目。</div>
    </section>
  </div>
</template>
```

- [ ] **Step 5: Add Settings view**

Create `src/components/SettingsView.vue`:

```vue
<script setup lang="ts">
import { ref } from "vue";
import { FolderOpen, RefreshCw } from "lucide-vue-next";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import type { AppSnapshot } from "../types";

defineProps<{ snapshot: AppSnapshot }>();

const emit = defineEmits<{
  snapshot: [value: AppSnapshot];
  error: [value: string];
}>();

const busy = ref(false);

async function run(action: () => Promise<AppSnapshot>) {
  busy.value = true;
  try {
    emit("snapshot", await action());
  } catch (cause) {
    emit("error", String(cause));
  } finally {
    busy.value = false;
  }
}

async function chooseCodexPath() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    await run(() => api.setCodexPath(selected));
  }
}
</script>

<template>
  <div class="settings-grid">
    <section class="settings-section">
      <h2>技能库</h2>
      <dl class="meta-list">
        <dt>位置</dt>
        <dd>{{ snapshot.state.skillLibraryPath }}</dd>
      </dl>
      <p class="muted">迁移技能库会在后续任务中接入，第一版先展示当前位置。</p>
    </section>

    <section class="settings-section">
      <h2>Codex 连接</h2>
      <dl class="meta-list">
        <dt>状态</dt>
        <dd>{{ snapshot.codexConnected ? "已连接" : "未连接" }}</dd>
        <dt>目录</dt>
        <dd>{{ snapshot.state.codexSkillsPath || "未设置" }}</dd>
      </dl>
      <div class="button-row">
        <button class="primary-button" :disabled="busy" @click="chooseCodexPath">
          <FolderOpen :size="16" />
          选择目录
        </button>
        <button class="primary-button" :disabled="busy" @click="run(api.syncCodex)">
          <RefreshCw :size="16" />
          同步 Codex
        </button>
      </div>
    </section>

    <section class="settings-section">
      <h2>诊断</h2>
      <ul class="diagnostics">
        <li v-for="item in snapshot.diagnostics" :key="item">{{ item }}</li>
        <li v-if="!snapshot.diagnostics.length">没有需要处理的问题。</li>
      </ul>
    </section>
  </div>
</template>
```

- [ ] **Step 6: Add styles**

Create `src/styles.css`:

```css
:root {
  font-family: Inter, "Segoe UI", system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
  color: #202426;
  background: #f6f7f7;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  min-width: 960px;
  min-height: 100vh;
}

button,
input,
select {
  font: inherit;
}

.app-shell {
  display: grid;
  grid-template-columns: 260px 1fr;
  min-height: 100vh;
  background: #f8faf9;
}

.sidebar {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 22px 14px;
  background: #eaf5f7;
  border-right: 1px solid #dbe5e8;
}

.brand {
  padding: 0 12px;
  font-weight: 700;
  font-size: 18px;
}

.nav-list {
  display: grid;
  gap: 6px;
}

.nav-list button,
.settings-button {
  display: grid;
  grid-template-columns: 22px 1fr auto;
  align-items: center;
  gap: 10px;
  width: 100%;
  min-height: 38px;
  padding: 8px 12px;
  border: 0;
  border-radius: 8px;
  color: #536166;
  background: transparent;
  text-align: left;
  cursor: pointer;
}

.nav-list button.active,
.settings-button.active {
  color: #1e2a2e;
  background: rgba(255, 255, 255, 0.72);
}

.settings-button {
  margin-top: auto;
}

.main-pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: #ffffff;
}

.topbar {
  display: flex;
  align-items: center;
  min-height: 72px;
  padding: 18px 28px;
  border-bottom: 1px solid #eceff0;
}

.topbar h1,
.detail-header h2,
.settings-section h2 {
  margin: 0;
  font-size: 20px;
  line-height: 1.25;
}

.topbar-note,
.muted,
.detail-header p,
.row-item small,
.rule-row small {
  margin: 4px 0 0;
  color: #7a8589;
  font-size: 13px;
}

.notice {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 16px 28px 0;
  padding: 10px 12px;
  border-radius: 8px;
  background: #fff7e6;
  color: #7a4a00;
}

.notice.error {
  background: #fff1f0;
  color: #a4262c;
}

.split-content {
  display: grid;
  grid-template-columns: minmax(320px, 40%) 1fr;
  min-height: 0;
  flex: 1;
}

.list-panel {
  padding: 20px;
  border-right: 1px solid #eceff0;
  overflow: auto;
}

.detail-panel {
  padding: 24px 28px;
  overflow: auto;
}

.toolbar,
.button-row {
  display: flex;
  gap: 8px;
  margin-bottom: 14px;
}

.search-input {
  width: 100%;
  height: 38px;
  padding: 0 12px;
  border: 1px solid #d7dde0;
  border-radius: 8px;
  outline: none;
}

.search-input:focus {
  border-color: #6aa6b8;
}

.icon-button,
.primary-button,
.danger-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 38px;
  padding: 0 12px;
  border: 1px solid #d7dde0;
  border-radius: 8px;
  background: #ffffff;
  color: #243033;
  cursor: pointer;
}

.primary-button {
  background: #1f6f82;
  border-color: #1f6f82;
  color: #ffffff;
}

.danger-button {
  margin-top: 22px;
  color: #9d1c24;
}

button:disabled {
  opacity: 0.55;
  cursor: default;
}

.row-item {
  display: grid;
  grid-template-columns: 1fr auto;
  align-items: center;
  gap: 12px;
  width: 100%;
  min-height: 58px;
  padding: 10px 12px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  text-align: left;
  cursor: pointer;
}

.row-item.active {
  background: #edf6f7;
}

.row-item strong,
.rule-row strong {
  display: block;
  font-size: 14px;
}

.status-dot {
  width: 9px;
  height: 9px;
  border-radius: 999px;
  background: #b8c1c4;
}

.status-dot.on {
  background: #238a54;
}

.status-dot.conflict {
  background: #d83b01;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  margin-bottom: 24px;
}

.switch-row {
  display: inline-flex;
  align-items: center;
  gap: 10px;
}

.meta-list {
  display: grid;
  grid-template-columns: 112px minmax(0, 1fr);
  gap: 10px 14px;
  margin: 0;
}

.meta-list dt {
  color: #7a8589;
}

.meta-list dd {
  margin: 0;
  overflow-wrap: anywhere;
}

.danger {
  color: #a4262c;
}

.rule-list {
  display: grid;
  gap: 10px;
}

.rule-row {
  display: grid;
  grid-template-columns: 1fr 180px;
  align-items: center;
  gap: 16px;
  min-height: 54px;
  padding: 10px 0;
  border-bottom: 1px solid #edf0f1;
}

.rule-row select {
  height: 36px;
  border: 1px solid #d7dde0;
  border-radius: 8px;
  background: #fff;
}

.settings-grid {
  display: grid;
  gap: 18px;
  padding: 24px 28px;
}

.settings-section {
  padding-bottom: 18px;
  border-bottom: 1px solid #edf0f1;
}

.diagnostics {
  margin: 12px 0 0;
  padding-left: 20px;
  color: #536166;
}

.content-empty {
  padding: 32px;
  color: #7a8589;
  text-align: center;
}
```

- [ ] **Step 7: Build frontend**

Run:

```powershell
npm run build
```

Expected: Vue typecheck and Vite build pass.

- [ ] **Step 8: Commit**

Run:

```powershell
git add src/App.vue src/components src/styles.css
git commit -m "feat: add SkillMaster desktop UI"
```

Expected: commit succeeds.

## Task 9: Add Skill Library Migration Command

**Files:**
- Modify: `src-tauri/src/skill_library.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src/api.ts`
- Modify: `src/components/SettingsView.vue`
- Test: Rust unit tests inside `skill_library.rs`

- [ ] **Step 1: Add failing migration test**

Append this test in `skill_library.rs`:

```rust
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
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
Set-Location src-tauri
cargo test migrates_skill_library_and_updates_skill_paths
Set-Location ..
```

Expected: FAIL because `migrate_skill_library` is undefined.

- [ ] **Step 3: Implement migration**

Add to `skill_library.rs`:

```rust
pub fn migrate_skill_library(state: &mut crate::models::AppState, target_root: &Path) -> Result<()> {
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
    Ok(())
}
```

- [ ] **Step 4: Add Tauri command**

In `commands.rs`, import migration:

```rust
use crate::skill_library::{
    delete_skill as delete_skill_from_library,
    import_skill as import_skill_into_library,
    migrate_skill_library,
};
```

Add command:

```rust
#[tauri::command]
pub fn migrate_library(
    app: AppHandle,
    target: PathBuf,
) -> std::result::Result<AppSnapshot, String> {
    let (paths, mut state) = load_command_state(&app).map_err(|error| error.to_string())?;
    migrate_skill_library(&mut state, &target).map_err(|error| error.to_string())?;
    persist(&paths, &state).map_err(|error| error.to_string())?;
    Ok(build_snapshot(state))
}
```

Register it in `lib.rs` inside `generate_handler!`:

```rust
commands::migrate_library
```

- [ ] **Step 5: Add frontend API and Settings action**

Add to `src/api.ts`:

```ts
export function migrateLibrary(target: string): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("migrate_library", { target });
}
```

In `SettingsView.vue`, add a button next to the skill library section:

```vue
<button class="primary-button" :disabled="busy" @click="chooseLibraryTarget">
  <FolderOpen :size="16" />
  迁移技能库
</button>
```

Add this function in the `<script setup>`:

```ts
async function chooseLibraryTarget() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    await run(() => api.migrateLibrary(selected));
  }
}
```

- [ ] **Step 6: Run verification**

Run:

```powershell
Set-Location src-tauri
cargo test migrates_skill_library_and_updates_skill_paths
Set-Location ..
npm run build
```

Expected: Rust migration test passes and Vue build succeeds.

- [ ] **Step 7: Commit**

Run:

```powershell
git add src-tauri/src/skill_library.rs src-tauri/src/commands.rs src-tauri/src/lib.rs src/api.ts src/components/SettingsView.vue
git commit -m "feat: migrate skill library"
```

Expected: commit succeeds.

## Task 10: End-to-End Verification and Packaging

**Files:**
- Create: `docs/verification/2026-05-28-manual-qa.md`
- Modify: `README.md`

- [ ] **Step 1: Add README**

Create `README.md`:

````markdown
# SkillMaster

SkillMaster is a local desktop app for managing agent skills.

## First Version

- Manage one local skill library.
- Import folders that contain `SKILL.md`.
- Set skills as default enabled or disabled.
- Add project folders and set project rules.
- Sync enabled skills to Codex through SkillMaster-managed links.

## Development

```powershell
npm install
npm run build
npm test
Set-Location src-tauri
cargo test
Set-Location ..
npm run tauri dev
```

## Packaging

```powershell
npm run tauri build
```
````

- [ ] **Step 2: Add manual QA checklist**

Create `docs/verification/2026-05-28-manual-qa.md`:

```markdown
# SkillMaster Manual QA

Date: 2026-05-28

## Setup

- Start the app with `npm run tauri dev`.
- Use a temporary Codex skills directory for QA.
- Prepare one valid skill folder with `SKILL.md`.
- Prepare one invalid folder without `SKILL.md`.

## Checks

- App opens to Skills.
- Import rejects the invalid folder.
- Import accepts the valid skill folder.
- Default enable can be toggled.
- Settings accepts a Codex skills directory.
- Sync creates a managed link for an enabled skill.
- Disabling the skill removes the managed link.
- Existing non-managed same-name Codex folder produces a visible conflict.
- Projects can add a folder.
- Project rule labels are exactly: 跟随默认, 在此项目启用, 在此项目停用.
- Setting a project as current context changes sync according to project rules.
- Skill library migration updates skill paths and keeps the old library unused.
```

- [ ] **Step 3: Run automated checks**

Run:

```powershell
npm test
npm run build
Set-Location src-tauri
cargo test
Set-Location ..
```

Expected: all frontend tests, frontend build, and Rust tests pass.

- [ ] **Step 4: Run desktop app locally**

Run:

```powershell
npm run tauri dev
```

Expected: a SkillMaster window opens with left sidebar entries `Skills`, `Projects`, and `Settings`.

- [ ] **Step 5: Run package build**

Run:

```powershell
npm run tauri build
```

Expected: Tauri produces a Windows installer on Windows. On macOS, Tauri produces the macOS bundle artifacts. If signing is not configured, record the unsigned artifact result in the QA document.

- [ ] **Step 6: Commit**

Run:

```powershell
git add README.md docs/verification/2026-05-28-manual-qa.md
git commit -m "docs: add verification guide"
```

Expected: commit succeeds.

- [ ] **Step 7: Push**

Run:

```powershell
git push
```

Expected: local commits are pushed to `origin/main`.

## Self-Review

- Spec coverage: tasks cover Tauri + Vue scaffold, local JSON state, skill library import/delete, default enable state, project rules, Codex path configuration, managed link sync, conflict detection, Settings-based Codex details, migration, verification, and packaging.
- Placeholder scan: the plan contains no unresolved markers or open-ended file names.
- Type consistency: Rust uses `AppState`, `Skill`, `ManagedLinks`, `Project`, and `ProjectRule`; TypeScript mirrors those DTOs with camelCase fields; command names match `src/api.ts`.
- Scope control: Claude Code, CodeBuddy, remote marketplace, skill versioning, dependency resolution, cloud backup, and multi-device sync stay outside this plan.

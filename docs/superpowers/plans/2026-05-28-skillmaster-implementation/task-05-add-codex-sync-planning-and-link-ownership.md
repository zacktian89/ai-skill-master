# Task 5: Add Codex Sync Planning and Link Ownership

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

---

Parent plan: [2026-05-28-skillmaster-implementation.md](../2026-05-28-skillmaster-implementation.md)

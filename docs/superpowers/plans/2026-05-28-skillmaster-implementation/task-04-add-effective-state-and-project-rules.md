# Task 4: Add Effective State and Project Rules

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

---

Parent plan: [2026-05-28-skillmaster-implementation.md](../2026-05-28-skillmaster-implementation.md)

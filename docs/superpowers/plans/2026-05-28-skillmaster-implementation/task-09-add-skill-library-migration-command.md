# Task 9: Add Skill Library Migration Command

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

---

Parent plan: [2026-05-28-skillmaster-implementation.md](../2026-05-28-skillmaster-implementation.md)

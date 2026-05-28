# Task 10: End-to-End Verification and Packaging

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

---

Parent plan: [2026-05-28-skillmaster-implementation.md](../2026-05-28-skillmaster-implementation.md)

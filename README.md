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

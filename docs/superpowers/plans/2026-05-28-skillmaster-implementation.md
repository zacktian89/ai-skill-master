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

## 子文档索引

这些子文档按执行顺序拆分，每个文件保留原计划中的 Files、Step、验证命令和提交要求，可直接分派给后续任务执行者。

| 顺序 | 子任务 | 子文档 |
| --- | --- | --- |
| 1 | Task 1: Bootstrap Tauri + Vue | [task-01-bootstrap-tauri-vue.md](2026-05-28-skillmaster-implementation/task-01-bootstrap-tauri-vue.md) |
| 2 | Task 2: Define Rust Models and State Store | [task-02-define-rust-models-and-state-store.md](2026-05-28-skillmaster-implementation/task-02-define-rust-models-and-state-store.md) |
| 3 | Task 3: Add App Paths and Skill Library Operations | [task-03-add-app-paths-and-skill-library-operations.md](2026-05-28-skillmaster-implementation/task-03-add-app-paths-and-skill-library-operations.md) |
| 4 | Task 4: Add Effective State and Project Rules | [task-04-add-effective-state-and-project-rules.md](2026-05-28-skillmaster-implementation/task-04-add-effective-state-and-project-rules.md) |
| 5 | Task 5: Add Codex Sync Planning and Link Ownership | [task-05-add-codex-sync-planning-and-link-ownership.md](2026-05-28-skillmaster-implementation/task-05-add-codex-sync-planning-and-link-ownership.md) |
| 6 | Task 6: Add Tauri Command API | [task-06-add-tauri-command-api.md](2026-05-28-skillmaster-implementation/task-06-add-tauri-command-api.md) |
| 7 | Task 7: Add Frontend Types, API Wrapper, and Store State | [task-07-add-frontend-types-api-wrapper-and-store-state.md](2026-05-28-skillmaster-implementation/task-07-add-frontend-types-api-wrapper-and-store-state.md) |
| 8 | Task 8: Build Two-Column Vue UI | [task-08-build-two-column-vue-ui.md](2026-05-28-skillmaster-implementation/task-08-build-two-column-vue-ui.md) |
| 9 | Task 9: Add Skill Library Migration Command | [task-09-add-skill-library-migration-command.md](2026-05-28-skillmaster-implementation/task-09-add-skill-library-migration-command.md) |
| 10 | Task 10: End-to-End Verification and Packaging | [task-10-end-to-end-verification-and-packaging.md](2026-05-28-skillmaster-implementation/task-10-end-to-end-verification-and-packaging.md) |

## 执行建议

- 按 Task 1 到 Task 10 顺序执行，除非前置产物已经存在并通过对应验证。
- 每个子任务完成后运行该子文档中的验证命令，再提交对应 commit。
- 如果实现时发现设计文档矛盾，只按原计划约定修改 `docs/superpowers/specs/2026-05-28-skillmaster-design.md`。

## Self-Review

- Spec coverage: tasks cover Tauri + Vue scaffold, local JSON state, skill library import/delete, default enable state, project rules, Codex path configuration, managed link sync, conflict detection, Settings-based Codex details, migration, verification, and packaging.
- Placeholder scan: the plan contains no unresolved markers or open-ended file names.
- Type consistency: Rust uses `AppState`, `Skill`, `ManagedLinks`, `Project`, and `ProjectRule`; TypeScript mirrors those DTOs with camelCase fields; command names match `src/api.ts`.
- Scope control: Claude Code, CodeBuddy, remote marketplace, skill versioning, dependency resolution, cloud backup, and multi-device sync stay outside this plan.

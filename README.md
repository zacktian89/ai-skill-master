# SkillMaster

[中文](README.zh.md)

SkillMaster is a local desktop app for the everyday mess around agent skills: scattered folders, duplicated copies, inconsistent project rules, and manual sync work. It keeps `SKILL.md` folders from Codex, Claude Code, project directories, and GitHub repositories in one local skill library, then distributes the same source skill through SkillMaster-managed references.

When you switch between projects, try different coding agents, download community skills, or need the same rules in multiple directories, SkillMaster handles import, preview, linking, disabling, and cleanup so you do not have to copy folders by hand, chase stale versions, or risk deleting the original skill.

![Skills view](docs/screenshots/skills.png)

## Features

- Central library: import skills from local folders and GitHub repositories into one library, then preview `SKILL.md` and README content before using them.
- Reuse one source: link the same skill into user, project, custom, or agent-specific skill directories through SkillMaster-managed references.
- Project rules: set skills to enabled, disabled, or inherited per project to keep one project's rules from leaking into another.
- Agent coverage: maintain skills roots for Codex, Claude Code, and other agents so different tools can share the same skill assets.
- Plugin insight: inspect installed plugins, included skills, MCP services, and supported agents to understand what each plugin actually adds.
- Store discovery: browse skills.sh leaderboards and start the download/import flow from a store entry instead of searching and copying manually.
- Safer cleanup: review impact before deletion; removing a reference clears only the managed link and keeps the original skill in the library.
- Local settings: switch theme and language, inspect storage paths, and migrate the skill library when needed.

## Screenshots

### Project Rules

![Projects view](docs/screenshots/projects.png)

### Skill Store

![Store view](docs/screenshots/store.png)

### Plugin Details

![Plugins view](docs/screenshots/plugins.png)

### Settings

![Settings view](docs/screenshots/settings.png)

## Installation

### 1. Download an installer

Open the [SkillMaster releases page](https://github.com/zacktian89/ai-skill-master/releases) and download the latest installer for your system.

Windows installers and macOS packages are attached to each release.

The macOS build is not signed or notarized. On first launch, open it with Control-click or right-click, choose Open, and confirm the macOS security prompt.

### 2. Install and launch

Run the downloaded installer, follow the system prompts, then launch SkillMaster from the Start menu or desktop shortcut.

### 3. Import a skill

1. Open the Skills page.
2. Click the add button.
3. Select a local folder that contains `SKILL.md`, or enter a GitHub repository URL.
4. Confirm the skills to import in the preview list.
5. After import, use the detail panel to inspect documentation and reference status.

### 4. Link a skill to an agent or project

1. Add an agent skills root on the Agents page, or add a project directory on the Projects page.
2. Select the skill you want to link.
3. Use Add Skill or Add Reference to link it into the target directory.
4. When a reference is removed later, SkillMaster removes only the managed reference and keeps the original skill in the library.

### 5. Configure project rules

1. Open the Projects page.
2. Select a project.
3. Set each skill to enabled, disabled, or inherited.
4. The active project context affects later sync calculations.

## Run From Source

Use the source workflow when contributing or debugging locally.

### 1. Install dependencies

```powershell
npm install
```

### 2. Start the development environment

```powershell
npm run dev       # Start the Vite browser preview only
npm run build     # Type-check and build the frontend
npm test          # Run frontend tests
npm run tauri dev # Start the Tauri desktop dev environment
```

Rust-side tests:

```powershell
Set-Location src-tauri
cargo test
Set-Location ..
```

Build installers:

```powershell
npm run tauri build
```

## Tech Stack

- Vue 3 + TypeScript
- Vite
- Tauri 2
- Rust
- Vitest

## Project Structure

```text
src/                 Vue frontend source
src/api/             Frontend API wrappers and browser mock data
src/components/      Shared UI components
src/views/           Skills, Store, Agents, Plugins, Projects, and Settings pages
src-tauri/           Tauri and Rust backend commands
docs/                Design docs, verification notes, and README screenshots
public/              Icons and static assets
```

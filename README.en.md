# SkillMaster

[中文](README.md)

SkillMaster is a local desktop app for managing, importing, previewing, and distributing agent skills. It keeps `SKILL.md` folders in one local skill library, then syncs them into Codex, Claude Code, and other target directories through SkillMaster-managed references.

![Skills view](docs/screenshots/skills.png)

## Features

- Skill library management: import skills from local folders or GitHub repositories, preview `SKILL.md` and README content, and review delete impact before removing a skill.
- Skill references: add or remove managed references for a skill in user, project, or custom directories.
- Project management: add local projects, scan existing project skills, and set per-project enable, disable, or inherit rules.
- Agent management: register skill root directories for different agents and link skills into those agent directories.
- Plugin inspection: view installed plugins, included skills, MCP services, and supported agents.
- Store browsing: search or browse skills.sh leaderboards and start the download/import flow from a store entry.
- Settings: switch dark/light theme, switch UI language, inspect storage paths, and migrate the local skill library.

## Screenshots

### Project Rules

![Projects view](docs/screenshots/projects.png)

### Agent Directories

![Agents view](docs/screenshots/agents.png)

### Plugin Details

![Plugins view](docs/screenshots/plugins.png)

### Settings

![Settings view](docs/screenshots/settings.png)

## Installation

### 1. Download an installer

Open the [SkillMaster 1.0.0 release page](https://github.com/zacktian89/ai-skill-master/releases/tag/v1.0.0) and download the installer for your system.

Windows users can download:

- [SkillMaster_1.0.0_x64-setup.exe](https://github.com/zacktian89/ai-skill-master/raw/main/releases/v1.0.0/SkillMaster_1.0.0_x64-setup.exe)
- [SkillMaster_1.0.0_x64_en-US.msi](https://github.com/zacktian89/ai-skill-master/raw/main/releases/v1.0.0/SkillMaster_1.0.0_x64_en-US.msi)

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

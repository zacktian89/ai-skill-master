# Codex Dense Theme Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Codex Dense black/white themes, persist the selected theme locally, and remove redundant UI labels from obvious regions.

**Architecture:** Keep theme state in the Vue app shell and persist it with `localStorage`. Apply visual changes through CSS variables in `src/styles.css`, with `data-theme="dark"` and `data-theme="light"` on `.app-shell`. Add the theme selector to `SettingsView` without touching Rust state or Tauri commands.

**Tech Stack:** Vue 3, TypeScript, Vitest, Vue Test Utils, Vite CSS.

---

## File Structure

- Modify `src/App.vue`: own `ThemeMode`, read/write `localStorage`, pass theme props/events to Settings, set `data-theme`.
- Modify `src/components/SettingsView.vue`: add an appearance group with black/white segmented controls.
- Modify `src/components/SkillsView.vue`: remove redundant list/detail headings where context is obvious.
- Modify `src/components/ProjectsView.vue`: remove redundant list/detail headings where context is obvious.
- Modify `src/styles.css`: replace warm theme tokens with Codex Dense dark/light variables and tighten common spacing, radius, and typography.
- Modify `src/__tests__/appShell.test.ts`: cover theme initialization, switching, persistence, and absence of redundant labels.

---

### Task 1: Add Theme Behavior Test

**Files:**
- Modify: `src/__tests__/appShell.test.ts`

- [ ] **Step 1: Write failing tests**

Add tests that mount `App`, verify default `data-theme="dark"`, switch to Settings, click `白色`, and assert `localStorage.skillmaster-theme` plus root `data-theme`.

- [ ] **Step 2: Run test to verify it fails**

Run: `npx vitest run src/__tests__/appShell.test.ts`

Expected: fails because `data-theme` and theme controls do not exist yet.

- [ ] **Step 3: Implement minimal App and Settings theme state**

Add `ThemeMode = "dark" | "light"` in `App.vue`, pass `theme-mode` and `update:theme-mode` to `SettingsView`, and add the appearance group in `SettingsView`.

- [ ] **Step 4: Run test to verify it passes**

Run: `npx vitest run src/__tests__/appShell.test.ts`

Expected: passes.

---

### Task 2: Remove Redundant UI Labels

**Files:**
- Modify: `src/components/SkillsView.vue`
- Modify: `src/components/ProjectsView.vue`
- Modify: `src/components/SettingsView.vue`
- Modify: `src/__tests__/appShell.test.ts`
- Modify: `src/__tests__/projectsView.test.ts`

- [ ] **Step 1: Write failing assertions**

Assert the app shell no longer renders redundant `技能库` and `Skill Detail` labels in the main skills view. Update `ProjectsView` tests so they no longer expect the redundant `技能列表` heading.

- [ ] **Step 2: Run tests to verify failure**

Run: `npx vitest run src/__tests__/appShell.test.ts src/__tests__/projectsView.test.ts`

Expected: fails while old headings remain.

- [ ] **Step 3: Remove redundant markup**

Remove list panel duplicate title blocks from Skills and Projects. Remove low-value eyebrows such as `Skill Detail`, `Project Skills`, and repeated Settings detail labels while preserving modal, diagnostic, destructive action, and safety labels.

- [ ] **Step 4: Run tests to verify pass**

Run: `npx vitest run src/__tests__/appShell.test.ts src/__tests__/projectsView.test.ts`

Expected: passes.

---

### Task 3: Apply Codex Dense CSS

**Files:**
- Modify: `src/styles.css`

- [ ] **Step 1: Update CSS tokens**

Replace warm red/orange variables with neutral dark and light theme variables. Add `[data-theme="light"]` overrides.

- [ ] **Step 2: Tighten shared UI rules**

Reduce large radii, panel padding, button heights, list row spacing, title sizes, and decorative gradients.

- [ ] **Step 3: Run build and tests**

Run: `npm run build`

Run: `npm test`

Expected: both exit 0.

---

### Task 4: Visual Verification

**Files:**
- No code files unless visual issues require CSS adjustment.

- [ ] **Step 1: Open local app**

Use the running Vite app at `http://127.0.0.1:5173/`, or start one with `npm run dev -- --host 127.0.0.1 --port 5173 --strictPort false`.

- [ ] **Step 2: Verify dark theme**

Check Skills, Projects, and Settings for black neutral Codex Dense styling, readable text, compact controls, and removed duplicate headings.

- [ ] **Step 3: Verify light theme**

Switch Settings to white theme, refresh, and verify the theme persists and the same pages remain readable.

- [ ] **Step 4: Final status**

Run `git diff --check`, `npm run build`, and `npm test` before reporting completion.


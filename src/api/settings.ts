import type { AppSnapshot } from "../types";
import { resolveInvoke, autoSync } from "./client";
import * as mockSettings from "./mock/settings";

export async function getSnapshot(): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("get_snapshot") : mockSettings.getSnapshot();
}

export async function setCodexPath(path: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("set_codex_path", { path }) : await mockSettings.setCodexPath(path);
  return autoSync(next);
}

export async function migrateLibrary(target: string): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("migrate_library", { target }) : await mockSettings.migrateLibrary(target);
  return autoSync(next);
}

export async function rebuildState(): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  const next = invoke ? await invoke<AppSnapshot>("rebuild_state") : await mockSettings.rebuildState();
  return autoSync(next);
}

export async function syncCodex(): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke ? invoke<AppSnapshot>("sync_codex") : mockSettings.syncCodex();
}

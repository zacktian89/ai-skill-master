import type { AppSnapshot } from "../types";
import { resolveInvoke } from "./client";
import * as mockSettings from "./mock/settings";

export async function setCodexPluginEnabled(pluginKey: string, enabled: boolean): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke
    ? invoke<AppSnapshot>("set_codex_plugin_enabled", { pluginKey, enabled })
    : mockSettings.getSnapshot();
}

export async function setCodexSkillEnabled(skillName: string, enabled: boolean): Promise<AppSnapshot> {
  const invoke = await resolveInvoke();
  return invoke
    ? invoke<AppSnapshot>("set_codex_skill_enabled", { skillName, enabled })
    : mockSettings.getSnapshot();
}

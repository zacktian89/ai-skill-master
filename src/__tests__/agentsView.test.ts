/**
 * @vitest-environment jsdom
 */
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import AgentsView from "../components/AgentsView.vue";
import type { AppSnapshot } from "../types";

const snapshot: AppSnapshot = {
  state: {
    schemaVersion: 1,
    skillLibraryPath: "/library",
    codexSkillsPath: "/codex/skills",
    currentProjectId: null,
    syncStatus: {
      phase: "healthy",
      message: null,
      pendingActions: [],
    },
    migrationNotice: null,
    skills: [],
    projects: [],
    agents: [],
  },
  targetProfiles: [],
  diagnostics: [],
  paths: {
    stateFile: "/config/skillmaster.json",
    backupFile: "/config/skillmaster.json.bak",
  },
  stateLoad: {
    phase: "clean",
    message: null,
  },
};

describe("AgentsView", () => {
  it("offers Gemini CLI and WorkBuddy as known skill targets", async () => {
    const wrapper = mount(AgentsView, {
      props: {
        snapshot,
        selectedAgentId: null,
      },
    });

    await wrapper.find('button[aria-label="添加 Agent"]').trigger("click");

    expect(wrapper.text()).not.toContain("选择预设主流 Agent");
    expect(wrapper.text()).not.toContain("提示：内置 Agent");
    expect(wrapper.text()).toContain("Gemini CLI");
    expect(wrapper.find(".modal-card--agent").exists()).toBe(true);
    expect(wrapper.find(".modal-step-section--scroll").exists()).toBe(true);
    expect(wrapper.find(".target-grid--agent-presets").exists()).toBe(true);
    await wrapper.findAll(".target-tile").find((button) => button.text() === "Gemini CLI")!.trigger("click");
    expect(wrapper.find('input[placeholder="输入或浏览目录路径（可使用 ~ 开头）"]').element).toMatchObject({
      value: "~/.gemini/config/skills",
    });
    expect(wrapper.text()).toContain("WorkBuddy");
  });
});

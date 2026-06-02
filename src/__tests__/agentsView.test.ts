/**
 * @vitest-environment jsdom
 */
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import AgentsView from "../views/agents/AgentsView.vue";
import type { AppSnapshot } from "../types";
import * as api from "../api";

const apiMocks = vi.hoisted(() => ({
  addAgent: vi.fn(),
  deleteAgent: vi.fn(),
  setAgentRule: vi.fn(),
  scanAgentSkills: vi.fn(),
  addSkillReference: vi.fn(),
  removeSkillReference: vi.fn(),
  importProjectSkill: vi.fn(),
}));

vi.mock("../api", () => ({
  addAgent: apiMocks.addAgent,
  deleteAgent: apiMocks.deleteAgent,
  setAgentRule: apiMocks.setAgentRule,
  scanAgentSkills: apiMocks.scanAgentSkills,
  addSkillReference: apiMocks.addSkillReference,
  removeSkillReference: apiMocks.removeSkillReference,
  importProjectSkill: apiMocks.importProjectSkill,
}));


const snapshot: AppSnapshot = {
  state: {
    schemaVersion: 1,
    skillLibraryPath: "/library",
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
  beforeEach(() => {
    apiMocks.addAgent.mockReset();
    apiMocks.deleteAgent.mockReset();
    apiMocks.deleteAgent.mockResolvedValue(snapshot);
    apiMocks.setAgentRule.mockReset();
    apiMocks.setAgentRule.mockResolvedValue(snapshot);
    apiMocks.scanAgentSkills.mockReset();
    apiMocks.scanAgentSkills.mockResolvedValue([]);
    apiMocks.addSkillReference.mockReset();
    apiMocks.addSkillReference.mockResolvedValue(snapshot);
    apiMocks.removeSkillReference.mockReset();
    apiMocks.removeSkillReference.mockResolvedValue(snapshot);
    apiMocks.importProjectSkill.mockReset();
    apiMocks.importProjectSkill.mockResolvedValue({ type: "success", snapshot });
  });

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

  it("shows add before delete and confirms before deleting an agent", async () => {
    const agentSnapshot: AppSnapshot = {
      ...snapshot,
      state: {
        ...snapshot.state,
        agents: [
          {
            id: "codex",
            name: "Codex",
            path: "~/.agents/skills",
            rules: {},
          },
        ],
      },
    };
    apiMocks.deleteAgent.mockResolvedValue({
      ...agentSnapshot,
      state: {
        ...agentSnapshot.state,
        agents: [],
      },
    });

    const wrapper = mount(AgentsView, {
      props: {
        snapshot: agentSnapshot,
        selectedAgentId: "codex",
      },
    });

    await flushPromises();

    const addButton = wrapper.find('button[aria-label="添加技能"]');
    const deleteButton = wrapper.find('button[aria-label="删除 Agent"]');
    expect(addButton.exists()).toBe(true);
    expect(deleteButton.exists()).toBe(true);
    expect((addButton.element.compareDocumentPosition(deleteButton.element) & Node.DOCUMENT_POSITION_FOLLOWING) > 0).toBe(true);

    await deleteButton.trigger("click");
    expect(wrapper.text()).toContain("确认删除 Agent");
    expect(api.deleteAgent).not.toHaveBeenCalled();

    await wrapper.findAll("button").find((button) => button.text() === "删除 Agent")!.trigger("click");
    await flushPromises();

    expect(api.deleteAgent).toHaveBeenCalledWith("codex");
  });
});

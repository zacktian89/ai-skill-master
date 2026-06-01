/**
 * @vitest-environment jsdom
 */
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import ProjectsView from "../components/ProjectsView.vue";
import type { AppSnapshot } from "../types";
import * as api from "../api";

const apiMocks = vi.hoisted(() => ({
  addProject: vi.fn(),
  setProjectRule: vi.fn(),
}));

const snapshot: AppSnapshot = {
  state: {
    schemaVersion: 1,
    skillLibraryPath: "/library",
    codexSkillsPath: "/codex/skills",
    currentProjectId: "acme",
    syncStatus: {
      phase: "healthy",
      message: null,
      pendingActions: [],
    },
    migrationNotice: null,
    skills: [
      {
        id: "writer-pro",
        name: "Writer Pro",
        description: "长文写作与风格控制",
        libraryPath: "/library/writer-pro",
        references: [],
        managedLinks: {
          codex: "/codex/skills/writer-pro",
        },
        conflict: null,
      },
      {
        id: "reviewer",
        name: "Reviewer",
        description: "审阅内容",
        libraryPath: "/library/reviewer",
        references: [],
        managedLinks: {
          codex: null,
        },
        conflict: null,
      },
      {
        id: "deploy-guard",
        name: "Deploy Guard",
        description: "发布检查",
        libraryPath: "/library/deploy-guard",
        references: [],
        managedLinks: {
          codex: null,
        },
        conflict: null,
      },
    ],
    projects: [
      {
        id: "acme",
        name: "Acme",
        path: "/work/acme",
        rules: {
          "writer-pro": "enable",
          reviewer: "disable",
        },
      },
    ],
  },
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

vi.mock("../api", () => ({
  addProject: apiMocks.addProject,
  setProjectRule: apiMocks.setProjectRule,
}));

vi.mock("../dialog", () => ({
  openDirectory: vi.fn(),
}));

describe("ProjectsView", () => {
  beforeEach(() => {
    apiMocks.addProject.mockReset();
    apiMocks.setProjectRule.mockReset();
    apiMocks.setProjectRule.mockResolvedValue(snapshot);
  });

  it("shows project skills without current-project controls", () => {
    const wrapper = mount(ProjectsView, {
      props: {
        snapshot,
        selectedProjectId: "acme",
      },
    });

    expect(wrapper.text()).not.toContain("技能列表");
    expect(wrapper.text()).toContain("Writer Pro");
    expect(wrapper.text()).toContain("Reviewer");
    expect(wrapper.text()).not.toContain("当前上下文");
    expect(wrapper.text()).not.toContain("回到全局默认");
    expect(wrapper.text()).not.toContain("全部规则");
  });

  it("adds and removes skills through project rules", async () => {
    const wrapper = mount(ProjectsView, {
      props: {
        snapshot,
        selectedProjectId: "acme",
      },
    });

    await wrapper.findAll("button").find((button) => button.text() === "添加技能")!.trigger("click");
    expect(wrapper.text()).toContain("Deploy Guard");

    await wrapper.find(".project-skill-pick-row").trigger("click");
    await flushPromises();

    expect(api.setProjectRule).toHaveBeenCalledWith({
      projectId: "acme",
      skillId: "deploy-guard",
      rule: "enable",
    });

    await wrapper.find('button[aria-label="从项目移除技能"]').trigger("click");
    await flushPromises();

    expect(api.setProjectRule).toHaveBeenCalledWith({
      projectId: "acme",
      skillId: "reviewer",
      rule: "inherit",
    });
  });
});

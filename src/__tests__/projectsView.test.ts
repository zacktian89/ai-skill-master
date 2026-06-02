/**
 * @vitest-environment jsdom
 */
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import ProjectsView from "../views/projects/ProjectsView.vue";
import type { AppSnapshot } from "../types";
import * as api from "../api";

const apiMocks = vi.hoisted(() => ({
  addProject: vi.fn(),
  setProjectRule: vi.fn(),
  scanProjectSkills: vi.fn(),
  addSkillReference: vi.fn(),
  removeSkillReference: vi.fn(),
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
        references: [
          {
            id: "ref-writer-pro",
            targetName: "Codex",
            targetPath: "/work/acme/.agent/skills/writer-pro",
            scope: "project",
            status: "healthy",
          }
        ],
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
    agents: [],
  },
  targetProfiles: [
    {
      id: "profile-1",
      targetName: "Codex",
      rootPath: "/work/acme/.agent/skills",
      scope: "project",
    }
  ],
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
  scanProjectSkills: apiMocks.scanProjectSkills,
  addSkillReference: apiMocks.addSkillReference,
  removeSkillReference: apiMocks.removeSkillReference,
}));

vi.mock("../utils/dialog", () => ({
  openDirectory: vi.fn(),
}));


describe("ProjectsView", () => {
  beforeEach(() => {
    apiMocks.addProject.mockReset();
    apiMocks.setProjectRule.mockReset();
    apiMocks.setProjectRule.mockResolvedValue(snapshot);
    apiMocks.scanProjectSkills.mockReset();
    apiMocks.scanProjectSkills.mockResolvedValue([
      {
        name: ".agent",
        path: "/work/acme/.agent",
        skills: [
          {
            id: "writer-pro",
            name: "Writer Pro",
            description: "长文写作与风格控制",
            path: "/work/acme/.agent/skills/writer-pro",
            isManaged: true,
          },
          {
            id: "reviewer",
            name: "Reviewer",
            description: "审阅内容",
            path: "/work/acme/.agent/skills/reviewer",
            isManaged: true,
          },
        ],
      },
    ]);
    apiMocks.addSkillReference.mockReset();
    apiMocks.addSkillReference.mockResolvedValue(snapshot);
    apiMocks.removeSkillReference.mockReset();
    apiMocks.removeSkillReference.mockResolvedValue(snapshot);
  });

  it("shows project skills without current-project controls", async () => {
    const wrapper = mount(ProjectsView, {
      props: {
        snapshot,
        selectedProjectId: "acme",
      },
    });

    await flushPromises();

    expect(wrapper.text()).not.toContain("技能列表");
    expect(wrapper.text()).toContain("Writer Pro");
    expect(wrapper.text()).toContain("Reviewer");
    expect(wrapper.text()).not.toContain("当前上下文");
    expect(wrapper.text()).not.toContain("回到全局默认");
    expect(wrapper.text()).not.toContain("全部规则");
  });

  it("shows the total project count in the list header", async () => {
    const wrapper = mount(ProjectsView, {
      props: {
        snapshot: {
          ...snapshot,
          state: {
            ...snapshot.state,
            projects: [
              ...snapshot.state.projects,
              {
                id: "beta",
                name: "Beta",
                path: "/work/beta",
                rules: {},
              },
            ],
          },
        },
        selectedProjectId: "acme",
      },
    });

    await flushPromises();

    expect(wrapper.find(".list-panel-head .search-row-count").text()).toBe("2");
  });

  it("toggles rules, adds and removes skills references", async () => {
    const wrapper = mount(ProjectsView, {
      props: {
        snapshot,
        selectedProjectId: "acme",
      },
    });

    await flushPromises();

    // 1. Toggle project rule for writer-pro (enable -> disable)
    const toggleInput = wrapper.find("input[type='checkbox']");
    await toggleInput.setValue(false);
    expect(api.setProjectRule).toHaveBeenCalledWith({
      projectId: "acme",
      skillId: "writer-pro",
      rule: "disable",
    });

    // 2. Open Add Skill dialog
    await wrapper.find('button[aria-label="添加"]').trigger("click");
    
    // Choose Codex target profile
    await wrapper.find(".target-tile").trigger("click");
    expect(wrapper.text()).toContain("Deploy Guard");

    // Select Deploy Guard and confirm
    const checkbox = wrapper.find("input[value='deploy-guard']");
    await checkbox.setValue(true);
    await wrapper.findAll("button").find((button) => button.text() === "确定")!.trigger("click");
    await flushPromises();

    expect(api.addSkillReference).toHaveBeenCalledWith({
      skillId: "deploy-guard",
      targetName: "Codex",
      rootPath: "/work/acme/.agents/skills",
      scope: "project",
      overwrite: true,
    });

    // 3. Remove writer-pro reference
    await wrapper.find('button[aria-label="从项目移除技能引用"]').trigger("click");
    await flushPromises();

    expect(api.removeSkillReference).toHaveBeenCalledWith("ref-writer-pro", true);
  });
});

/**
 * @vitest-environment jsdom
 */
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import ProjectsView from "../views/projects/ProjectsView.vue";
import type { AppSnapshot } from "../types";
import * as api from "../api";
import { clearSkillScannerCaches } from "../composables/useSkillScanner";

const apiMocks = vi.hoisted(() => ({
  addProject: vi.fn(),
  deleteProject: vi.fn(),
  setProjectRule: vi.fn(),
  scanProjectSkills: vi.fn(),
  addSkillReference: vi.fn(),
  removeSkillReference: vi.fn(),
  readSkillFile: vi.fn(),
}));

const snapshot: AppSnapshot = {
  state: {
    schemaVersion: 1,
    skillLibraryPath: "/library",
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
        managedLinks: {},
        conflict: null,
      },
      {
        id: "reviewer",
        name: "Reviewer",
        description: "审阅内容",
        libraryPath: "/library/reviewer",
        references: [],
        managedLinks: {},
        conflict: null,
      },
      {
        id: "deploy-guard",
        name: "Deploy Guard",
        description: "发布检查",
        libraryPath: "/library/deploy-guard",
        references: [],
        managedLinks: {},
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
  deleteProject: apiMocks.deleteProject,
  setProjectRule: apiMocks.setProjectRule,
  scanProjectSkills: apiMocks.scanProjectSkills,
  addSkillReference: apiMocks.addSkillReference,
  removeSkillReference: apiMocks.removeSkillReference,
  readSkillFile: apiMocks.readSkillFile,
}));

vi.mock("../utils/dialog", () => ({
  openDirectory: vi.fn(),
}));


describe("ProjectsView", () => {
  beforeEach(() => {
    clearSkillScannerCaches();
    apiMocks.addProject.mockReset();
    apiMocks.deleteProject.mockReset();
    apiMocks.deleteProject.mockResolvedValue({
      ...snapshot,
      state: {
        ...snapshot.state,
        projects: [],
      },
    });
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
    apiMocks.readSkillFile.mockReset();
    apiMocks.readSkillFile.mockResolvedValue(`---
name: "Writer Pro"
description: "长文写作与风格控制"
license: "MIT"
version: "2.1"
---

# Writer Pro

这是详情内容。`);
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

  it("renders the card description on its own line so long text is not squeezed by the skill id", async () => {
    const longDescription = "A production-grade memory management system using attention-weighted architecture with semantic routing and dependency modeling.";
    apiMocks.scanProjectSkills.mockResolvedValue([
      {
        name: ".agent",
        path: "/work/acme/.agent",
        skills: [
          {
            id: "attention-memory",
            name: "attention-memory",
            description: longDescription,
            path: "/work/acme/.agent/skills/attention-memory",
            isManaged: true,
          },
        ],
      },
    ]);

    const wrapper = mount(ProjectsView, {
      props: {
        snapshot: {
          ...snapshot,
          state: {
            ...snapshot.state,
            skills: [
              {
                id: "attention-memory",
                name: "attention-memory",
                description: longDescription,
                libraryPath: "/library/attention-memory",
                references: [],
                managedLinks: {},
                conflict: null,
              },
            ],
          },
        },
        selectedProjectId: "acme",
      },
    });

    await flushPromises();

    const description = wrapper.find(".project-skill-description");
    expect(description.exists()).toBe(true);
    expect(description.text()).toBe(longDescription);
  });

  it("opens a skill preview in the right panel and closes it with the back button", async () => {
    const wrapper = mount(ProjectsView, {
      props: {
        snapshot,
        selectedProjectId: "acme",
      },
    });

    await flushPromises();

    expect(wrapper.text()).toContain("Acme");
    expect(wrapper.text()).not.toContain("返回");

    const detailPanel = wrapper.find(".detail-panel").element as HTMLElement;
    detailPanel.scrollTop = 180;

    await wrapper.findAll(".project-skill-row")[0]!.trigger("click");
    await flushPromises();

    expect(wrapper.text()).toContain("Writer Pro");
    expect(wrapper.text()).toContain("name");
    expect(wrapper.text()).toContain("license");
    expect(wrapper.text()).toContain("这是详情内容。");
    expect(wrapper.find('button[aria-label="返回技能列表"]').exists()).toBe(true);
    expect(wrapper.find('button[aria-label="更多技能操作"]').exists()).toBe(true);

    await wrapper.find('button[aria-label="返回技能列表"]').trigger("click");
    await flushPromises();

    expect(wrapper.text()).toContain("Acme");
    expect(wrapper.find('button[aria-label="返回技能列表"]').exists()).toBe(false);
    expect(detailPanel.scrollTop).toBe(180);
  });

  it("does not show the total project count in the list header", async () => {
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

    expect(wrapper.find(".list-panel-head .search-row-count").exists()).toBe(false);
  });

  it("uses cached project scans until the rescan button is clicked", async () => {
    const wrapper = mount(ProjectsView, {
      props: {
        snapshot,
        selectedProjectId: "acme",
      },
    });

    await flushPromises();

    expect(api.scanProjectSkills).toHaveBeenCalledTimes(1);

    await wrapper.setProps({
      snapshot: {
        ...snapshot,
        state: {
          ...snapshot.state,
          skills: [
            ...snapshot.state.skills,
            {
              id: "style-guide",
              name: "Style Guide",
              description: "写作规范",
              libraryPath: "/library/style-guide",
              references: [],
              managedLinks: {},
              conflict: null,
            },
          ],
        },
      },
    });
    await flushPromises();

    expect(api.scanProjectSkills).toHaveBeenCalledTimes(1);

    await wrapper.find('button[aria-label="重新扫描"]').trigger("click");
    await flushPromises();

    expect(api.scanProjectSkills).toHaveBeenCalledTimes(2);
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
    await wrapper.find('button[aria-label="更多技能操作"]').trigger("click", { clientX: 300, clientY: 80 });
    await flushPromises();
    expect((document.body.querySelector(".global-context-menu") as HTMLElement).style.left).toBe("152px");
    const closeItem = Array.from(document.body.querySelectorAll(".global-context-menu-item")).find(
      (item) => item.textContent?.includes("关闭")
    ) as HTMLButtonElement;
    closeItem.click();
    await flushPromises();

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
    await wrapper.find('button[aria-label="更多技能操作"]').trigger("click", { clientX: 300, clientY: 80 });
    const deleteReferenceItem = Array.from(document.body.querySelectorAll(".global-context-menu-item")).find(
      (item) => item.textContent?.includes("删除引用")
    ) as HTMLButtonElement;
    deleteReferenceItem.click();
    await flushPromises();

    expect(wrapper.text()).toContain("确认从项目中移除这个技能引用吗？");
    await wrapper.findAll("button").find((button) => button.text() === "移除引用")!.trigger("click");
    await flushPromises();

    expect(api.removeSkillReference).toHaveBeenCalledWith("ref-writer-pro", true);
  });

  it("confirms before deleting a project", async () => {
    const wrapper = mount(ProjectsView, {
      props: {
        snapshot,
        selectedProjectId: "acme",
      },
    });

    await flushPromises();

    await wrapper.find('button[aria-label="删除项目"]').trigger("click");
    expect(wrapper.text()).toContain("确认删除项目");
    expect(api.deleteProject).not.toHaveBeenCalled();

    await wrapper.findAll("button").find((button) => button.text() === "删除项目")!.trigger("click");
    await flushPromises();

    expect(api.deleteProject).toHaveBeenCalledWith("acme");
  });
});

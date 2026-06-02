/**
 * @vitest-environment jsdom
 */
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, describe, expect, it, vi } from "vitest";
import * as api from "../api";
import SkillsView from "../views/skills/SkillsView.vue";
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
    skills: [
      {
        id: "writer-pro",
        name: "Writer Pro",
        description: "长文写作与风格控制",
        libraryPath: "/library/writer-pro",
        references: [
          {
            id: "ref-claude-writer-pro",
            targetName: "Claude Code",
            targetPath: "/claude/skills/writer-pro",
            scope: "user",
            status: "healthy",
          },
        ],
        managedLinks: {
          codex: "/codex/skills/writer-pro",
        },
        conflict: null,
      },
    ],
    projects: [],
    agents: [],
  },
  targetProfiles: [
    {
      id: "claude-user",
      targetName: "Claude Code",
      rootPath: "/claude/skills",
      scope: "user",
    },
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

describe("SkillsView references tab", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("opens references first and toggles between list, graph, and detail views", async () => {
    const wrapper = mount(SkillsView, {
      props: {
        snapshot,
        selectedSkillId: "writer-pro",
      },
    });

    await wrapper.findAll(".detail-tab")[1].trigger("click");

    expect(wrapper.find(".detail-tab.active").text()).toBe("引用");
    expect(wrapper.text()).not.toContain("软链接引用");
    expect(wrapper.text()).not.toContain("软链接已指向当前 skill");
    expect(wrapper.text()).toContain("/codex/skills/writer-pro");
    expect(wrapper.text()).toContain("/claude/skills/writer-pro");
    expect(wrapper.find(".reference-row").exists()).toBe(true);

    await wrapper.find('button[aria-label="连线图"]').trigger("click");

    expect(wrapper.find(".reference-graph").exists()).toBe(true);
    expect(wrapper.find(".reference-row").exists()).toBe(false);

    await wrapper.findAll(".detail-tab")[0].trigger("click");
    await flushPromises();

    expect(wrapper.find(".description-pane").exists()).toBe(true);
    expect(wrapper.text()).toContain("长文写作与风格控制");
  });

  it("opens add and delete reference dialogs from the references list", async () => {
    const wrapper = mount(SkillsView, {
      props: {
        snapshot,
        selectedSkillId: "writer-pro",
      },
    });

    await wrapper.findAll(".detail-tab")[1].trigger("click");

    await wrapper.find('button[aria-label="新增引用"]').trigger("click");

    expect(wrapper.text()).toContain("Claude Code");
    expect(wrapper.text()).toContain("选择 skills 目录");

    await wrapper.find(".target-tile").trigger("click");

    expect(wrapper.text()).toContain("目标路径");
    expect(wrapper.text()).toContain("/claude/skills/writer-pro");
    expect(wrapper.text()).not.toContain("/library/writer-pro");

    await wrapper.find('button[aria-label="关闭"]').trigger("click");
    await wrapper.find('button[aria-label="删除引用"]').trigger("click");

    expect(wrapper.text()).toContain("删除引用");
    expect(wrapper.text()).toContain("只移除这个托管引用");
  });

  it("shows an in-app overwrite confirmation when a reference link points elsewhere", async () => {
    const addSkillReference = vi
      .spyOn(api, "addSkillReference")
      .mockRejectedValueOnce("路径无效：托管链接已指向其他位置")
      .mockResolvedValueOnce(snapshot);
    const wrapper = mount(SkillsView, {
      props: {
        snapshot,
        selectedSkillId: "writer-pro",
      },
    });

    await wrapper.findAll(".detail-tab")[1].trigger("click");

    await wrapper.find('button[aria-label="新增引用"]').trigger("click");
    await wrapper.find(".target-tile").trigger("click");
    await wrapper.findAll("button").find((button) => button.text() === "新增引用")!.trigger("click");
    await flushPromises();

    expect(wrapper.text()).toContain("覆盖引用链接");
    expect(wrapper.text()).toContain("引用链接已存在，且指向其他位置");
    expect(wrapper.text()).toContain("/claude/skills/writer-pro");

    await wrapper.findAll("button").find((button) => button.text() === "覆盖引用")!.trigger("click");
    await flushPromises();

    expect(addSkillReference).toHaveBeenLastCalledWith({
      skillId: "writer-pro",
      targetName: "Claude Code",
      rootPath: "/claude/skills",
      scope: "user",
      overwrite: true,
    });
  });

  it("scans import candidates and defaults ready skills to selected", async () => {
    const wrapper = mount(SkillsView, {
      props: {
        snapshot,
        selectedSkillId: "writer-pro",
      },
    });

    await wrapper.find('button[aria-label="新增 Skill"]').trigger("click");
    expect(wrapper.text()).toContain("新增 Skill");

    await wrapper.findAll(".import-source-tabs button")[1].trigger("click");
    await wrapper.find('input[placeholder="GitHub URL"]').setValue("https://github.com/acme/skills");
    await wrapper.findAll(".secondary-button").find((button) => button.text() === "扫描")!.trigger("click");
    await flushPromises();

    expect(wrapper.findAll(".import-candidate-row")).toHaveLength(2);
    expect(wrapper.find(".import-check-all input").element).toMatchObject({ checked: true });

    await wrapper.find(".import-candidate-row input").setValue(false);

    expect(wrapper.find(".import-check-all input").element).toMatchObject({ checked: false });
  });

  it("shows delete confirmation prompt when deleting a reference link that points elsewhere", async () => {
    const removeSkillReference = vi
      .spyOn(api, "removeSkillReference")
      .mockRejectedValueOnce("路径无效：托管链接已指向其他位置")
      .mockResolvedValueOnce(snapshot);

    const wrapper = mount(SkillsView, {
      props: {
        snapshot,
        selectedSkillId: "writer-pro",
      },
    });

    // Go to references tab
    await wrapper.findAll(".detail-tab")[1].trigger("click");

    // Open delete dialog
    await wrapper.find('button[aria-label="删除引用"]').trigger("click");
    expect(wrapper.text()).toContain("删除引用");

    // Confirm delete (triggers error because of mockRejectedValueOnce)
    await wrapper.findAll("button").find((button) => button.text() === "删除引用")!.trigger("click");
    await flushPromises();

    // Check if the prompt for conflict/mismatch is displayed
    expect(wrapper.text()).toContain("托管链接已指向其他位置（或存在内容冲突）");
    expect(wrapper.text()).toContain("是否同时删除该外部链接？");

    // Trigger click on "否（只移除记录）"
    await wrapper.findAll("button").find((button) => button.text() === "否（只移除记录）")!.trigger("click");
    await flushPromises();

    expect(removeSkillReference).toHaveBeenCalledWith("ref-claude-writer-pro", false);
  });
});

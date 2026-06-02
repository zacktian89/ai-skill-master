/**
 * @vitest-environment jsdom
 */
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import AgentsView from "../views/agents/AgentsView.vue";
import type { AppSnapshot } from "../types";
import * as api from "../api";
import { clearSkillScannerCaches } from "../composables/useSkillScanner";
import { useI18n } from "../composables/useI18n";

const apiMocks = vi.hoisted(() => ({
  addAgent: vi.fn(),
  deleteAgent: vi.fn(),
  setAgentRule: vi.fn(),
  scanAgentSkills: vi.fn(),
  addSkillReference: vi.fn(),
  removeSkillReference: vi.fn(),
  importProjectSkill: vi.fn(),
  readSkillFile: vi.fn(),
  readSkillFileAtPath: vi.fn(),
}));

vi.mock("../api", () => ({
  addAgent: apiMocks.addAgent,
  deleteAgent: apiMocks.deleteAgent,
  setAgentRule: apiMocks.setAgentRule,
  scanAgentSkills: apiMocks.scanAgentSkills,
  addSkillReference: apiMocks.addSkillReference,
  removeSkillReference: apiMocks.removeSkillReference,
  importProjectSkill: apiMocks.importProjectSkill,
  readSkillFile: apiMocks.readSkillFile,
  readSkillFileAtPath: apiMocks.readSkillFileAtPath,
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
    useI18n().locale.value = "zh";
    clearSkillScannerCaches();
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
    apiMocks.readSkillFile.mockReset();
    apiMocks.readSkillFile.mockResolvedValue(`---
name: "Writer Pro"
description: "长文写作"
license: "MIT"
version: "2.1"
---

# Writer Pro

这是详情内容。`);
    apiMocks.readSkillFileAtPath.mockReset();
    apiMocks.readSkillFileAtPath.mockResolvedValue(`---
name: "Local Skill"
description: "本地 skill 描述"
---

# Local Skill

这是本地详情内容。`);
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

    const moreButton = wrapper.find('button[aria-label="更多操作"]');
    expect(moreButton.exists()).toBe(true);

    await moreButton.trigger("click");
    await flushPromises();

    const cancelManageItem = Array.from(document.body.querySelectorAll(".global-context-menu-item")).find(
      (item) => item.textContent?.includes("取消管理")
    ) as HTMLButtonElement;
    expect(cancelManageItem).toBeDefined();
    cancelManageItem.click();
    await flushPromises();

    expect(wrapper.text()).toContain("确认取消管理 Agent");
    expect(api.deleteAgent).not.toHaveBeenCalled();

    await wrapper.findAll("button").find((button) => button.text() === "删除 Agent")!.trigger("click");
    await flushPromises();

    expect(api.deleteAgent).toHaveBeenCalledWith("codex");
  });

  it("opens a skill preview in the right panel and closes it with the back button", async () => {
    const agentSnapshot: AppSnapshot = {
      ...snapshot,
      state: {
        ...snapshot.state,
        skills: [
          {
            id: "writer-pro",
            name: "Writer Pro",
            description: "长文写作",
            libraryPath: "/library/writer-pro",
            references: [],
            managedLinks: {},
            conflict: null,
          },
        ],
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

    apiMocks.scanAgentSkills.mockResolvedValue([
      {
        name: ".",
        path: "~/.agents/skills",
        skills: [
          {
            id: "writer-pro",
            name: "Writer Pro",
            description: "长文写作",
            path: "~/.agents/skills/writer-pro",
            isManaged: true,
          },
        ],
      },
    ]);

    const wrapper = mount(AgentsView, {
      props: {
        snapshot: agentSnapshot,
        selectedAgentId: "codex",
      },
    });

    await flushPromises();

    expect(wrapper.text()).toContain("Codex");
    expect(wrapper.find('button[aria-label="返回技能列表"]').exists()).toBe(false);

    const detailPanel = wrapper.find(".detail-panel").element as HTMLElement;
    detailPanel.scrollTop = 220;

    await wrapper.find(".project-skill-row").trigger("click");
    await flushPromises();

    expect(wrapper.text()).toContain("Writer Pro");
    expect(wrapper.text()).toContain("name");
    expect(wrapper.text()).toContain("license");
    expect(wrapper.text()).toContain("这是详情内容。");
    expect(wrapper.find('button[aria-label="返回技能列表"]').exists()).toBe(true);
    expect(wrapper.find('button[aria-label="更多技能操作"]').exists()).toBe(true);

    await wrapper.find('button[aria-label="返回技能列表"]').trigger("click");
    await flushPromises();

    expect(wrapper.text()).toContain("Codex");
    expect(wrapper.find('button[aria-label="返回技能列表"]').exists()).toBe(false);
    expect(detailPanel.scrollTop).toBe(220);
  });

  it("uses cached agent scans until the rescan button is clicked", async () => {
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

    const wrapper = mount(AgentsView, {
      props: {
        snapshot: agentSnapshot,
        selectedAgentId: "codex",
      },
    });

    await flushPromises();

    expect(api.scanAgentSkills).toHaveBeenCalledTimes(1);

    await wrapper.setProps({
      snapshot: {
        ...agentSnapshot,
        state: {
          ...agentSnapshot.state,
          skills: [
            {
              id: "writer-pro",
              name: "Writer Pro",
              description: "长文写作",
              libraryPath: "/library/writer-pro",
              references: [],
              managedLinks: {},
              conflict: null,
            },
          ],
        },
      },
    });
    await flushPromises();

    expect(api.scanAgentSkills).toHaveBeenCalledTimes(1);

    await wrapper.find('button[aria-label="重新扫描"]').trigger("click");
    await flushPromises();

    expect(api.scanAgentSkills).toHaveBeenCalledTimes(2);
  });

  it("refreshes scanned skills after importing an unmanaged agent skill", async () => {
    const agentSnapshot: AppSnapshot = {
      ...snapshot,
      state: {
        ...snapshot.state,
        skills: [
          {
            id: "legacy-review",
            name: "Legacy Review",
            description: "旧系统审计",
            libraryPath: "/library/legacy-review",
            references: [],
            managedLinks: {},
            conflict: null,
          },
        ],
        agents: [
          {
            id: "codex",
            name: "Codex",
            path: "/agents/skills",
            rules: {},
          },
        ],
      },
    };

    apiMocks.scanAgentSkills
      .mockResolvedValueOnce([
        {
          name: ".",
          path: "/agents/skills",
          skills: [
            {
              id: "legacy-review",
              name: "Legacy Review",
              description: "旧系统审计",
              path: "/agents/skills/legacy-review",
              isManaged: false,
            },
          ],
        },
      ])
      .mockResolvedValueOnce([
        {
          name: ".",
          path: "/agents/skills",
          skills: [
            {
              id: "legacy-review",
              name: "Legacy Review",
              description: "旧系统审计",
              path: "/agents/skills/legacy-review",
              isManaged: true,
            },
          ],
        },
      ]);
    apiMocks.importProjectSkill.mockResolvedValue({ type: "success", snapshot: agentSnapshot });

    const wrapper = mount(AgentsView, {
      props: {
        snapshot: agentSnapshot,
        selectedAgentId: "codex",
      },
    });

    await flushPromises();

    await wrapper.find('button[aria-label="更多技能操作"]').trigger("click", { clientX: 300, clientY: 120 });
    await flushPromises();
    const importItem = Array.from(document.body.querySelectorAll(".global-context-menu-item")).find(
      (item) => item.textContent?.includes("托管")
    ) as HTMLButtonElement;
    importItem.click();
    await flushPromises();

    expect(api.importProjectSkill).toHaveBeenCalledWith("Codex", "/agents/skills/legacy-review", undefined);
    expect(api.scanAgentSkills).toHaveBeenCalledTimes(2);

    await wrapper.find('button[aria-label="更多技能操作"]').trigger("click", { clientX: 300, clientY: 120 });
    await flushPromises();

    const menuText = document.body.querySelector(".global-context-menu")?.textContent ?? "";
    expect(menuText).toContain("关闭");
    expect(menuText).toContain("删除引用");
    expect(menuText).not.toContain("托管");
  });

  it("loads local unmanaged skill details from the skill path", async () => {
    const agentSnapshot: AppSnapshot = {
      ...snapshot,
      state: {
        ...snapshot.state,
        agents: [
          {
            id: "codex",
            name: "Codex",
            path: "/agents/skills",
            rules: {},
          },
        ],
      },
    };

    apiMocks.scanAgentSkills.mockResolvedValue([
      {
        name: ".",
        path: "/agents/skills",
        skills: [
          {
            id: "speckit-checklist",
            name: "speckit-checklist",
            description: "",
            path: "/agents/skills/speckit-checklist",
            isManaged: false,
          },
        ],
      },
    ]);

    const wrapper = mount(AgentsView, {
      props: {
        snapshot: agentSnapshot,
        selectedAgentId: "codex",
      },
    });

    await flushPromises();
    await wrapper.find(".project-skill-row").trigger("click");
    await flushPromises();

    expect(api.readSkillFile).not.toHaveBeenCalled();
    expect(api.readSkillFileAtPath).toHaveBeenCalledWith("/agents/skills/speckit-checklist");
    expect(wrapper.text()).toContain("这是本地详情内容。");
  });
});

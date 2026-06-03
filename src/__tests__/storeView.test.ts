/**
 * @vitest-environment jsdom
 */
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import StoreView from "../views/store/StoreView.vue";
import type { AppSnapshot } from "../types";
import * as api from "../api";
import { useI18n } from "../composables/useI18n";

vi.mock("../api", async () => {
  const actual = await vi.importActual<typeof import("../api")>("../api");
  return {
    ...actual,
    fetchStoreLeaderboard: vi.fn(),
    searchStoreSkills: vi.fn(),
    previewImportSkills: vi.fn(),
    confirmImportSkills: vi.fn(),
  };
});

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
    skills: [
      {
        id: "installed-skill",
        name: "Installed Skill",
        description: "已经安装的技能",
        libraryPath: "/library/installed-skill",
        source: {
          kind: "openclawMarket",
          label: "skills.sh",
          url: "https://github.com/acme/skills.git",
          subdir: "installed-skill",
        },
        references: [],
        managedLinks: {},
        conflict: null,
      },
    ],
    projects: [],
    agents: [],
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

describe("StoreView", () => {
  beforeEach(() => {
    useI18n().locale.value = "zh";
    vi.mocked(api.fetchStoreLeaderboard).mockResolvedValue([
      {
        id: "acme/skills/installed-skill",
        skillId: "installed-skill",
        name: "Installed Skill",
        source: "acme/skills",
        installs: 1200,
      },
      {
        id: "openai/skills/playwright",
        skillId: "playwright",
        name: "Playwright",
        source: "openai/skills",
        installs: 4000,
      },
    ]);
    vi.mocked(api.searchStoreSkills).mockResolvedValue([
      {
        id: "openai/skills/playwright",
        skillId: "playwright",
        name: "Playwright",
        source: "openai/skills",
        installs: 4000,
      },
    ]);
    vi.mocked(api.previewImportSkills).mockResolvedValue({
      candidates: [
        {
          candidateId: "playwright",
          id: "playwright",
          name: "Playwright",
          description: "Browser automation skill",
          relativePath: "playwright",
          status: "ready",
          message: null,
        },
      ],
    });
    vi.mocked(api.confirmImportSkills).mockResolvedValue(snapshot);
  });

  it("loads the store leaderboard and marks installed skills", async () => {
    const wrapper = mount(StoreView, {
      props: {
        snapshot,
      },
    });

    await flushPromises();

    expect(api.fetchStoreLeaderboard).toHaveBeenCalledWith("alltime");
    expect(wrapper.text()).toContain("Installed Skill");
    expect(wrapper.text()).toContain("Playwright");
    expect(wrapper.text()).toContain("已安装");
  });

  it("switches leaderboard tabs and uses search results", async () => {
    const wrapper = mount(StoreView, {
      props: {
        snapshot,
      },
    });

    await flushPromises();
    await wrapper.findAll(".segmented-control button")[2]!.trigger("click");
    await flushPromises();

    expect(api.fetchStoreLeaderboard).toHaveBeenLastCalledWith("trending");

    await wrapper.find('input[type="search"]').setValue("playwright");
    await flushPromises();

    expect(api.searchStoreSkills).toHaveBeenCalledWith("playwright", 60);
    expect(wrapper.text()).toContain("Playwright");
  });

  it("downloads a store skill through the existing import preview flow", async () => {
    const wrapper = mount(StoreView, {
      props: {
        snapshot,
      },
    });

    await flushPromises();
    await wrapper.findAll(".store-list-item")[1]!.trigger("click");
    await flushPromises();
    await wrapper.find('button[aria-label="下载 Skill"]').trigger("click");
    await flushPromises();

    expect(api.previewImportSkills).toHaveBeenCalledWith({
      kind: "github",
      url: "https://github.com/openai/skills.git",
      ref: null,
      subdir: null,
    });

    expect(wrapper.text()).toContain("导入 Skill");
    expect(wrapper.text()).toContain("Playwright");
  });
});

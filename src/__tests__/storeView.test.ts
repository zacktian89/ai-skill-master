/**
 * @vitest-environment jsdom
 */
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import StoreView from "../views/store/StoreView.vue";
import type { AppSnapshot, StoreSkill } from "../types";
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
    vi.clearAllMocks();
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

  afterEach(() => {
    vi.unstubAllGlobals();
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

  it("shows the initial loading animation before the first store response resolves", async () => {
    let resolveLeaderboard: ((value: StoreSkill[]) => void) | undefined;
    vi.mocked(api.fetchStoreLeaderboard).mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveLeaderboard = resolve;
        })
    );

    const wrapper = mount(StoreView, {
      props: {
        snapshot,
      },
    });

    await flushPromises();

    const loadingAnimation = wrapper.get("[data-testid='app-loading-animation']");
    expect(loadingAnimation.classes()).toContain("app-loading-animation--panel");
    expect(loadingAnimation.classes()).toContain("store-initial-loading");
    expect(wrapper.findAll("[data-testid='loading-skeleton-row']")).toHaveLength(7);

    if (!resolveLeaderboard) {
      throw new Error("expected leaderboard request to be pending");
    }

    resolveLeaderboard([
      {
        id: "acme/skills/installed-skill",
        skillId: "installed-skill",
        name: "Installed Skill",
        source: "acme/skills",
        installs: 1200,
      },
    ]);
    await flushPromises();

    expect(wrapper.find("[data-testid='app-loading-animation']").exists()).toBe(false);
    expect(wrapper.text()).toContain("Installed Skill");
  });

  it("switches leaderboard tabs and uses search results", async () => {
    vi.useFakeTimers();
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
    vi.advanceTimersByTime(250);
    await flushPromises();

    expect(api.searchStoreSkills).toHaveBeenCalledWith("playwright", 60);
    expect(wrapper.text()).toContain("Playwright");
    vi.useRealTimers();
  });

  it("debounces store search input before loading results", async () => {
    vi.useFakeTimers();
    const wrapper = mount(StoreView, {
      props: {
        snapshot,
      },
    });

    await flushPromises();
    vi.mocked(api.searchStoreSkills).mockClear();

    const searchInput = wrapper.find('input[type="search"]');
    await searchInput.setValue("p");
    await searchInput.setValue("pl");
    await searchInput.setValue("playwright");

    expect(api.searchStoreSkills).not.toHaveBeenCalled();

    vi.advanceTimersByTime(249);
    await flushPromises();
    expect(api.searchStoreSkills).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1);
    await flushPromises();

    expect(api.searchStoreSkills).toHaveBeenCalledTimes(1);
    expect(api.searchStoreSkills).toHaveBeenCalledWith("playwright", 60);
    vi.useRealTimers();
  });

  it("does not run import preview while selecting store skills for details", async () => {
    const wrapper = mount(StoreView, {
      props: {
        snapshot,
      },
    });

    await flushPromises();
    vi.mocked(api.previewImportSkills).mockClear();

    await wrapper.findAll(".store-list-item")[1]!.trigger("click");
    await flushPromises();

    expect(api.previewImportSkills).not.toHaveBeenCalled();
    expect(wrapper.text()).toContain("Playwright");
  });

  it("loads store skill markdown from the common skills subdirectory without import preview", async () => {
    vi.mocked(api.fetchStoreLeaderboard).mockResolvedValue([
      {
        id: "anthropics/skills/frontend-design",
        skillId: "frontend-design",
        name: "frontend-design",
        source: "anthropics/skills",
        installs: 497552,
      },
    ]);
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      return {
        ok: url.includes("/skills/frontend-design/SKILL.md"),
        statusText: "Not Found",
        text: async () => "# Frontend Design\n\nLoaded from skills subdir.",
      } as Response;
    });
    vi.stubGlobal("fetch", fetchMock);

    const wrapper = mount(StoreView, {
      props: {
        snapshot,
      },
    });

    await flushPromises();
    await flushPromises();

    expect(api.previewImportSkills).not.toHaveBeenCalled();
    expect(fetchMock).toHaveBeenCalledWith(
      "https://raw.githubusercontent.com/anthropics/skills/main/skills/frontend-design/SKILL.md"
    );
    expect(wrapper.text()).toContain("Frontend Design");
    expect(wrapper.text()).not.toContain("加载 SKILL.md 失败");
  });

  it("resolves nested store markdown paths from the GitHub tree when direct paths miss", async () => {
    vi.mocked(api.fetchStoreLeaderboard).mockResolvedValue([
      {
        id: "microsoft/azure-skills/azure-cost-optimization",
        skillId: "azure-cost-optimization",
        name: "azure-cost-optimization",
        source: "microsoft/azure-skills",
        installs: 206876,
      },
    ]);
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/git/trees/main?recursive=1")) {
        return {
          ok: true,
          json: async () => ({
            tree: [
              { path: "skills/azure-cost/SKILL.md", type: "blob" },
              { path: "skills/azure-cost/cost-optimization/workflow.md", type: "blob" },
            ],
          }),
        } as Response;
      }
      return {
        ok: url.includes("/skills/azure-cost/SKILL.md"),
        statusText: "Not Found",
        text: async () => "# Azure Cost\n\nResolved from GitHub tree.",
      } as Response;
    });
    vi.stubGlobal("fetch", fetchMock);

    const wrapper = mount(StoreView, {
      props: {
        snapshot,
      },
    });

    await flushPromises();
    await flushPromises();

    expect(fetchMock).toHaveBeenCalledWith(
      "https://api.github.com/repos/microsoft/azure-skills/git/trees/main?recursive=1"
    );
    expect(fetchMock).toHaveBeenCalledWith(
      "https://raw.githubusercontent.com/microsoft/azure-skills/main/skills/azure-cost/SKILL.md"
    );
    expect(wrapper.text()).toContain("Azure Cost");
    expect(wrapper.text()).not.toContain("加载 SKILL.md 失败");
  });

  it("resolves store markdown by SKILL frontmatter name when directory and id differ", async () => {
    vi.mocked(api.fetchStoreLeaderboard).mockResolvedValue([
      {
        id: "vercel-labs/agent-skills/vercel-react-best-practices",
        skillId: "vercel-react-best-practices",
        name: "vercel-react-best-practices",
        source: "vercel-labs/agent-skills",
        installs: 448733,
      },
    ]);
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/git/trees/main?recursive=1")) {
        return {
          ok: true,
          json: async () => ({
            tree: [
              { path: "skills/composition-patterns/SKILL.md", type: "blob" },
              { path: "skills/react-best-practices/SKILL.md", type: "blob" },
            ],
          }),
        } as Response;
      }
      if (url.includes("/skills/composition-patterns/SKILL.md")) {
        return {
          ok: true,
          text: async () => "---\nname: vercel-composition-patterns\n---\n# Composition",
        } as Response;
      }
      return {
        ok: url.includes("/skills/react-best-practices/SKILL.md"),
        statusText: "Not Found",
        text: async () => "---\nname: vercel-react-best-practices\n---\n# Vercel React Best Practices",
      } as Response;
    });
    vi.stubGlobal("fetch", fetchMock);

    const wrapper = mount(StoreView, {
      props: {
        snapshot,
      },
    });

    await flushPromises();
    await flushPromises();

    expect(fetchMock).toHaveBeenCalledWith(
      "https://raw.githubusercontent.com/vercel-labs/agent-skills/main/skills/react-best-practices/SKILL.md"
    );
    expect(wrapper.text()).toContain("Vercel React Best Practices");
    expect(wrapper.text()).not.toContain("加载 SKILL.md 失败");
  });

  it("downloads using the markdown path resolved for store details", async () => {
    vi.mocked(api.fetchStoreLeaderboard).mockResolvedValue([
      {
        id: "microsoft/azure-skills/azure-cost-optimization",
        skillId: "azure-cost-optimization",
        name: "azure-cost-optimization",
        source: "microsoft/azure-skills",
        installs: 206876,
      },
    ]);
    vi.mocked(api.previewImportSkills).mockResolvedValue({
      candidates: [
        {
          candidateId: ".",
          id: "azure-cost",
          name: "azure-cost",
          description: "Azure cost skill",
          relativePath: ".",
          status: "ready",
          message: null,
        },
      ],
    });
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/git/trees/main?recursive=1")) {
        return {
          ok: true,
          json: async () => ({
            tree: [
              { path: "skills/azure-cost/SKILL.md", type: "blob" },
              { path: "skills/azure-cost/cost-optimization/workflow.md", type: "blob" },
            ],
          }),
        } as Response;
      }
      return {
        ok: url.includes("/skills/azure-cost/SKILL.md"),
        statusText: "Not Found",
        text: async () => "# Azure Cost\n\nResolved from GitHub tree.",
      } as Response;
    });
    vi.stubGlobal("fetch", fetchMock);

    const wrapper = mount(StoreView, {
      props: {
        snapshot,
      },
    });

    await flushPromises();
    await flushPromises();
    await wrapper.find('button[aria-label="下载 Skill"]').trigger("click");
    await flushPromises();

    expect(api.previewImportSkills).toHaveBeenCalledWith({
      kind: "github",
      url: "https://github.com/microsoft/azure-skills.git",
      ref: null,
      subdir: "skills/azure-cost",
    });
    expect(api.confirmImportSkills).toHaveBeenCalledWith({
      source: {
        kind: "github",
        url: "https://github.com/microsoft/azure-skills.git",
        ref: null,
        subdir: "skills/azure-cost",
      },
      candidateIds: ["."],
    });
  });

  it("downloads a store skill directly after resolving the matching import candidate", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => ({
        ok: false,
        statusText: "Not Found",
        json: async () => ({ tree: [] }),
        text: async () => "",
      }))
    );
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
        {
          candidateId: "writer",
          id: "writer",
          name: "Writer",
          description: "Writing skill",
          relativePath: "writer",
          status: "ready",
          message: null,
        },
      ],
    });
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

    expect(wrapper.text()).not.toContain("导入 Skill");
    expect(wrapper.text()).not.toContain("Writer");

    expect(api.confirmImportSkills).toHaveBeenCalledWith({
      source: {
        kind: "github",
        url: "https://github.com/openai/skills.git",
        ref: null,
        subdir: null,
      },
      candidateIds: ["playwright"],
    });
  });
});

/**
 * @vitest-environment jsdom
 */
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App.vue";
import { router } from "../router";
import { useI18n } from "../composables/useI18n";

vi.mock("../api", () => ({
  getSnapshot: vi.fn().mockResolvedValue({
    state: {
      schemaVersion: 1,
      skillLibraryPath: "D:/skills",
      currentProjectId: null,
      syncStatus: {
        phase: "idle",
        message: null,
        pendingActions: [],
      },
      migrationNotice: null,
      skills: [
        {
          id: "writer-pro",
          name: "Writer Pro",
          description: "长文写作",
          libraryPath: "D:/skills/writer-pro",
          references: [],
          managedLinks: {},
          conflict: null,
        },
      ],
      projects: [],
    },
    diagnostics: [],
    paths: {
      stateFile: "D:/config/skillmaster.json",
      backupFile: "D:/config/skillmaster.json.bak",
    },
    stateLoad: {
      phase: "clean",
      message: null,
    },
  }),
  readSkillFile: vi.fn().mockResolvedValue(""),
}));

describe("App shell", () => {
  beforeEach(async () => {
    localStorage.clear();
    useI18n().locale.value = "zh";
    router.push("/");
    await router.isReady();
  });

  it("renders the SkillMaster navigation after loading a snapshot", async () => {
    const wrapper = mount(App, {
      global: {
        plugins: [router],
      },
    });

    await vi.dynamicImportSettled();

    expect(wrapper.text()).toContain("SkillMaster");
    expect(wrapper.text()).toContain("技能");
    expect(wrapper.text()).toContain("商店");
    expect(wrapper.text()).toContain("项目");
    expect(wrapper.text()).toContain("设置");
  });

  it("navigates to the store workspace from the sidebar", async () => {
    const wrapper = mount(App, {
      global: {
        plugins: [router],
      },
    });

    await vi.dynamicImportSettled();

    await wrapper.findAll("button").find((button) => button.text().includes("商店"))!.trigger("click");
    await flushPromises();

    expect(router.currentRoute.value.name).toBe("store");
  });

  it("uses the application logo in the top brand mark", async () => {
    const wrapper = mount(App, {
      global: {
        plugins: [router],
      },
    });

    await vi.dynamicImportSettled();

    const brandLogo = wrapper.get(".rail-brand-mark img");
    expect(brandLogo.attributes("src")).toContain("skillmaster-logo-dark.png");
    expect(wrapper.findComponent({ name: "Sparkles" }).exists()).toBe(false);
  });

  it("defaults to the dark theme", async () => {
    const wrapper = mount(App, {
      global: {
        plugins: [router],
      },
    });

    await vi.dynamicImportSettled();

    expect(wrapper.find(".app-shell").attributes("data-theme")).toBe("dark");
  });

  it("switches theme from settings and persists the preference", async () => {
    const wrapper = mount(App, {
      global: {
        plugins: [router],
      },
    });

    await vi.dynamicImportSettled();
    await wrapper.findAll("button").find((button) => button.text().includes("设置"))!.trigger("click");
    await flushPromises();
    await wrapper.findAll(".theme-toggle button")[1]!.trigger("click");

    expect(wrapper.find(".app-shell").attributes("data-theme")).toBe("light");
    expect(localStorage.getItem("skillmaster-theme")).toBe("light");
  });

  it("does not render redundant skills headings", async () => {
    const wrapper = mount(App, {
      global: {
        plugins: [router],
      },
    });

    await vi.dynamicImportSettled();

    expect(wrapper.text()).not.toContain("技能库");
    expect(wrapper.text()).not.toContain("Skill Detail");
  });
});



/**
 * @vitest-environment jsdom
 */
import { mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App.vue";
vi.mock("../api", () => ({
  getSnapshot: vi.fn().mockResolvedValue({
    state: {
      schemaVersion: 1,
      skillLibraryPath: "D:/skills",
      codexSkillsPath: null,
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
          managedLinks: {
            codex: null,
          },
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
}));

describe("App shell", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("renders the SkillMaster navigation after loading a snapshot", async () => {
    const wrapper = mount(App);

    await vi.dynamicImportSettled();

    expect(wrapper.text()).toContain("SkillMaster");
    expect(wrapper.text()).toContain("Skills");
    expect(wrapper.text()).toContain("Projects");
    expect(wrapper.text()).toContain("Settings");
  });

  it("defaults to the dark theme", async () => {
    const wrapper = mount(App);

    await vi.dynamicImportSettled();

    expect(wrapper.find(".app-shell").attributes("data-theme")).toBe("dark");
  });

  it("switches theme from settings and persists the preference", async () => {
    const wrapper = mount(App);

    await vi.dynamicImportSettled();
    await wrapper.findAll("button").find((button) => button.text().includes("Settings"))!.trigger("click");
    await wrapper.vm.$nextTick();
    await wrapper.findAll("button").find((button) => button.text().includes("白色"))!.trigger("click");

    expect(wrapper.find(".app-shell").attributes("data-theme")).toBe("light");
    expect(localStorage.getItem("skillmaster-theme")).toBe("light");
  });

  it("does not render redundant skills headings", async () => {
    const wrapper = mount(App);

    await vi.dynamicImportSettled();

    expect(wrapper.text()).not.toContain("技能库");
    expect(wrapper.text()).not.toContain("Skill Detail");
  });
});

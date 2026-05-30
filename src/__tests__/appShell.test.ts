/**
 * @vitest-environment jsdom
 */
import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
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
      skills: [],
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
  it("renders the SkillMaster navigation after loading a snapshot", async () => {
    const wrapper = mount(App);

    await vi.dynamicImportSettled();

    expect(wrapper.text()).toContain("SkillMaster");
    expect(wrapper.text()).toContain("Skills");
    expect(wrapper.text()).toContain("Projects");
    expect(wrapper.text()).toContain("Settings");
  });
});

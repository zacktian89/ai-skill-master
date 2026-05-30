/**
 * @vitest-environment jsdom
 */
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import SkillsView from "../components/SkillsView.vue";
import type { AppSnapshot } from "../types";

const snapshot: AppSnapshot = {
  state: {
    schemaVersion: 1,
    skillLibraryPath: "/skills",
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
        libraryPath: "/skills/writer-pro",
        managedLinks: {
          codex: "/codex/skills/writer-pro",
        },
        conflict: null,
      },
    ],
    projects: [],
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

describe("SkillsView references tab", () => {
  it("opens references first and toggles between list, graph, and detail views", async () => {
    const wrapper = mount(SkillsView, {
      props: {
        snapshot,
        selectedSkillId: "writer-pro",
      },
    });

    expect(wrapper.find(".detail-tab.active").text()).toBe("引用");
    expect(wrapper.text()).not.toContain("软链接引用");
    expect(wrapper.text()).not.toContain("软链接已指向当前 skill");
    expect(wrapper.text()).toContain("/codex/skills/writer-pro");
    expect(wrapper.find(".reference-row").exists()).toBe(true);

    await wrapper.find('button[aria-label="连线图"]').trigger("click");

    expect(wrapper.find(".reference-graph").exists()).toBe(true);
    expect(wrapper.find(".reference-row").exists()).toBe(false);

    await wrapper.findAll(".detail-tab")[1].trigger("click");

    expect(wrapper.find(".description-pane").exists()).toBe(true);
    expect(wrapper.text()).toContain("长文写作与风格控制");
  });
});

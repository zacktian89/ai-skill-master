/**
 * @vitest-environment jsdom
 */
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "../api";
import SkillsView from "../views/skills/SkillsView.vue";
import type { AppSnapshot } from "../types";
import { useI18n } from "../composables/useI18n";

vi.mock("@tauri-apps/plugin-opener", () => ({
  openPath: vi.fn(),
  openUrl: vi.fn(),
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
        managedLinks: {},
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
  beforeEach(() => {
    localStorage.clear();
    useI18n().locale.value = "zh";
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("opens references first and shows the references list without graph controls", async () => {
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
    expect(wrapper.text()).toContain("/claude/skills/writer-pro");
    expect(wrapper.find(".reference-row").exists()).toBe(true);
    expect(wrapper.find('button[aria-label="连线图"]').exists()).toBe(false);
    expect(wrapper.find(".reference-graph").exists()).toBe(false);

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

    await wrapper.find('button[aria-label="更多操作"]').trigger("click");
    await flushPromises();
    const addReferenceItem = Array.from(document.body.querySelectorAll(".global-context-menu-item")).find(
      (item) => item.textContent?.includes("增加引用")
    ) as HTMLButtonElement;
    addReferenceItem.click();
    await flushPromises();

    expect(wrapper.text()).toContain("Claude Code");
    expect(wrapper.text()).toContain("选择 skills 目录");

    await wrapper.find(".target-tile").trigger("click");

    expect(wrapper.text()).toContain("目标路径");
    expect(wrapper.text()).toContain("/claude/skills/writer-pro");
    expect(wrapper.text()).not.toContain("/library/writer-pro");

    await wrapper.find('button[aria-label="关闭"]').trigger("click");
    await wrapper.findAll(".detail-tab")[1].trigger("click");
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

    await wrapper.find('button[aria-label="更多操作"]').trigger("click");
    await flushPromises();
    const addReferenceItem = Array.from(document.body.querySelectorAll(".global-context-menu-item")).find(
      (item) => item.textContent?.includes("增加引用")
    ) as HTMLButtonElement;
    addReferenceItem.click();
    await flushPromises();
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
    await wrapper.find('input[type="url"]').setValue("https://github.com/acme/skills");
    await wrapper.find(".scan-btn").trigger("click");
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

  it("navigates import steps: allows going back to step 1 and clicking next", async () => {
    const wrapper = mount(SkillsView, {
      props: {
        snapshot,
        selectedSkillId: "writer-pro",
      },
    });

    await wrapper.find('button[aria-label="新增 Skill"]').trigger("click");
    expect(wrapper.text()).toContain("新增 Skill");

    // Initially in step 1
    expect(wrapper.text()).toContain("设置路径");
    expect(wrapper.text()).toContain("本地");
    expect(wrapper.text()).toContain("GitHub");

    await wrapper.findAll(".import-source-tabs button")[1].trigger("click");
    await wrapper.find('input[type="url"]').setValue("https://github.com/acme/skills");
    await wrapper.find(".scan-btn").trigger("click");
    await flushPromises();

    // After scanning, automatically moves to step 2
    expect(wrapper.text()).toContain("已选 2/2");
    expect(wrapper.find('button.primary-button').text()).toContain("导入");

    // Click back to step 1
    await wrapper.findAll("button").find(b => b.text() === "上一步")!.trigger("click");
    await flushPromises();

    expect(wrapper.text()).toContain("设置路径");
    expect(wrapper.text()).toContain("下一步");

    // Click next to step 2 again
    await wrapper.findAll("button").find(b => b.text() === "下一步")!.trigger("click");
    await flushPromises();

    expect(wrapper.text()).toContain("已选 2/2");
  });

  it("renders a GitHub link for GitHub-imported skills in details", async () => {
    const githubSnapshot: AppSnapshot = {
      ...snapshot,
      state: {
        ...snapshot.state,
        skills: [
          {
            id: "writer-git",
            name: "Writer Git",
            description: "Git 仓库里的写作技能",
            libraryPath: "/library/writer-git",
            source: {
              kind: "github",
              label: "GitHub",
              url: "git@github.com:test-owner/test-repo.git",
            },
            references: [],
            managedLinks: {},
            conflict: null,
          },
        ],
      },
    };

    const wrapper = mount(SkillsView, {
      props: {
        snapshot: githubSnapshot,
        selectedSkillId: "writer-git",
      },
    });

    await flushPromises();

    const githubLink = wrapper.find("a.github-link");
    expect(githubLink.exists()).toBe(true);
    expect(githubLink.text()).toContain("打开 GitHub");
    expect(githubLink.attributes("href")).toBe("https://github.com/test-owner/test-repo");

    // Mock openUrl
    const opener = await import("@tauri-apps/plugin-opener");
    const openUrlSpy = vi.spyOn(opener, "openUrl");

    await githubLink.trigger("click");
    expect(openUrlSpy).toHaveBeenCalledWith("https://github.com/test-owner/test-repo");
  });

  it("aggregates skills from the same GitHub repository when toggle button is clicked", async () => {
    const complexSnapshot: AppSnapshot = {
      ...snapshot,
      state: {
        ...snapshot.state,
        skills: [
          {
            id: "skill-a",
            name: "Skill A",
            description: "GitHub Skill 1",
            libraryPath: "/library/skill-a",
            source: {
              kind: "github",
              url: "https://github.com/org-a/repo-1",
            },
            references: [],
            managedLinks: {},
            conflict: null,
          },
          {
            id: "skill-b",
            name: "Skill B",
            description: "GitHub Skill 2",
            libraryPath: "/library/skill-b",
            source: {
              kind: "github",
              url: "https://github.com/org-a/repo-1",
            },
            references: [],
            managedLinks: {},
            conflict: null,
          },
          {
            id: "skill-c",
            name: "Skill C",
            description: "GitHub Skill 3 from diff repo",
            libraryPath: "/library/skill-c",
            source: {
              kind: "github",
              url: "https://github.com/org-b/repo-2.git",
            },
            references: [],
            managedLinks: {},
            conflict: null,
          },
          {
            id: "skill-local",
            name: "Skill Local",
            description: "Local Skill",
            libraryPath: "/library/skill-local",
            source: {
              kind: "local",
              path: "/local/path",
            },
            references: [],
            managedLinks: {},
            conflict: null,
          },
        ],
      },
    };

    const wrapper = mount(SkillsView, {
      props: {
        snapshot: complexSnapshot,
        selectedSkillId: "skill-a",
      },
    });

    await flushPromises();

    // Verify non-grouped layout initially
    expect(wrapper.find(".skill-group").exists()).toBe(false);
    expect(wrapper.find(".list-stack").text()).toContain("Skill A");
    expect(wrapper.find(".list-stack").text()).toContain("Skill B");
    expect(wrapper.find(".list-stack").text()).toContain("Skill C");
    expect(wrapper.find(".list-stack").text()).toContain("Skill Local");

    // Click the toggle aggregation button
    const toggleBtn = wrapper.find('button[aria-label="按 GitHub 仓库聚合"]');
    expect(toggleBtn.exists()).toBe(true);
    await toggleBtn.trigger("click");
    await flushPromises();

    // Grouping should be active now
    expect(wrapper.find(".skill-group").exists()).toBe(true);

    const groupHeaders = wrapper.findAll(".skill-group-header");
    expect(groupHeaders).toHaveLength(3);

    // Group 1: org-a/repo-1 (alphabetically sorted)
    expect(groupHeaders[0].find(".skill-group-title").text()).toBe("org-a/repo-1");
    expect(groupHeaders[0].find(".skill-group-count").text()).toBe("2");

    // Group 2: org-b/repo-2
    expect(groupHeaders[1].find(".skill-group-title").text()).toBe("org-b/repo-2");
    expect(groupHeaders[1].find(".skill-group-count").text()).toBe("1");

    // Group 3: Local/Others
    expect(groupHeaders[2].find(".skill-group-title").text()).toBe("其他 / 本地");
    expect(groupHeaders[2].find(".skill-group-count").text()).toBe("1");

    // Verify skills are visible initially
    expect(wrapper.find(".list-stack").text()).toContain("Skill A");
    expect(wrapper.find(".list-stack").text()).toContain("Skill B");

    // Click Group 1 header to collapse it
    await groupHeaders[0].trigger("click");
    await flushPromises();

    // Verify chevron class has collapsed state
    expect(groupHeaders[0].find(".skill-group-chevron").classes()).toContain("collapsed");

    // The skills inside Group 1 (Skill A and Skill B) should be hidden in the list
    expect(wrapper.find(".list-stack").text()).not.toContain("Skill A");
    expect(wrapper.find(".list-stack").text()).not.toContain("Skill B");
    // Other groups' skills should still be visible in the list
    expect(wrapper.find(".list-stack").text()).toContain("Skill C");
    expect(wrapper.find(".list-stack").text()).toContain("Skill Local");

    // Click Group 1 header again to expand it
    await groupHeaders[0].trigger("click");
    await flushPromises();

    // Verify chevron class has no collapsed state
    expect(groupHeaders[0].find(".skill-group-chevron").classes()).not.toContain("collapsed");

    // The skills should be visible again in the list
    expect(wrapper.find(".list-stack").text()).toContain("Skill A");
    expect(wrapper.find(".list-stack").text()).toContain("Skill B");
  });

  it("renders and switches to the Readme tab for GitHub-imported skills, fetching and parsing the raw README.md", async () => {
    const githubSnapshot: AppSnapshot = {
      ...snapshot,
      state: {
        ...snapshot.state,
        skills: [
          {
            id: "writer-git",
            name: "Writer Git",
            description: "Git 仓库里的写作技能",
            libraryPath: "/library/writer-git",
            source: {
              kind: "github",
              url: "https://github.com/test-owner/test-repo",
            },
            references: [],
            managedLinks: {},
            conflict: null,
          },
        ],
      },
    };

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      text: () => Promise.resolve("# Git Repo README\nThis is the readme content."),
    });
    vi.stubGlobal("fetch", fetchMock);

    const wrapper = mount(SkillsView, {
      props: {
        snapshot: githubSnapshot,
        selectedSkillId: "writer-git",
      },
    });

    await flushPromises();

    // Verify Readme tab button exists because it's a GitHub skill
    const tabs = wrapper.findAll(".detail-tab");
    const readmeTab = tabs.find((tab) => tab.text() === "Readme");
    expect(readmeTab).toBeDefined();

    // Click on Readme tab
    await readmeTab!.trigger("click");
    await flushPromises();

    // Verify fetch was called with the correct raw github user content URL
    expect(fetchMock).toHaveBeenCalledWith(
      "https://raw.githubusercontent.com/test-owner/test-repo/main/README.md"
    );

    // Verify the README content is parsed and rendered inside the description pane
    const pane = wrapper.find(".description-pane");
    expect(pane.exists()).toBe(true);
    expect(pane.html()).toContain("<h1>Git Repo README</h1>");
    expect(pane.text()).toContain("This is the readme content.");

    vi.unstubAllGlobals();
  });

  it("falls back to the master branch if the fetch from main returns a 404", async () => {
    const githubSnapshot: AppSnapshot = {
      ...snapshot,
      state: {
        ...snapshot.state,
        skills: [
          {
            id: "writer-git",
            name: "Writer Git",
            description: "Git 仓库里的写作技能",
            libraryPath: "/library/writer-git",
            source: {
              kind: "github",
              url: "https://github.com/test-owner/test-repo",
            },
            references: [],
            managedLinks: {},
            conflict: null,
          },
        ],
      },
    };

    const fetchMock = vi.fn().mockImplementation((url: string) => {
      if (url.includes("/main/")) {
        return Promise.resolve({
          ok: false,
          status: 404,
          statusText: "Not Found",
        });
      }
      return Promise.resolve({
        ok: true,
        text: () => Promise.resolve("# Legacy README\nMaster branch readme."),
      });
    });
    vi.stubGlobal("fetch", fetchMock);

    const wrapper = mount(SkillsView, {
      props: {
        snapshot: githubSnapshot,
        selectedSkillId: "writer-git",
      },
    });

    await flushPromises();

    const readmeTab = wrapper.findAll(".detail-tab").find((tab) => tab.text() === "Readme");
    await readmeTab!.trigger("click");
    await flushPromises();

    // Verify fetch was tried for main, and then fell back to master
    expect(fetchMock).toHaveBeenCalledWith(
      "https://raw.githubusercontent.com/test-owner/test-repo/main/README.md"
    );
    expect(fetchMock).toHaveBeenCalledWith(
      "https://raw.githubusercontent.com/test-owner/test-repo/master/README.md"
    );

    // Verify master branch content is rendered
    expect(wrapper.find(".description-pane").html()).toContain("<h1>Legacy README</h1>");
    expect(wrapper.find(".description-pane").text()).toContain("Master branch readme.");

    vi.unstubAllGlobals();
  });

  it("persists the groupByGitHub selection to localStorage and initializes it on mount", async () => {
    // Set localStorage value to true initially
    localStorage.setItem("skillmaster-group-by-github", "true");

    const complexSnapshot: AppSnapshot = {
      ...snapshot,
      state: {
        ...snapshot.state,
        skills: [
          {
            id: "skill-a",
            name: "Skill A",
            description: "GitHub Skill 1",
            libraryPath: "/library/skill-a",
            source: {
              kind: "github",
              url: "https://github.com/org-a/repo-1",
            },
            references: [],
            managedLinks: {},
            conflict: null,
          },
        ],
      },
    };

    const wrapper = mount(SkillsView, {
      props: {
        snapshot: complexSnapshot,
        selectedSkillId: "skill-a",
      },
    });

    await flushPromises();

    // Verify it is grouped on mount
    expect(wrapper.find(".skill-group").exists()).toBe(true);

    // Toggle it off
    const toggleBtn = wrapper.find('button[aria-label="按 GitHub 仓库聚合"]');
    await toggleBtn.trigger("click");
    await flushPromises();

    // Verify it is no longer grouped
    expect(wrapper.find(".skill-group").exists()).toBe(false);

    // Verify localStorage has been updated
    expect(localStorage.getItem("skillmaster-group-by-github")).toBe("false");
  });
});

import { ref, computed, watch } from "vue";
import { marked } from "marked";
import * as api from "../api";

export function useSkillMarkdown(
  getSelectedSkillId: () => string | null | undefined,
  getActiveTab?: () => string
) {
  const skillMarkdown = ref("");
  const isMarkdownLoading = ref(false);

  async function loadSkillMarkdown() {
    const skillId = getSelectedSkillId();
    if (!skillId) {
      skillMarkdown.value = "";
      return;
    }
    isMarkdownLoading.value = true;
    try {
      const content = await api.readSkillFile(skillId);
      skillMarkdown.value = content;
    } catch (err) {
      console.error("加载 SKILL.md 失败", err);
      skillMarkdown.value = "";
    } finally {
      isMarkdownLoading.value = false;
    }
  }

  function parseFrontMatter(content: string) {
    const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
    if (!match) return { metadata: {} as Record<string, string>, body: content };
    const yamlStr = match[1];
    const body = content.slice(match[0].length).trim();
    const metadata: Record<string, string> = {};
    yamlStr.split(/\r?\n/).forEach((line) => {
      const idx = line.indexOf(":");
      if (idx > -1) {
        const key = line.slice(0, idx).trim();
        const value = line
          .slice(idx + 1)
          .trim()
          .replace(/^['"]|['"]$/g, "");
        if (key) metadata[key] = value;
      }
    });
    return { metadata, body };
  }

  const parsedMarkdown = computed(() => {
    const { metadata, body } = parseFrontMatter(skillMarkdown.value);
    return { metadata, body };
  });

  const renderedMarkdown = computed(() => {
    if (!parsedMarkdown.value.body) return "";
    try {
      return marked.parse(parsedMarkdown.value.body) as string;
    } catch (err) {
      console.error("Markdown 解析失败:", err);
      return parsedMarkdown.value.body;
    }
  });

  // Watch for changes in selectedSkillId or tab
  if (getActiveTab) {
    watch(
      [getSelectedSkillId, getActiveTab],
      async ([newSkillId, newTab]) => {
        if (newSkillId && newTab === "description") {
          await loadSkillMarkdown();
        }
      },
      { immediate: true }
    );
  } else {
    watch(
      getSelectedSkillId,
      async (newSkillId) => {
        if (newSkillId) {
          await loadSkillMarkdown();
        }
      },
      { immediate: true }
    );
  }

  return {
    skillMarkdown,
    isMarkdownLoading,
    parsedMarkdown,
    renderedMarkdown,
    loadSkillMarkdown,
  };
}

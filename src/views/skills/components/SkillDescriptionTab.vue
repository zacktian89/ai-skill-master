<script setup lang="ts">
defineProps<{
  isMarkdownLoading: boolean;
  skillMarkdown: string | null;
  parsedMarkdown: {
    metadata: Record<string, string>;
    body: string;
  };
  renderedMarkdown: string;
}>();
</script>

<template>
  <section class="description-pane">
    <div v-if="isMarkdownLoading" class="preview-loading">
      <span>加载中...</span>
    </div>
    <div v-else-if="skillMarkdown">
      <!-- Front matter metadata tags -->
      <div class="skill-meta-tags" v-if="Object.keys(parsedMarkdown.metadata).length">
        <div v-for="(val, key) in parsedMarkdown.metadata" :key="key" class="skill-meta-tag">
          <span class="skill-meta-tag-key">{{ key }}</span>:
          <span class="skill-meta-tag-val">{{ val }}</span>
        </div>
      </div>

      <!-- Markdown Body -->
      <div class="markdown-body" v-html="renderedMarkdown"></div>
    </div>
    <p v-else class="description-empty">暂无描述。</p>
  </section>
</template>

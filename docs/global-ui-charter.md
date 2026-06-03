# SkillMaster AI UI 开发宪章 (Global UI Coding Charter)

本宪章是后续 AI 模型在开发、修改和重构 SkillMaster 项目页面或组件时必须严格遵守的全局开发行为准则。

---

## 1. 核心开发原则 (Core Directives)

### 1.1 严禁硬编码与内联样式 (No Hardcoding & Inline Styles)
- **CSS 变量**: 严禁在 CSS 中硬编码具体的颜色值（如 `#111`、`#fff`）、具体的圆角半径或非标准的间距值。必须使用 [tokens.css](file:///d:/code/SkillMaster/src/styles/tokens.css) 中声明的 CSS 变量。
- **内联样式 (`style="..."`)**: Vue 模板中严禁写入静态内联样式。仅当涉及动态计算的布局（如可拖拽宽度计算等绑定变量的场景）时允许使用 `:style`。

### 1.2 严禁使用 Tailwind CSS (No Tailwind CSS)
- 本项目采用纯 Vanilla CSS 结合 CSS 自定义变量的方案进行样式开发。在编写 Vue 组件或 CSS 时，**严禁**引入 Tailwind 样式类或类似的实用工具类（如 `flex items-center justify-between p-4 bg-gray-100` 等）。

### 1.3 样式文件模块化与归档 (Modular Style Files)
- 严禁在 Vue 单文件组件中的 `<style>` 标签内写入大篇幅的样式代码。
- 所有的样式定义必须存放于以下结构：
  - 全局/布局级样式：放在 [layout.css](file:///d:/code/SkillMaster/src/styles/layout.css)。
  - 组件级通用样式：在 `src/styles/components/` 目录下新建独立的 `.css` 文件，并在 [components.css](file:///d:/code/SkillMaster/src/styles/components.css) 中通过 `@import` 导入。
  - 页面级/视图级样式：在 `src/styles/views/` 目录下新建同名 `.css` 文件，并在 [components.css](file:///d:/code/SkillMaster/src/styles/components.css) 中通过 `@import` 导入。

---

## 2. 主题与无障碍适配规范 (Theme & Accessibility)

- **双主题适配 (Dark & Light)**:
  在创建或修改页面时，必须确保所有文本颜色、背景色和边框颜色在深色（默认）和浅色模式下都具备良好的对比度。
  - **测试用例**: 每当修改 UI 时，AI 必须使用浏览器/预览工具分别在默认模式（Dark）和设置页切换到的 Light 模式下，验证界面的呈现效果和文字可读性。
  - **边框与阴影**: 浅色模式下为确保层次感，可合理调整容器的 `--border-default` 边框，但要避免大面积添加阴影。

---

## 3. 信息密度与减噪规范 (Density & Decluttering)

- **极致紧凑 (High Density)**:
  - 列表行宽、间隙、按钮高度（默认 34px）应严格对齐已有的 Codex 风格。
  - 使用 8px 网格（即使用 `--spacing-*`）配置页面元素的外边距与内边距。
- **文案减噪 (Zero Redundancy)**:
  - 页面不展示在其他层级（如侧边导航、父级容器标题）中已经明确表达的标题或 Eyebrow 说明。
  - 仅保留有实际区分价值的子项标题（例如“危险操作区域”）。

---

## 4. UI 修改与开发检查清单 (AI Checklist)

在提交任何 UI 代码修改或页面开发前，请按照以下清单进行自我核对：

| 检查项 | 验证条件 |
| :--- | :--- |
| **CSS 变量** | 是否所有的 `color`, `background`, `font-size`, `margin`, `padding`, `border-radius` 都使用了 `var(--*)`？ |
| **Tailwind 检查** | 检查 Vue 模板和 CSS，确认无任何 Tailwind CSS 语法或样式类引入？ |
| **双主题验证** | 切换到 Light 模式下，页面是否显示正常，没有出现不可读的白色背景白字或低对比度元素？ |
| **模块化 CSS** | 新样式是否提取到了 `src/styles/` 的相应组件/视图文件中，而没有滞留在 Vue 的 `<style scoped>` 中？ |
| **文案冗余** | 顶部区域是否清除了重复的 Eyebrow（如 `Skills` 列表顶部的 `Skills` 大标题）？ |
| **测试构建** | 终端执行 `npm run build` 是否无任何 CSS/Vue 编译错误？ |

# SkillMaster UI 风格设计规范

## 1. 核心设计理念

SkillMaster 采用 **Codex Dense UI** 风格。这是一种专为开发者和技术用户设计的安静、中性、紧凑的工具型界面。
其核心特征包括：
- **低视觉噪点**：移除不必要的彩色边框、大面积装饰和无意义的层级阴影，以低对比度的中性灰为基础。
- **高信息密度**：紧凑的内边距（padding）、较小的行高（line-height）与控件高度（例如按钮默认高度 34px），使界面在单屏内承载尽可能多的核心配置与数据。
- **零冗余文案**：依赖导航上下文和直观的控件形态来标示区域，避免重复的 Eyebrow（小标题）或多余的分类标题（例如在左侧侧边栏已激活 Skills 选项时，右侧主区域头部不再出现“Skills”或“技能库”的冗余标题）。

---

## 2. 主题与色彩系统 (Theme & Tokens)

应用支持 **Dark（深色）** 和 **Light（浅色）** 双主题，主题状态保存在 `localStorage` 中并通过根节点的 `data-theme` 属性生效。

核心变量定义在 [tokens.css](file:///d:/code/SkillMaster/src/styles/tokens.css) 中：

### 2.1 深色主题 (Dark Theme - 默认)
深色主题以中性暗色灰度为基础，摒弃了高饱和度或彩色的背景色。
- **应用背景 (App Background)**: `#1c1c1e` (`--bg-app`)
- **主工作区面板 (Main Panel)**: `#252528` (`--bg-main`, `--bg-detail-panel`, `--bg-panel`)
- **悬停与选中状态 (Hover & Selected)**:
  - 悬停：`rgba(255, 255, 255, 0.05)` (`--bg-hover`)
  - 选中：`rgba(255, 255, 255, 0.09)` (`--bg-selected`)
  - 列表行悬停：`#2a2a2e` (`--bg-hover-row`)
  - 列表行选中：`#333339` (`--bg-selected-row`)
- **文本颜色 (Text)**:
  - 主要文字：`#e4e4e7` (`--text-primary`)
  - 次要文字：`#a1a1aa` (`--text-secondary`)
  - 辅助/微弱文字：`#71717a` (`--text-tertiary`), `#52525b` (`--text-muted`)
  - 反色文字（用于高亮按钮）：`#1c1c1e` (`--text-inverse`)

### 2.2 浅色主题 (Light Theme)
浅色主题选用柔和、低蓝光的中性暖灰/米白色（Sepia-like tone），确保长时间使用的视觉舒适度。
- **应用背景**: `#f4f4f0` (`--bg-app`)
- **主工作区面板**: `#fafaf9` (`--bg-main`, `--bg-detail-panel`, `--bg-panel`)
- **侧边栏 (Sidebar)**: `#ebebe6` (`--bg-sidebar`)
- **列表面板 (List Panel)**: `#f2f2ed` (`--bg-list-panel`)
- **悬停与选中**:
  - 悬停：`rgba(0, 0, 0, 0.045)` (`--bg-hover`)
  - 选中：`rgba(0, 0, 0, 0.08)` (`--bg-selected`)
  - 列表行悬停：`#eaeae4` (`--bg-hover-row`)
  - 列表行选中：`#dedee4` (`--bg-selected-row`)
- **文本颜色**:
  - 主要文字：`#1c1917` (`--text-primary`)
  - 次要文字：`#57534e` (`--text-secondary`)
  - 辅助/微弱文字：`#78716c` (`--text-tertiary`), `#a8a29e` (`--text-muted`)
  - 反色文字：`#ffffff` (`--text-inverse`)

### 2.3 品牌色与状态色
- **品牌强调色 (Brand/Accent)**: 深色下使用橘黄/琥珀色系（`#d97706` / `--brand-500` 到 `#fbbf24` / `--brand-700`），浅色下使用暗琥珀色（`#b45309` / `--brand-500` 到 `#92400e` / `--brand-700`）。
- **成功状态 (Success)**: 深色下为绿色字配低对比绿底，浅色下为高对比绿字配浅绿底。
- **警告状态 (Warning)**: 黄褐色。
- **危险/错误状态 (Danger)**: 淡红/红褐色。
- **离线/未连接 (Offline)**: 中性灰。

---

## 3. 间距与网格系统 (Spacing & Grid)

基于 **8px (8pt) 网格** 的梯度尺度，严禁随意输入像素值：
- `--spacing-2xs`: `4px`
- `--spacing-xs`: `8px`
- `--spacing-sm`: `10px`
- `--spacing-md`: `12px`
- `--spacing-lg`: `14px`
- `--spacing-xl`: `16px`
- `--spacing-2xl`: `18px`
- `--spacing-3xl`: `20px`
- `--spacing-4xl`: `24px`
- `--spacing-5xl`: `28px`
- `--spacing-6xl`: `32px`
- `--spacing-7xl`: `40px`

开发布局时必须通过 `var(--spacing-*)` 引用以上变量，用于控制 padding、margin 和 grid-gap。

---

## 4. 字体与排版 (Typography)

### 4.1 字体栈 (Font Family)
```css
font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "PingFang SC", "Segoe UI", sans-serif;
```
优先调用系统原生字体，禁止引入外部自定义字体文件。

### 4.2 字号级别 (Font Size)
- `--font-size-xs`: `11px` (极小辅助文字、徽章)
- `--font-size-sm`: `12px` (小标签、辅助说明)
- `--font-size-md`: `13px` (表单说明、状态项)
- `--font-size-lg`: `14px` (正文、输入框内容、标准按钮文本)
- `--font-size-xl`: `15px` (导航节点名称)
- `--font-size-2xl`: `18px` (子标题、卡片标题)
- `--font-size-3xl`: `22px` (区域/详情大标题)
- `--font-size-4xl`: `24px` (最大标题级别)

### 4.3 字重级别 (Font Weight)
- `--font-weight-regular`: `400`
- `--font-weight-medium`: `550` (用于中度强调)
- `--font-weight-semibold`: `600` (重度强调)
- `--font-weight-bold`: `650` (标题级字重)

---

## 5. 圆角与边框 (Radius & Border)

- **边框样式**: 采用超低对比的中性半透明线条，深色下为 `rgba(255, 255, 255, 0.065)` (`--border-default`)。
- **圆角规则**:
  - `--radius-xs`: `4px` (小徽章、折叠图标)
  - `--radius-sm`: `6px` (小卡片、下拉项)
  - `--radius-md`: `7px` (应用标志、中型元素)
  - `--radius-lg`: `8px` (标准表单控件、标准卡片、按钮)
  - `--radius-xl`: `10px` (按钮或轻量弹框)
  - `--radius-2xl`: `12px` (大面板、主弹窗)

---

## 6. 基础布局结构 (Layout Architecture)

页面布局规则详细定义在 [layout.css](file:///d:/code/SkillMaster/src/styles/layout.css) 中，核心架构层次如下：
1. **主外壳 (`.app-shell`)**: 采用 grid 布局，第一列为固定或可折叠的导航 rail，第二列为工作区。
2. **侧边导航栏 (`.sidebar-rail`)**: 宽度由变量 `--sidebar-width` 动态控制，具备 `.sidebar-rail--collapsed` 收起状态，右侧包含可拖拽大小的 `.sidebar-resize-handle`。
3. **工作区外壳 (`.workspace-shell`)**: 包裹实际应用内容，提供统一的背景。
4. **分栏布局 (`.split-content`)**: 经典的双栏设计，左侧为固定/限制宽度的列表面板 (`.list-panel`)，右侧为自适应宽度的详情面板 (`.detail-panel`)。

---

## 7. 预设组件规范 (Standard Components)

所有常用组件的样式都在 [components.css](file:///d:/code/SkillMaster/src/styles/components.css) 中声明并被模块化拆分：

- **按钮 (Buttons)**:
  - 核心包含 `.primary-button` (高亮强调色)、`.secondary-button` (默认中性暗色)、`.danger-button` (警告红) 和 `.ghost-icon-button` (无边框悬浮图标按钮)。
- **表单布局 (Form Layouts)**:
  - 字段栈 `.field-stack`：垂直排列标签 (`label`)、控件、辅助说明文字 (`small`)，使用标准 8px 间隙。
  - 选择瓦片 `.target-tile`：大面积的快速配置卡片，悬停时产生中性灰亮度跃迁。
- **状态徽章 (Status Tags)**:
  - `.status-tag`：支持 `.status-tag--success`、`.status-tag--danger`、`.status-tag--warning`、`.status-tag--offline`，用于以小字标示各类状态。

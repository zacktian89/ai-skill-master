# SkillMaster

[English](README.en.md)

SkillMaster 是一个本地桌面应用，用来集中管理、导入、预览和分发 agent skills。它把分散在不同 agent 或项目里的 `SKILL.md` 目录统一收纳到本地技能库，再通过 SkillMaster 管理的引用链接同步到 Codex、Claude Code 等目标目录。

![Skills view](docs/screenshots/skills.png)

## 功能介绍

- 技能库管理：导入本地文件夹或 GitHub 仓库中的 skills，预览 `SKILL.md` 和 README 内容，删除前查看影响范围。
- 技能引用：为单个 skill 添加或移除到用户目录、项目目录或自定义目录的托管引用。
- 项目管理：添加本地项目，扫描项目内已有 skills，为项目设置启用、禁用或继承默认规则。
- 智能体管理：维护不同 agent 的 skills 根目录，并把技能关联到对应 agent。
- 插件查看：识别已安装插件，查看插件包含的 skills、MCP 服务和支持的 agent。
- 商店浏览：搜索或查看 skills.sh 技能榜单，并从商店条目进入下载导入流程。
- 系统设置：切换深色/浅色主题、切换界面语言、查看存储路径并迁移技能库。

## 截图

### 项目规则

![Projects view](docs/screenshots/projects.png)

### 智能体目录

![Agents view](docs/screenshots/agents.png)

### 插件详情

![Plugins view](docs/screenshots/plugins.png)

### 系统设置

![Settings view](docs/screenshots/settings.png)

## 安装使用

### 1. 下载安装包

打开 [SkillMaster 安装包目录](https://github.com/zacktian89/ai-skill-master/tree/main/releases/v0.1.0)，下载适合当前系统的安装包。

Windows 用户可以直接下载：

- [SkillMaster_0.1.0_x64-setup.exe](https://github.com/zacktian89/ai-skill-master/raw/main/releases/v0.1.0/SkillMaster_0.1.0_x64-setup.exe)
- [SkillMaster_0.1.0_x64_en-US.msi](https://github.com/zacktian89/ai-skill-master/raw/main/releases/v0.1.0/SkillMaster_0.1.0_x64_en-US.msi)

### 2. 安装并启动

运行下载的安装包，按系统提示完成安装，然后从开始菜单或桌面快捷方式启动 SkillMaster。

### 3. 导入 skill

1. 打开「技能」页面。
2. 点击新增按钮。
3. 选择本地包含 `SKILL.md` 的文件夹，或输入 GitHub 仓库地址。
4. 在预览列表中确认要导入的 skills。
5. 完成导入后，在详情页查看说明和引用状态。

### 4. 关联到 agent 或项目

1. 在「智能体」页面添加 agent 的 skills 根目录，或在「项目」页面添加项目目录。
2. 选择需要关联的 skill。
3. 使用「增加技能」或「新增引用」把 skill 链接到目标目录。
4. 后续删除引用时，SkillMaster 只移除托管引用，不会删除技能库中的原始 skill。

### 5. 调整项目规则

1. 打开「项目」页面。
2. 选择项目。
3. 为每个 skill 设置启用、禁用或继承默认规则。
4. 项目上下文会影响后续同步计算。

## 从源码运行

如果需要参与开发或在本地调试，可以使用源码方式运行。

### 1. 安装依赖

```powershell
npm install
```

### 2. 启动开发环境

```powershell
npm run dev       # 仅启动 Vite 浏览器预览
npm run build     # 类型检查并构建前端
npm test          # 运行前端测试
npm run tauri dev # 启动 Tauri 桌面开发环境
```

Rust 侧测试：

```powershell
Set-Location src-tauri
cargo test
Set-Location ..
```

构建安装包：

```powershell
npm run tauri build
```

## 技术栈

- Vue 3 + TypeScript
- Vite
- Tauri 2
- Rust
- Vitest

## 项目结构

```text
src/                 Vue 前端源码
src/api/             前端 API 封装和浏览器 mock 数据
src/components/      通用界面组件
src/views/           Skills、Store、Agents、Plugins、Projects、Settings 页面
src-tauri/           Tauri 与 Rust 后端命令
docs/                设计文档、验证记录和 README 截图
public/              图标和静态资源
```

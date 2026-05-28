# SkillMaster 设计规格

日期：2026-05-28

## 目标

SkillMaster 是一个本地可安装的跨平台桌面应用，用来管理 agent skill。
第一版只做一个小而完整的闭环：

- 快速查看本地技能库中有哪些 skill。
- 从本地导入并管理 skill。
- 通过一个应用管理的技能库作为唯一来源。
- 按默认状态启用或停用 skill。
- 按项目文件夹设置单独规则。
- 将生效的 skill 同步到 Codex。

第一版支持 Windows 和 macOS。

## 产品原则

- 用户能理解的概念要少。
- 低频配置放到 Settings。
- 首屏优先回答“有哪些 skill，默认是否启用”。
- 只管理 SkillMaster 自己创建的文件链接。
- 不覆盖用户原本放在 Codex 目录里的 skill。
- 项目级规则要明确、可逆、容易恢复默认。

用户主要理解三个概念：

- 技能库：SkillMaster 管理的本地 skill 集合。
- 默认启用：没有项目规则时，skill 是否生效。
- 项目规则：某个项目跟随默认、单独启用、单独停用。

实现中可以存在适配器、唯一来源、托管链接、状态清单等概念，但界面首屏不主动展示这些词。

## 技术路线

使用 Tauri + Vue。

- Vue 负责左右两栏桌面界面。
- Tauri 命令负责文件系统操作、路径检测、链接创建、链接移除、导入、删除、迁移和诊断。
- Rust 负责会影响本地文件系统的操作。
- 第一版使用本地 JSON 文件保存应用状态。

选择 Tauri 的原因：应用是本地优先、跨平台、文件系统操作较多，Tauri 的体积和权限模型更适合这个工具。

## 范围

### 第一版包含

- Windows 和 macOS 可安装应用。
- 应用管理的默认技能库目录。
- 支持迁移技能库到自定义目录。
- 导入包含 `SKILL.md` 的本地 skill 文件夹。
- 删除由 SkillMaster 管理的 skill。
- 每个 skill 可设置默认启用或默认停用。
- 添加项目文件夹。
- 每个项目可设置 skill 规则：跟随默认、在此项目启用、在此项目停用。
- 第一版内置 Codex 同步能力。
- 自动检测 Codex skills 目录，检测失败时允许手动选择。
- 检测 Codex 目录中的同名冲突。
- 为 SkillMaster 管理的 skill 创建和移除链接。

### 第一版不包含

- 远程 skill 市场。
- skill 版本管理。
- skill 依赖解析。
- skill 自动更新。
- Claude Code 和 CodeBuddy 内置支持。
- 多设备同步。
- 云备份。

## 界面设计

应用采用类似 Codex 的安静工具型两栏布局。

### 左栏

左栏是固定窄侧边栏，使用浅色背景。
顶层入口只保留三个：

- Skills
- Projects
- Settings

Codex 连接和同步细节放到 Settings 中，不作为顶层入口。

左栏可以在合适场景展示轻量对象列表，例如项目列表或 skill 分组。首屏避免出现实现术语。

### 右栏

右栏是主工作区，包含简单标题栏和当前内容。

选择 Skills 时展示：

- 搜索框。
- skill 列表。
- 默认启用开关。
- Codex 同步或冲突的小状态标记。
- 导入 skill 操作。

选择某个 skill 时展示：

- 从 `SKILL.md` 读取的名称和描述。
- 技能库路径。
- 默认启用状态。
- Codex 同步状态。
- 仅在异常时展示冲突详情。

选择 Projects 时展示：

- 项目文件夹列表。
- 添加项目操作。
- 当前项目选择。
- 当前项目下的 skill 规则。

项目规则文案固定为：

- 跟随默认。
- 在此项目启用。
- 在此项目停用。

选择 Settings 时展示：

- 技能库位置。
- 迁移技能库。
- Codex 连接。
- Codex skills 目录。
- 同步和冲突诊断。
- 状态文件位置。

## 首次启动

首次启动流程：

1. 创建默认技能库目录。
2. 创建应用状态文件。
3. 检测 Codex skills 目录。
4. 检测成功时打开 Skills 首页。
5. 检测失败时仍然打开 Skills 首页，并显示一条轻量提示，引导用户到 Settings 设置 Codex 目录。

除非必要目录无法创建，应用不使用阻塞式向导。

## 数据模型

第一版使用一个 JSON 状态文件，放在应用配置目录。
技能库保存 skill 文件夹本体，状态文件保存启用规则、项目规则和外部链接状态。

示例结构：

```json
{
  "schemaVersion": 1,
  "skillLibraryPath": "...",
  "codexSkillsPath": "...",
  "skills": [
    {
      "id": "markdown-go",
      "name": "markdown-go",
      "description": "Convert Markdown to WeChat HTML",
      "libraryPath": ".../skills/markdown-go",
      "defaultEnabled": true,
      "managedLinks": {
        "codex": "..."
      }
    }
  ],
  "projects": [
    {
      "id": "project-hash",
      "name": "SkillMaster",
      "path": "D:/code/SkillMaster",
      "rules": {
        "markdown-go": "disable"
      }
    }
  ]
}
```

项目规则取值：

- `inherit`
- `enable`
- `disable`

如果某个项目没有配置某个 skill，该 skill 在该项目中跟随默认启用状态。

## 同步规则

### 默认状态

当 skill 默认启用时，SkillMaster 从技能库中的 skill 文件夹创建链接到 Codex skills 目录。

当 skill 默认停用时，SkillMaster 只移除状态文件中记录的 SkillMaster 托管链接。

### 项目规则

项目规则优先于默认状态。

第一版由 SkillMaster 保存项目规则。用户把某个项目设为当前上下文后，SkillMaster 计算该项目下真正生效的 skill，然后同步 Codex skills 目录。

这样可以在 Codex 暂无项目级 skill 目录机制的情况下，实现项目级启用和停用。

### 冲突处理

如果 Codex skills 目录中已经存在同名文件夹，且它不是 SkillMaster 管理的链接，SkillMaster 不覆盖它。

应用会把该 skill 标记为冲突，并在 skill 详情和 Settings 诊断中展示。

### 链接归属

SkillMaster 只移除状态文件中记录的 `managedLinks`。
不会删除 Codex 目录里的普通文件夹或用户自己创建的 skill。

## Skill 导入

导入条件：

- 文件夹存在。
- 文件夹包含 `SKILL.md`。
- 导入后的 skill id 不和已有托管 skill 冲突。

导入时，SkillMaster 将该文件夹复制到技能库。
导入后，技能库中的副本成为 SkillMaster 管理的来源。

## Skill 删除

删除 skill 时移除：

- 技能库中的 skill 文件夹。
- 该 skill 的 SkillMaster 托管链接。
- 所有项目中针对该 skill 的规则。

删除前显示确认，说明将影响哪些托管资源。

## 技能库迁移

用户可以在 Settings 中迁移技能库。

迁移流程：

1. 选择新的目标目录。
2. 检查写入权限和命名冲突。
3. 复制 skill 文件夹到目标目录。
4. 复制成功后更新状态文件。
5. 重新计算托管链接目标。
6. 用户确认后重新同步 Codex 链接。

迁移失败时，继续使用旧技能库。

## 异常处理

- 技能库缺失：在 Settings 中展示修复、重新选择或重建入口。
- 状态文件损坏：优先尝试读取备份，失败后提示重建。
- Codex 路径缺失：skill 仍可管理，Codex 显示未连接。
- 链接创建失败：展示文件系统错误原因，并保留原状态。
- Codex 目录已有非托管同名文件夹：标记冲突，不覆盖。
- 导入目录无效：没有 `SKILL.md` 时拒绝导入。

## 测试重点

- 导入合法 skill。
- 拒绝没有 `SKILL.md` 的目录。
- 删除托管 skill。
- 切换默认启用状态。
- 创建和移除 Codex 链接。
- 检测 Codex 同名冲突。
- 保留 Codex 目录中非托管文件夹。
- 项目规则优先于默认状态。
- 选择项目上下文后同步真正生效的 skill。
- 迁移技能库后路径正确。
- Codex 路径缺失时应用仍可打开和管理 skill。

## 成功标准

第一版完成时应满足：

- 用户可以在 Windows 和 macOS 打开桌面应用。
- 应用可以创建并展示本地技能库。
- 用户可以导入包含 `SKILL.md` 的 skill 文件夹。
- 用户可以设置 skill 的默认启用状态。
- 应用可以检测或手动配置 Codex skills 目录。
- 应用可以通过创建托管链接把启用的 skill 同步到 Codex。
- 应用只移除 SkillMaster 托管链接。
- 用户可以添加项目文件夹并设置项目规则。
- 选择项目上下文后，Codex 按项目规则同步。
- 冲突清晰可见，且不会通过覆盖用户文件夹来解决冲突。

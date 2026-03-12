# Oclaw

> 专为 AI Agent 打造的桌面浏览器，配套 OpenClaw 使用。

---

## 这是什么

Oclaw 是一款基于 **Tauri 2 + Vue 3** 的轻量桌面浏览器，核心目的只有一个：**让 AI Agent 能够操控浏览器完成网页任务**。

普通浏览器是给人用的，Oclaw 是给 AI 用的——同时，人也可以坐在旁边看着，随时接管。

---

## 功能截图

### 一键安装 OpenClaw

首次启动时，若检测到本地 OpenClaw 尚未运行，应用会自动弹出安装向导。点击「开始安装」，自动完成 Node.js 检测、OpenClaw 安装全流程，实时显示终端输出。

![安装向导](public/document/1.png)
![安装中](public/document/2.png)

安装完成后，提供两种初始化方式：**可视化配置**（推荐）和**内嵌终端**。

![选择初始化方式](public/document/3.png)

### 可视化配置向导

无需手动编辑配置文件，向导逐步引导完成模型提供商选择、API Key 填写等所有配置项。

![初始化确认](public/document/4.png)
![配置向导](public/document/5.png)

配置完成后，Gateway 自动启动，点击「开始使用」即可。

![初始化完成](public/document/6.png)

### 设置页

管理 OpenClaw 连接配置，支持一键自动获取 Token、检测连接状态，以及 Gateway 配置检测与自动修复。

![设置页](public/document/7.png)
![设置页-已连接](public/document/8.png)

### AI 对话控制台

点击右上角「OpenClaw」进入对话界面，直接向大虾发出任务指令，实时流式输出思考过程。

![AI 对话](public/document/9.png)

### 技能管理

内置技能管理页，查看、安装、新建 OpenClaw 技能，支持在线编辑技能内容。

![技能管理](public/document/10.png)

---

## 主要功能

- **AI 对话控制台** — 内置与 OpenClaw 通信的对话界面，支持流式输出，支持临时会话模式
- **多标签浏览** — 多 Tab 并行浏览，支持网址直跳和关键词搜索
- **身份隔离** — 默认 / 工作 / 个人三套浏览器 Profile，Cookie 互不干扰
- **一键安装向导** — 自动检测环境，智能选择安装策略，无需手动折腾终端
- **可视化配置** — 图形界面完成 OpenClaw 初始化，无需手动编辑配置文件
- **Gateway 管理** — 一键检测连接状态、自动修复配置、重启 Gateway
- **技能管理** — 查看和管理 OpenClaw 技能，支持在线编辑

---

## 安装策略

向导会根据你的环境自动选择最合适的策略：

| 你的环境 | 安装策略 |
|---------|---------|
| 已有 Node.js ≥ 22 | 直接用系统 npm 全局安装，零侵入 |
| 已有 fnm | 用你的 fnm 安装 Node.js 22 |
| 已有 nvm | 用你的 nvm 安装 Node.js 22 |
| 以上均无 | 用应用内置 fnm 自动安装（独立隔离目录） |

安装完成后，`openclaw` 命令自动写入终端全局 PATH。

---

## 使用方式

1. 下载并启动 Oclaw
2. 按向导提示完成 OpenClaw 安装与初始化（约 1 分钟）
3. 点击右上角 **OpenClaw** 打开对话框
4. 对大虾说出你的任务

---

## 下载

前往 [Releases](../../releases) 下载对应平台的安装包（macOS / Windows）。

---

## macOS 安装后无法打开？

由于应用暂未签名，macOS 会阻止首次启动。解除方法：

**1. 打开终端，输入以下命令（末尾有空格，先不要按回车）：**

```
sudo xattr -rd com.apple.quarantine
```

**2. 将 Oclaw 应用图标直接拖入终端窗口**

**3. 按回车，输入开机密码，再次回车**

**4. 重新打开应用即可**

---

## 本地开发

```bash
pnpm install
pnpm tauri dev
```

构建：

```bash
pnpm tauri build
```

---

## 相关项目

- [OpenClaw](https://github.com/OpenClaw) — AI Agent 平台，大虾的大本营

# Claw Browser

> 专为 AI Agent 打造的桌面浏览器，配套 OpenClaw 使用。

---

## 这是什么

Claw Browser 是一款轻量桌面浏览器，核心目的只有一个：**让 AI Agent 能够操控浏览器完成网页任务**。

普通浏览器是给人用的，Claw Browser 是给 AI 用的——同时，人也可以坐在旁边看着，随时接管。

---

## 张大妈是谁

**张大妈**是内置在 OpenClaw 中的 AI 网页助手角色。当你对 OpenClaw 说：

- 「张大妈帮我查一下…」
- 「大妈帮我比比价格…」
- 「大虾帮我看看这个…」

张大妈就会接管 Claw Browser，帮你完成搜资讯、比价格、追热点、提取内容、填写表单……一系列网页任务。

你不需要动手，盯着屏幕看就行。

---

## 核心能力

**AI 可以做什么：**

- 打开任意网页、搜索内容
- 点击按钮、填写表单、选择下拉框
- 滚动页面、等待元素加载
- 提取列表数据、读取文章正文
- 执行任意 JavaScript

**人随时可以：**

- 接管浏览器，手动操作
- AI 暂停等待，不干扰
- 操作完成后，让 AI 继续

---

## 使用方式

1. 启动 Claw Browser
2. 打开 OpenClaw，连接到本地浏览器
3. 对张大妈说出你的任务

浏览器会在后台自动完成操作，你只需要等结果。

---

## 下载

前往 [Releases](../../releases) 下载对应平台的安装包（macOS / Windows）。

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

- [OpenClaw](https://github.com/OpenClaw) — AI Agent 平台，张大妈的大本营

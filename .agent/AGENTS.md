# Claw Browser — Agent 说明

本仓库为 **Claw Browser** 的工程，与 OpenClaw 结合使用，采用干净骨架，便于在此基础上扩展与 OpenClaw 相关的浏览器能力。

## 角色与目标

- **项目定位**：为与 OpenClaw 配合提供桌面浏览器壳（Tauri + Vue），不承载原业务逻辑，仅保留可运行的最小框架。
- **协作方式**：在实现新功能、修 bug 或重构时，优先遵循本仓库约定与 `.agent/skills/` 下技能说明。

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue 3 (Composition API) + TypeScript + Vite |
| 路由 | vue-router + unplugin-vue-router（文件路由，`src/pages/`） |
| 布局 | vite-plugin-vue-layouts（`src/layouts/`，默认 `default`） |
| 状态 | Pinia（`src/stores/`） |
| UI/样式 | Element Plus（按需）+ UnoCSS |
| 桌面壳 | Tauri 2，Rust 侧仅保留最小命令（如 `greet`），便于后续扩展 |

## 目录约定

- `src/pages/`：URL 路由页（unplugin-vue-router 自动注册），目前只有首页 `index.vue`。
- `src/views/`：应用级全屏面板（OpenclawPage、SettingsPage、SetupPage、SkillsPage），通过 Pinia `specialView` 状态切换，不注册 URL 路由。
- `src/components/`：小型可复用 UI 组件（TabBar、AIConsole 等）。
- `src/layouts/`：布局组件，`default.vue` 为默认布局，内含 specialView 条件渲染逻辑。
- `src/stores/`：Pinia stores（tabs、settings、profile、recording、installer、openclaw 等）。
- `src/api/`：Tauri invoke 封装层，按领域分文件。
- `src-tauri/src/`：Rust 后端，各功能模块化（api.rs HTTP 服务、webview/、installer、skills 等）。

## 扩展时注意

- 新增 Tauri 能力：在 `lib.rs` 中增加 `#[tauri::command]`，并在 `generate_handler![]` 中注册；前端通过 `@tauri-apps/api/core` 的 `invoke` 调用。
- 新增页面：在 `src/pages/` 下新增 `.vue`，路由自动生成。
- 项目专属的 AI 行为与约定见 **`.agent/skills/`** 下的技能文件。

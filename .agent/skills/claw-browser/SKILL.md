---
name: claw-browser
description: Claw Browser 项目开发约定与 Tauri/Vue 架构规范。在为本仓库添加功能、调用 Tauri 命令、改路由或状态时使用；新增代码必须符合本技能中的前端与 Rust 规范。
---

# Claw Browser 开发技能

## 项目身份

- 仓库：Claw Browser，与 OpenClaw 结合使用的桌面浏览器，干净骨架，无历史业务逻辑。
- 栈：Tauri 2 + Vue 3 + TypeScript + Vite + Pinia + UnoCSS + Element Plus。

---

## 一、前端架构规范

### 1.1 目录结构（必须遵守）

```
src/
├── api/              # Tauri 命令封装层，按领域分文件，不直接在组件/页面里 invoke
├── components/       # 通用 Vue 组件，可被多页面复用
├── composables/     # 组合式逻辑（URL 解析、搜索、侧边栏等），页面只做组装
├── layouts/         # 布局组件，default.vue 为默认
├── pages/           # 页面组件，文件即路由（unplugin-vue-router）
├── stores/          # Pinia stores，可被 unplugin-auto-import 自动导入
├── types/           # 全局 TypeScript 类型与接口
└── styles/          # 全局样式，main.css 入口
```

- **禁止**：在 `pages/` 或 `components/` 中直接 `import { invoke } from '@tauri-apps/api/core'` 并写业务型 invoke 调用；应通过 `src/api/` 封装后使用。
- **禁止**：在页面里写大段纯逻辑（如 URL 判断、搜索串处理）；应抽到 `composables/` 或 `utils/`。

### 1.2 API 层（Tauri 调用）

- 所有与 Tauri 的通信集中在 `src/api/`，按领域分文件，例如：
  - `src/api/webview.ts`：create_tab_webview、show_webview、hide_webview、close_webview、resize_all_webviews、eval_in_webview、get_dom_snapshot
  - `src/api/sidecar.ts`：test_sidecar 等 sidecar 相关
  - `src/api/greet.ts` 或合并到 `src/api/app.ts`：greet 等杂项
- 每个 API 函数要有明确 TypeScript 类型（参数与返回值），并在 `types/` 或同文件内定义接口。
- 示例：

```ts
// src/api/webview.ts
import { invoke } from '@tauri-apps/api/core'

export async function createTabWebview(label: string, url: string): Promise<void> {
  await invoke('create_tab_webview', { label, url })
}
```

- Store 或 composable 中只调用这些 API 函数，不直接 invoke 命令名。

### 1.3 组件与页面约定

- **Composition API**：统一使用 `<script setup lang="ts">`，不使用 Options API。
- **页面**：`src/pages/` 下 `.vue` 即路由，新页面放在 `pages/` 或 `pages/子目录/`，文件名与期望路径一致；页面只负责布局与组装，业务逻辑放在 composables 或 stores。
- **组件**：可复用 UI 放在 `src/components/`，命名 PascalCase；通过 props/emits 或 store 通信，不直接调用 Tauri。
- **状态**：需要跨组件/跨页面的状态用 Pinia stores（`src/stores/`）；仅单页内的状态用 `ref`/`reactive`。

### 1.4 Composables

- 可复用逻辑（如「是否像 URL」「归一化 URL」「从输入得到目标 URL」）放在 `src/composables/`，以 `use` 前缀命名（如 `useUrlInput`）。
- 返回 `ref`/`reactive` 与方法，供页面或组件组合使用，避免在页面里写一长串纯函数。

### 1.5 类型

- 全局类型、与 Tauri 返回一致的接口放在 `src/types/`，或在对应 `api/`、`stores/` 旁用 `.ts` 导出。CSS 与 UI 约定见下文「四、CSS 与 UI 使用规范」。

### 1.6 新增前端功能时的检查

- [ ] 新页面在 `src/pages/`（或子目录），且仅做组装与布局。
- [ ] 新 Tauri 调用已封装到 `src/api/` 对应领域文件，并写好类型。
- [ ] 纯逻辑已抽到 `composables/` 或 `utils/`，不在页面堆逻辑。
- [ ] 新全局状态在 `src/stores/` 中定义。

---

## 二、Rust 架构规范

### 2.1 模块化目标结构（必须朝向此结构演进）

**禁止**：把所有命令、常量、辅助函数全部写在 `lib.rs` 单文件中。

目标目录与模块划分：

```
src-tauri/src/
├── main.rs           # 仅作入口，调用 lib::run()
├── lib.rs            # 只做：mod 声明、插件注册、generate_handler![] 汇总、run()
├── bridge.js         # 注入到 webview 的脚本，保持不变
├── config.rs         # 常量与配置（如 TAB_BAR_HEIGHT、LEFT_PANEL_WIDTH）
├── webview/          # 与 webview 相关的逻辑
│   ├── mod.rs
│   ├── rect.rs       # 如 calc_webview_rect
│   ├── commands.rs   # create_tab_webview, show/hide/close_webview, resize_all, eval_in_webview, get_dom_snapshot
│   └── navigation.rs # on_navigation 相关解析（claw://webview-click、claw://dom-snapshot 等）
├── sidecar.rs        # test_sidecar 等 sidecar 命令
└── app.rs            # greet、on_webview_click 等杂项命令
```

- `lib.rs` 中只保留：`mod config; mod webview; mod sidecar; mod app;` 以及 `tauri::Builder::default().plugin(...).invoke_handler(tauri::generate_handler![...]).run(...)`，具体命令实现全部在对应模块中。
- 新命令按领域放入 `webview/commands.rs`、`sidecar.rs` 或 `app.rs`，并在 `lib.rs` 的 `generate_handler![]` 中注册。

### 2.2 命令与错误处理

- 每个 `#[tauri::command]` 函数放在对应模块中，命名清晰（如 `create_tab_webview` 在 webview 模块）。
- 返回 `Result<T, E>` 时，`E` 建议使用 `String` 或项目内统一 error 类型；错误信息应对前端可读，便于弹窗或日志。
- 需要 `AppHandle` 时，签名为 `fn 命令名(app: AppHandle, ...)`，由 Tauri 注入。

### 2.3 常量与配置

- 魔法数字（如 88.0、320.0）一律放到 `config.rs`（或 `config/mod.rs`），通过常量命名表达含义，例如 `TAB_BAR_HEIGHT`、`LEFT_PANEL_WIDTH`。
- 新功能涉及尺寸、超时等配置时，在 config 中新增常量，不在命令或 webview 逻辑里硬编码。

### 2.4 新增 Tauri 命令时的检查

- [ ] 命令实现写在对应领域模块（webview/commands、sidecar、app），而不是 `lib.rs`。
- [ ] 在 `lib.rs` 的 `generate_handler![..., 新命令]` 中注册。
- [ ] 新常量已加入 `config.rs`。
- [ ] 前端已通过 `src/api/` 封装并带类型调用。

---

## 三、Tauri 与前端协作（简要）

- **命令定义**：Rust 侧在对应模块中写 `#[tauri::command] fn 命令名(...) -> ...`，在 `lib.rs` 的 `tauri::generate_handler![]` 中注册。
- **前端调用**：仅通过 `src/api/` 封装函数调用，内部使用 `import { invoke } from '@tauri-apps/api/core'` 与 `await invoke('命令名', { 参数 })`。
- **依赖**：新 Rust 依赖加在 `src-tauri/Cargo.toml`；插件用 `tauri_plugin_*`，在 `lib.rs` 的 `Builder::default().plugin(...)` 中初始化。

---

## 四、CSS 与 UI 使用规范

基于当前项目：UnoCSS（`unocss.config.ts`）+ 组件内 scoped CSS，Element Plus 按需、图标用 unplugin-icons + 本地 `src/assets/icons`。新增或修改样式/UI 时须符合下列约定。

### 4.1 样式策略与优先级

- **优先使用 UnoCSS 原子类**：能用 `flex`、`gap-4`、`text-secondary-500`、`rounded-lg` 等表达的，写在模板的 class 中，避免在 scoped 里重复写相同含义的 CSS。
- **复杂或组件专属样式**：用 `<style scoped>` + 类名（见 4.2），可与 UnoCSS 混用；同一块 UI 内风格统一（要么原子类为主，要么 scoped 为主，不混用同一属性两处写）。
- **颜色/间距**：优先用 UnoCSS theme，避免散落硬编码色值。
  - 主题 token 见 `unocss.config.ts` → `theme.colors`：`primary`（蓝）、`secondary`（紫，当前品牌主色 #5f47ce）、`accent`（红）、`neutral`（灰）。
  - 原子类示例：`text-primary`、`text-secondary-500`、`bg-secondary-50`、`border-secondary-200`、`text-accent-500`（错误/危险）。
  - 若在 scoped 中写色值，请与 theme 一致（如 `#5f47ce` 对应 secondary-500），或使用 `theme.colors` 中已有值，便于后续统一换肤。
- **快捷类**：项目在 UnoCSS 中配置了 `btn`、`btn-plain` 等 shortcut，通用按钮可优先使用；自定义 shortcut 加在 `unocss.config.ts` 的 `shortcuts` 中并写清注释。
- **全局样式**：仅放在 `src/styles/main.css`，不在组件内写无 scoped 的全局类；如需全局变量，在 `main.css` 用 `:root` 定义。

### 4.2 类名与 scoped 约定

- **类名**：BEM 或语义化 kebab-case 块名，与现有风格一致。例如：`.layout`、`.layout-body`、`.tab-bar`、`.tab-item`、`.tab-item.active`、`.ai-console-header`。
- **命名**：同一组件内块名统一前缀（如 `ai-console-*`、`search-bar`），避免单字母或无意义缩写；状态/修饰用 `--modifier` 或 `.is-active` 等。
- **scoped**：组件内只用 `<style scoped>`，不新增全局 `<style>`；深度选择器仅在必要时使用（如 `:deep(.el-input__inner)` 覆盖 Element 内部样式）。

### 4.3 布局与响应式

- **布局**：与现有布局一致：顶部 TabBar（44px）、主体 flex、左侧边栏 320px（`.layout-sidebar`）、右侧内容区 flex:1；尺寸常量若多处使用，考虑放到 UnoCSS theme 或 `main.css` 变量。
- **响应式**：使用 UnoCSS 的 `mobile:` 等 variants（见 `unocss.config.ts` 的 `variants`）；或使用 Uno 预设的 breakpoint 类（如 `md:flex`）。需与现有 `pc:/app:` 等环境前缀区分用途（环境前缀用于运行时 class，媒体查询用于纯 CSS 响应式）。

### 4.4 Element Plus 使用

- **按需使用**：项目已通过 unplugin-vue-components + ElementPlusResolver 按需引入，无需在 `main.ts` 全量 `app.use(ElementPlus)`。
- **使用场景**：表单（ElForm、ElInput、ElSelect 等）、反馈（ElMessage、ElMessageBox）、弹窗（ElDialog）、表格（ElTable）等复杂 UI 时优先用 Element 组件，保证一致性与可维护性；简单按钮/链接可用原生 + UnoCSS 或 `btn`/`btn-plain`。
- **反馈**：Toast、确认框用 `ElMessage`、`ElMessageBox`（已配置 auto-import），不手写 div 弹窗。
- **样式覆盖**：尽量通过 Element 的 props（如 `size`、`type`）和 UnoCSS 类控制；必须改内部样式时用 scoped + `:deep()`，并注明原因，避免污染全局。

### 4.5 图标

- **优先**：本地 SVG 集合 `src/assets/icons`，通过 unplugin-icons 的 `icon-local-*` 使用（如 `<icon-local-shi-du />`）；图标会经 loader 处理为 currentColor，便于与文字同色。
- **复杂/动态**：使用 `@iconify/vue` 的 `<Icon>`（已配置 IconParkResolver），按需传入 icon 名。
- **内联 SVG**：仅当图标与交互强绑定或需精确控制 viewBox 时在模板内写 `<svg>`，并保持 `fill="currentColor"` 或 `stroke="currentColor"` 以继承颜色。

### 4.6 新增/修改 UI 时的检查

- [ ] 颜色/间距优先用 UnoCSS theme（primary/secondary/accent/neutral）或已有 shortcut。
- [ ] 类名符合 BEM 或 kebab-case 块前缀，scoped 无全局泄漏。
- [ ] 复杂表单/反馈/弹窗优先 Element Plus；简单控件可用原生 + UnoCSS。
- [ ] 图标优先 `icon-local-*` 或 `<Icon>`，内联 SVG 仅必要时使用。

---

## 五、添加新功能时的总清单

1. **前端**
   - 新页面/组件放在约定目录；Tauri 调用只通过 `src/api/`；逻辑抽到 composables/stores；类型与样式符合**四、CSS 与 UI 使用规范**。
2. **Rust**
   - 新命令进对应模块（webview/sidecar/app），常量进 config，在 lib.rs 注册 handler。
3. **联调**
   - 前端用 API 层函数 + 类型调用；Rust 返回清晰错误信息，便于前端展示或排查。

生成或修改代码时，必须遵循本技能中的目录结构、API 层、composables、Rust 模块化、以及 **CSS 与 UI 使用规范**，确保项目可持续演进。

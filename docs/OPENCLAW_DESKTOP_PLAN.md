# OpenClaw 桌面化方案（分步实施）

OpenClaw 是 **TypeScript/Node.js** 项目（[openclaw/openclaw](https://github.com/openclaw/openclaw)），用 pnpm 管理。要在 Claw Browser 内「一键运行 OpenClaw」，不能当 Rust 库用，只能以**子进程**方式启动（打包成单文件二进制，或用系统/内嵌 Node 运行脚本）。

---

## 目标

- 用户打开 Claw Browser 后，在应用内点击「启动 OpenClaw」即可使用，无需单独开终端或安装 Node。
- OpenClaw 以 Gateway 形式监听 18789，用户在本机浏览器或应用内连接 18789 对话；OpenClaw 通过 Shell 调 curl 控制 Claw Browser 的 18790 API。

---

## 方案概览

| 方案 | 做法 | 优点 | 缺点 |
|------|------|------|------|
| **A. 单文件二进制** | 用 pkg/ncc 把 OpenClaw 打成可执行文件，放入 Tauri Sidecar | 用户无需 Node；分发简单 | 需在 OpenClaw 仓库维护打包脚本；体积较大 |
| **B. 内嵌 Node + 源码** | 把 Node 运行时 + OpenClaw 源码打进 Tauri resources，用 `node xxx/index.js` 启动 | 可继续用 pnpm、Skill 插件；灵活 | 打包与路径配置稍复杂；需带 Node 或便携版 |

建议：**先按 B 跑通（开发阶段用系统 Node）**，再视需要做 A（为发行版做 pkg 单文件）。

---

## 第一步：在本地跑通 OpenClaw（不依赖 Tauri）

目标：确认 OpenClaw 能独立运行、起 Gateway、并能用 curl 控制 Claw Browser。

1. **克隆并安装**
   ```bash
   git clone https://github.com/openclaw/openclaw.git
   cd openclaw
   pnpm install
   ```

2. **看启动方式**
   - 查 `package.json` 的 `scripts`（如 `start`、`gateway`、`dev`）。
   - 确认默认是否监听 18789，或需环境变量/配置文件。

3. **启动 Gateway**
   ```bash
   pnpm run gateway   # 或 pnpm start / pnpm dev，以仓库为准
   ```
   - 浏览器打开 http://127.0.0.1:18789/，能见 Web UI 即成功。

4. **确认与 Claw Browser 联动**
   - 先启动 Claw Browser（保证 18790 已起）。
   - 在 OpenClaw 里发一条「用 claw 打开 https://example.com」或「claw 快照」。
   - 若浏览器有反应，说明 Skill 调 18790 正常。

**产出**：文档里记下「启动 OpenClaw 的完整命令」（例如 `pnpm run gateway` 或 `node dist/gateway.js`），以及工作目录、环境变量（如有）。

---

## 第二步：在 Claw Browser 里用「系统 Node」启动 OpenClaw（开发态）

目标：不打包，用本机已安装的 `node`，由 Tauri 拉起 OpenClaw 进程。

1. **确定可执行命令**
   - 例如：在 OpenClaw 仓库根目录执行 `node dist/index.js` 或 `pnpm run gateway`。
   - `pnpm run` 会依赖 PATH 里的 pnpm；更稳的是直接用 `node` + 入口文件路径。

2. **在 Tauri 里用 Shell 调起**
   - 不用 Sidecar 二进制，改用 `tauri_plugin_shell` 的 `Command::new("node")`，`args` 里传 OpenClaw 入口路径。
   - 入口路径有两种：
     - **开发阶段**：写死或配置为「用户本机 OpenClaw 克隆目录」（如 `~/code/openclaw`），用 `node ~/code/openclaw/dist/index.js`。
     - **打包后**：用 Tauri `resources` 把 OpenClaw 打进安装包，运行时用 `resource_dir()` 拼出路径，例如 `node ${resource_dir}/openclaw/dist/index.js`。

3. **进程与生命周期**
   - 把 `Command::spawn()` 拿到的 child 存到 app state（如 `OpenClawProcess(Mutex<Option<Child>>)`）。
   - 提供 `start_openclaw` / `stop_openclaw` 两个 Tauri command；前端「启动 OpenClaw」「停止 OpenClaw」按钮分别调用。
   - 应用退出时（`RunEvent::Exit` 或 `ExitRequested`）从 state 取出 child 并 `kill()`，避免残留进程。

4. **工作目录**
   - 启动时 `current_dir(openclaw_root)`，保证 OpenClaw 能读到自己的配置、Skill、node_modules（若用 B 方案带源码）。

**产出**：开发环境下，在 Claw Browser 里点「启动 OpenClaw」后，18789 可用，且能控制 18790。

---

## 第三步：打包时把 OpenClaw 带进安装包（方案 B）

目标：安装 Claw Browser 后，不依赖用户本机是否有 OpenClaw 或 Node。

1. **Node 运行时**
   - **选项 1**：不内嵌 Node，安装包要求用户已装 Node，启动时用 `node`（PATH）。
   - **选项 2**：把便携版 Node（如 node-v20-win-x64、darwin arm64）放进 `bundle.resources`，启动时用 `path/to/node path/to/openclaw/dist/index.js`。

2. **OpenClaw 源码**
   - 在 Claw Browser 的构建脚本里（如 `beforeBuildCommand` 或单独脚本）：
     - `git clone` 或 `cp -r` OpenClaw 到 `src-tauri/resources/openclaw`；
     - 在 resources 目录下执行 `pnpm install --prod` 和 `pnpm build`（或 `tsc`）；
   - 在 `tauri.conf.json` 的 `bundle.resources` 里加入 `openclaw` 目录（或打包成 zip 再解压到 resource_dir）。

3. **运行时路径**
   - Rust 里用 `app.path().resource_dir()` 得到安装后的 resources 目录，拼出 `openclaw_dir` 和 `node_bin`（若内嵌 Node），再 `Command::new(node_bin).args([openclaw_entry]).current_dir(openclaw_dir)`。

**产出**：打包后的 Claw Browser 安装包内带 OpenClaw（及可选 Node），用户无需单独安装。

---

## 第四步：（可选）方案 A——把 OpenClaw 打成单文件二进制

目标：不依赖 Node 运行时，一个 exe（或二进制）搞定 OpenClaw。

1. **在 OpenClaw 仓库里**
   - 用 `pkg` 或 `vercel/ncc` 把入口打成单文件：
     - 例：`npx pkg . --targets node18-macos-arm64 --output binaries/openclaw`
     - 或 `ncc build src/index.ts -o dist-single` 再配合 pkg 打包成可执行文件。
   - 输出按 Tauri 约定命名：`openclaw-aarch64-apple-darwin`、`openclaw-x86_64-pc-windows-msvc.exe` 等。

2. **放入 Claw Browser**
   - 把上述二进制放到 `src-tauri/bin/`（或 `binaries/`），与 `tauri.conf.json` 里 `externalBin` 一致。
   - 当前配置是 `externalBin: ["bin/openclaw"]`，Tauri 会按平台找 `openclaw-<target>`。

3. **Rust 侧**
   - 保持用 Sidecar 启动：`shell.sidecar("openclaw").spawn()`，存 child 到 state，提供 start/stop。

**产出**：发行版可选用「单文件 OpenClaw + Sidecar」，用户无需 Node。

---

## 第五步：前端与体验

1. **左侧 AI 控制台**
   - 增加「启动 OpenClaw」「停止 OpenClaw」按钮，调用 `start_openclaw` / `stop_openclaw`。
   - 显示状态：「OpenClaw：未运行 / 已运行」；可用 `is_openclaw_running` 或启动时发事件通知前端。

2. **引导**
   - 首次使用或 18789 连不上时，提示「请先点击『启动 OpenClaw』」。
   - 可选：在应用内嵌一个 webview 打开 http://127.0.0.1:18789/，或提供「在默认浏览器中打开 OpenClaw」链接。

3. **退出时杀进程**
   - 在 Tauri 的 `RunEvent::ExitRequested` 或 `run()` 的 event 循环里，若 state 中有 OpenClaw child，则 `kill()`。

---

## 建议执行顺序（一步一步来）

| 顺序 | 内容 | 说明 |
|------|------|------|
| 1 | 在本地用 pnpm 跑通 OpenClaw，确认 18789 + 18790 联动 | 不写代码，先验证环境与命令 |
| 2 | 在 Claw Browser 里用 Shell 启动「本机 OpenClaw 目录」的 node 入口，并实现 start/stop + 退出杀进程 | 开发态可用 |
| 3 | 前端加「启动/停止 OpenClaw」按钮与状态展示 | 体验闭环 |
| 4 | 选 B：把 OpenClaw（+ 可选 Node）打进 resources，打包后从 resource_dir 启动 | 发行版不依赖本机 Node |
| 5 | 可选：做方案 A，用 pkg 打 OpenClaw 单文件，用 Sidecar 启动 | 不依赖 Node 的发行方式 |

当前仓库已有：18790 HTTP API、18789 WebSocket 连接与 AI 控制台、Sidecar 配置（`bin/openclaw`）。接下来只需按上面顺序实现「用 Node 或单文件启动 OpenClaw + 进程管理 + 前端按钮」即可。

---

## 已实现（本仓库）

- **Rust**：`openclaw_process` 模块，支持两种启动方式：
  1. **环境变量 `OPENCLAW_ENTRY`**：设为 OpenClaw 入口脚本绝对路径（如 `/path/to/openclaw/dist/index.js`），则用系统 `node` 执行该脚本。
  2. **未设置时**：使用 Sidecar 二进制 `bin/openclaw`（需自行用 pkg 等打好并放入 `src-tauri/bin/`）。
- **命令**：`start_openclaw`、`stop_openclaw`、`is_openclaw_running`。
- **前端**：左侧 AI 控制台有「OpenClaw」区块，显示状态（已运行/未运行）及「启动」「停止」按钮。

**开发时用 Node 跑 OpenClaw**：在 OpenClaw 仓库 `pnpm build` 后，在启动 Claw Browser 前设置环境变量再运行，例如：

```bash
export OPENCLAW_ENTRY=/path/to/openclaw/dist/index.js
pnpm tauri dev
```

然后在应用内点击「启动 OpenClaw」即可。

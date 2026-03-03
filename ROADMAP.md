# Claw Browser - 沉浸式 AI 浏览器项目蓝图与实施路线图

## 1. 项目愿景与核心价值

**Claw Browser** 的定位不再是一个普通的“套壳浏览器”，而是**为 OpenClaw 量身定制的“沉浸式 AI 浏览器”**。
它的核心价值在于解决当前 AI Agent（尤其是 Web Automation 领域）的三个核心痛点：
1. **黑盒焦虑**：用户不知道 AI 在后台点击了什么、输入了什么。
2. **状态脱节**：AI 遇到复杂验证码或弹窗卡住时，用户难以无缝介入接管。
3. **安全与隐私失控**：AI 可能会误操作支付或向第三方泄露敏感信息。

通过 Tauri 的系统级能力、定制化的多子 Webview 架构，结合 OpenClaw 的智能决策，Claw Browser 将实现**“人机协同”、“全透明执行”与“安全护城河”**。

---

## 2. 技术架构设计

为避免后期推翻重构，项目架构应围绕“通信与隔离”进行设计：

*   **前端展示层 (Vue 3 + UnoCSS)**：
    *   **分屏布局 (Split-View)**：左侧 AI 控制台（对话、思考链、工具日志），右侧真实 Webview。
*   **宿主控制层 (Tauri + Rust)**：
    *   **进程管理**：作为 Sidecar 启动并守护 OpenClaw 二进制文件。
    *   **通信枢纽 (IPC Bridge)**：负责前端、OpenClaw 和目标 Webview 之间的消息中转。
    *   **安全网关**：拦截高危指令，触发前端权限确认弹窗。
*   **目标执行层 (Tauri Webview + 注入脚本)**：
    *   利用 Tauri 的 `initialization_script` 向每个子 Webview 注入隐藏的 `bridge.js`。
    *   负责执行高亮渲染、事件监听、简化 DOM 提取，并跨域绕过部分安全限制将状态回传给 Rust。

---

## 3. 实施路线图 (Phased Roadmap)

为了避免踩坑（特别是跨域通信和 CSP 限制的坑），项目分为 4 个阶段，**强烈建议按顺序实现，不要跨阶段跃进**。

### 阶段 0：技术可行性验证 (PoC) - 扫清基建地雷
*目标：确认 Tauri 的能力边界，确保核心链路畅通。*

*   [x] **Sidecar 联通**：在 Rust 中将 OpenClaw 配置为 Sidecar，成功启动并通过 stdin/stdout 发送测试指令并获取响应。（当前使用 mock 脚本 `bin/openclaw-<target>`，主页可点击「阶段 0：测试 Sidecar」验证。）
*   [x] **Webview 注入与双向通信**：子 Webview 使用 `initialization_script` 注入 `bridge.js`，并注入 `window.__clawBridgeLabel`；页面点击通过 `on_webview_click` 回传 Rust。
*   [x] **动作执行与高亮测试**：Rust 提供 `eval_in_webview`，可对指定 label 的 Webview 执行任意 JS；注入的 `window.__clawBridge.highlight(selector)` 可高亮元素。前端可据此在阶段 1 做「红框延迟执行」。
*   *防坑指南：如果某些网站因严苛的 CSP (Content Security Policy) 阻止了注入脚本的执行，需评估是否引入 Rust 本地 HTTP 代理层 (Local Proxy) 来剥离 CSP 头。*

### 阶段 1：核心交互框架 - “打通手和眼”
*目标：搭建沉浸式 UI，让用户能直观看到 AI 的工作流。*

*   [x] **分屏 UI 改造**：左侧 320px AI 控制台（`AIConsole.vue`），右侧 Webview 区域；`calc_webview_rect` 已按 `LEFT_PANEL_WIDTH` 计算，窗口 resize 时调用 `resize_all_webviews`。
*   [ ] **指令解析与可视化流**：解析 OpenClaw 输出的思考链 (Thought) 和动作 (Action)，在左侧面板流式渲染。（UI 已就绪：`AIConsole` 内 `streamItems` 占位，接通 OpenClaw 后推送即可。）
*   [x] **视觉高亮 (Action Highlighting)**：左侧控制台提供「高亮测试」输入框，可对当前 tab 内任意选择器执行红框高亮（`eval_in_webview` → `__clawBridge.highlight`）。AI 指令驱动下的「先高亮再执行」在接通 OpenClaw 后接入。
*   [ ] **人机混合接管 (Hybrid Control)**：增加全局快捷键或面板开关，允许用户随时暂停 AI，自己在右侧 Webview 中完成操作（如拖动滑块验证码），随后通知 AI “状态已更新，继续”。

### 阶段 2：AI 专属增强 - “提效与降本”
*目标：提升 OpenClaw 的执行成功率，降低大模型的 Token 消耗。*

*   [ ] **智能 DOM 提纯 (DOM Pre-processor)**：
    *   在注入的 `bridge.js` 或 Rust 代理层中，过滤掉无用的 `<script>`、`<style>` 和广告节点。
    *   提取关键的交互元素（a, button, input）及其坐标，生成精简的 JSON 或 Markdown 发送给 OpenClaw。
*   [ ] **操作录制与回放 (Session Recording)**：
    *   记录 AI 的操作轨迹（URL、动作、简要状态）。
    *   实现“时光倒流”功能：任务失败时，重置 Webview 并按脚本自动回放至失败前的一步，允许用户修改 prompt 重新尝试。
*   [ ] **多身份隔离 (Sandbox Profiles)**：
    *   为不同的任务设定独立的 Tauri 数据目录。实现“工作环境”与“个人环境”的 Cookie、LocalStorage 物理隔离，防止账号串车。

### 阶段 3：安全与隐私中心 - “构建护城河”
*目标：将风险降到最低，打造企业级/重度用户的信任基础。*

*   [ ] **权限拦截弹窗 (Approval Gate)**：
    *   在 Rust 层建立敏感词词库或正则（如“支付”、“提交订单”、“delete”）。
    *   当拦截到高风险动作时，暂停 Sidecar 进程，弹出原生/前端 Dialog：“AI 正试图进行敏感操作，是否允许？”
*   [ ] **隐私数据脱敏 (Privacy Shield)**：
    *   全局清洗左侧日志中的敏感信息（密码、API Key、身份证号等），显示为 `***`，确保即便用户截图分享也不会泄露隐私。

---

## 4. 关键技术踩坑预警 (Pitfalls & Mitigations)

1.  **DOM 获取与跨域问题**：Tauri 2 对子 Webview 的控制力有限。绝对不要指望能直接从主进程读取跨域页面的 DOM。必须依赖 `initialization_script` 注入。
2.  **SPA 路由变化感知**：单页应用 (SPA) 路由跳转不会触发整个页面的重新加载，注入的脚本需要监听 `popstate` 和 `pushState`，或使用 `MutationObserver` 实时感知 DOM 变化并通知 Rust。
3.  **大模型幻觉导致的死循环**：OpenClaw 可能会反复点击一个无效按钮。需要在 Rust/前端层做**防抖和死循环检测**，若同一页面同一元素短时间内被点击超过 N 次，自动暂停并向用户求助。
4.  **Sidecar 僵尸进程**：必须妥善处理 Tauri 退出时的生命周期，确保 OpenClaw Sidecar 在主程序退出、崩溃时被正确 Kill 掉，否则会占用系统资源。

---

> **结语**：不要一开始就做大而全。先用最简单的 HTML 页面跑通 **阶段 0** 的全链路，再逐步攻克复杂第三方网页。Claw Browser 的最终形态，将是未来 AI 时代的标准浏览器范式。
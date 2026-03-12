# OpenClaw 与 Oclaw 的集成方向

**结论：不以「Oclaw 连接 OpenClaw Gateway」为主，而是以「OpenClaw 控制 Oclaw」为正道。**

## 为什么不让本应用去连 OpenClaw？

- 本应用作为 Gateway 客户端（连 ws、发 chat、收 response）时，OpenClaw 仍然操作的是自己的浏览器或扩展，**无法直接驱动本应用内的 webview**，用户看到的仍是「两个世界」。
- **与 OpenClaw 对话可直接使用其 Web UI**：访问 http://127.0.0.1:18789/ 即可，无需在本应用内再实现一套对话界面。

## 正道：OpenClaw 控制本应用

1. **Oclaw 暴露 HTTP API**  
   本应用在本地起一个小型 HTTP 服务（如 `127.0.0.1:18790`），提供例如：
   - `POST /navigate`：在当前 tab 打开 URL
   - `POST /snapshot` 或 `GET /snapshot`：返回当前页的 DOM 快照（可交互元素列表）
   - `POST /click`：根据 selector 或坐标执行点击
   - （可选）`POST /eval`：执行脚本、高亮等

2. **OpenClaw 侧增加 skill**  
   在 OpenClaw 的 skill 目录下新增一个技能，该技能在「打开网页」「点击」「取快照」等意图时，去请求上述 API，从而**由 OpenClaw 驱动 Oclaw 内的网页**。

3. **效果**  
   用户在一个窗口内：左侧是 AI 控制台（可后续从 OpenClaw 拉思考链或仅做本地展示），右侧是 Oclaw 的 webview，**所有浏览器操作都由 OpenClaw 通过 API 控制**，实现「一个窗口、全透明、可接管」。

## 已实现的 HTTP API（127.0.0.1:18790）

应用启动时在本地起服，**当前活动 tab** 由前端在切换/打开/关闭 tab 时通过 `set_active_tab_label` 同步给 Rust；在首页或无 tab 时活动 label 为 `null`，此时 `/navigate` 会发事件让前端打开新 tab 并加载 URL（仍返回 204），snapshot/click 则返回 400。

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/navigate` | Body: `{ "url": "https://..." }`。有活动 tab 时在该 tab 跳转；无活动 tab 时自动打开新 tab 并加载 URL。204。 |
| GET  | `/snapshot` | 返回当前活动页的 DOM 快照 JSON。Body: `{ "snapshot": "..." }`。无活动 tab 则 400。 |
| POST | `/click`    | Body: `{ "selector": "CSS selector" }`。在当前活动页对首个匹配元素执行 click。204。 |

## 下一步建议

- 编写并配置 OpenClaw 的「Oclaw」skill，在技能内按需请求上述 API。

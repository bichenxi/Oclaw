读取并理解当前项目中 OpenClaw Agent 的技能文件，给出现状摘要。

请依次执行：

1. 读取 `openclaw-skill/SKILL.md` —— Agent 的通用浏览器操控技能（全部 API 端点、操作模式、场景模板）
2. 读取 `src-tauri/src/api.rs` —— Rust 端暴露的实际 HTTP 接口（确认 skill 文件描述的 API 是否与代码一致）

完成后输出以下摘要：

### 当前 Agent 技能概览
- **通用 API 端点**（端点 / 方法 / 参数 / 用途）
- **关键约束**（禁止事项、必须完成的步骤）
- **代码一致性**：skill 文件描述的 API 与 `api.rs` 中实际暴露的端点是否匹配，列出任何不一致之处

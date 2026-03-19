# Oclaw 项目版本发布记录

## 项目基本信息
- **项目名称**：Oclaw（已从 claw-browser 重命名）
- **版本号位置**：
  - `package.json` 的 `version` 字段
  - `src-tauri/tauri.conf.json` 的 `version` 字段（需同步）
- **当前版本**：1.4.4 (发布于 2026-03-16)

## 版本发布流程
1. 更新 `package.json` 中的版本号（patch 递增）
2. 更新 `src-tauri/tauri.conf.json` 中的版本号（保持同步）
3. git add -A
4. git commit -m "chore: bump version to X.X.X"
5. git tag vX.X.X
6. git push origin main
7. git push origin vX.X.X（触发 GitHub Actions）

## 关键文件
- 应用入口：`src-tauri/src/main.rs`
- 应用配置：`src-tauri/tauri.conf.json`（包含 productName: Oclaw）
- 前端入口：`src/main.ts`
- 技能定义：`openclaw-skill/SKILL.md`（skill name: oclaw-control）
- 数据目录：通过 `src-tauri/src/installer.rs` 配置

## 重要约定
- 应用名统一使用 **Oclaw**（产品名）
- 内部 skill 名统一使用 **oclaw-control**
- Rust 包名：oclaw，lib 名：oclaw_lib
- 每次发版时，这两个版本号文件必须同步更新

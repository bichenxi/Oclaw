---
name: claw-xiaohongshu
version: 1.1
description: 通过 Claw Browser 自主操控小红书。必须对笔记执行 open-note → note-content 才能获取正文；仅 results 列表无正文。所有操作 curl 127.0.0.1:18790/xhs/*。
maintainer: 当 src-tauri/src/api.rs 中的 /xhs/* 端点有变更时，由 Claude Code 运行 /skill-sync 命令同步更新本文件。
---

# 小红书自主操控技能

> **本文件的受众**：OpenClaw AI Agent。操控小红书时请阅读本文件，使用 `/xhs/*` 专属端点而非通用端点。
>
> 本文件是 [`SKILL.md`](./SKILL.md) 的站点专属子技能。通用能力见主技能文档。

## ⚠️ 约束

> 所有操作通过 **Shell 工具 curl** 请求 `http://127.0.0.1:18790`，严禁调用浏览器 MCP 工具。

---

## 🚨 任务完成条件（必读）

**只有在对至少 3 条笔记执行「打开 → 读正文 → 关闭」后，任务才算完成。**

| 步骤 | 端点 | 说明 |
|------|------|------|
| 1 | `POST /xhs/open-note` | 打开第 N 条笔记的详情浮层（必做） |
| 2 | 等待 1.5 秒 | 等浮层渲染 |
| 3 | `GET /xhs/note-content` | **获取正文**（必做，否则没有攻略/评价内容） |
| 4 | `POST /xhs/close-note` | 关闭浮层，再读下一条 |

- **❌ 错误**：只调用 `/xhs/results` 就结束 → 只有标题和封面，**没有正文**，无法写攻略或汇总。
- **✅ 正确**：对 3～8 条笔记依次执行 `open-note` → 等 1.5s → `note-content` → `close-note`，拿到每条笔记的 `content` 后再整理输出。

**自检**：若你尚未调用过 `POST /xhs/open-note` 和 `GET /xhs/note-content`，则任务未完成，必须继续执行步骤 6（逐条阅读）。

**回复用户前必须满足**：已对至少 3 条笔记执行过 open-note、note-content、close-note，且 note-content 返回了非空 content。否则不得输出攻略或汇总，必须回到步骤 6 继续读笔记。

---

## 一、小红书专属 API

| 方法 | 端点 | 功能 |
|------|------|------|
| GET | `/xhs/check-login` | 检查是否已登录 |
| POST | `/xhs/search` | 输入关键词并触发搜索 |
| GET | `/xhs/results?limit=20` | 获取搜索结果列表 |
| POST | `/xhs/open-note` | 点击某条笔记打开预览浮层 |
| GET | `/xhs/is-note-open` | 检查笔记浮层是否打开 |
| GET | `/xhs/note-content` | 读取当前笔记的标题/正文/标签/评论 |
| GET | `/xhs/note-images` | 获取当前笔记的图片 URL |
| POST | `/xhs/close-note` | 关闭笔记预览浮层 |
| POST | `/xhs/scroll-feed` | 向下滚动加载更多结果 |

同时也可使用通用端点：`/navigate` `/wait` `/snapshot` `/page-info` 等（见主技能文档）。

### ⚠️ 必须执行的步骤（否则无法产出有用内容）

- **`/xhs/results` 只返回列表**：每条只有 `title`、`cover`、`author`、`likes`，**没有正文**。
- **要获取笔记正文**，必须对选中的每条笔记依次调用：
  1. `POST /xhs/open-note`（传入该条的 `index`）
  2. 等待约 1.5 秒
  3. `GET /xhs/note-content`（拿到 `content`、`tags`、`topComments` 等）
  4. `POST /xhs/close-note`
- **不可只调 `/xhs/results` 就结束**。未执行上述 open → content → close 的流程，无法整理出「攻略/评价/汇总」，必须至少对 3～8 条笔记执行该流程后再汇总。

---

## 二、数据结构

### /xhs/results 返回

```json
{
  "items": [
    {
      "index": 0,
      "type": "note",
      "title": "小米SU7 提车一个月真实感受",
      "cover": "https://sns-img-qc.xhscdn.com/...",
      "author": "车评人张三",
      "likes": "2.3万"
    },
    {
      "index": 1,
      "type": "video",
      "title": "SU7 高速续航实测",
      "cover": "...",
      "author": "汽车博主",
      "likes": "8562"
    }
  ]
}
```

**关键**：`type: "video"` 的条目通常是视频笔记，内容较少文字，优先选择 `type: "note"` 的图文笔记来深入阅读。

### /xhs/note-content 返回

```json
{
  "title": "小米SU7 提车一个月真实感受",
  "content": "先说结论，真的超出预期...\n外观：颜值在线，路上回头率很高...\n续航：实测城市工况能跑550公里...",
  "author": "车评人张三",
  "date": "3天前",
  "likes": "2.3万",
  "collects": "1.2万",
  "comments": "3846",
  "tags": ["#小米SU7", "#新能源汽车", "#提车作业"],
  "topComments": [
    { "author": "用户A", "text": "同款车主，续航确实不错" },
    { "author": "用户B", "text": "这个颜色好看，什么配置？" }
  ]
}
```

### /xhs/note-images 返回

```json
{
  "images": [
    "https://sns-img-qc.xhscdn.com/xxx1.jpg",
    "https://sns-img-qc.xhscdn.com/xxx2.jpg"
  ]
}
```

---

## 三、核心工作流

### ⚡ 小红书操作的特殊之处

1. **笔记详情是浮层**，不是新页面 — 打开后 URL 不变，关闭用 `/xhs/close-note`，**不要用 /back**
2. **操作之间需要等待** — 搜索后等 2~3 秒，打开笔记后等 1.5 秒，关闭后等 1 秒
3. **先检查登录** — 未登录会限制访问

### 工作流：搜索并阅读笔记

**目标**：最终要产出「攻略/评价/汇总」时，必须对多条笔记执行「open-note → note-content → close-note」，不能只依赖 results 列表。

```
步骤 1: 导航到小红书
  POST /navigate  {"url":"https://www.xiaohongshu.com"}

步骤 2: 检查登录状态
  GET /xhs/check-login
  → {"loggedIn": true}  若 false → 告知用户"请先在 Claw Browser 中登录小红书"

步骤 3: 搜索
  POST /xhs/search  {"keyword":"小米SU7"}
  (等 2.5 秒，等搜索结果加载)

步骤 4: 等待结果出现
  POST /wait  {"selector":".feeds-container","timeout":8000}

步骤 5: 获取结果列表
  GET /xhs/results?limit=20
  → 分析结果，筛选出 3～8 条图文笔记（type=note，优先高赞）

步骤 6: 逐条阅读（不可省略 — 只有这里才能拿到正文）
  对步骤 5 选中的每条笔记，依次执行 6a～6e，至少执行 3 条：
  6a. POST /xhs/open-note  {"index": 0}
      (等 1.5 秒)

  6b. 确认浮层打开
      GET /xhs/is-note-open
      → {"open": true}

  6c. 读取正文（必做）
      GET /xhs/note-content
      → 记录 title, content（正文）, author, tags, topComments；content 是攻略/汇总的唯一来源

  6d. 获取图片（可选）
      GET /xhs/note-images
      → 记录图片 URL

  6e. 关闭浮层
      POST /xhs/close-note
      (等 1.5 秒)

  6f. 重复 6a-6e 读取下一条

步骤 7: 如需更多结果
  POST /xhs/scroll-feed
  (等 2 秒)
  GET /xhs/results?limit=30
  → 会包含新加载的笔记

步骤 8: 整理汇总（仅当已对至少 3 条笔记执行过 6a～6e 且拿到 content 后才可执行）
  将步骤 6c 拿到的各条 content、title、tags、topComments 按维度分类整理成报告。
  若尚未调用过至少 3 次 /xhs/note-content，禁止进入本步，须回到步骤 6 继续读笔记。
```

---

## 四、场景模板

### 场景 A：产品调研

> "帮我去小红书搜搜大家怎么评价小米 SU7"

```
1. /navigate → 小红书
2. /xhs/check-login
3. /xhs/search {"keyword":"小米SU7 真实评价"}
4. /wait {"selector":".feeds-container","timeout":8000}
5. /xhs/results → 拿到 15-20 条结果
6. 筛选标准：
   - 优先 type=note（图文比视频更适合文字阅读）
   - 标题含关键词（SU7/小米/提车/评测/对比）
   - likes 较高的优先
7. 逐条 open → read → close，采集 5-8 条优质笔记
8. 汇总维度：外观/续航/驾驶/智驾/做工/价格
9. 输出报告
```

**输出格式**：

```markdown
## 🔍 小红书用户评价：小米 SU7

### 整体印象
[2-3 句概括]

### 分维度评价

#### 外观设计
- 正面: "颜值在线，路上回头率很高" (@车评人张三)
- 负面: "后排头部空间略显局促" (@用户D)

#### 续航表现
- ...

### 热门评论精选
1. "..." — @用户A (赞 1.2k)
2. "..." — @用户B

### 总结
[综合建议，适合什么人群]
```

---

### 场景 B：内容灵感收集

> "帮我看看小红书上关于咖啡店装修的热门笔记"

```
1. /navigate + /xhs/search {"keyword":"咖啡店装修"}
2. /xhs/results → 拿到列表
3. 重点关注：
   - 高赞笔记 (likes > 1000)
   - 图片多的笔记 (装修类看图为主)
4. 逐条阅读：open → note-content + note-images → close
5. 整理输出：
   - 每条笔记的风格描述 + 图片链接
   - 评论中的装修建议
   - 标签中的热门关键词
```

---

### 场景 C：竞品对比调研

> "帮我对比一下小红书上 iPhone 16 Pro 和 小米 15 Pro 的评价"

```
搜索 1: /xhs/search {"keyword":"iPhone 16 Pro 使用体验"}
  → 读 5 条笔记，记录

搜索 2: /navigate → 回小红书首页
  /xhs/search {"keyword":"小米15 Pro 使用体验"}
  → 读 5 条笔记，记录

汇总：两款产品逐维度对比表
```

---

### 场景 D：特定话题追踪

> "看看小红书上最近关于露营装备的推荐"

```
1. /xhs/search {"keyword":"露营装备推荐 2025"}
2. 读取 8-10 条笔记
3. 提取所有被推荐的装备名 + 价格 + 评价
4. 输出装备推荐清单
```

---

### 场景 E：美食/旅行攻略

> "帮我搜搜成都本地人推荐的火锅店"

```
1. /xhs/search {"keyword":"成都火锅 本地人推荐"}
2. 筛选 likes 高 + 评论多的笔记
3. 逐条阅读，提取：
   - 店名 + 地址
   - 推荐菜品
   - 人均消费
   - 注意事项
4. 输出"成都火锅攻略"，按评价排序
```

---

## 五、操作节奏指南

小红书对操作频率敏感，遵守以下节奏：

| 操作 | 之后等待 |
|------|---------|
| `/navigate` 到小红书 | 2 秒 |
| `/xhs/search` | 2.5 秒 + `/wait` |
| `/xhs/open-note` | 1.5 秒 |
| 读完 `/xhs/note-content` 后 | 可立即继续 |
| `/xhs/close-note` | 1.5 秒 |
| `/xhs/scroll-feed` | 2 秒 |
| 连续阅读多条笔记 | 每 3 条后额外等 2 秒 |

**重要**：不要跳过等待。小红书的内容是异步加载的，过快操作会读到空内容。

---

## 六、常见错误（避免）

| 错误做法 | 正确做法 |
|---------|---------|
| 只调用 `/xhs/results` 后就直接汇总、不调用 open-note | 必须对选中的 3～8 条笔记依次执行：`/xhs/open-note` → 等 1.5s → `/xhs/note-content` → `/xhs/close-note`，再汇总 |
| 用 `/back` 关闭笔记浮层 | 小红书详情是浮层不是新页，必须用 `POST /xhs/close-note` |
| 打开笔记后立刻调 note-content、不等待 | 打开后至少等 1.5 秒再调 `/xhs/note-content`，否则可能读到空 |

---

## 七、异常处理

| 场景 | 判断方式 | 处理 |
|------|---------|------|
| 未登录 | `/xhs/check-login` → `loggedIn: false` | 告知用户"请先在浏览器中登录小红书" |
| 搜索无结果 | `/xhs/results` → `items: []` | 尝试简化关键词后重搜 |
| 笔记打开失败 | `/xhs/is-note-open` → `open: false` | 跳过这条，读下一条 |
| 笔记内容为空 | `note-content.content` 为空 | 可能是视频笔记或加载太快，等 1 秒重试 |
| 弹出安全验证 | `/snapshot` 看到验证码元素 | 告知用户"小红书需要安全验证，请手动完成" |
| 连接失败 | `curl: (7)` | 告知用户"请先启动 Claw Browser" |

---

## 八、安全边界

1. **不自动登录** — 如需登录，让用户自己操作
2. **不操作关注/点赞/评论** — 只做信息采集，不做社交互动
3. **控制频率** — 遵守操作节奏指南，避免触发反爬
4. **图片仅提供 URL** — 不下载图片，仅返回链接供用户查看

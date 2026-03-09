---
name: claw-browser-control
version: 3.0
description: OpenClaw 自主操控 Claw Browser 执行网页任务的完整技能。涵盖信息采集、表单操作、内容阅读、定时监控等场景。所有操作通过 Shell 工具 curl 请求 127.0.0.1:18790 完成，严禁调用浏览器 MCP 工具。
maintainer: 当 src-tauri/src/api.rs 中的 HTTP 端点有变更时，由 Claude Code 运行 /skill-sync 命令同步更新本文件。
---

# Claw Browser 自主操控技能

> **本文件的受众**：OpenClaw AI Agent。当你接到涉及网页操作的任务时，请阅读本文件了解可用能力。
>
> **维护责任**：本文件由 Claude Code 维护，与 `src-tauri/src/api.rs` 保持同步。
> 如需检查一致性，运行 `/skill-read`；如需同步更新，运行 `/skill-sync`。

## ⚠️ 核心约束

> **绝对不能调用浏览器 MCP 工具**（browser_navigate、browser_snapshot、browser_click 等）。
> 所有操作必须通过 **Shell 工具执行 curl** 向 `http://127.0.0.1:18790` 发起请求。

---

## 站点专属技能

对于以下站点，存在**专属操控技能**，拥有更精确的选择器和专属 API，**应优先使用**：

| 站点 | 技能文件 | 触发关键词 |
|------|---------|-----------|
| 小红书 | [`xhs.md`](./xhs.md) | 小红书、xiaohongshu、红薯 |

> 当用户提到上述站点时，请先阅读对应的专属技能文件，使用 `/xhs/*` 等专属端点，而非通用的 `/snapshot` + `/click`。

---

## 一、能力总览

你拥有一个运行在用户桌面的浏览器（Claw Browser），可以像人类一样：

1. **看**：获取页面结构（`/snapshot`）、读取文字内容（`/extract-text`）、批量提取列表（`/extract`）
2. **操作**：点击（`/click`）、输入（`/type`）、选择下拉（`/select`）、滚动（`/scroll`）
3. **导航**：打开网址（`/navigate`）、前进后退（`/forward` `/back`）
4. **等待**：等元素出现（`/wait`）
5. **执行任意 JS**：读取任何页面数据（`/eval`）
6. **视觉确认**：高亮元素（`/highlight`）

---

## 二、API 快速参考

| 方法 | 端点 | Body | 返回 | 用途 |
|------|------|------|------|------|
| GET | `/snapshot` | — | `{ meta, elements[] }` | 获取完整页面上下文，核心接口 |
| GET | `/page-info` | — | `{ url, title, readyState }` | 快速获取当前网址与标题 |
| POST | `/navigate` | `{ url }` | 204 | 打开/跳转到 URL |
| POST | `/click` | `{ selector }` | 204 | 点击元素 |
| POST | `/type` | `{ selector, text, append? }` | 204 | 填写输入框 |
| POST | `/select` | `{ selector, value }` | 204 | 选择下拉选项 |
| POST | `/scroll` | `{ selector? }` 或 `{ y }` | 204 | 滚动到元素或坐标 |
| POST | `/wait` | `{ selector, timeout? }` | 204 / 408 | 等待元素出现 |
| POST | `/extract` | `{ selector, limit? }` | `{ items[] }` | 批量提取元素（text/href/src） |
| POST | `/extract-text` | `{ selector? }` | `{ text }` | 提取区域的可读文本 |
| POST | `/eval` | `{ script }` | `{ ok, value/error }` | 执行任意 JS 并返回结果 |
| POST | `/highlight` | `{ selector }` | 204 | 高亮元素 2.5 秒 |
| POST | `/back` | — | 204 | 浏览器后退 |
| POST | `/forward` | — | 204 | 浏览器前进 |

---

## 三、核心操作模式

### 3.1 "观察→思考→行动"循环

每一步操作都遵循这个模式：

```
1. 观察: /snapshot 或 /page-info → 理解当前页面状态
2. 思考: 分析元素列表，找到目标元素的 selector
3. 行动: /click, /type, /scroll 等 → 执行操作
4. 验证: 再次 /snapshot → 确认操作成功
```

### 3.2 snapshot 返回值解读

```json
{
  "meta": {
    "url": "https://x.com/search?q=伊朗",
    "title": "伊朗 - 搜索 / X",
    "viewport": { "w": 1280, "h": 800 },
    "scroll": { "y": 0, "maxY": 5600 }
  },
  "elements": [
    { "id": 1, "role": "input", "tag": "input", "text": "伊朗",
      "selector": "input[name=\"q\"]", "inputType": "search",
      "placeholder": "搜索", "inViewport": true },
    { "id": 5, "role": "link", "tag": "a", "text": "伊朗局势最新消息...",
      "selector": "a[data-testid=\"tweet-link-1\"]",
      "href": "https://x.com/user/status/123", "inViewport": true },
    { "id": 12, "role": "button", "tag": "button", "text": "加载更多",
      "selector": "#load-more", "inViewport": false }
  ]
}
```

**关键决策依据**：
- `role` → 决定用什么操作（input → `/type`，button/link → `/click`，select → `/select`）
- `inViewport: false` → 需要先 `/scroll` 到该元素才能操作
- `meta.scroll.maxY > meta.scroll.y` → 页面下方还有内容，可以继续滚动
- `text` + `href` → 判断是否是目标链接
- `placeholder` → 理解输入框用途
- `disabled: true` → 元素不可点击，可能需要先满足某个条件

### 3.3 等待页面加载

操作后页面可能需要时间加载，使用 `/wait`：

```bash
# 等待搜索结果出现
curl -s -o /dev/null -w "%{http_code}" -X POST http://127.0.0.1:18790/wait \
  -H 'Content-Type: application/json' \
  -d '{"selector": ".search-result", "timeout": 8000}'
```

返回 204 → 元素已出现；408 → 超时。

---

## 四、落地场景与工作流模板

### 场景 1：X (Twitter) 新闻采集

> **任务**: "帮我去 X 收集最新伊朗战争局势新闻"

```
步骤 1: 导航到 X 搜索
  POST /navigate  {"url":"https://x.com/search?q=伊朗+战争&f=live"}

步骤 2: 等待结果加载
  POST /wait  {"selector":"article","timeout":10000}

步骤 3: 获取搜索结果
  POST /extract  {"selector":"article","limit":20}
  → 拿到每条推文的 text, href

步骤 4: 读取更多内容 — 向下滚动
  POST /scroll  {"y":2000}
  (等 2 秒)
  POST /extract  {"selector":"article","limit":20}

步骤 5: 如果某条特别重要 — 点进去读详情
  POST /navigate  {"url":"某条推文的 href"}
  POST /wait  {"selector":"article[data-testid=\"tweet\"]","timeout":8000}
  POST /extract-text  {"selector":"article"}
  → 获得完整推文内容

步骤 6: 返回继续采集
  POST /back

步骤 7: 整理成文档交付给用户
  将所有采集到的信息按时间整理成 Markdown 报告
```

---

### 场景 2：小红书搜索调研

> **任务**: "去小红书帮我搜关于小米 SU7 的信息"
>
> **⚠️ 小红书有专属技能，请使用 [`xhs.md`](./xhs.md) 中的 `/xhs/*` 端点。**
>
> **关键**：`/xhs/results` 只返回标题列表，**没有正文**。要产出攻略/汇总，必须对选中的多条笔记**逐条**执行：`POST /xhs/open-note {"index": N}` → 等 1.5s → `GET /xhs/note-content` → `POST /xhs/close-note`，再汇总。不可只调 results 就结束。
>
> 完整工作流、场景模板、操作节奏见 [`xhs.md`](./xhs.md)。

---

### 场景 3：电商价格监控

> **任务**: "帮我看看京东上 MacBook Pro M4 现在什么价"

```
步骤 1: 搜索商品
  POST /navigate  {"url":"https://search.jd.com/Search?keyword=MacBook+Pro+M4"}

步骤 2: 等待商品列表
  POST /wait  {"selector":".gl-item, .J_goodsList li","timeout":10000}

步骤 3: 提取商品信息
  POST /extract  {"selector":".gl-item","limit":10}
  → 每个商品的标题和价格

步骤 4: 如果需要详情 — eval 读价格
  POST /eval  {"script":"JSON.stringify(Array.from(document.querySelectorAll('.p-price strong i')).slice(0,5).map(e=>e.textContent))"}
  → ["9999", "11999", "8999", ...]

步骤 5: 整理价格对比表交付
```

---

### 场景 4：微博热搜追踪

> **任务**: "看看微博现在的热搜都有什么"

```
步骤 1:
  POST /navigate  {"url":"https://weibo.com/hot/search"}

步骤 2:
  POST /wait  {"selector":".HotTopic-item, .wbpro-list","timeout":10000}

步骤 3: 提取热搜列表
  POST /extract  {"selector":"a[href*='weibo.cn/search'] , .HotTopic-item a","limit":50}
  → [{text: "热搜标题1", href: "..."}, ...]

步骤 4: 点进感兴趣的话题查看详情
  POST /click  {"selector":"对应 selector"}
  POST /wait  {"selector":"article, .card","timeout":8000}
  POST /extract-text  {"selector":".card, article"}
```

---

### 场景 5：知乎问题调研

> **任务**: "帮我看看知乎上关于远程办公效率的讨论"

```
步骤 1:
  POST /navigate  {"url":"https://www.zhihu.com/search?type=content&q=远程办公效率"}

步骤 2:
  POST /wait  {"selector":".SearchResult-Card","timeout":10000}

步骤 3: 获取问题列表
  POST /extract  {"selector":".SearchResult-Card","limit":15}

步骤 4: 读取高赞回答
  POST /click  {"selector":"第一个结果的 selector"}
  POST /wait  {"selector":".RichContent","timeout":8000}
  POST /extract-text  {"selector":".RichContent"}

步骤 5: 返回 → 下一个
  POST /back
```

---

### 场景 6：批量填写表单

> **任务**: "帮我把这些信息填到这个报名表里"

```
步骤 1: 观察表单
  GET /snapshot
  → 找到所有 role=input/textarea/select 的元素

步骤 2: 逐项填写
  POST /type  {"selector":"input[name=\"name\"]","text":"张三"}
  POST /type  {"selector":"input[name=\"email\"]","text":"zhangsan@example.com"}
  POST /type  {"selector":"textarea[name=\"bio\"]","text":"软件工程师，5年经验..."}
  POST /select  {"selector":"select[name=\"city\"]","value":"beijing"}

步骤 3: 确认填写结果
  GET /snapshot → 检查 value 是否正确

步骤 4: 提交
  POST /click  {"selector":"button[type=\"submit\"]"}
  POST /wait  {"selector":".success, .result","timeout":10000}
```

---

### 场景 7：GitHub 仓库调研

> **任务**: "帮我看看 GitHub 上最近有什么热门的 AI Agent 项目"

```
步骤 1:
  POST /navigate  {"url":"https://github.com/search?q=ai+agent&type=repositories&s=stars&o=desc"}

步骤 2:
  POST /wait  {"selector":".repo-list-item, .Box-row","timeout":10000}

步骤 3: 提取仓库列表
  POST /extract  {"selector":".Box-row a.v-align-middle","limit":20}

步骤 4: 点入查看 README
  POST /click  {"selector":"第一个仓库的 selector"}
  POST /wait  {"selector":"article, .markdown-body","timeout":8000}
  POST /extract-text  {"selector":".markdown-body"}

步骤 5: 汇总仓库名 + star 数 + 简介
```

---

### 场景 8：新闻网站每日摘要

> **任务**: "每天帮我看看 36Kr 的头条新闻"

```
步骤 1:
  POST /navigate  {"url":"https://36kr.com"}

步骤 2:
  POST /wait  {"selector":"article, .article-item","timeout":10000}

步骤 3: 提取首页文章列表
  POST /extract  {"selector":"a.article-item-title, .flow-item a","limit":20}

步骤 4: 逐篇阅读
  POST /click  {"selector":"第一篇文章"}
  POST /wait  {"selector":".article-content","timeout":8000}
  POST /extract-text  {"selector":".article-content"}
  POST /back

步骤 5: 生成每日摘要
  标题 + 关键信息 + 一句话总结
```

---

### 场景 9：B 站视频信息收集

> **任务**: "帮我看看 B 站最近关于 Rust 编程的热门视频"

```
步骤 1:
  POST /navigate  {"url":"https://search.bilibili.com/all?keyword=Rust编程&order=click"}

步骤 2:
  POST /wait  {"selector":".video-list-item, .bili-video-card","timeout":10000}

步骤 3: 提取视频列表（标题 + 播放量 + 链接）
  POST /extract  {"selector":".bili-video-card","limit":20}

步骤 4: 如有需要读评论
  POST /click  {"selector":"视频链接"}
  POST /wait  {"selector":".reply-list, .comment","timeout":10000}
  POST /scroll  {"y": 800}
  POST /extract  {"selector":".reply-item","limit":30}
```

---

### 场景 10：多平台比价

> **任务**: "帮我对比下淘宝和京东上 AirPods Pro 2 的价格"

```
平台 1 — 京东:
  POST /navigate  {"url":"https://search.jd.com/Search?keyword=AirPods+Pro+2"}
  POST /wait  {"selector":".gl-item","timeout":10000}
  POST /extract  {"selector":".gl-item","limit":5}

平台 2 — 淘宝:
  POST /navigate  {"url":"https://s.taobao.com/search?q=AirPods+Pro+2"}
  POST /wait  {"selector":".items .item","timeout":10000}
  POST /extract  {"selector":".items .item","limit":5}

汇总: 两平台价格对比表 + 推荐
```

---

## 五、高级技巧

### 5.1 处理无限滚动页面

很多现代网站（微博/小红书/知乎）使用无限滚动加载。策略：

```
记录 = []
最大滚动次数 = 10
for i in 1..最大滚动次数:
    items = POST /extract {"selector":"目标元素"}
    记录 += items（去重）
    old_max_y = snapshot.meta.scroll.maxY
    POST /scroll {"y": 当前y + 800}
    等 2 秒
    new_snapshot = GET /snapshot
    if new_snapshot.meta.scroll.maxY == old_max_y:
        break  # 没有更多内容了
```

### 5.2 处理登录态页面

如果页面需要登录：
1. 先 `/snapshot` 看是否有登录表单
2. 如果有，通知用户"需要先登录 X 网站"
3. 用户手动登录后（浏览器保持 Cookie），再继续任务
4. 不要尝试自动输入密码——安全原因

### 5.3 处理弹窗/Cookie 提示

```
1. /snapshot 后检查是否有 role=button 且 text 包含"接受"/"同意"/"关闭"/"×" 的元素
2. 如果有 → /click 关掉它
3. 重新 /snapshot 获取真正的页面内容
```

### 5.4 读取 SPA 页面数据

React/Vue/Next.js 等 SPA 应用的数据通常挂在全局变量上：

```bash
# Next.js
POST /eval {"script":"JSON.stringify(window.__NEXT_DATA__.props.pageProps)"}

# Nuxt.js
POST /eval {"script":"JSON.stringify(window.__NUXT__)"}

# 通用：读取 Redux Store
POST /eval {"script":"JSON.stringify(window.__REDUX_STATE__ || window.__INITIAL_STATE__)"}
```

### 5.5 当 selector 找不到目标时

如果 `/snapshot` 返回的 elements 中没有你要的目标：
1. 目标可能在视口外 → `/scroll` 向下翻后重新 `/snapshot`
2. 目标可能还没加载 → `/wait` 等一等
3. 目标可能在 iframe 中 → 无法直接操作，告知用户
4. 用 `/eval` 写更灵活的 JS 查询：
   ```bash
   POST /eval {"script":"document.querySelector('.some-class')?.textContent"}
   ```

### 5.6 长文本内容提取

`/extract-text` 专为阅读文章设计，它会提取 p/h1-h6/li/blockquote 等语义块：

```bash
# 读取整篇文章
POST /extract-text  {"selector":"article"}
# → {"text": "标题\n正文第一段\n正文第二段\n..."}

# 读取整个页面（不指定 selector）
POST /extract-text  {}
```

---

## 六、输出格式指南

采集完数据后，按以下格式整理：

### 新闻类

```markdown
## 📰 [主题] 最新动态 — YYYY-MM-DD

### 1. [新闻标题]
- 来源: [平台/作者]
- 时间: [发布时间]
- 摘要: [100字以内的关键内容]
- 链接: [URL]

### 2. [新闻标题]
...

---
**总结**: [3-5句概括当天总体态势]
```

### 产品调研类

```markdown
## 🔍 [产品名] 调研报告

### 用户评价汇总
| 维度 | 正面 | 负面 |
|------|------|------|
| 外观 | ... | ... |
| 性能 | ... | ... |
| 价格 | ... | ... |

### 典型用户声音
1. "[原文摘录]" —— @用户名
2. ...

### 结论
[综合评价与建议]
```

### 价格对比类

```markdown
## 💰 [商品名] 价格对比

| 平台 | 价格 | 优惠 | 链接 |
|------|------|------|------|
| 京东 | ¥xxx | 满减xx | [链接] |
| 淘宝 | ¥xxx | 无 | [链接] |

**推荐**: [最佳购买建议]
```

---

## 七、错误处理

| 错误 | 原因 | 处理 |
|------|------|------|
| `curl: (7) Failed to connect` | Claw Browser 未运行 | 告知用户「请先启动 Claw Browser」 |
| HTTP 400 `No active tab` | 无活动标签页 | 先 `/navigate` 打开一个页面 |
| HTTP 408 `/wait` 超时 | 元素未出现 | 可能页面结构变了，用 `/snapshot` 重新观察 |
| `eval timeout` / `snapshot timeout` | 页面响应慢 | 等 3 秒后重试，最多重试 2 次 |
| HTTP 5xx | 内部错误 | 告知用户检查 Claw Browser |

**通用重试策略**：任何操作失败后，先 `/page-info` 确认页面是否还活着，再决定重试还是报错。

---

## 八、安全与边界

1. **不自动输入密码** — 如果需要登录，提示用户手动登录
2. **不操作支付页面** — 涉及金钱的操作必须让用户确认
3. **不绕过反爬机制** — 如遇到验证码，告知用户手动处理
4. **尊重频率** — 连续操作之间至少间隔 1 秒，避免触发反爬
5. **数据仅用于汇报** — 采集的内容只用于整理后呈现给用户

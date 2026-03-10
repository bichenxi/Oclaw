---
name: xhs
version: 2.0
description: 小红书（xiaohongshu.com）专属操控技能。在小红书页面上优先使用 window.__clawXhs 系列辅助函数，通过 /eval 调用，比通用 selector 更稳定。
---

# 小红书专属技能

> 在小红书（xiaohongshu.com / xhslink.com）页面上，**优先使用本文件的专属 API**，
> 而非通用 `/click`、`/type` 等端点。所有专属函数通过 `/eval` 调用。

---

## 前置检查

进入小红书任何页面后，**第一步**先确认登录状态：

```bash
curl -s -X POST http://127.0.0.1:18790/eval \
  -H 'Content-Type: application/json' \
  -d '{"script":"window.__clawXhs.isLoggedIn()"}'
# 返回 {"ok":true,"value":false} → 告知用户「请先手动登录小红书，完成后告诉我」
# 返回 {"ok":true,"value":true}  → 继续操作
```

---

## 专属 API 速查

所有函数均通过 `POST /eval` 的 `script` 字段调用。

### 搜索

```bash
# 1. 填写搜索关键词
curl -s -X POST http://127.0.0.1:18790/eval \
  -H 'Content-Type: application/json' \
  -d '{"script":"window.__clawXhs.setSearchInput(\"关键词\")"}'

# 2. 触发搜索（点击搜索按钮或回车）
curl -s -X POST http://127.0.0.1:18790/eval \
  -H 'Content-Type: application/json' \
  -d '{"script":"window.__clawXhs.clickSearch()"}'

# 等结果加载
curl -s -X POST http://127.0.0.1:18790/wait \
  -H 'Content-Type: application/json' \
  -d '{"selector":".feeds-container, #feeds-container","timeout":8000}'
```

### 获取 Feed 列表

```bash
# 返回 [{index, type:"note"|"video", title, cover, author, likes}, ...]
curl -s -X POST http://127.0.0.1:18790/eval \
  -H 'Content-Type: application/json' \
  -d '{"script":"JSON.stringify(window.__clawXhs.getResults(20))"}'
```

返回字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `index` | number | 在列表中的位置（用于 openNote） |
| `type` | `"note"` \| `"video"` | 笔记类型 |
| `title` | string | 标题 |
| `author` | string | 作者名 |
| `likes` | string | 点赞数（文本） |

### 打开 / 关闭笔记

```bash
# 打开第 0 条（index 来自 getResults 返回）
curl -s -X POST http://127.0.0.1:18790/eval \
  -H 'Content-Type: application/json' \
  -d '{"script":"window.__clawXhs.openNote(0)"}'

# 等弹层出现
curl -s -X POST http://127.0.0.1:18790/wait \
  -H 'Content-Type: application/json' \
  -d '{"selector":".note-detail-mask, .note-scroller","timeout":6000}'

# 读笔记内容（标题、正文、作者、日期、点赞/收藏/评论数、标签、前10条评论）
curl -s -X POST http://127.0.0.1:18790/eval \
  -H 'Content-Type: application/json' \
  -d '{"script":"JSON.stringify(window.__clawXhs.getNoteContent())"}'

# 读笔记图片列表（URL 数组）
curl -s -X POST http://127.0.0.1:18790/eval \
  -H 'Content-Type: application/json' \
  -d '{"script":"JSON.stringify(window.__clawXhs.getNoteImages())"}'

# 关闭弹层
curl -s -X POST http://127.0.0.1:18790/eval \
  -H 'Content-Type: application/json' \
  -d '{"script":"window.__clawXhs.closeNote()"}'
```

### 滚动加载更多

```bash
# 向下滚动 800px，触发瀑布流加载
curl -s -X POST http://127.0.0.1:18790/eval \
  -H 'Content-Type: application/json' \
  -d '{"script":"window.__clawXhs.scrollFeed()"}'

# 等新内容加载后再调 getResults
```

---

## 标准任务模板

### 搜索并汇报前 N 条笔记

```
1. navigate → https://www.xiaohongshu.com
2. eval → isLoggedIn()，未登录则停
3. eval → setSearchInput("关键词")
4. eval → clickSearch()
5. wait → .feeds-container（8s）
6. eval → JSON.stringify(getResults(10))
7. 汇报标题、作者、点赞数
```

### 读取某篇笔记详情

```
1. eval → openNote(index)
2. wait → .note-scroller（6s）
3. eval → JSON.stringify(getNoteContent())
4. 汇报标题、正文、标签、热门评论
5. eval → closeNote()
```

---

## 注意事项

1. **`window.__clawXhs` 只在小红书页面有效**，其他网站调用会返回 `undefined`
2. 笔记弹层打开时，`isNoteOpen()` 返回 `true`，此时 `closeNote()` 才有效
3. 小红书反爬较严，每步操作之间等待 **800ms 以上**
4. 遇到滑块验证码，立即叫用户手动完成，不要尝试自动化
5. 函数返回 `false` 或空数组，通常说明页面结构变化，回退到通用 `/snapshot` + `/click` 方式

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useTabsStore } from '@/stores/tabs'

const store = useTabsStore()
const highlightSelector = ref('')
const highlightError = ref('')

// 占位：后续由 OpenClaw 流式推送
const streamItems = ref<{ type: 'thought' | 'tool'; text: string }[]>([
  { type: 'thought', text: '（接通 OpenClaw 后，思考链将在此流式显示）' },
  { type: 'tool', text: '（工具调用与执行结果将在此显示）' },
])

async function runHighlight() {
  const sel = highlightSelector.value.trim()
  if (!sel) {
    highlightError.value = '请输入选择器'
    return
  }
  const label = store.activeTabId
  if (!label) {
    highlightError.value = '请先打开一个网页 tab'
    return
  }
  highlightError.value = ''
  // 使用自包含脚本：不依赖 __clawBridge，避免注入被 CSP 限制时仍能高亮
  const script = `(function(){
    var el = document.querySelector(${JSON.stringify(sel)});
    if (!el) return;
    var r = el.getBoundingClientRect();
    var wrap = document.createElement('div');
    wrap.setAttribute('data-claw-highlight','1');
    wrap.style.cssText = 'position:fixed;left:' + r.left + 'px;top:' + r.top + 'px;width:' + r.width + 'px;height:' + r.height + 'px;border:3px solid #e11;box-shadow:0 0 0 2px #f88;pointer-events:none;z-index:2147483647;box-sizing:border-box;';
    document.body.appendChild(wrap);
    setTimeout(function(){ if(wrap.parentNode) wrap.parentNode.removeChild(wrap); }, 2500);
  })();`
  try {
    await invoke('eval_in_webview', { label, script })
  } catch (e) {
    highlightError.value = String(e)
  }
}
</script>

<template>
  <div class="ai-console">
    <div class="ai-console-header">
      <span class="ai-console-title">AI 控制台</span>
      <span class="ai-console-sub">思考链 · 工具调用</span>
    </div>
    <div class="ai-console-body">
      <div class="ai-console-takeover">
        <button
          type="button"
          class="takeover-btn"
          :class="{ active: store.aiPaused }"
          @click="store.setAiPaused(!store.aiPaused)"
        >
          {{ store.aiPaused ? '继续 AI' : '暂停 AI（手动接管）' }}
        </button>
        <p v-if="store.aiPaused" class="takeover-hint">
          你正在手动操作右侧网页，完成验证码等操作后点击「继续 AI」。
        </p>
      </div>
      <div class="ai-console-stream">
        <div
          v-for="(item, i) in streamItems"
          :key="i"
          class="stream-item"
          :class="item.type"
        >
          <span class="stream-tag">{{ item.type === 'thought' ? 'Thought' : 'Tool' }}</span>
          <span class="stream-text">{{ item.text }}</span>
        </div>
      </div>
      <div class="ai-console-highlight">
        <label class="highlight-label">高亮测试（当前 tab 内元素）</label>
        <div class="highlight-row">
          <input
            v-model="highlightSelector"
            type="text"
            placeholder="如: #login-btn 或 button"
            class="highlight-input"
            @keydown.enter="runHighlight"
          />
          <button type="button" class="highlight-btn" @click="runHighlight">
            高亮
          </button>
        </div>
        <p v-if="highlightError" class="highlight-error">{{ highlightError }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ai-console {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #faf8ff;
  border-right: 1px solid #e8e2f4;
}

.ai-console-header {
  flex-shrink: 0;
  padding: 12px 16px;
  border-bottom: 1px solid #e8e2f4;
}

.ai-console-title {
  font-size: 14px;
  font-weight: 600;
  color: #5f47ce;
  display: block;
}

.ai-console-sub {
  font-size: 11px;
  color: #9b8ec4;
  margin-top: 2px;
  display: block;
}

.ai-console-body {
  flex: 1;
  overflow: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.ai-console-takeover {
  flex-shrink: 0;
  padding-bottom: 12px;
  border-bottom: 1px solid #e8e2f4;
}

.takeover-btn {
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  color: #5f47ce;
  background: rgba(95, 71, 206, 0.08);
  border: 1px solid rgba(95, 71, 206, 0.25);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.takeover-btn:hover {
  background: rgba(95, 71, 206, 0.12);
}

.takeover-btn.active {
  color: #22c55e;
  background: rgba(34, 197, 94, 0.1);
  border-color: rgba(34, 197, 94, 0.3);
}

.takeover-hint {
  font-size: 12px;
  color: #8a80a7;
  margin: 8px 0 0 0;
  line-height: 1.4;
}

.ai-console-stream {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.stream-item {
  font-size: 12px;
  padding: 8px 10px;
  border-radius: 8px;
  border-left: 3px solid transparent;
}

.stream-item.thought {
  background: rgba(95, 71, 206, 0.06);
  border-left-color: #5f47ce;
}

.stream-item.tool {
  background: rgba(34, 197, 94, 0.06);
  border-left-color: #22c55e;
}

.stream-tag {
  display: block;
  font-weight: 600;
  color: #8a80a7;
  margin-bottom: 4px;
}

.stream-text {
  color: #1f1f2e;
  line-height: 1.4;
}

.ai-console-highlight {
  margin-top: auto;
  padding-top: 12px;
  border-top: 1px solid #e8e2f4;
}

.highlight-label {
  font-size: 12px;
  color: #8a80a7;
  display: block;
  margin-bottom: 6px;
}

.highlight-row {
  display: flex;
  gap: 8px;
}

.highlight-input {
  flex: 1;
  padding: 6px 10px;
  font-size: 12px;
  border: 1px solid #e8e2f4;
  border-radius: 6px;
  outline: none;
}

.highlight-input:focus {
  border-color: #5f47ce;
}

.highlight-btn {
  padding: 6px 12px;
  font-size: 12px;
  color: #5f47ce;
  background: rgba(95, 71, 206, 0.1);
  border: 1px solid rgba(95, 71, 206, 0.25);
  border-radius: 6px;
  cursor: pointer;
}

.highlight-btn:hover {
  background: rgba(95, 71, 206, 0.15);
}

.highlight-error {
  font-size: 11px;
  color: #ef4444;
  margin: 6px 0 0 0;
}
</style>

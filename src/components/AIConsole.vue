<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import { useTabsStore } from '@/stores/tabs'
import { useRecordingStore } from '@/stores/recording'
import type { RecordingStep } from '@/stores/recording'
import { evalInWebview, getDomSnapshot } from '@/api/webview'
import { simulateStream } from '@/api/app'
import {
  startOpenclawProcess,
  stopOpenclawProcess,
  isOpenclawProcessRunning,
  openclawSendV1,
  type OpenclawV1Params,
} from '@/api/openclaw'

const store = useTabsStore()
const recordingStore = useRecordingStore()
const replaying = ref(false)
const highlightSelector = ref('')
const highlightError = ref('')
const domSnapshot = ref<string | null>(null)
const domSnapshotLoading = ref(false)

// 由 OpenClaw / stream-item 事件流式推送；user 为本地发送的消息
const streamItems = ref<{ type: 'thought' | 'tool' | 'user'; text: string }[]>([
  { type: 'thought', text: '（接通 OpenClaw 后，思考链将在此流式显示）' },
  { type: 'tool', text: '（工具调用与执行结果将在此显示）' },
])
const isPlaceholder = (items: { type: string; text: string }[]) =>
  items.length === 2 &&
  items[0].text.includes('接通 OpenClaw') &&
  items[1].text.includes('工具调用与执行结果')
const streamSimulating = ref(false)

const openclawRunning = ref(false)
const openclawBusy = ref(false)
async function refreshOpenclawStatus() {
  try {
    openclawRunning.value = await isOpenclawProcessRunning()
  } catch {
    openclawRunning.value = false
  }
}
async function runStartOpenclaw() {
  openclawBusy.value = true
  try {
    await startOpenclawProcess()
    await refreshOpenclawStatus()
  } catch (e) {
    console.error(e)
  } finally {
    openclawBusy.value = false
  }
}
async function runStopOpenclaw() {
  openclawBusy.value = true
  try {
    await stopOpenclawProcess()
    await refreshOpenclawStatus()
  } catch (e) {
    console.error(e)
  } finally {
    openclawBusy.value = false
  }
}

const openclawInput = ref('')
const openclawToken = ref('')
const openclawSessionKey = ref('agent:main:main2')
const openclawSendLoading = ref(false)
const openclawSendError = ref('')
async function runOpenclawSend() {
  const input = openclawInput.value.trim()
  if (!input) {
    openclawSendError.value = '请输入内容'
    return
  }
  openclawSendError.value = ''
  if (isPlaceholder(streamItems.value)) {
    streamItems.value = []
  }
  streamItems.value.push({ type: 'user', text: input })
  openclawSendLoading.value = true
  try {
    const params: OpenclawV1Params = {
      input,
      stream: true,
    }
    if (openclawToken.value.trim()) params.token = openclawToken.value.trim()
    if (openclawSessionKey.value.trim()) params.session_key = openclawSessionKey.value.trim()
    await openclawSendV1(params)
  } catch (e) {
    openclawSendError.value = String(e)
  } finally {
    openclawSendLoading.value = false
  }
}

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
    await evalInWebview(label, script)
  } catch (e) {
    highlightError.value = String(e)
  }
}

async function fetchDomSnapshot() {
  const label = store.activeTabId
  if (!label) {
    domSnapshot.value = null
    return
  }
  domSnapshotLoading.value = true
  domSnapshot.value = null
  try {
    await getDomSnapshot(label)
    // 结果通过 dom-snapshot 事件异步回传，由 listen 写入 domSnapshot
  } catch (e) {
    domSnapshot.value = '获取失败: ' + String(e)
  } finally {
    domSnapshotLoading.value = false
  }
}

function stepSummary(step: RecordingStep): string {
  if (step.type === 'navigate') return step.url
  const s = step.script.slice(0, 50)
  return (s.length < step.script.length ? s + '…' : s) || 'eval'
}

async function replayToStepIndex(index: number) {
  const label = store.activeTabId
  if (!label) return
  replaying.value = true
  try {
    await recordingStore.replayToStep(label, index)
  } finally {
    replaying.value = false
  }
}

async function runSimulateStream() {
  streamSimulating.value = true
  try {
    await simulateStream()
  } finally {
    streamSimulating.value = false
  }
}

onMounted(() => {
  refreshOpenclawStatus()
  listen<string>('dom-snapshot', (e) => {
    domSnapshot.value = e.payload
  })
  listen<{ type: string; text: string }>('stream-item', (e) => {
    const payload = e.payload
    if (!payload?.type || !payload?.text) return
    const type = payload.type === 'tool' ? 'tool' : 'thought'
    if (isPlaceholder(streamItems.value)) {
      streamItems.value = [{ type, text: payload.text }]
    } else {
      streamItems.value.push({ type, text: payload.text })
    }
  })
})
</script>

<template>
  <div class="ai-console">
    <div class="ai-console-header">
      <span class="ai-console-title">AI 控制台</span>
      <span class="ai-console-sub">思考链 · 工具调用</span>
    </div>
    <div class="ai-console-body">
      <div class="ai-console-openclaw-process">
        <span class="openclaw-process-label">OpenClaw</span>
        <span class="openclaw-process-status">{{ openclawRunning ? '已运行' : '未运行' }}</span>
        <button
          type="button"
          class="openclaw-process-btn"
          :disabled="openclawBusy || openclawRunning"
          @click="runStartOpenclaw"
        >
          启动
        </button>
        <button
          type="button"
          class="openclaw-process-btn"
          :disabled="openclawBusy || !openclawRunning"
          @click="runStopOpenclaw"
        >
          停止
        </button>
      </div>
      <div class="ai-console-openclaw-send">
        <label class="openclaw-send-label">发送到 OpenClaw (HTTP /v1/responses)</label>
        <input
          v-model="openclawToken"
          type="password"
          placeholder="Bearer Token（可选，或设 OPENCLAW_BEARER_TOKEN）"
          class="openclaw-send-input"
          autocomplete="off"
        />
        <input
          v-model="openclawSessionKey"
          type="text"
          placeholder="x-openclaw-session-key"
          class="openclaw-send-input"
        />
        <textarea
          v-model="openclawInput"
          placeholder="输入消息，如：帮我去小红书搜 3 月青州旅游"
          class="openclaw-send-textarea"
          rows="2"
        />
        <p v-if="openclawSendError" class="openclaw-send-error">{{ openclawSendError }}</p>
        <button
          type="button"
          class="openclaw-send-btn"
          :disabled="openclawSendLoading"
          @click="runOpenclawSend"
        >
          {{ openclawSendLoading ? '发送中…' : '发送' }}
        </button>
      </div>
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
        <div class="stream-header-row">
          <span class="stream-label">思考链 · 工具调用</span>
          <button
            type="button"
            class="stream-simulate-btn"
            :disabled="streamSimulating"
            @click="runSimulateStream"
          >
            {{ streamSimulating ? '模拟中…' : '模拟流式输出' }}
          </button>
        </div>
        <div
          v-for="(item, i) in streamItems"
          :key="i"
          class="stream-item"
          :class="item.type"
        >
          <span class="stream-tag">
            {{ item.type === 'thought' ? 'Thought' : item.type === 'tool' ? 'Tool' : '我' }}
          </span>
          <span class="stream-text">{{ item.text }}</span>
        </div>
      </div>
      <div class="ai-console-recording">
        <label class="recording-label">操作录制</label>
        <div v-if="recordingStore.steps.length === 0" class="recording-empty">
          打开网页或执行操作后将在此记录步骤
        </div>
        <ul v-else class="recording-list">
          <li
            v-for="(step, i) in recordingStore.steps"
            :key="i"
            class="recording-item"
          >
            <span class="recording-step-num">{{ i + 1 }}</span>
            <span class="recording-step-type">{{ step.type }}</span>
            <span class="recording-step-summary">{{ stepSummary(step) }}</span>
            <button
              type="button"
              class="recording-replay-btn"
              :disabled="replaying || !store.activeTabId"
              @click="replayToStepIndex(i)"
            >
              回放到此步
            </button>
          </li>
        </ul>
        <button
          v-if="recordingStore.steps.length > 0"
          type="button"
          class="recording-clear-btn"
          @click="recordingStore.clearSteps()"
        >
          清空录制
        </button>
      </div>
      <div class="ai-console-domsnapshot">
        <label class="snapshot-label">DOM 提纯（可交互元素快照）</label>
        <button
          type="button"
          class="snapshot-btn"
          :disabled="!store.activeTabId || domSnapshotLoading"
          @click="fetchDomSnapshot"
        >
          {{ domSnapshotLoading ? '获取中…' : '获取 DOM 快照' }}
        </button>
        <div v-if="domSnapshot" class="snapshot-output">
          <pre>{{ domSnapshot }}</pre>
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

.ai-console-openclaw-process {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e8e2f4;
}
.openclaw-process-label { font-weight: 600; font-size: 13px; color: #5f47ce; }
.openclaw-process-status { font-size: 12px; color: #9b8ec4; }
.openclaw-process-btn {
  padding: 4px 10px;
  font-size: 12px;
  color: #5f47ce;
  background: rgba(95, 71, 206, 0.08);
  border: 1px solid rgba(95, 71, 206, 0.25);
  border-radius: 6px;
  cursor: pointer;
}
.openclaw-process-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.ai-console-openclaw-send {
  flex-shrink: 0;
  padding-bottom: 12px;
  border-bottom: 1px solid #e8e2f4;
}
.openclaw-send-label { font-size: 12px; color: #5f47ce; font-weight: 600; display: block; margin-bottom: 6px; }
.openclaw-send-input {
  width: 100%;
  padding: 6px 8px;
  font-size: 12px;
  border: 1px solid #e8e2f4;
  border-radius: 6px;
  margin-bottom: 6px;
  box-sizing: border-box;
}
.openclaw-send-textarea {
  width: 100%;
  padding: 8px;
  font-size: 13px;
  border: 1px solid #e8e2f4;
  border-radius: 6px;
  margin-bottom: 6px;
  resize: vertical;
  min-height: 52px;
  box-sizing: border-box;
}
.openclaw-send-error { font-size: 12px; color: #c00; margin-bottom: 6px; }
.openclaw-send-btn {
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  color: #fff;
  background: #5f47ce;
  border: none;
  border-radius: 8px;
  cursor: pointer;
}
.openclaw-send-btn:disabled { opacity: 0.7; cursor: not-allowed; }

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

.stream-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 4px;
}

.stream-label {
  font-size: 11px;
  color: #8a80a7;
}

.stream-simulate-btn {
  padding: 4px 8px;
  font-size: 11px;
  color: #5f47ce;
  background: rgba(95, 71, 206, 0.08);
  border: 1px solid rgba(95, 71, 206, 0.2);
  border-radius: 4px;
  cursor: pointer;
}

.stream-simulate-btn:hover:not(:disabled) {
  background: rgba(95, 71, 206, 0.14);
}

.stream-simulate-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
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

.stream-item.user {
  background: rgba(99, 102, 241, 0.08);
  border-left-color: #6366f1;
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

.ai-console-recording {
  flex-shrink: 0;
}

.recording-label {
  font-size: 12px;
  color: #8a80a7;
  display: block;
  margin-bottom: 6px;
}

.recording-empty {
  font-size: 12px;
  color: #9b8ec4;
  padding: 8px 0;
}

.recording-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 180px;
  overflow: auto;
}

.recording-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  font-size: 11px;
  border-radius: 6px;
  margin-bottom: 4px;
  background: rgba(95, 71, 206, 0.05);
  border: 1px solid rgba(95, 71, 206, 0.1);
}

.recording-step-num {
  flex-shrink: 0;
  width: 18px;
  text-align: center;
  color: #5f47ce;
  font-weight: 600;
}

.recording-step-type {
  flex-shrink: 0;
  color: #8a80a7;
  text-transform: uppercase;
}

.recording-step-summary {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #1f1f2e;
}

.recording-replay-btn {
  flex-shrink: 0;
  padding: 4px 8px;
  font-size: 10px;
  color: #5f47ce;
  background: rgba(95, 71, 206, 0.1);
  border: 1px solid rgba(95, 71, 206, 0.2);
  border-radius: 4px;
  cursor: pointer;
}

.recording-replay-btn:hover:not(:disabled) {
  background: rgba(95, 71, 206, 0.18);
}

.recording-replay-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.recording-clear-btn {
  margin-top: 8px;
  padding: 4px 10px;
  font-size: 11px;
  color: #8a80a7;
  background: transparent;
  border: 1px solid #e8e2f4;
  border-radius: 4px;
  cursor: pointer;
}

.recording-clear-btn:hover {
  color: #5f47ce;
  border-color: rgba(95, 71, 206, 0.3);
}

.ai-console-domsnapshot {
  flex-shrink: 0;
}

.snapshot-label {
  font-size: 12px;
  color: #8a80a7;
  display: block;
  margin-bottom: 6px;
}

.snapshot-btn {
  width: 100%;
  padding: 6px 12px;
  font-size: 12px;
  color: #5f47ce;
  background: rgba(95, 71, 206, 0.08);
  border: 1px solid rgba(95, 71, 206, 0.2);
  border-radius: 6px;
  cursor: pointer;
  margin-bottom: 8px;
}

.snapshot-btn:hover:not(:disabled) {
  background: rgba(95, 71, 206, 0.12);
}

.snapshot-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.snapshot-output {
  max-height: 160px;
  overflow: auto;
  font-size: 11px;
  background: #1f1f2e;
  color: #e8e2f4;
  border-radius: 6px;
  padding: 8px;
}

.snapshot-output pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
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

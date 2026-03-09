<script setup lang="ts">
import { useSettingsStore } from '@/stores/settings'
import { useTabsStore } from '@/stores/tabs'
import {
  isOpenclawProcessRunning,
  startOpenclawProcess,
  stopOpenclawProcess,
} from '@/api/openclaw'

const settings = useSettingsStore()
const store = useTabsStore()

const tokenInput = ref(settings.bearerToken)
const sessionKeyInput = ref(settings.sessionKey)
const baseUrlInput = ref(settings.baseUrl)
const saved = ref(false)
const openclawRunning = ref(false)
const openclawBusy = ref(false)

async function refreshStatus() {
  try {
    openclawRunning.value = await isOpenclawProcessRunning()
  } catch {
    openclawRunning.value = false
  }
}

async function startProcess() {
  openclawBusy.value = true
  try {
    await startOpenclawProcess()
    await refreshStatus()
  } catch (e) {
    console.error(e)
  } finally {
    openclawBusy.value = false
  }
}

async function stopProcess() {
  openclawBusy.value = true
  try {
    await stopOpenclawProcess()
    await refreshStatus()
  } catch (e) {
    console.error(e)
  } finally {
    openclawBusy.value = false
  }
}

function save() {
  settings.save(tokenInput.value, sessionKeyInput.value, baseUrlInput.value)
  saved.value = true
  setTimeout(() => {
    saved.value = false
  }, 2000)
}

onMounted(() => {
  refreshStatus()
})
</script>

<template>
  <div class="h-full flex flex-col bg-[#f8f6ff] overflow-hidden">
    <!-- Header -->
    <div class="shrink-0 flex items-center justify-between px-6 py-3.5 bg-white border-b border-[#e8e2f4]">
      <div class="flex items-center gap-3">
        <img
          class="w-9 h-9 rounded-[10px] object-cover shadow-[0_2px_10px_rgba(0,0,0,0.12)] shrink-0"
          src="/logo.jpg"
          alt="logo"
        />
        <div class="flex flex-col">
          <span class="text-[16px] font-bold text-[#1f1f2e] leading-[1.2]">设置</span>
          <span class="text-[11px] text-[#9b8ec4] mt-px">OpenClaw 连接配置</span>
        </div>
      </div>
      <button
        type="button"
        class="flex items-center gap-1.5 px-3.5 py-[7px] text-[13px] text-secondary bg-secondary/8 border border-secondary/20 rounded-[8px] cursor-pointer transition hover:bg-secondary/13"
        @click="store.switchToSpecialView('openclaw')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
        </svg>
        前往对话
      </button>
    </div>

    <!-- Body -->
    <div class="sp-body flex-1 overflow-y-auto p-6 flex flex-col gap-4 max-w-[700px] w-full self-center box-border">
      <!-- OpenClaw 进程控制 -->
      <div class="bg-white border border-[#e8e2f4] rounded-[12px] p-5 flex flex-col gap-4">
        <div class="flex items-center gap-[7px] text-[13px] font-semibold text-secondary">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
          </svg>
          OpenClaw 进程
        </div>
        <div class="flex items-center gap-2.5">
          <!-- 状态徽章 -->
          <div
            class="flex items-center gap-[5px] px-3 py-[5px] rounded-[20px] text-[12px] font-medium"
            :class="openclawRunning
              ? 'bg-[rgba(34,197,94,0.1)] text-[#16a34a]'
              : 'bg-[rgba(107,114,128,0.1)] text-[#6b7280]'"
          >
            <span
              class="w-1.5 h-1.5 rounded-full bg-current"
              :class="{ 'animate-[sp-pulse_1.5s_ease-in-out_infinite]': openclawRunning }"
            />
            {{ openclawRunning ? '运行中' : '未运行' }}
          </div>
          <!-- 启动按钮 -->
          <button
            v-if="!openclawRunning"
            type="button"
            class="px-3.5 py-1.5 text-[12px] rounded-[6px] border cursor-pointer transition disabled:opacity-50 disabled:cursor-not-allowed text-[#16a34a] border-[rgba(34,197,94,0.3)] bg-[rgba(34,197,94,0.08)] hover:not-disabled:bg-[rgba(34,197,94,0.14)]"
            :disabled="openclawBusy"
            @click="startProcess"
          >
            启动
          </button>
          <!-- 停止按钮 -->
          <button
            v-else
            type="button"
            class="px-3.5 py-1.5 text-[12px] rounded-[6px] border cursor-pointer transition disabled:opacity-50 disabled:cursor-not-allowed text-[#dc2626] border-[rgba(239,68,68,0.3)] bg-[rgba(239,68,68,0.07)] hover:not-disabled:bg-[rgba(239,68,68,0.13)]"
            :disabled="openclawBusy"
            @click="stopProcess"
          >
            停止
          </button>
          <!-- 刷新状态 -->
          <button
            type="button"
            class="px-3.5 py-1.5 text-[12px] rounded-[6px] border border-secondary/25 text-secondary bg-secondary/7 cursor-pointer transition disabled:opacity-50 disabled:cursor-not-allowed hover:not-disabled:bg-secondary/13"
            :disabled="openclawBusy"
            @click="refreshStatus"
          >
            刷新状态
          </button>
        </div>
      </div>

      <!-- 认证配置 -->
      <div class="bg-white border border-[#e8e2f4] rounded-[12px] p-5 flex flex-col gap-4">
        <div class="flex items-center gap-[7px] text-[13px] font-semibold text-secondary">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
          </svg>
          认证配置
        </div>
        <!-- Bearer Token -->
        <div class="flex flex-col gap-2">
          <label class="flex flex-col gap-0.5 text-[13px] font-semibold text-[#1f1f2e]">
            OPENCLAW_BEARER_TOKEN
            <span class="text-[11px] font-normal text-[#9b8ec4]">Bearer Token，用于 API 鉴权</span>
          </label>
          <div class="relative">
            <input
              v-model="tokenInput"
              type="password"
              class="w-full px-3.5 py-2.5 text-[13px] font-[inherit] border-[1.5px] border-[#e8e2f4] rounded-[8px] outline-none box-border text-[#1f1f2e] bg-[#fafafa] transition placeholder-[#c4bdd8] placeholder:text-[12px] focus:border-[#7c5cfc] focus:bg-white focus:shadow-[0_0_0_3px_rgba(95,71,206,0.08)]"
              placeholder="输入 Bearer Token（若已设置环境变量可留空）"
              autocomplete="off"
              @blur="tokenInput = tokenInput.trim()"
            />
          </div>
        </div>
        <!-- Session Key -->
        <div class="flex flex-col gap-2">
          <label class="flex flex-col gap-0.5 text-[13px] font-semibold text-[#1f1f2e]">
            OPENCLAW_SESSION_KEY
            <span class="text-[11px] font-normal text-[#9b8ec4]">会话标识，用于隔离不同对话</span>
          </label>
          <div class="relative">
            <input
              v-model="sessionKeyInput"
              type="text"
              class="w-full px-3.5 py-2.5 text-[13px] font-[inherit] border-[1.5px] border-[#e8e2f4] rounded-[8px] outline-none box-border text-[#1f1f2e] bg-[#fafafa] transition placeholder-[#c4bdd8] placeholder:text-[12px] focus:border-[#7c5cfc] focus:bg-white focus:shadow-[0_0_0_3px_rgba(95,71,206,0.08)]"
              placeholder="如：agent:main:main2"
            />
          </div>
        </div>
      </div>

      <!-- 连接配置 -->
      <div class="bg-white border border-[#e8e2f4] rounded-[12px] p-5 flex flex-col gap-4">
        <div class="flex items-center gap-[7px] text-[13px] font-semibold text-secondary">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10" />
            <line x1="2" y1="12" x2="22" y2="12" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
          </svg>
          连接配置
        </div>
        <div class="flex flex-col gap-2">
          <label class="flex flex-col gap-0.5 text-[13px] font-semibold text-[#1f1f2e]">
            Base URL
            <span class="text-[11px] font-normal text-[#9b8ec4]">OpenClaw HTTP 服务地址（留空使用默认）</span>
          </label>
          <div class="relative">
            <input
              v-model="baseUrlInput"
              type="text"
              class="w-full px-3.5 py-2.5 text-[13px] font-[inherit] border-[1.5px] border-[#e8e2f4] rounded-[8px] outline-none box-border text-[#1f1f2e] bg-[#fafafa] transition placeholder-[#c4bdd8] placeholder:text-[12px] focus:border-[#7c5cfc] focus:bg-white focus:shadow-[0_0_0_3px_rgba(95,71,206,0.08)]"
              placeholder="http://127.0.0.1:18789"
            />
          </div>
        </div>
      </div>

      <!-- 保存按钮 -->
      <div class="flex flex-col items-start gap-2">
        <button
          type="button"
          class="flex items-center gap-[7px] px-[22px] py-2.5 text-[14px] font-medium text-white border-none rounded-[10px] cursor-pointer transition"
          :class="saved
            ? 'bg-[linear-gradient(135deg,#22c55e_0%,#16a34a_100%)] shadow-[0_2px_10px_rgba(34,197,94,0.25)]'
            : 'bg-[linear-gradient(135deg,#7c5cfc_0%,#5f47ce_100%)] shadow-[0_2px_10px_rgba(95,71,206,0.25)] hover:shadow-[0_4px_16px_rgba(95,71,206,0.35)] hover:-translate-y-px active:translate-y-0'"
          @click="save"
        >
          <svg v-if="!saved" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
            <polyline points="17 21 17 13 7 13 7 21" />
            <polyline points="7 3 7 8 15 8" />
          </svg>
          <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12" />
          </svg>
          {{ saved ? '已保存' : '保存设置' }}
        </button>
        <p class="text-[11px] text-[#b8b0cc] m-0">设置保存在本地，不会上传到任何服务器</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 滚动条样式无法原子化 */
.sp-body::-webkit-scrollbar {
  width: 4px;
}
.sp-body::-webkit-scrollbar-thumb {
  background: rgba(95, 71, 206, 0.15);
  border-radius: 2px;
}

/* 运行状态圆点 pulse 动画 */
@keyframes sp-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>

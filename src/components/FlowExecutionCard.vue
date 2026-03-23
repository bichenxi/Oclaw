<script setup lang="ts">
import { useOpenclawStore } from '@/stores/openclaw'
import type { FlowNodeState } from '@/stores/openclaw'

const props = defineProps<{ executionId: string }>()
const ocStore = useOpenclawStore()

const exec = computed(() => ocStore.flowExecutions[props.executionId])

function nodeById(id: string): FlowNodeState | undefined {
  return exec.value?.nodes.find(n => n.id === id)
}

const overallIcon = computed(() => {
  const s = exec.value?.status
  if (s === 'completed') return 'ok'
  if (s === 'failed') return 'err'
  return 'running'
})

const elapsed = ref(0)
let timer: ReturnType<typeof setInterval> | null = null
onMounted(() => { timer = setInterval(() => elapsed.value++, 1000) })
onUnmounted(() => { if (timer) clearInterval(timer) })
watch(() => exec.value?.status, s => { if (s !== 'running' && timer) { clearInterval(timer); timer = null } })

const hasBranches = computed(() => (exec.value?.branches.length ?? 0) > 1)
</script>

<template>
  <div v-if="exec" class="w-full max-w-[540px] rounded-[14px] border border-[#e8e2f4] bg-white shadow-[0_2px_12px_rgba(95,71,206,0.07)] overflow-hidden">

    <!-- 卡片头 -->
    <div class="flex items-start gap-3 px-4 py-3 border-b border-[#f0ecfa] bg-[#faf9ff]">
      <div class="w-8 h-8 rounded-[9px] bg-[linear-gradient(135deg,#7c5cfc,#5f47ce)] flex-center shrink-0 mt-0.5">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="5" cy="12" r="2"/><circle cx="19" cy="5" r="2"/><circle cx="19" cy="19" r="2"/>
          <line x1="7" y1="12" x2="17" y2="6"/><line x1="7" y1="12" x2="17" y2="18"/>
        </svg>
      </div>
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <span class="text-[13px] font-semibold text-[#1f1f2e] truncate">{{ exec.flowName }}</span>
          <span
            class="shrink-0 flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold"
            :class="{
              'bg-blue-50 text-blue-600 border border-blue-200': overallIcon === 'running',
              'bg-emerald-50 text-emerald-600 border border-emerald-200': overallIcon === 'ok',
              'bg-red-50 text-red-500 border border-red-200': overallIcon === 'err',
            }"
          >
            <svg v-if="overallIcon === 'running'" class="animate-spin" width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M21 12a9 9 0 1 1-6.2-8.6"/></svg>
            <svg v-else-if="overallIcon === 'ok'" width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
            <svg v-else width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            {{ overallIcon === 'running' ? `执行中 ${elapsed}s` : overallIcon === 'ok' ? '已完成' : '执行失败' }}
          </span>
        </div>
        <div class="text-[11px] text-[#9b8ec4] mt-0.5 truncate">{{ exec.task }}</div>
      </div>
    </div>

    <div class="px-4 py-3 flex flex-col gap-2">

      <!-- ── 并行分支区（列排列） ── -->
      <div
        v-if="hasBranches"
        class="grid gap-3 items-start"
        :style="`grid-template-columns: repeat(${exec.branches.length}, 1fr)`"
      >
        <div v-for="(branch, bi) in exec.branches" :key="bi" class="flex flex-col gap-1">
          <template v-for="(nodeId, ni) in branch" :key="nodeId">
            <!-- 节点卡 -->
            <div
              v-if="nodeById(nodeId)"
              class="flex flex-col gap-1 p-2.5 rounded-[10px] border transition-all duration-300"
              :class="{
                'bg-[#f8f8fb] border-[#ede8f8]': nodeById(nodeId)!.status === 'pending',
                'bg-blue-50 border-blue-200': nodeById(nodeId)!.status === 'running',
                'bg-emerald-50 border-emerald-200': nodeById(nodeId)!.status === 'completed',
                'bg-red-50 border-red-200': nodeById(nodeId)!.status === 'failed',
              }"
            >
              <div class="flex items-center gap-1.5">
                <span class="shrink-0 w-4 h-4 flex-center">
                  <svg v-if="nodeById(nodeId)!.status === 'running'" class="animate-spin" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="3"><path d="M21 12a9 9 0 1 1-6.2-8.6"/></svg>
                  <svg v-else-if="nodeById(nodeId)!.status === 'completed'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                  <svg v-else-if="nodeById(nodeId)!.status === 'failed'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#ef4444" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  <svg v-else width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#c4bdd8" stroke-width="2"><circle cx="12" cy="12" r="9"/></svg>
                </span>
                <span
                  class="text-[12px] font-semibold truncate"
                  :class="{
                    'text-[#b8b0cc]': nodeById(nodeId)!.status === 'pending',
                    'text-blue-700': nodeById(nodeId)!.status === 'running',
                    'text-emerald-700': nodeById(nodeId)!.status === 'completed',
                    'text-red-600': nodeById(nodeId)!.status === 'failed',
                  }"
                >{{ nodeById(nodeId)!.label }}</span>
              </div>
              <div class="text-[11px] leading-[1.5] pl-[22px]">
                <span v-if="nodeById(nodeId)!.status === 'pending'" class="text-[#c4bdd8]">等待中</span>
                <span v-else-if="nodeById(nodeId)!.status === 'running'" class="text-blue-400 flex items-center gap-1">
                  <span class="inline-flex gap-0.5">
                    <span class="w-[4px] h-[4px] rounded-full bg-blue-400" style="animation:td 1.2s ease-in-out infinite"/>
                    <span class="w-[4px] h-[4px] rounded-full bg-blue-400" style="animation:td 1.2s ease-in-out infinite 0.2s"/>
                    <span class="w-[4px] h-[4px] rounded-full bg-blue-400" style="animation:td 1.2s ease-in-out infinite 0.4s"/>
                  </span>
                </span>
                <span v-else-if="nodeById(nodeId)!.status === 'failed'" class="text-red-500 line-clamp-2 break-words">{{ nodeById(nodeId)!.error }}</span>
                <span v-else class="text-[#4b4568] line-clamp-2 break-words">{{ nodeById(nodeId)!.output }}</span>
              </div>
            </div>
            <!-- 分支内节点间连接箭头 -->
            <div v-if="ni < branch.length - 1" class="flex justify-center py-0.5">
              <div class="flex flex-col items-center">
                <div class="w-px h-2.5 bg-[#ddd8f0]"/>
                <svg width="7" height="5" viewBox="0 0 7 5" fill="#c4bdd8"><path d="M3.5 5L0 0h7z"/></svg>
              </div>
            </div>
          </template>
        </div>
      </div>

      <!-- ── 单分支（顺序流） ── -->
      <template v-else-if="(exec.branches.length === 1)">
        <template v-for="(nodeId, ni) in exec.branches[0]" :key="nodeId">
          <div v-if="nodeById(nodeId)" class="flex flex-col gap-1 p-2.5 rounded-[10px] border transition-all duration-300"
            :class="{
              'bg-[#f8f8fb] border-[#ede8f8]': nodeById(nodeId)!.status === 'pending',
              'bg-blue-50 border-blue-200': nodeById(nodeId)!.status === 'running',
              'bg-emerald-50 border-emerald-200': nodeById(nodeId)!.status === 'completed',
              'bg-red-50 border-red-200': nodeById(nodeId)!.status === 'failed',
            }"
          >
            <div class="flex items-center gap-1.5">
              <span class="shrink-0 w-4 h-4 flex-center">
                <svg v-if="nodeById(nodeId)!.status === 'running'" class="animate-spin" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="3"><path d="M21 12a9 9 0 1 1-6.2-8.6"/></svg>
                <svg v-else-if="nodeById(nodeId)!.status === 'completed'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                <svg v-else-if="nodeById(nodeId)!.status === 'failed'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#ef4444" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                <svg v-else width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#c4bdd8" stroke-width="2"><circle cx="12" cy="12" r="9"/></svg>
              </span>
              <span class="text-[12px] font-semibold truncate"
                :class="{
                  'text-[#b8b0cc]': nodeById(nodeId)!.status === 'pending',
                  'text-blue-700': nodeById(nodeId)!.status === 'running',
                  'text-emerald-700': nodeById(nodeId)!.status === 'completed',
                  'text-red-600': nodeById(nodeId)!.status === 'failed',
                }"
              >{{ nodeById(nodeId)!.label }}</span>
            </div>
            <div class="text-[11px] leading-[1.5] pl-[22px]">
              <span v-if="nodeById(nodeId)!.status === 'pending'" class="text-[#c4bdd8]">等待中</span>
              <span v-else-if="nodeById(nodeId)!.status === 'running'" class="text-blue-400 flex items-center gap-1">
                <span class="inline-flex gap-0.5">
                  <span class="w-[4px] h-[4px] rounded-full bg-blue-400" style="animation:td 1.2s ease-in-out infinite"/>
                  <span class="w-[4px] h-[4px] rounded-full bg-blue-400" style="animation:td 1.2s ease-in-out infinite 0.2s"/>
                  <span class="w-[4px] h-[4px] rounded-full bg-blue-400" style="animation:td 1.2s ease-in-out infinite 0.4s"/>
                </span>
              </span>
              <span v-else-if="nodeById(nodeId)!.status === 'failed'" class="text-red-500 line-clamp-2 break-words">{{ nodeById(nodeId)!.error }}</span>
              <span v-else class="text-[#4b4568] line-clamp-2 break-words">{{ nodeById(nodeId)!.output }}</span>
            </div>
          </div>
          <div v-if="ni < exec.branches[0].length - 1" class="flex justify-center py-0.5">
            <div class="flex flex-col items-center">
              <div class="w-px h-3 bg-[#ddd8f0]"/>
              <svg width="7" height="5" viewBox="0 0 7 5" fill="#c4bdd8"><path d="M3.5 5L0 0h7z"/></svg>
            </div>
          </div>
        </template>
      </template>

      <!-- ── 汇聚分隔线（有分支且有汇聚节点时显示） ── -->
      <div v-if="hasBranches && exec.convergeIds.length > 0" class="flex items-center gap-2 pt-1">
        <div class="flex-1 h-px bg-[#e8e2f4]"/>
        <div class="flex items-center gap-1 text-[10px] font-medium text-[#9b8ec4]">
          <svg width="10" height="7" viewBox="0 0 10 7" fill="#c4bdd8"><path d="M5 7L0 0h10z"/></svg>
          汇聚
        </div>
        <div class="flex-1 h-px bg-[#e8e2f4]"/>
      </div>

      <!-- ── 汇聚区节点 ── -->
      <template v-for="(nodeId, ci) in exec.convergeIds" :key="nodeId">
        <div v-if="nodeById(nodeId)" class="flex flex-col gap-1 p-2.5 rounded-[10px] border transition-all duration-300"
          :class="{
            'bg-[#f8f8fb] border-[#ede8f8]': nodeById(nodeId)!.status === 'pending',
            'bg-blue-50 border-blue-200': nodeById(nodeId)!.status === 'running',
            'bg-emerald-50 border-emerald-200': nodeById(nodeId)!.status === 'completed',
            'bg-red-50 border-red-200': nodeById(nodeId)!.status === 'failed',
          }"
        >
          <div class="flex items-center gap-1.5">
            <span class="shrink-0 w-4 h-4 flex-center">
              <svg v-if="nodeById(nodeId)!.status === 'running'" class="animate-spin" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="3"><path d="M21 12a9 9 0 1 1-6.2-8.6"/></svg>
              <svg v-else-if="nodeById(nodeId)!.status === 'completed'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
              <svg v-else-if="nodeById(nodeId)!.status === 'failed'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#ef4444" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              <svg v-else width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#c4bdd8" stroke-width="2"><circle cx="12" cy="12" r="9"/></svg>
            </span>
            <span class="text-[12px] font-semibold truncate"
              :class="{
                'text-[#b8b0cc]': nodeById(nodeId)!.status === 'pending',
                'text-blue-700': nodeById(nodeId)!.status === 'running',
                'text-emerald-700': nodeById(nodeId)!.status === 'completed',
                'text-red-600': nodeById(nodeId)!.status === 'failed',
              }"
            >{{ nodeById(nodeId)!.label }}</span>
          </div>
          <div class="text-[11px] leading-[1.5] pl-[22px]">
            <span v-if="nodeById(nodeId)!.status === 'pending'" class="text-[#c4bdd8]">等待中</span>
            <span v-else-if="nodeById(nodeId)!.status === 'running'" class="text-blue-400 flex items-center gap-1">
              <span class="inline-flex gap-0.5">
                <span class="w-[4px] h-[4px] rounded-full bg-blue-400" style="animation:td 1.2s ease-in-out infinite"/>
                <span class="w-[4px] h-[4px] rounded-full bg-blue-400" style="animation:td 1.2s ease-in-out infinite 0.2s"/>
                <span class="w-[4px] h-[4px] rounded-full bg-blue-400" style="animation:td 1.2s ease-in-out infinite 0.4s"/>
              </span>
            </span>
            <span v-else-if="nodeById(nodeId)!.status === 'failed'" class="text-red-500 line-clamp-2 break-words">{{ nodeById(nodeId)!.error }}</span>
            <span v-else class="text-[#4b4568] line-clamp-2 break-words">{{ nodeById(nodeId)!.output }}</span>
          </div>
        </div>
        <div v-if="ci < exec.convergeIds.length - 1" class="flex justify-center py-0.5">
          <div class="flex flex-col items-center">
            <div class="w-px h-3 bg-[#ddd8f0]"/>
            <svg width="7" height="5" viewBox="0 0 7 5" fill="#c4bdd8"><path d="M3.5 5L0 0h7z"/></svg>
          </div>
        </div>
      </template>

    </div>
  </div>
</template>

<style scoped>
@keyframes td {
  0%, 80%, 100% { opacity: 0.3; transform: scale(0.8); }
  40% { opacity: 1; transform: scale(1); }
}
</style>

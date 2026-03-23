<script setup lang="ts">
import { useOpenclawStore } from '@/stores/openclaw'
import type { FlowNodeState } from '@/stores/openclaw'

const props = defineProps<{ executionId: string }>()
const ocStore = useOpenclawStore()
const exec = computed(() => ocStore.flowExecutions[props.executionId])
const hasBranches = computed(() => (exec.value?.branches?.length ?? 0) > 1)
const branchCount = computed(() => exec.value?.branches.length ?? 0)

const statusStats = computed(() => {
  const nodes = exec.value?.nodes ?? []
  const completed = nodes.filter(n => n.status === 'completed').length
  const runningCount = nodes.filter(n => n.status === 'running').length
  const failed = nodes.filter(n => n.status === 'failed').length
  const total = nodes.length
  const pending = Math.max(total - completed - runningCount - failed, 0)
  const percent = total > 0 ? Math.round((completed / total) * 100) : 0
  return { total, completed, running: runningCount, failed, pending, percent }
})

const formattedElapsed = computed(() => {
  const sec = elapsed.value
  const minutes = Math.floor(sec / 60)
  const seconds = sec % 60
  if (minutes > 0) return `${minutes}m ${seconds}s`
  return `${seconds}s`
})

const elapsed = ref(0)
let timer: ReturnType<typeof setInterval> | null = null
onMounted(() => { timer = setInterval(() => elapsed.value++, 1000) })
onUnmounted(() => { if (timer) clearInterval(timer) })
watch(() => exec.value?.status, s => { if (s !== 'running' && timer) { clearInterval(timer); timer = null } })

const modalNode = ref<FlowNodeState | null>(null)
function openModal(nodeId: string) {
  const n = nodeById(nodeId)
  if (n) modalNode.value = n
}

// 弹窗内容跟随节点实时更新
watch(() => exec.value?.nodes, () => {
  if (modalNode.value) {
    modalNode.value = nodeById(modalNode.value.id) ?? null
  }
}, { deep: true })

function nodeById(id: string): FlowNodeState | undefined {
  return exec.value?.nodes.find(n => n.id === id)
}

function statusBadgeClass(status: FlowNodeState['status']) {
  if (status === 'running') return 'text-cyan-700 bg-cyan-100 border border-cyan-200'
  if (status === 'completed') return 'text-emerald-700 bg-emerald-100 border border-emerald-200'
  if (status === 'failed') return 'text-red-600 bg-red-50 border border-red-200'
  return 'text-[#6b5f8a] bg-[#f3efff] border border-[#e4dcf7]'
}
function statusLabel(status: FlowNodeState['status']) {
  if (status === 'running') return '运行中'
  if (status === 'completed') return '完成'
  if (status === 'failed') return '失败'
  return '待执行'
}
function nodeSnippet(nodeId: string) {
  const node = nodeById(nodeId)
  if (!node) return ''
  if (node.status === 'failed') return node.error ?? '发生错误'
  if (node.output) {
    return node.output.length > 140 ? `${node.output.slice(0, 140)}…` : node.output
  }
  if (node.status === 'running') return '正在生成输出...'
  return '等待上游结果'
}

</script>

<template>
  <div v-if="exec" class="w-full max-w-[960px] rounded-[24px] overflow-hidden border border-[#dfd3ff] bg-white shadow-[0_35px_90px_rgba(20,5,60,0.18)]">

    <!-- Main grid -->
    <div class="px-8 py-6 flex flex-col gap-6">

      <!-- Flow lanes -->
      <section class="flex flex-col gap-6">
        <div class="flex flex-col gap-1">
          <p class="text-[11px] uppercase tracking-[0.5em] text-[#a391d9]">flow map</p>
          <h3 class="text-[18px] font-semibold text-[#1b1531]">执行路径</h3>
          <p class="text-[13px] text-[#6b5f8a] leading-relaxed">
            {{ exec.task || '当前执行未附带描述' }}
          </p>
          <div class="text-[11px] text-[#988fbd] flex items-center gap-3">
            <span>总计 {{ statusStats.total }} 步</span>
            <span>并行 {{ hasBranches ? branchCount : 0 }}</span>
            <span>耗时 {{ formattedElapsed }}</span>
          </div>
        </div>

        <div v-if="exec.branches.length > 0" class="flex gap-3 items-start">
          <div
            v-for="(branch, bi) in exec.branches"
            :key="bi"
            class="flex-1 min-w-0 rounded-[22px] border border-[#e6defa] bg-[#fcfbff] px-5 py-4 flex flex-col gap-3"
          >
            <div class="flex items-center justify-between text-[#5f47ce] text-[12px] font-semibold">
              <span>分支 {{ bi + 1 }}</span>
              <span class="text-[11px] font-mono text-[#9b8ec4]">{{ branch.length }} 步</span>
            </div>
            <template v-for="(nodeId, idx) in branch" :key="nodeId">
              <div
                v-if="nodeById(nodeId)"
                class="flex flex-col gap-2 rounded-[18px] border bg-white px-4 py-3 cursor-pointer transition-all hover:border-[#c4b0ff] hover:shadow-[0_2px_8px_rgba(95,71,206,0.08)]"
                :class="nodeById(nodeId)!.status === 'failed' ? 'ring-1 ring-red-200 border-red-200' : 'border-[#e6defa]'"
                @click="openModal(nodeId)"
              >
                <div class="flex items-center justify-between gap-2">
                  <span class="text-[11px] font-mono uppercase tracking-[0.25em] text-[#9b8ec4]">第 {{ idx + 1 }} 步</span>
                  <span class="px-2 py-0.5 text-[10px] font-mono rounded-full" :class="statusBadgeClass(nodeById(nodeId)!.status)">
                    {{ statusLabel(nodeById(nodeId)!.status) }}
                  </span>
                </div>
                <span class="text-[13px] font-semibold text-[#1f1b2e] truncate">{{ nodeById(nodeId)!.label }}</span>
                <p v-if="nodeById(nodeId)?.flow_role" class="text-[10px] text-[#8b7fd4] leading-snug line-clamp-2 m-0">
                  {{ nodeById(nodeId)!.flow_role }}
                </p>
                <p class="text-[11px] text-[#6b5f8a] leading-relaxed min-h-[34px]">
                  {{ nodeSnippet(nodeId) }}
                  <span v-if="nodeById(nodeId)!.status === 'running'" class="cursor-blink text-[#5f47ce]">█</span>
                </p>
              </div>
            </template>
          </div>
        </div>
        <div v-else class="rounded-[18px] border border-dashed border-[#dfd3ff] bg-[#fbf7ff] px-5 py-10 text-center text-[13px] text-[#9b8ec4]">
          还没有 agent 节点，先在右侧添加智能体吧。
        </div>

        <div
          v-if="exec.convergeIds.length"
          class="rounded-[22px] border border-[#f2d9b5] bg-[#fffaf2] px-5 py-4 flex flex-col gap-4"
        >
          <div class="flex items-center justify-between text-[#b0620b] text-[12px] font-semibold">
            <span>合流区</span>
            <span class="text-[11px] font-mono text-[#d19a4b]">{{ exec.convergeIds.length }} 节点</span>
          </div>
          <div class="grid grid-cols-2 gap-3">
            <template v-for="(nodeId, idx) in exec.convergeIds" :key="nodeId">
              <div
                v-if="nodeById(nodeId)"
                class="flex flex-col gap-2 rounded-[18px] border bg-white px-4 py-3 cursor-pointer transition-all hover:border-[#f0b768]/60 hover:shadow-[0_2px_8px_rgba(222,146,28,0.08)]"
                :class="nodeById(nodeId)!.status === 'failed' ? 'ring-1 ring-red-200 border-red-200' : 'border-[#f7dcb5]'"
                @click="openModal(nodeId)"
              >
                <div class="flex items-center justify-between gap-2">
                  <span class="text-[11px] font-mono uppercase tracking-[0.25em] text-[#d19a4b]">第 {{ idx + 1 }} 步</span>
                  <span class="px-2 py-0.5 text-[10px] font-mono rounded-full" :class="statusBadgeClass(nodeById(nodeId)!.status)">
                    {{ statusLabel(nodeById(nodeId)!.status) }}
                  </span>
                </div>
                <span class="text-[13px] font-semibold text-[#3c2f13] truncate">{{ nodeById(nodeId)!.label }}</span>
                <p v-if="nodeById(nodeId)?.flow_role" class="text-[10px] text-[#b08a4a] leading-snug line-clamp-2 m-0">
                  {{ nodeById(nodeId)!.flow_role }}
                </p>
                <p class="text-[11px] text-[#7b6841] leading-relaxed min-h-[34px]">
                  {{ nodeSnippet(nodeId) }}
                  <span v-if="nodeById(nodeId)!.status === 'running'" class="cursor-blink text-[#b0620b]">█</span>
                </p>
              </div>
            </template>
          </div>
        </div>
      </section>

    </div>

    <!-- 节点详情 -->
    <Teleport to="body">
      <div
        v-if="modalNode"
        class="fixed inset-0 z-[9999] flex items-center justify-center"
        @click.self="modalNode = null"
      >
        <div class="absolute inset-0 bg-black/60 backdrop-blur-[2px]" @click="modalNode = null" />
        <div class="relative w-[640px] max-w-[90vw] max-h-[80vh] flex flex-col rounded-[16px] overflow-hidden border border-[#2a1f4e] shadow-[0_40px_100px_rgba(0,0,0,0.6)]">
          <!-- title bar -->
          <div class="shrink-0 flex items-center justify-between px-5 py-3 bg-[#1a1330] border-b border-[#2a1f4e]">
            <div class="flex items-center gap-3">
              <div class="flex gap-1.5">
                <span class="w-3 h-3 rounded-full bg-[#ff5f57] cursor-pointer" @click="modalNode = null" />
                <span class="w-3 h-3 rounded-full bg-[#febc2e]" />
                <span class="w-3 h-3 rounded-full bg-[#28c840]" />
              </div>
              <span class="font-mono text-[12px] text-[#7c6aa8]">~/node/{{ modalNode.label }}</span>
            </div>
            <span class="px-2 py-0.5 text-[10px] font-mono rounded-full" :class="statusBadgeClass(modalNode.status)">
              {{ statusLabel(modalNode.status) }}
            </span>
          </div>
          <!-- output -->
          <div class="flex-1 overflow-y-auto bg-[#0d0b18] px-6 py-5">
            <template v-if="modalNode.output">
              <pre class="font-mono text-[12px] text-[#c4b0ff] whitespace-pre-wrap leading-[1.8] m-0 break-words">{{ modalNode.output }}<span v-if="modalNode.status === 'running'" class="cursor-blink text-cyan-400">█</span></pre>
            </template>
            <template v-else-if="modalNode.status === 'failed'">
              <pre class="font-mono text-[12px] text-red-400 whitespace-pre-wrap leading-[1.8] m-0 break-words">{{ modalNode.error ?? 'unknown error' }}</pre>
            </template>
            <template v-else>
              <span class="font-mono text-[12px] text-[#4a3e7a]">等待输出...<span class="cursor-blink text-cyan-400">█</span></span>
            </template>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.cursor-blink {
  animation: blink 1s step-end infinite;
}
@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}
</style>

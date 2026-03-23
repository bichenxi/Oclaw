import { defineStore } from 'pinia'
import { listen } from '@tauri-apps/api/event'

export type MessageType = 'thought' | 'tool' | 'user' | 'flow'

export interface Message {
  type: MessageType
  text: string
  streaming: boolean
  executionId?: string
}

export interface FlowNodeState {
  id: string
  label: string
  status: 'pending' | 'running' | 'completed' | 'failed'
  output: string
  error?: string
}

export interface FlowExecutionState {
  id: string
  flowName: string
  task: string
  status: 'running' | 'completed' | 'failed'
  nodes: FlowNodeState[]
  /** 各层节点 id，用于卡片布局 */
  levelIds: string[][]
  /** nodeId → 前驱 agent 节点的 label 列表（已过滤 start/end） */
  predecessors: Record<string, string[]>
}

export const useOpenclawStore = defineStore('openclaw', () => {
  const messages = ref<Message[]>([])
  const sending = ref(false)
  const sendError = ref('')

  /** 为 true 时，stream-item / stream-done 事件不写入 messages（flow 执行期间使用） */
  const suppressStream = ref(false)

  /** 所有工作流执行状态，reactive 以驱动卡片更新 */
  const flowExecutions = ref<Record<string, FlowExecutionState>>({})

  let listenersStarted = false

  function startListeners() {
    if (listenersStarted) return
    listenersStarted = true

    listen<{ type: string; text: string }>('stream-item', (e) => {
      if (suppressStream.value) return
      const payload = e.payload
      if (!payload?.type || !payload?.text) return
      const type: MessageType = payload.type === 'tool' ? 'tool' : 'thought'
      const last = messages.value[messages.value.length - 1]
      if (last && last.streaming && last.type === type) {
        last.text += payload.text
      } else {
        messages.value.push({ type, text: payload.text, streaming: true })
      }
    })

    listen('stream-done', () => {
      if (suppressStream.value) return
      const last = messages.value[messages.value.length - 1]
      if (last && last.streaming) last.streaming = false
      sending.value = false
    })
  }

  function createFlowExecution(
    flowName: string,
    task: string,
    levels: { id: string; label: string }[][],
    edges: { source: string; target: string }[],
  ): string {
    const id = `fexec-${Date.now()}`
    const allNodes = levels.flat()
    const agentIds = new Set(allNodes.map(n => n.id))
    const labelMap = new Map(allNodes.map(n => [n.id, n.label]))

    // 计算每个 agent 节点的前驱 agent label（过滤掉 start/end）
    const predecessors: Record<string, string[]> = {}
    for (const e of edges) {
      if (!agentIds.has(e.target)) continue
      if (!agentIds.has(e.source)) continue  // source 是 start 节点，跳过
      if (!predecessors[e.target]) predecessors[e.target] = []
      const lbl = labelMap.get(e.source)
      if (lbl) predecessors[e.target].push(lbl)
    }

    flowExecutions.value[id] = {
      id,
      flowName,
      task,
      status: 'running',
      nodes: allNodes.map(n => ({ id: n.id, label: n.label, status: 'pending', output: '' })),
      levelIds: levels.map(l => l.map(n => n.id)),
      predecessors,
    }
    return id
  }

  function updateFlowNode(execId: string, nodeId: string, patch: Partial<FlowNodeState>) {
    const exec = flowExecutions.value[execId]
    if (!exec) return
    const node = exec.nodes.find(n => n.id === nodeId)
    if (node) Object.assign(node, patch)
  }

  function finishFlowExecution(execId: string, status: 'completed' | 'failed') {
    const exec = flowExecutions.value[execId]
    if (exec) exec.status = status
  }

  return {
    messages, sending, sendError, suppressStream,
    flowExecutions,
    startListeners,
    createFlowExecution, updateFlowNode, finishFlowExecution,
  }
})

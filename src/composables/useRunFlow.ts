import { useOpenclawStore } from '@/stores/openclaw'
import { useSettingsStore } from '@/stores/settings'
import { runFlowNode } from '@/api/flows'
import type { AgentFlow, FlowNode } from '@/api/flows'
import { buildFlowNodePrompt } from '@/utils/flowNodePrompt'

function getExecutionLevels(flow: AgentFlow): FlowNode[][] {
  const nodeMap = new Map(flow.nodes.map(n => [n.id, n]))
  const inDegree = new Map<string, number>()
  const childrenOf = new Map<string, string[]>()
  for (const n of flow.nodes) { inDegree.set(n.id, 0); childrenOf.set(n.id, []) }
  for (const e of flow.edges) {
    inDegree.set(e.target, (inDegree.get(e.target) ?? 0) + 1)
    childrenOf.get(e.source)?.push(e.target)
  }

  const levels: FlowNode[][] = []
  const visited = new Set<string>()
  let frontier = flow.nodes.filter(n => (inDegree.get(n.id) ?? 0) === 0)

  while (frontier.length > 0) {
    const agentLevel = frontier.filter(n => n.type === 'agent')
    if (agentLevel.length > 0) levels.push(agentLevel)
    const next: FlowNode[] = []
    for (const n of frontier) {
      visited.add(n.id)
      for (const cid of childrenOf.get(n.id) ?? []) {
        const deg = (inDegree.get(cid) ?? 0) - 1
        inDegree.set(cid, deg)
        if (deg === 0 && !visited.has(cid)) {
          const child = nodeMap.get(cid)
          if (child) next.push(child)
        }
      }
    }
    frontier = next
  }
  return levels
}

function computeBranchLayout(
  flow: AgentFlow,
  levels: FlowNode[][],
): { branches: string[][]; convergeIds: string[] } {
  const agentNodes = flow.nodes.filter(n => n.type === 'agent')
  const agentIds = new Set(agentNodes.map(n => n.id))

  const agentInDeg = new Map<string, number>()
  const agentSucc = new Map<string, string[]>()
  for (const n of agentNodes) { agentInDeg.set(n.id, 0); agentSucc.set(n.id, []) }
  for (const e of flow.edges) {
    if (agentIds.has(e.target) && agentIds.has(e.source)) {
      agentInDeg.set(e.target, (agentInDeg.get(e.target) ?? 0) + 1)
      agentSucc.get(e.source)?.push(e.target)
    }
  }

  const isConverge = (id: string) => (agentInDeg.get(id) ?? 0) > 1
  const roots = agentNodes.filter(n => agentInDeg.get(n.id) === 0)
  const branches: string[][] = roots.map(root => {
    const path = [root.id]
    let cur = root.id
    while (true) {
      const nexts = (agentSucc.get(cur) ?? []).filter(id => !isConverge(id))
      if (nexts.length !== 1) break
      path.push(nexts[0])
      cur = nexts[0]
    }
    return path
  })

  const branchSet = new Set(branches.flat())
  const convergeIds = levels.flat()
    .map(n => n.id)
    .filter(id => !branchSet.has(id))
    .filter((id, i, arr) => arr.indexOf(id) === i)

  return { branches, convergeIds }
}

export function useRunFlow() {
  const ocStore = useOpenclawStore()
  const settings = useSettingsStore()

  async function runFlow(flow: AgentFlow, initialTask: string): Promise<void> {
    const levels = getExecutionLevels(flow)
    const token = settings.bearerToken

    if (!token) throw new Error('未配置 Bearer Token')
    if (levels.length === 0) throw new Error('工作流没有 Agent 节点')

    const { branches, convergeIds } = computeBranchLayout(flow, levels)
    const execId = ocStore.createFlowExecution(
      flow.name,
      initialTask,
      levels.flat().map(n => ({ id: n.id, label: n.label, flow_role: n.flow_role })),
      branches,
      convergeIds,
    )
    ocStore.messages.push({ type: 'flow', text: '', streaming: false, executionId: execId })

    const nodeOutputs = new Map<string, string>()
    const baseUrlOpt = settings.baseUrl ? { baseUrl: settings.baseUrl } : {}
    let finalOutput = ''
    let allSucceeded = true

    for (const levelNodes of levels) {
      if (levelNodes.length === 1) {
        const node = levelNodes[0]
        const input = buildFlowNodePrompt(node, flow, nodeOutputs, initialTask)
        ocStore.updateFlowNode(execId, node.id, { status: 'running' })
        try {
          const output = await runFlowNode({
            token,
            nodeId: node.id,
            sessionKey: `agent:${node.agent_work}:${node.agent_work}`,
            input,
            ...baseUrlOpt,
          })
          nodeOutputs.set(node.id, output)
          finalOutput = output
          ocStore.updateFlowNode(execId, node.id, { status: 'completed', output })
        } catch (err) {
          const errMsg = err instanceof Error ? err.message : String(err)
          ocStore.updateFlowNode(execId, node.id, { status: 'failed', error: errMsg })
          allSucceeded = false
          break
        }
      } else {
        levelNodes.forEach(n => ocStore.updateFlowNode(execId, n.id, { status: 'running' }))
        const results = await Promise.allSettled(
          levelNodes.map(node => runFlowNode({
            token,
            nodeId: node.id,
            sessionKey: `agent:${node.agent_work}:${node.agent_work}`,
            input: buildFlowNodePrompt(node, flow, nodeOutputs, initialTask),
            ...baseUrlOpt,
          }))
        )
        let anyFailed = false
        const parallelOutputs: string[] = []
        for (let i = 0; i < results.length; i++) {
          const res = results[i]
          const node = levelNodes[i]
          if (res.status === 'fulfilled') {
            nodeOutputs.set(node.id, res.value)
            parallelOutputs.push(res.value)
            ocStore.updateFlowNode(execId, node.id, { status: 'completed', output: res.value })
          } else {
            const errMsg = res.reason instanceof Error ? res.reason.message : String(res.reason)
            ocStore.updateFlowNode(execId, node.id, { status: 'failed', error: errMsg })
            anyFailed = true
            allSucceeded = false
          }
        }
        if (parallelOutputs.length > 0) finalOutput = parallelOutputs[parallelOutputs.length - 1]
        if (anyFailed) break
      }
    }

    ocStore.finishFlowExecution(execId, allSucceeded ? 'completed' : 'failed')
    if (allSucceeded && finalOutput) {
      ocStore.messages.push({ type: 'thought', text: finalOutput, streaming: false })
    }
  }

  return { runFlow }
}

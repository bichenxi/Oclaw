import type { AgentFlow, FlowNode } from '@/api/flows'

/**
 * 为工作流中的单个 Agent 节点拼装发给模型的提示词。
 * 若节点配置了 flow_role，会作为「流程内职责」注入，便于协作时各 Agent 明确本步交付物。
 */
export function buildFlowNodePrompt(
  node: FlowNode,
  flow: AgentFlow,
  outputs: Map<string, string>,
  initialTask: string,
): string {
  const sourceOutputs = flow.edges
    .filter(e => e.target === node.id)
    .map(e => {
      const src = flow.nodes.find(n => n.id === e.source)
      const out = outputs.get(e.source)
      return out ? { label: src?.label ?? e.source, text: out } : null
    })
    .filter(Boolean) as { label: string; text: string }[]

  const parts: string[] = []
  parts.push(`【总体任务】\n${initialTask}`)

  const duty = node.flow_role?.trim()
  const agentName = node.agent_work ?? node.label
  if (duty) {
    parts.push(
      `【本节点在流程中的职责】\n${duty}\n\n`
      + `你是流程中的节点「${node.label}」（使用 Agent「${agentName}」）。请严格按上述职责与【总体任务】完成本步应交付的内容；输出应便于下游节点继续处理。`,
    )
  } else {
    parts.push(
      `【你的角色】\n你是「${node.label}」（Agent：${agentName}）。请围绕【总体任务】与上游输出，完成本工作流中这一环应交付的内容。`,
    )
  }

  if (sourceOutputs.length === 1) {
    parts.push(`【上游输出（来自「${sourceOutputs[0].label}」）】\n${sourceOutputs[0].text}`)
  } else if (sourceOutputs.length > 1) {
    const combined = sourceOutputs.map(s => `— 来自「${s.label}」：\n${s.text}`).join('\n\n')
    parts.push(`【上游输出】\n${combined}`)
  }

  return parts.join('\n\n')
}

import { invoke } from '@tauri-apps/api/core'

export function greet(name: string): Promise<string> {
  return invoke<string>('greet', { name })
}

/** 模拟 OpenClaw 流式输出，用于验证思考链/工具流 UI */
export function simulateStream(): Promise<void> {
  return invoke('simulate_stream')
}

import { invoke } from '@tauri-apps/api/core'

export interface GatewayConfigStatus {
  already_ok: boolean
  fixed: boolean
  needs_restart: boolean
  error: string | null
}

/** 检查并自动修复 ~/.openclaw/openclaw.json 中 gateway 所需配置 */
export async function checkAndFixGatewayConfig(): Promise<GatewayConfigStatus> {
  return invoke<GatewayConfigStatus>('check_and_fix_gateway_config')
}

/** 执行 `openclaw gateway restart` */
export async function restartOpenclawGateway(): Promise<void> {
  await invoke('restart_openclaw_gateway')
}

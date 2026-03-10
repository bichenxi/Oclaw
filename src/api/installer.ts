import { invoke } from '@tauri-apps/api/core'

/** 开始安装（fnm install node 22 → npm install -g openclaw → openclaw onboard） */
export async function startInstall(): Promise<void> {
  await invoke('start_install')
}

/** 取消正在进行的安装 */
export async function cancelInstall(): Promise<void> {
  await invoke('cancel_install')
}

/** 检测 OpenClaw 是否已安装（~/.openclaw/openclaw.json 存在） */
export async function checkOpenclawInstalled(): Promise<boolean> {
  return invoke<boolean>('check_openclaw_installed')
}

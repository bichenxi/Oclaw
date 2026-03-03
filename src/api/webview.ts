import { invoke } from '@tauri-apps/api/core'

export async function createTabWebview(label: string, url: string): Promise<void> {
  await invoke('create_tab_webview', { label, url })
}

export async function showWebview(label: string): Promise<void> {
  await invoke('show_webview', { label })
}

export async function hideWebview(label: string): Promise<void> {
  await invoke('hide_webview', { label })
}

export async function closeWebview(label: string): Promise<void> {
  await invoke('close_webview', { label })
}

export async function resizeAllWebviews(labels: string[]): Promise<void> {
  await invoke('resize_all_webviews', { labels })
}

export async function evalInWebview(label: string, script: string): Promise<void> {
  await invoke('eval_in_webview', { label, script })
}

export async function getDomSnapshot(label: string): Promise<void> {
  await invoke('get_dom_snapshot', { label })
}

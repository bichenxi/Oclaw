import { invoke } from '@tauri-apps/api/core'

export function getCurrentProfile(): Promise<string> {
  return invoke<string>('get_current_profile')
}

export function setCurrentProfile(name: string): Promise<void> {
  return invoke('set_current_profile', { name })
}

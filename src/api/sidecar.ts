import { invoke } from '@tauri-apps/api/core'

export async function testSidecar(): Promise<string> {
  return invoke<string>('test_sidecar')
}

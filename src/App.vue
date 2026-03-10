<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import { useTabsStore } from '@/stores/tabs'
import { useInstallerStore } from '@/stores/installer'
import { checkOpenclawAlive } from '@/api/openclaw'
import { checkOpenclawInstalled } from '@/api/installer'

const store = useTabsStore()
const installerStore = useInstallerStore()

const onResize = useDebounceFn(() => {
  store.resizeAllWebviews()
}, 100)

let unlistenApiOpenTab: (() => void) | null = null

onMounted(async () => {
  window.addEventListener('resize', onResize)
  listen<{ url: string }>('api_open_tab', (e) => {
    store.openTab(e.payload.url)
  }).then((fn) => {
    unlistenApiOpenTab = fn
  }).catch(() => {})

  // 三态检测：running / installed-not-running / not-installed
  const alive = await checkOpenclawAlive().catch(() => false)
  if (!alive) {
    const installed = await checkOpenclawInstalled().catch(() => false)
    installerStore.isInstalled = installed
    store.switchToSpecialView('setup')
  }
})

onUnmounted(() => {
  window.removeEventListener('resize', onResize)
  unlistenApiOpenTab?.()
})
</script>

<template>
  <RouterView />
</template>

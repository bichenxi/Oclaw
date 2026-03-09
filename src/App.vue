<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import { useTabsStore } from '@/stores/tabs'

const store = useTabsStore()

const onResize = useDebounceFn(() => {
  store.resizeAllWebviews()
}, 100)

let unlistenApiOpenTab: (() => void) | null = null

onMounted(() => {
  window.addEventListener('resize', onResize)
  listen<{ url: string }>('api_open_tab', (e) => {
    store.openTab(e.payload.url)
  }).then((fn) => {
    unlistenApiOpenTab = fn
  }).catch(() => {})
})

onUnmounted(() => {
  window.removeEventListener('resize', onResize)
  unlistenApiOpenTab?.()
})
</script>

<template>
  <RouterView />
</template>

import { defineStore } from 'pinia'
import * as webviewApi from '@/api/webview'
import { useRecordingStore } from '@/stores/recording'

const WEBVIEW_LOADING_DELAY_MS = 1200

export interface TabItem {
  id: string
  label: string
  url: string
  title: string
}

export const useTabsStore = defineStore('tabs', () => {
  const tabs = ref<TabItem[]>([])
  const activeTabId = ref<string | null>(null)
  const loadingTabId = ref<string | null>(null)
  let tabIndex = 0
  let loadingTimer: ReturnType<typeof setTimeout> | null = null

  const isHome = computed(() => activeTabId.value === null)
  const isWebviewLoading = computed(
    () => loadingTabId.value !== null && loadingTabId.value === activeTabId.value,
  )

  // 人机混合接管：true 表示已暂停 AI，用户可手动操作右侧网页，完成后「继续 AI」
  const aiPaused = ref(false)
  function setAiPaused(value: boolean) {
    aiPaused.value = value
    // 接通 OpenClaw 后在此通知 sidecar 暂停/继续
  }

  function scheduleShowWebview(label: string) {
    if (loadingTimer) clearTimeout(loadingTimer)
    loadingTabId.value = label
    loadingTimer = setTimeout(() => {
      loadingTimer = null
      loadingTabId.value = null
      webviewApi.showWebview(label).catch(() => {})
    }, WEBVIEW_LOADING_DELAY_MS)
  }

  async function openTab(url: string) {
    tabIndex++
    const id = `tab-${Date.now()}-${tabIndex}`
    const title = url.replace(/^https?:\/\//, '').split('/')[0]

    await webviewApi.createTabWebview(id, url)
    useRecordingStore().pushStep({ type: 'navigate', url })

    tabs.value.push({ id, label: id, url, title })

    if (activeTabId.value) {
      await webviewApi.hideWebview(activeTabId.value).catch(() => {})
    }
    activeTabId.value = id
    scheduleShowWebview(id)
  }

  async function switchTab(id: string) {
    if (id === activeTabId.value) return

    if (activeTabId.value) {
      await webviewApi.hideWebview(activeTabId.value).catch(() => {})
    }

    activeTabId.value = id
    await webviewApi.showWebview(id).catch(() => {})
  }

  async function switchToHome() {
    if (activeTabId.value) {
      await webviewApi.hideWebview(activeTabId.value).catch(() => {})
    }
    activeTabId.value = null
  }

  async function closeTab(id: string) {
    const idx = tabs.value.findIndex((t) => t.id === id)
    if (idx === -1) return

    await webviewApi.closeWebview(id).catch(() => {})
    tabs.value.splice(idx, 1)

    if (activeTabId.value === id) {
      if (tabs.value.length > 0) {
        const nextIdx = Math.min(idx, tabs.value.length - 1)
        await switchTab(tabs.value[nextIdx].id)
      } else {
        activeTabId.value = null
      }
    }
  }

  async function resizeAllWebviews() {
    const labels = tabs.value.map((t) => t.label)
    if (labels.length === 0) return
    await webviewApi.resizeAllWebviews(labels).catch(() => {})
  }

  return {
    tabs,
    activeTabId,
    loadingTabId,
    isHome,
    isWebviewLoading,
    aiPaused,
    setAiPaused,
    openTab,
    switchTab,
    switchToHome,
    closeTab,
    resizeAllWebviews,
  }
})

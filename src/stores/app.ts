import { defineStore } from 'pinia'

/**
 * 应用级状态 - Claw Browser
 * 可在此扩展全局 UI 状态、设备就绪等
 */
export const useAppStore = defineStore('app', () => {
  const ready = ref(false)

  function setReady(value: boolean) {
    ready.value = value
  }

  return { ready, setReady }
})

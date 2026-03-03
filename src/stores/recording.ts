import { defineStore } from 'pinia'
import * as webviewApi from '@/api/webview'

export type RecordingStep =
  | { type: 'navigate'; url: string }
  | { type: 'eval'; script: string }

const NAVIGATE_WAIT_MS = 2200
const EVAL_WAIT_MS = 600

export const useRecordingStore = defineStore('recording', () => {
  const steps = ref<RecordingStep[]>([])

  function pushStep(step: RecordingStep) {
    steps.value.push(step)
  }

  function clearSteps() {
    steps.value = []
  }

  /** 回放到第 toIndex 步（含），需传入当前 tab 的 webview label */
  async function replayToStep(label: string, toIndex: number): Promise<void> {
    const list = steps.value.slice(0, toIndex + 1)
    for (let i = 0; i < list.length; i++) {
      const step = list[i]
      if (step.type === 'navigate') {
        const script = `window.location.href = ${JSON.stringify(step.url)};`
        await webviewApi.evalInWebview(label, script)
        await new Promise((r) => setTimeout(r, NAVIGATE_WAIT_MS))
      } else {
        await webviewApi.evalInWebview(label, step.script)
        await new Promise((r) => setTimeout(r, EVAL_WAIT_MS))
      }
    }
  }

  return { steps, pushStep, clearSteps, replayToStep }
})

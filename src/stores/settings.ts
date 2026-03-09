import { defineStore } from 'pinia'

const LS_TOKEN = 'openclaw_bearer_token'
const LS_SESSION_KEY = 'openclaw_session_key'
const LS_BASE_URL = 'openclaw_base_url'

export const useSettingsStore = defineStore('settings', () => {
  const bearerToken = ref(localStorage.getItem(LS_TOKEN) ?? '')
  const sessionKey = ref(localStorage.getItem(LS_SESSION_KEY) ?? 'agent:main:main2')
  const baseUrl = ref(localStorage.getItem(LS_BASE_URL) ?? '')

  function save(token: string, key: string, url: string) {
    bearerToken.value = token.trim()
    sessionKey.value = key.trim()
    baseUrl.value = url.trim()
    localStorage.setItem(LS_TOKEN, token)
    localStorage.setItem(LS_SESSION_KEY, key)
    if (url) {
      localStorage.setItem(LS_BASE_URL, url)
    } else {
      localStorage.removeItem(LS_BASE_URL)
    }
  }

  return { bearerToken, sessionKey, baseUrl, save }
})

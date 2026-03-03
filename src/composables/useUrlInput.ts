const DEFAULT_SEARCH_ENGINE_URL = 'https://www.google.com/search'

export function useUrlInput(searchEngineUrl = DEFAULT_SEARCH_ENGINE_URL) {
  function isLikelyUrl(raw: string): boolean {
    const s = raw.trim()
    if (!s || /\s/.test(s)) return false
    if (/^https?:\/\//i.test(s)) return true
    if (/^.+\.[a-z]{2,6}(\/.*)?$/i.test(s)) return true
    return false
  }

  function normalizeUrl(raw: string): string {
    const s = raw.trim()
    if (!s) return ''
    if (/^https?:\/\//i.test(s)) return s
    return `https://${s}`
  }

  function getTargetUrl(raw: string): string {
    const s = raw.trim()
    if (!s) return ''
    if (isLikelyUrl(s)) return normalizeUrl(s)
    const q = encodeURIComponent(s)
    return `${searchEngineUrl}?q=${q}`
  }

  return { isLikelyUrl, normalizeUrl, getTargetUrl }
}

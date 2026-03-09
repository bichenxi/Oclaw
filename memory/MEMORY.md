# Project Memory: openclaw-auntie-browser

## Stack
- Tauri 2 + Vue 3 + TypeScript + Pinia
- Vite, vite-plugin-pages (file-based routing), unplugin-auto-import
- Single route (`/`) rendered in `src/pages/index.vue`; layout wraps via `src/layouts/default.vue`

## Project Structure
- `src/layouts/default.vue` — Main layout: profile row + TabBar + content area
- `src/pages/index.vue` — Home page (search bar)
- `src/components/TabBar.vue` — Tab bar with Home/webpage tabs + OpenClaw/Settings nav buttons
- `src/components/OpenclawPage.vue` — Full-page OpenClaw chat interface (NEW)
- `src/components/SettingsPage.vue` — Settings page for token/session key (NEW)
- `src/components/AIConsole.vue` — Old AI console, no longer imported (dead code)
- `src/stores/tabs.ts` — Tab state + specialView ('openclaw'|'settings'|null)
- `src/stores/settings.ts` — Persists bearerToken, sessionKey, baseUrl to localStorage (NEW)
- `src/stores/recording.ts` — Step recording store
- `src/stores/profile.ts` — Browser profile switching

## Key Architecture Decisions
- `specialView` in tabs store controls which full-page view is shown
  - `null` → normal (home or webview)
  - `'openclaw'` → OpenclawPage fills content area
  - `'settings'` → SettingsPage fills content area
- Switching to specialView hides the active webview (Tauri native layer)
- `isHome` = `activeTabId === null && specialView === null`
- Webview occupies full content width (LEFT_PANEL_WIDTH = 0.0 in config.rs)

## Rust Config (src-tauri/src/config.rs)
- `TAB_BAR_HEIGHT = 88.0` (profile row ~44px + tab bar 44px)
- `LEFT_PANEL_WIDTH = 0.0` (no sidebar; webview is full width)

## OpenClaw API
- `src/api/openclaw.ts` — invoke Tauri commands: start/stop process, openclawSendV1
- Stream responses arrive via Tauri event `stream-item` with `{type, text}` payload
- OpenclawPage listens to `stream-item` events and appends to messages array

## Settings Storage
- localStorage keys: `openclaw_bearer_token`, `openclaw_session_key`, `openclaw_base_url`
- Managed by `useSettingsStore` in `src/stores/settings.ts`

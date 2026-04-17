# HA Status Bar — Architecture

A macOS-only menu bar app built with Tauri 2 that polls a Home Assistant instance and displays selected entity values directly in the macOS menu bar.

---

## High-level overview

```
macOS Menu Bar
  ☀️ 3.22 kWh  🔋 78 %
  │
  ├─ [dropdown menu]
  │    ☀️ Solar Today: 3.22 kWh
  │    🔋 Powerwall: 78 %
  │    ─────────────────────
  │    ⚙ Settings...
  │    ✓ Launch at Login
  │    Quit
  │
  └─ [Settings window — hidden by default]
       URL / Token / Entity picker / Interval


Rust polling loop  ──── GET /api/states/<id> ────►  Home Assistant
(Tokio, every 30s) ◄─── entity state + unit ──────  REST API
        │
        └── updates TrayIcon (menu items + title text)
```

---

## Project structure

```
ha-status-bar/
├── src/                        # Frontend (settings window)
│   ├── index.html              # Settings UI markup
│   ├── main.js                 # Settings logic (vanilla JS, window.__TAURI__)
│   └── styles.css              # macOS-native-feeling styles, dark mode
│
└── src-tauri/
    ├── Info.plist              # Sets LSUIElement=true (hides Dock icon in bundle)
    ├── tauri.conf.json         # App config: window, bundle, macOS plist path
    ├── capabilities/
    │   └── default.json        # IPC permissions for settings window
    └── src/
        ├── main.rs             # Thin entry point
        ├── lib.rs              # Core: tray setup, commands, polling loop
        ├── config.rs           # Config struct (serde, Default)
        └── ha_client.rs        # reqwest HTTP client for HA REST API
```

---

## Rust backend (`lib.rs`)

### Startup sequence

1. Register `tauri-plugin-store` and `tauri-plugin-autostart` plugins.
2. Set `ActivationPolicy::Accessory` (hides Dock icon in dev mode).
3. Manage an `Arc<tokio::sync::Notify>` in app state — used to wake the poll loop early when config is saved.
4. Build the initial tray menu and `TrayIcon` (id: `"main"`).
5. Attach a `on_window_event` handler to the settings window to intercept the OS close button: calls `api.prevent_close()` + `win.hide()` so the window is never destroyed.
6. Spawn the background polling task.

### Polling loop

Runs in `tauri::async_runtime::spawn`. Each iteration:

1. Load config from store.
2. If token + entities are configured, call `ha_client::fetch_selected` for each entity sequentially.
3. On success: rebuild the tray menu, update the title, and **hide the tray icon** (leaving only the text).
4. On error: restore the tray icon and show `HA ⚠` as the title.
5. Wait via `tokio::select!` for either the poll interval to expire or the `Notify` to fire (triggered by `save_config`).

### Tray display

- **Title**: all selected entity values joined by two spaces — e.g. `☀️ 3.22 kWh  🔋 78 %`. Numeric values are formatted to 2 decimal places.
- **Icon**: shown only when no entity data is available (startup, error, nothing selected); hidden once real values are displayed.
- **Menu items**: one disabled item per entity showing emoji + friendly name + value + unit. Followed by Settings, Launch at Login (check item), Quit.

### Emoji mapping (`entity_emoji`)

Resolves an emoji for each entity by matching keywords in `entity_id + friendly_name` (e.g. `solar` → ☀️, `battery`/`powerwall` → 🔋, `grid` → ⚡), then falls back to domain-based mapping (`light` → 💡, `climate` → 🌡️, etc.), then `📊`.

---

## Frontend (`src/`)

A plain HTML/JS settings window — no bundler, no framework. Served as static files from `src/` via `frontendDist: "../src"`. Accesses Tauri APIs via `window.__TAURI__` (injected by `withGlobalTauri: true`).

### Lifecycle

1. `init()` — load saved config, populate form fields, attach event listeners.
2. If credentials are already set, auto-call `handleTestConnection` to pre-populate the entity list.
3. Register `onCloseRequested` handler: reverts `selectedEntities` to the last saved state, calls `preview_selection`, then hides the window.

### Entity selection flow

- **Test Connection** → `invoke("test_connection")` → `invoke("fetch_entities")` → renders checkbox list grouped by domain with emojis.
- Each checkbox change updates `selectedEntities` in memory and immediately calls `invoke("preview_selection", { states })` — the Rust side rebuilds the tray with the preview state so the user sees changes live.
- **Save** → `invoke("save_config")` → persists to store, signals the poll loop via `Notify`, hides window, updates `originalSelectedEntities`.
- **Cancel (X button)** → reverts to `originalSelectedEntities`, sends a revert preview to the tray, hides window. Handled both in JS (`onCloseRequested`) and as a safety net in Rust (`on_window_event`).

---

## Configuration persistence

`tauri-plugin-store` writes a single JSON key `"config"` to `config.json` in the app data directory. The `Config` struct is serialised/deserialised via serde.

```rust
pub struct Config {
    pub ha_url: String,              // e.g. "http://homeassistant.local:8123"
    pub ha_token: String,            // Long-lived access token
    pub selected_entities: Vec<String>, // e.g. ["sensor.solar_power", "sensor.powerwall"]
    pub refresh_interval_secs: u64,  // min 5, default 30
}
```

---

## Key dependencies

| Crate / Package | Purpose |
|---|---|
| `tauri 2` | App shell, tray icon, webview window, IPC |
| `tauri-plugin-store 2` | JSON config persistence |
| `tauri-plugin-autostart 2` | LaunchAgent registration for login item |
| `reqwest 0.12` (rustls) | Async HTTP client for HA REST API |
| `tokio 1` (full) | Async runtime, `Notify`, `select!`, `sleep` |
| `@tauri-apps/api 2` | JS-side IPC and window management |

---

## Dock icon suppression

Two complementary mechanisms ensure no Dock icon appears:

1. **`Info.plist`** (`LSUIElement = true`) — effective for the bundled `.app`.
2. **`ActivationPolicy::Accessory`** in Rust setup — effective during `tauri dev`.

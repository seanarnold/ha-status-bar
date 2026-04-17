# HA Status Bar

A macOS menu bar app that displays live Home Assistant entity values — solar generation, battery charge, grid usage, temperature, or anything else — right in your menu bar.

Built with [Tauri 2](https://tauri.app), no Dock icon, no background windows.

---

## Screenshot

<!-- Add a screenshot here -->

---

## Features

- Displays any Home Assistant entity value in the macOS menu bar
- Polls your HA instance on a configurable interval (default 30 seconds)
- Emoji icons auto-assigned based on entity type (☀️ solar, 🔋 battery, ⚡ grid, 🌡️ temperature, etc.)
- Live preview — see menu bar update as you select entities in Settings
- Persists configuration across restarts
- Optional launch at login via macOS LaunchAgent
- No Dock icon

## Requirements

- macOS 11 or later
- A running [Home Assistant](https://www.home-assistant.io) instance accessible on your network
- A Home Assistant [Long-Lived Access Token](https://developers.home-assistant.io/docs/auth_api/#long-lived-access-token)

---

## Installation

### Download (recommended)

1. Download the latest `.dmg` from the [Releases](../../releases) page
2. Open the `.dmg` and drag **HA Status Bar** to Applications
3. Open the app — it will appear in your menu bar showing `HA`

> **Note:** macOS may show a security warning since the app isn't notarised. Go to **System Settings → Privacy & Security** and click **Open Anyway**.

### Build from source

Requirements: [Rust](https://rustup.rs) and [Node.js](https://nodejs.org) 18+.

```bash
git clone https://github.com/your-username/ha-status-bar
cd ha-status-bar
npm install
npm run tauri build
```

The built app will be at `src-tauri/target/release/bundle/macos/HA Status Bar.app`.

---

## Setup

1. Click the `HA` icon in your menu bar and choose **⚙ Settings...**
2. Enter your Home Assistant URL (e.g. `http://homeassistant.local:8123`)
3. Paste your Long-Lived Access Token
4. Click **Test Connection** — your entity list will load automatically
5. Check the entities you want to display
6. Click **Save**

The menu bar will update within one poll cycle. Numeric values are shown to 2 decimal places.

### Getting a Long-Lived Access Token

1. In Home Assistant, click your profile picture (bottom left)
2. Scroll to **Long-Lived Access Tokens**
3. Click **Create Token**, give it a name, and copy the token

---

## Usage

| Action | Result |
|---|---|
| Click menu bar | Opens the dropdown showing current entity values |
| **⚙ Settings...** | Opens the settings window |
| **Launch at Login** | Toggles automatic startup via macOS LaunchAgent |
| **Quit** | Exits the app |
| Close settings with X | Reverts any unsaved entity selection changes |

### Supported entity types

The app works with any Home Assistant entity that has a readable state. Emojis are assigned automatically:

| Keyword match | Emoji |
|---|---|
| solar, pv | ☀️ |
| battery, powerwall | 🔋 |
| grid, power, energy, watt, kwh | ⚡ |
| temperature | 🌡️ |
| humidity, water | 💧 |
| door | 🚪 |
| motion, occupancy | 🚶 |
| lock | 🔒 |
| light (domain) | 💡 |
| media_player (domain) | 🎵 |
| everything else | 📊 |

---

## Development

```bash
npm install
. "$HOME/.cargo/env"   # if Rust was just installed
npm run tauri dev
```

The app hot-reloads Rust changes automatically. The settings window reloads on JS/CSS changes.

See [ARCHITECTURE.md](ARCHITECTURE.md) for a full breakdown of how the app is structured.

---

## License

MIT

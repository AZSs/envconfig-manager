# EnvConfig Manager

English | [中文](./README.zh-CN.md)

A cross-platform desktop app for managing shell configuration files and environment variables. Built with Tauri 2 + Vue 3 + TypeScript.

## Features

- **Config File Editor** — Scan and edit `.zshrc`, `.bashrc`, `.bash_profile` and more with a built-in CodeMirror 6 editor
- **Environment Variables** — View, create, edit, and delete env vars with grouping and search
- **Backup & Restore** — Auto-backup before changes, manual snapshots, one-click rollback
- **Config Profiles** — SwitchHosts-style profile management with toggle switches to quickly enable/disable config sets, import/export support
- **Config Validation** — Real-time syntax checking (`bash -n` / `zsh -n`), PATH existence detection, duplicate export/alias detection
- **Theme** — Light/Dark theme (Catppuccin Latte/Mocha), adjustable editor font size
- **Auto Update** — In-app updates via GitHub Releases
- **Privilege Elevation** — macOS/Linux system auth dialog, Windows UAC

## Screenshots

<!-- Add screenshots here -->

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop Framework | Tauri 2.x (Rust) |
| Frontend | Vue 3 + TypeScript + Vite |
| State Management | Pinia |
| Router | Vue Router 4 |
| Code Editor | CodeMirror 6 |
| Styling | CSS Variables (Catppuccin theme) |

## Installation

Download the latest release from the [Releases](https://github.com/AZSs/envconfig-manager/releases) page:

- **macOS** (Apple Silicon / Intel) — `.dmg`
- **Windows** — `.msi` / `.exe`

## Development

### Prerequisites

- Node.js 18+
- Rust 1.70+
- [Tauri CLI prerequisites](https://v2.tauri.app/start/prerequisites/)

### Setup

```bash
# Install dependencies
npm install

# Start dev mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

## System Requirements

- macOS 12+ (Apple Silicon / Intel)
- Windows 10/11

## License

MIT

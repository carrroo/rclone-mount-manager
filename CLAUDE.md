# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rclone Mount Manager — a macOS menu bar app for managing rclone mounts via GUI. Built with **Tauri v2** (Rust backend) + **Vue 3 + TypeScript** (frontend). Bilingual (English/Chinese) with vue-i18n.

## Development Commands

```bash
# Full Tauri dev mode (Rust + frontend hot-reload on port 1420)
npm run tauri-dev

# Frontend only (Vite dev server)
npm run dev

# Production build — outputs .app and .dmg in src-tauri/target/release/bundle/
npm run tauri-build

# Type-check frontend (vue-tsc --noEmit)
npm run build

# Build Rust backend only
cargo build --manifest-path src-tauri/Cargo.toml
```

There are no tests, linter, or formatter configured.

## Architecture

### Frontend (`src/`)

- **Entry:** `main.ts` → mounts Vue app with Pinia + vue-i18n
- **App.vue** — root shell: header with mount-count badge, toggles between MountList/MountForm views, dependency check modal, about dialog. Polls mount status every 5s via `setInterval`.
- **Components:** `MountList.vue` (cards with inline editing, mount/unmount/delete), `MountForm.vue` (add custom mount with remote selector from rclone.conf), `DependencyCheck.vue` (rclone/macFUSE status), `AboutDialog.vue`
- **Pinia stores:**
  - `mounts.ts` — central state: `items`, `loading`, `mountedCount`. Bridges frontend ↔ Rust via Tauri `invoke()` calls. Persists custom mounts and remote config overrides in localStorage (`rclone-remote-configs`, `rclone-mounts` keys).
  - `settings.ts` — language setting (`"zh" | "en" | "system"`). Syncs with Rust backend via `get_language`/`set_language` commands and listens for `language-changed` Tauri events.
- **Locales:** `en.ts`, `zh.ts` — full translation dictionaries; `index.ts` sets up vue-i18n with system locale detection.

### Backend (`src-tauri/src/`)

- **main.rs** — app entry: initializes language state, registers `RcloneManager` as managed state, sets up macOS menu (Open/About/Quit + language submenu), system tray (Open/Quit), intercepts window close to hide instead of quit.
- **commands.rs** — 8 Tauri command handlers: `get_all_mounts`, `mount_remote`, `unmount_remote`, `update_remote_config`, `start_auto_reconnect`, `check_dependencies`, `get_language`, `set_language`. All return `ApiResponse<T>`.
- **rclone.rs** — core logic:
  - `RcloneManager` holds path to `~/.config/rclone/rclone.conf`
  - Parses rclone.conf via regex (section headers + key=value)
  - Detects mounted filesystems by running `/sbin/mount`
  - Mount: validates paths (must be under `/Volumes/` or home), blocks shell metacharacters via `is_safe_arg()`, runs `rclone mount` with `nohup`, logs to `~/Library/Caches/rclone-mount-manager/rclone-mount.log`
  - Unmount: runs `diskutil unmount force`
  - Config updates: modifies rclone.conf in-place, whitelisted to `host`/`user`/`pass`/`port` keys
  - Reconnect monitor: background thread polls `/sbin/mount` every 30s, auto-remounts dropped custom mounts
- **lang.rs** — language state via `AtomicBool` globals, persists to `~/.config/rclone-mount-manager/language`, emits `language-changed` event to frontend, provides Rust-side `t()` translations for menu items.

### Key Design Points

- **Menu bar app pattern:** closing window hides it; tray icon persists; dock click reopens via macOS `Reopen` event.
- **Dual persistence:** custom mounts/config overrides in localStorage; actual rclone.conf managed by Rust backend.
- **Language stored in two places:** Rust writes to `~/.config/rclone-mount-manager/language`; frontend syncs via Tauri commands.
- **Security:** `is_safe_arg()` blocks shell metacharacters; `is_path_allowed()` restricts mount points to `/Volumes/` or home directory; config updates whitelisted to specific keys.
- **Tauri capabilities** defined in `src-tauri/capabilities/default.json`.

## macOS Development Notes

### Launch Services Stale Entries

Development can leave orphaned Launch Services records, causing duplicate app icons in Launchpad. Root causes:

1. **Bundle ID changed** — macOS treats old and new `identifier` in `tauri.conf.json` as separate apps
2. **Build artifacts scattered** — `npm run tauri-build` creates `.app` under `target/release/bundle/macos/`; double-clicking it registers the app
3. **Running from DMG** — opening the app directly from a mounted DMG before dragging it to `/Applications` leaves orphan records

**Full cleanup (both steps required):**

```bash
# 1. Delete the Launch Services database
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -delete

# 2. sudo reboot to force a complete rebuild (plain reboot is not enough)
sudo reboot
```

**Prevention:**
- Keep the Bundle ID stable once set
- Remove `target/release/bundle/macos/*.app` after each build (see `clean-bundle` script)
- Always drag `.app` to `/Applications` first; never run directly from the DMG
- Use `npm run tauri-dev` for local testing instead of double-clicking build artifacts

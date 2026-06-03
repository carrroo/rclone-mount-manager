/**
 * API layer — type-safe wrappers around all Tauri invoke() calls.
 *
 * Every backend command is exposed as a plain async function here.
 * Stores and components should import from this module instead of
 * calling invoke() directly, so that command names and parameter
 * shapes are defined in one place.
 */
import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "../types";

/** Type-safe wrapper around Tauri's invoke(). */
async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<ApiResponse<T>> {
  return invoke<ApiResponse<T>>(cmd, args);
}

// ── Mounts ────────────────────────────────────────────────

/** Fetch all mount items (config-sourced + custom), merging mount status. */
export async function getAllMounts(customMounts: unknown) {
  return call<unknown[]>("get_all_mounts", { customMounts });
}

/** Mount a remote path to a local mount point. */
export async function mountRemote(remotePath: string, mountPoint: string, extraArgs: string[]) {
  return call<null>("mount_remote", { remotePath, mountPoint, extraArgs });
}

/** Force-unmount a mount point via diskutil. */
export async function unmountRemote(mountPoint: string) {
  return call<null>("unmount_remote", { mountPoint });
}

// ── Config ────────────────────────────────────────────────

/** Update whitelisted keys (host/user/pass/port) in a remote section of rclone.conf. */
export async function updateRemoteConfig(name: string, updates: Record<string, string>) {
  return call<null>("update_remote_config", { name, updates });
}

// ── Auto-reconnect ───────────────────────────────────────

/** Start the background reconnect monitor for custom mounts. */
export async function startAutoReconnect(configs: unknown) {
  return call<null>("start_auto_reconnect", { configs });
}

// ── Dependencies ──────────────────────────────────────────

/** Check whether rclone and macFUSE are installed. */
export async function checkDependencies() {
  return call<unknown>("check_dependencies");
}

// ── Language ──────────────────────────────────────────────

/** Get the current language setting from the Rust backend. */
export async function getLanguage(): Promise<string> {
  return invoke<string>("get_language");
}

/** Persist a language change to the Rust backend and update native menus. */
export async function setLanguage(lang: string): Promise<void> {
  return invoke("set_language", { lang });
}

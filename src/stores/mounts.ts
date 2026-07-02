/**
 * Mount store — central state for all mount items.
 *
 * Bridges the Vue frontend with the Rust backend via the api/ layer.
 * Persists custom mounts and remote config overrides in localStorage
 * so they survive page reloads; actual rclone.conf is managed by Rust.
 */
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { MountItem, SavedRemoteConfig, ApiResponse } from "../types";
import * as api from "../api";

/** Generate a unique ID for custom mounts. */
function generateId(): string {
  return "custom:" + crypto.randomUUID();
}

/** localStorage keys for persisted mount data. */
const REMOTE_CONFIGS_KEY = "rclone-remote-configs";
const CUSTOM_MOUNTS_KEY = "rclone-mounts";

/** Load saved remote config overrides from localStorage. */
function getSavedRemoteConfigs(): Record<string, SavedRemoteConfig> {
  const saved = localStorage.getItem(REMOTE_CONFIGS_KEY);
  if (saved) {
    try {
      return JSON.parse(saved);
    } catch {
      return {};
    }
  }
  return {};
}

/** Persist remote config overrides to localStorage. */
function saveRemoteConfigs(configs: Record<string, SavedRemoteConfig>) {
  localStorage.setItem(REMOTE_CONFIGS_KEY, JSON.stringify(configs));
}

/** Load custom mounts from localStorage. */
function getCustomMounts(): MountItem[] {
  const saved = localStorage.getItem(CUSTOM_MOUNTS_KEY);
  if (saved) {
    try {
      return JSON.parse(saved);
    } catch {
      return [];
    }
  }
  return [];
}

/** Persist custom mounts to localStorage. */
function saveCustomMounts(customs: MountItem[]) {
  localStorage.setItem(CUSTOM_MOUNTS_KEY, JSON.stringify(customs));
}

export const useMountStore = defineStore("mounts", () => {
  const items = ref<MountItem[]>([]);

  /** Global loading flag for list-level operations (load/refresh). */
  const loading = ref(false);

  /** Per-item operation tracking — maps item.id → operation type string.
   *  Prevents one stuck operation from blocking all UI interactions. */
  const pendingOps = ref<Record<string, string>>({});

  const error = ref<string | null>(null);

  const mountedCount = computed(() => {
    return items.value.filter((i) => i.mounted).length;
  });

  /** Check whether a specific item has a pending operation. */
  function isPending(id: string): boolean {
    return id in pendingOps.value;
  }

  /**
   * Fetch all mounts from backend and merge with saved config overrides.
   *
   * IMPORTANT: this function only READS from savedConfigs; it never writes.
   * savedConfigs is the authoritative source for user-edited remote_path
   * and mount_point. Writing during background polling risks overwriting
   * user preferences with backend defaults when rclone is in a bad state.
   */
  async function loadMounts() {
    loading.value = true;
    error.value = null;
    try {
      const customMounts = getCustomMounts();
      const res = (await api.getAllMounts(customMounts)) as ApiResponse<MountItem[]>;
      if (res.success && res.data) {
        const savedConfigs = getSavedRemoteConfigs();
        const merged = res.data.map((item) => {
          if (item.source !== "config") return item;

          const saved = savedConfigs[item.name];
          if (saved) {
            // User has edited this remote before — use saved paths as
            // the authoritative source, regardless of mount state.
            return {
              ...item,
              remote_path: saved.remote_path,
              mount_point: saved.mount_point,
              host: saved.host || item.host,
              user: saved.user || item.user,
              pass: saved.pass || item.pass,
              port: saved.port || item.port,
            };
          }

          // No saved config yet — use whatever the backend returned.
          return item;
        });
        items.value = merged;
      } else {
        error.value = res.error || "error.load_failed";
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  /** Mount a remote and persist config override if it's from rclone.conf. */
  async function doMount(item: MountItem) {
    pendingOps.value[item.id] = "mount";
    error.value = null;
    try {
      const res = await api.mountRemote(item.remote_path, item.mount_point, item.extra_args);
      if (res.success) {
        item.mounted = true;
        if (item.source === "config") {
          const saved = getSavedRemoteConfigs();
          saved[item.name] = {
            name: item.name,
            remote_path: item.remote_path,
            mount_point: item.mount_point,
            host: item.host,
            user: item.user,
            port: item.port,
            // pass intentionally not persisted for security
          };
          saveRemoteConfigs(saved);
        }
      } else {
        error.value = res.error || "error.mount_failed";
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      delete pendingOps.value[item.id];
    }
  }

  /** Unmount a remote by its mount point. */
  async function doUnmount(item: MountItem) {
    pendingOps.value[item.id] = "unmount";
    error.value = null;
    try {
      const res = await api.unmountRemote(item.mount_point);
      if (res.success) {
        item.mounted = false;
      } else {
        error.value = res.error || "error.unmount_failed";
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      delete pendingOps.value[item.id];
    }
  }

  /** Update a mount's config (remote path, mount point, credentials).
   *  Syncs changes to rclone.conf first, then updates in-memory state. */
  async function updateMountConfig(item: MountItem, remotePath: string, mountPoint: string, host: string, user: string, pass: string, port: string) {
    const updates: Record<string, string> = {};
    if (remotePath !== item.remote_path) updates["remote_path"] = remotePath;
    if (mountPoint !== item.mount_point) updates["mount_point"] = mountPoint;
    if (host !== item.host) updates["host"] = host;
    if (user !== item.user) updates["user"] = user;
    if (pass !== item.pass) updates["pass"] = pass;
    if (port !== item.port) updates["port"] = port;

    if (item.source === "config") {
      const confUpdates: Record<string, string> = {};
      if ("host" in updates) confUpdates["host"] = host;
      if ("user" in updates) confUpdates["user"] = user;
      if ("pass" in updates) confUpdates["pass"] = pass;
      if ("port" in updates) confUpdates["port"] = port;
      if (Object.keys(confUpdates).length > 0) {
        const res = await api.updateRemoteConfig(item.name, confUpdates);
        if (!res.success) {
          error.value = res.error || "error.write_conf_failed";
          return;
        }
      }
    }

    item.remote_path = remotePath;
    item.mount_point = mountPoint;
    item.host = host;
    item.user = user;
    item.pass = pass;
    item.port = port;

    if (item.source === "config") {
      const saved = getSavedRemoteConfigs();
      saved[item.name] = {
        name: item.name,
        remote_path: remotePath,
        mount_point: mountPoint,
        host,
        user,
        port,
        // pass intentionally not persisted for security
      };
      saveRemoteConfigs(saved);
    }
  }

  /** Add a new custom mount and persist it to localStorage. */
  function addCustomMount(item: Omit<MountItem, "id" | "mounted">) {
    const newItem: MountItem = {
      ...item,
      id: generateId(),
      mounted: false,
    };
    const customs = getCustomMounts();
    customs.push(newItem);
    saveCustomMounts(customs);
    items.value.push(newItem);
  }

  /** Remove a custom mount by ID from both localStorage and in-memory state. */
  function removeCustomMount(id: string) {
    const customs = getCustomMounts().filter((m) => m.id !== id);
    saveCustomMounts(customs);
    items.value = items.value.filter((m) => m.id !== id);
  }

  return {
    items,
    loading,
    pendingOps,
    isPending,
    error,
    mountedCount,
    loadMounts,
    doMount,
    doUnmount,
    updateMountConfig,
    addCustomMount,
    removeCustomMount,
  };
});

/**
 * Mount store — central state for all mount items.
 *
 * Bridges the Vue frontend with the Rust backend via the api/ layer.
 * All mount configs are stored in rclone.conf (managed by Rust backend).
 */
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { MountItem, ApiResponse } from "../types";
import * as api from "../api";

export const useMountStore = defineStore("mounts", () => {
  const items = ref<MountItem[]>([]);

  /** Global loading flag for list-level operations (load/refresh). */
  const loading = ref(false);

  /** Per-item operation tracking — maps item.id → operation type string.
   *  Prevents one stuck operation from blocking all UI interactions. */
  const pendingOps = ref<Record<string, string>>({});

  /**
   * Recently completed mount/unmount operations that have not yet been
   * confirmed by a backend poll. Used to keep the UI in the optimistic
   * state (`mounted` or `unmounted`) for a short grace period so that a
   * 5-second background poll does not briefly flip the button back.
   */
  const recentOps = ref<Record<string, { op: "mount" | "unmount"; until: number }>>({});

  const error = ref<string | null>(null);

  const mountedCount = computed(() => {
    return items.value.filter((i) => i.mounted).length;
  });

  /** Check whether a specific item has a pending operation. */
  function isPending(id: string): boolean {
    return id in pendingOps.value;
  }

  /** Fetch all mounts from backend (rclone.conf).
   *  Backend uses saved preferences for user-configured paths. */
  async function loadMounts(clearError = true) {
    if (loading.value) return; // skip if a refresh is already in flight
    loading.value = true;
    if (clearError) error.value = null;
    try {
      const res = (await api.getAllMounts([])) as ApiResponse<MountItem[]>;
      if (res.success && res.data) {
        const now = Date.now();
        for (const item of res.data) {
          const recent = recentOps.value[item.id];
          if (recent && now < recent.until) {
            // Keep the optimistic mounted state until the grace period expires.
            item.mounted = recent.op === "mount";
          }
        }
        // Clean up expired entries.
        for (const id of Object.keys(recentOps.value)) {
          if (now >= recentOps.value[id].until) {
            delete recentOps.value[id];
          }
        }
        items.value = res.data;
      } else {
        error.value = res.error || "error.load_failed";
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  /** Mount a remote. */
  async function doMount(item: MountItem) {
    pendingOps.value[item.id] = "mount";
    error.value = null;
    try {
      // Save path preferences first
      await api.saveMountPref(item.name, item.remote_path, item.mount_point);

      const res = await api.mountRemote(item.remote_path, item.mount_point, item.extra_args);
      if (res.success) {
        item.mounted = true;
        recentOps.value[item.id] = { op: "mount", until: Date.now() + 3000 };
      } else {
        error.value = res.error || "error.mount_failed";
      }
      // Sync real state immediately so any slow-mount race (e.g. backend
      // timeout while the OS still finishes mounting) is reconciled quickly.
      await loadMounts(false);
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
        recentOps.value[item.id] = { op: "unmount", until: Date.now() + 3000 };
      } else {
        error.value = res.error || "error.unmount_failed";
      }
      // Sync real state immediately.
      await loadMounts();
    } catch (e) {
      error.value = String(e);
    } finally {
      delete pendingOps.value[item.id];
    }
  }

  /** Update a mount's config in rclone.conf. */
  async function updateMountConfig(item: MountItem, configType: string, remotePath: string, mountPoint: string, host: string, user: string, pass: string, port: string) {
    const confUpdates: Record<string, string> = {};
    if (configType !== item.config_type) confUpdates["type"] = configType;
    if (host !== item.host) confUpdates["host"] = host;
    if (user !== item.user) confUpdates["user"] = user;
    if (pass !== item.pass) confUpdates["pass"] = pass;
    if (port !== item.port) confUpdates["port"] = port;

    if (Object.keys(confUpdates).length > 0) {
      const res = await api.updateRemoteConfig(item.name, confUpdates);
      if (!res.success) {
        error.value = res.error || "error.write_conf_failed";
        return;
      }
    }

    item.config_type = configType;
    item.remote_path = remotePath;
    item.mount_point = mountPoint;
    item.host = host;
    item.user = user;
    item.pass = pass;
    item.port = port;

    // Save path preferences
    await api.saveMountPref(item.name, remotePath, mountPoint);
  }

  /** Add a new remote to rclone.conf and mount it. */
  async function addAndMount(name: string, configType: string, remotePath: string, mountPoint: string, options: Record<string, string>, extraArgs: string[]): Promise<boolean> {
    error.value = null;
    try {
      // 1. Write to rclone.conf
      const res = await api.addRemoteConfig(name, configType, options);
      if (!res.success) {
        error.value = res.error || "error.write_conf_failed";
        return false;
      }

      // 2. Save path preferences first (so they persist even if mount fails)
      await api.saveMountPref(name, remotePath, mountPoint);

      // 3. Add to in-memory list immediately with user's paths
      const newItem: MountItem = {
        id: `config:${name}`,
        name,
        remote_path: remotePath,
        mount_point: mountPoint,
        source: "config",
        mounted: false,
        config_type: configType,
        extra_args: extraArgs,
        host: options.host || "",
        user: options.user || "",
        pass: options.pass || "",
        port: options.port || "",
      };
      items.value.push(newItem);

      // 4. Mount with the user's paths
      const mountRes = await api.mountRemote(remotePath, mountPoint, extraArgs);
      if (mountRes.success) {
        newItem.mounted = true;
        recentOps.value[newItem.id] = { op: "mount", until: Date.now() + 3000 };
      } else {
        error.value = mountRes.error || "error.mount_failed";
      }
      // Sync real state immediately.
      await loadMounts();
      return mountRes.success;
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  let orderTimer: ReturnType<typeof setTimeout> | null = null;

  /** Move item up in the list. */
  function moveUp(index: number) {
    if (index <= 0 || index >= items.value.length) return;
    const item = items.value.splice(index, 1)[0];
    items.value.splice(index - 1, 0, item);
    debouncedSaveOrder();
  }

  /** Move item down in the list. */
  function moveDown(index: number) {
    if (index < 0 || index >= items.value.length - 1) return;
    const item = items.value.splice(index, 1)[0];
    items.value.splice(index + 1, 0, item);
    debouncedSaveOrder();
  }

  /** Debounced save order — waits 500ms after last change. */
  function debouncedSaveOrder() {
    if (orderTimer) clearTimeout(orderTimer);
    orderTimer = setTimeout(() => {
      const order = items.value.map(i => i.name);
      api.saveMountOrder(order);
    }, 500);
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
    addAndMount,
    moveUp,
    moveDown,
  };
});

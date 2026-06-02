import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface MountItem {
  id: string;
  name: string;
  remote_path: string;
  mount_point: string;
  source: string;
  mounted: boolean;
  config_type: string;
  extra_args: string[];
  host: string;
  user: string;
  pass: string;
  port: string;
}

interface SavedRemoteConfig {
  name: string;
  remote_path: string;
  mount_point: string;
  host: string;
  user: string;
  pass: string;
  port: string;
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

function generateId(): string {
  return "custom:" + Date.now().toString(36) + Math.random().toString(36).substr(2);
}

const REMOTE_CONFIGS_KEY = "rclone-remote-configs";
const CUSTOM_MOUNTS_KEY = "rclone-mounts";

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

function saveRemoteConfigs(configs: Record<string, SavedRemoteConfig>) {
  localStorage.setItem(REMOTE_CONFIGS_KEY, JSON.stringify(configs));
}

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

function saveCustomMounts(customs: MountItem[]) {
  localStorage.setItem(CUSTOM_MOUNTS_KEY, JSON.stringify(customs));
}

export const useMountStore = defineStore("mounts", () => {
  const items = ref<MountItem[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const mountedCount = computed(() => {
    return items.value.filter((i) => i.mounted).length;
  });

  async function loadMounts() {
    loading.value = true;
    error.value = null;
    try {
      const customMounts = getCustomMounts();
      const res = await invoke<ApiResponse<MountItem[]>>("get_all_mounts", {
        customMounts: customMounts,
      });
      if (res.success && res.data) {
        let savedConfigs = getSavedRemoteConfigs();
        const merged = res.data.map((item) => {
          if (item.source !== "config") return item;

          if (item.mounted) {
            // Remember the actual config when mounted
            savedConfigs[item.name] = {
              name: item.name,
              remote_path: item.remote_path,
              mount_point: item.mount_point,
              host: item.host,
              user: item.user,
              pass: item.pass,
              port: item.port,
            };
            return item;
          } else {
            // Use saved config if available, otherwise keep defaults
            const saved = savedConfigs[item.name];
            if (saved) {
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
            return item;
          }
        });
        saveRemoteConfigs(savedConfigs);
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

  async function doMount(item: MountItem) {
    loading.value = true;
    try {
      const res = await invoke<ApiResponse<null>>("mount_remote", {
        remotePath: item.remote_path,
        mountPoint: item.mount_point,
        extraArgs: item.extra_args,
      });
      if (res.success) {
        item.mounted = true;
        // Save the config used for mounting
        if (item.source === "config") {
          const saved = getSavedRemoteConfigs();
          saved[item.name] = {
            name: item.name,
            remote_path: item.remote_path,
            mount_point: item.mount_point,
            host: item.host,
            user: item.user,
            pass: item.pass,
            port: item.port,
          };
          saveRemoteConfigs(saved);
        }
      } else {
        error.value = res.error || "error.mount_failed";
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function doUnmount(item: MountItem) {
    loading.value = true;
    try {
      const res = await invoke<ApiResponse<null>>("unmount_remote", {
        mountPoint: item.mount_point,
      });
      if (res.success) {
        item.mounted = false;
      } else {
        error.value = res.error || "error.unmount_failed";
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function updateMountConfig(item: MountItem, remotePath: string, mountPoint: string, host: string, user: string, pass: string, port: string) {
    // Build updates map with only changed fields
    const updates: Record<string, string> = {};
    if (remotePath !== item.remote_path) updates["remote_path"] = remotePath;
    if (mountPoint !== item.mount_point) updates["mount_point"] = mountPoint;
    if (host !== item.host) updates["host"] = host;
    if (user !== item.user) updates["user"] = user;
    if (pass !== item.pass) updates["pass"] = pass;
    if (port !== item.port) updates["port"] = port;

    // Sync changed fields to rclone.conf first (before updating in-memory state)
    if (item.source === "config") {
      const confUpdates: Record<string, string> = {};
      if ("host" in updates) confUpdates["host"] = host;
      if ("user" in updates) confUpdates["user"] = user;
      if ("pass" in updates) confUpdates["pass"] = pass;
      if ("port" in updates) confUpdates["port"] = port;
      if (Object.keys(confUpdates).length > 0) {
        try {
          const res = await invoke<ApiResponse<null>>("update_remote_config", {
            name: item.name,
            updates: confUpdates,
          });
          if (!res.success) {
            error.value = res.error || "error.write_conf_failed";
            return;
          }
        } catch (e) {
          error.value = String(e);
          return;
        }
      }
    }

    // Only update in-memory state after backend succeeds
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
        pass,
        port,
      };
      saveRemoteConfigs(saved);
    }
  }

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

  function removeCustomMount(id: string) {
    const customs = getCustomMounts().filter((m) => m.id !== id);
    saveCustomMounts(customs);
    items.value = items.value.filter((m) => m.id !== id);
  }

  async function refresh() {
    await loadMounts();
  }

  return {
    items,
    loading,
    error,
    mountedCount,
    loadMounts,
    doMount,
    doUnmount,
    updateMountConfig,
    addCustomMount,
    removeCustomMount,
    refresh,
  };
});

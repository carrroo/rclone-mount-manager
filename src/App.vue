<template>
  <div class="app">
    <header class="header">
      <h1>Rclone Mount Manager</h1>
      <div class="actions">
        <span class="status-badge" :class="{ active: store.mountedCount > 0 }">
          {{ t('badge.mountedCount', { count: store.mountedCount }) }}
        </span>
        <button class="btn" @click="checkDeps()" :title="t('header.checkDeps')">
          {{ t('header.checkDeps') }}
        </button>
        <button class="btn btn-primary" @click="showForm = true">
          {{ t('header.addMount') }}
        </button>
        <button class="btn" @click="store.refresh">{{ t('header.refresh') }}</button>
      </div>
    </header>

    <main class="main">
      <MountList v-if="!showForm" />
      <MountForm v-else @close="showForm = false" @saved="onSaved" />
    </main>

    <div v-if="store.error" class="toast toast-error" @click="store.error = null">
      {{ translateError(store.error) }}
    </div>

    <DependencyCheck ref="depCheck" />
    <AboutDialog />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { useMountStore } from "./stores/mounts";
import { useSettingsStore } from "./stores/settings";
import MountList from "./components/MountList.vue";
import MountForm from "./components/MountForm.vue";
import DependencyCheck, { type Deps } from "./components/DependencyCheck.vue";
import AboutDialog from "./components/AboutDialog.vue";

const { t, te } = useI18n();
const store = useMountStore();
const settings = useSettingsStore();
const showForm = ref(false);
const depCheck = ref<InstanceType<typeof DependencyCheck> | null>(null);

function onSaved() {
  showForm.value = false;
}

function translateError(msg: string): string {
  if (msg.startsWith('error.')) {
    const key = msg.split(':')[0];
    const rest = msg.substring(key.length + 1);
    if (te(key)) {
      return t(key, { msg: rest }) as string;
    }
  }
  return msg;
}

async function checkDeps(showOnlyOnError = false) {
  try {
    const res = await invoke<{ success: boolean; data: Deps | null }>("check_dependencies");
    if (res.success && res.data) {
      const { rclone_installed, macfuse_installed } = res.data;
      if (!showOnlyOnError || !rclone_installed || !macfuse_installed) {
        depCheck.value?.open(res.data);
      }
    }
  } catch {
    // ignore
  }
}

onMounted(async () => {
  await settings.initLanguage();
  await store.loadMounts();
  await checkDeps(true);

  setInterval(() => {
    store.loadMounts();
  }, 5000);
});
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  background: #f5f5f5;
  color: #333;
}

.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.header {
  background: #fff;
  border-bottom: 1px solid #e0e0e0;
  padding: 16px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header h1 {
  font-size: 20px;
  font-weight: 600;
}

.actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.status-badge {
  padding: 4px 12px;
  border-radius: 12px;
  background: #e8e8e8;
  font-size: 13px;
  color: #666;
}

.status-badge.active {
  background: #d4edda;
  color: #155724;
}

.btn {
  padding: 8px 16px;
  border: 1px solid #ddd;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.btn:hover {
  background: #f0f0f0;
}

.btn-primary {
  background: #007bff;
  color: #fff;
  border-color: #007bff;
}

.btn-primary:hover {
  background: #0056b3;
}

.btn-success {
  background: #28a745;
  color: #fff;
  border-color: #28a745;
}

.btn-success:hover {
  background: #1e7e34;
}

.btn-danger {
  background: #dc3545;
  color: #fff;
  border-color: #dc3545;
}

.btn-danger:hover {
  background: #c82333;
}

.main {
  flex: 1;
  padding: 24px;
}

.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 12px 24px;
  border-radius: 8px;
  color: #fff;
  cursor: pointer;
  z-index: 1000;
}

.toast-error {
  background: #dc3545;
}

input,
select,
textarea {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  width: 100%;
}

input:focus,
select:focus,
textarea:focus {
  outline: none;
  border-color: #007bff;
}

label {
  font-size: 13px;
  color: #666;
  margin-bottom: 4px;
  display: block;
}

.form-group {
  margin-bottom: 16px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}
</style>

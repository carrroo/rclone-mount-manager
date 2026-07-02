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
        <button class="btn" @click="store.loadMounts">{{ t('header.refresh') }}</button>
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
import { ref, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useMountStore } from "./stores/mounts";
import { useSettingsStore } from "./stores/settings";
import MountList from "./components/MountList.vue";
import MountForm from "./components/MountForm.vue";
import DependencyCheck from "./components/DependencyCheck.vue";
import AboutDialog from "./components/AboutDialog.vue";
import type { Deps, ApiResponse } from "./types";
import { checkDependencies } from "./api";
import { useToast } from "./composables/useToast";

const { t } = useI18n();
const { translateError } = useToast();
const store = useMountStore();
const settings = useSettingsStore();
const showForm = ref(false);
const depCheck = ref<InstanceType<typeof DependencyCheck> | null>(null);

let pollTimer: ReturnType<typeof setInterval> | null = null;

function onSaved() {
  showForm.value = false;
}

async function checkDeps(showOnlyOnError = false) {
  try {
    const res = await checkDependencies() as ApiResponse<Deps>;
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

function startPolling() {
  if (pollTimer) return;
  pollTimer = setInterval(() => {
    store.loadMounts();
  }, 5000);
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
}

function onVisibilityChange() {
  if (document.hidden) {
    stopPolling();
  } else {
    startPolling();
  }
}

onMounted(async () => {
  await settings.initLanguage();
  await store.loadMounts();
  await checkDeps(true);

  startPolling();

  // Pause polling when window is hidden, resume when shown
  document.addEventListener("visibilitychange", onVisibilityChange);
});

onUnmounted(() => {
  stopPolling();
  document.removeEventListener("visibilitychange", onVisibilityChange);
});
</script>

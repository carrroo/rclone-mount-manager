<template>
  <div v-if="show" class="modal-overlay" @click.self="show = false">
    <div class="modal">
      <h3>Rclone Mount Manager</h3>
      <div class="about-body">
        <p class="version">v0.1.0</p>
        <p class="desc">{{ t('about.desc') }}</p>
        <p class="license">MIT License</p>
      </div>
      <div class="modal-actions">
        <button class="btn" @click="show = false">OK</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

const { t } = useI18n();
const show = ref(false);
let unlisten: UnlistenFn | null = null;

onMounted(async () => {
  unlisten = await listen('show-about', () => {
    show.value = true;
  });
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.modal {
  background: #fff;
  border-radius: 12px;
  padding: 24px;
  width: 360px;
  max-width: 90vw;
  text-align: center;
}

.modal h3 {
  margin-bottom: 12px;
  font-size: 18px;
}

.about-body {
  margin-bottom: 20px;
}

.version {
  font-size: 14px;
  color: #666;
  margin-bottom: 8px;
}

.desc {
  font-size: 14px;
  color: #333;
  margin-bottom: 8px;
}

.license {
  font-size: 13px;
  color: #999;
}

.modal-actions {
  display: flex;
  justify-content: center;
}
</style>

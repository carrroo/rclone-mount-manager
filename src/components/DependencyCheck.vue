<template>
  <div v-if="show" class="modal-overlay" @click.self="show = false">
    <div class="modal">
      <h3>{{ t('deps.title') }}</h3>

      <div class="dep-list">
        <div class="dep-item" :class="{ ok: deps?.rclone_installed }">
          <span class="dep-icon">{{ deps?.rclone_installed ? '✓' : '✗' }}</span>
          <div class="dep-info">
            <span class="dep-name">rclone</span>
            <span class="dep-status">{{ deps?.rclone_installed ? (deps?.rclone_version || t('deps.installed')) : t('deps.notInstalled') }}</span>
          </div>
        </div>

        <div class="dep-item" :class="{ ok: deps?.macfuse_installed }">
          <span class="dep-icon">{{ deps?.macfuse_installed ? '✓' : '✗' }}</span>
          <div class="dep-info">
            <span class="dep-name">macFUSE</span>
            <span class="dep-status">{{ deps?.macfuse_installed ? t('deps.installed') : t('deps.notInstalled') }}</span>
          </div>
        </div>
      </div>

      <div v-if="!allOk" class="dep-hint">
        <p>{{ t('deps.missing') }}</p>
        <ul>
          <li v-if="!deps?.rclone_installed">
            <strong>rclone</strong> — {{ t('deps.installCmd') }}<code>brew install rclone</code>
          </li>
          <li v-if="!deps?.macfuse_installed">
            <strong>macFUSE</strong> — {{ t('deps.installCmd') }}<code>brew install macfuse</code>
            <br><small>{{ t('deps.restartHint') }}</small>
          </li>
        </ul>
      </div>

      <div class="modal-actions">
        <button class="btn" @click="show = false">{{ t('deps.ok') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Deps } from '../types';

const { t } = useI18n();

const show = ref(false);
const deps = ref<Deps | null>(null);

const allOk = computed(() => {
  return deps.value?.rclone_installed && deps.value?.macfuse_installed;
});

function open(d: Deps) {
  deps.value = d;
  show.value = true;
}

defineExpose({ open });
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
  width: 420px;
  max-width: 90vw;
}

.modal h3 {
  margin-bottom: 16px;
  font-size: 18px;
}

.dep-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 16px;
}

.dep-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 8px;
  background: #ffebee;
}

.dep-item.ok {
  background: #e8f5e9;
}

.dep-icon {
  font-size: 20px;
  font-weight: bold;
  width: 28px;
  text-align: center;
}

.dep-item.ok .dep-icon {
  color: #2e7d32;
}

.dep-item:not(.ok) .dep-icon {
  color: #c62828;
}

.dep-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.dep-name {
  font-weight: 600;
  font-size: 15px;
}

.dep-status {
  font-size: 13px;
  color: #666;
}

.dep-hint {
  background: #fff3e0;
  padding: 14px;
  border-radius: 8px;
  margin-bottom: 16px;
  font-size: 14px;
}

.dep-hint p {
  margin-bottom: 8px;
  font-weight: 500;
}

.dep-hint ul {
  margin: 0;
  padding-left: 20px;
}

.dep-hint li {
  margin-bottom: 8px;
}

.dep-hint code {
  background: #f5f5f5;
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'SF Mono', monospace;
  font-size: 13px;
}

.dep-hint small {
  color: #888;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
}
</style>

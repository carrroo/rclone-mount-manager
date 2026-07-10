<template>
  <div class="mount-list">
    <div v-if="store.items.length === 0" class="empty">
      <p>{{ t('empty.noRemote') }}</p>
      <p class="hint">{{ t('empty.hint') }}</p>
    </div>

    <div v-else class="cards">
      <div
        v-for="(item, index) in store.items"
        :key="item.id"
        class="card"
        :class="{ mounted: item.mounted }"
      >
        <div class="card-header">
          <div class="card-title">
            <span class="indicator" :class="{ active: item.mounted }"></span>
            <span>{{ item.name }}</span>
          </div>
          <div class="card-header-right">
            <div class="move-buttons">
              <button class="btn-icon" @click="store.moveUp(index)" :disabled="index === 0" title="上移">↑</button>
              <button class="btn-icon" @click="store.moveDown(index)" :disabled="index === store.items.length - 1" title="下移">↓</button>
            </div>
            <span class="badge" :class="{ active: item.mounted }">
              {{ item.mounted ? t('mount.mounted') : t('mount.unmounted') }}
            </span>
          </div>
        </div>

        <div class="card-body">
          <div class="info-row">
            <label>{{ t('config.type') }}</label>
            <template v-if="editingId === item.id && !item.mounted">
              <select v-model="editForm.config_type" class="edit-select">
                <option value="sftp">{{ t('form.typeSftp') }}</option>
                <option value="webdav">{{ t('form.typeWebdav') }}</option>
                <option value="http">{{ t('form.typeHttp') }}</option>
                <option value="ftp">{{ t('form.typeFtp') }}</option>
              </select>
            </template>
            <template v-else>
              <code>{{ item.config_type }}</code>
            </template>
          </div>

          <div class="info-row">
            <label>{{ t('form.remoteDir') }}</label>
            <template v-if="editingId === item.id && !item.mounted">
              <div class="path-input">
                <span class="path-prefix">{{ item.name }}:</span>
                <input v-model="editForm.remote_dir" class="edit-input" />
              </div>
            </template>
            <template v-else>
              <code>{{ item.name }}:{{ getRemoteDir(item.remote_path, item.name) }}</code>
            </template>
          </div>

          <div class="info-row">
            <label>{{ t('config.mountPoint') }}</label>
            <template v-if="editingId === item.id && !item.mounted">
              <input v-model="editForm.mount_point" class="edit-input" />
            </template>
            <template v-else>
              <code>{{ item.mount_point }}</code>
            </template>
          </div>

          <template v-if="editingId === item.id && !item.mounted">
            <div class="edit-row">
              <div class="info-row">
                <label>{{ t('config.host') }}</label>
                <input v-model="editForm.host" class="edit-input" :placeholder="t('form.hostPlaceholder')" />
              </div>
              <div class="info-row">
                <label>{{ t('config.port') }}</label>
                <input v-model="editForm.port" class="edit-input" :placeholder="t('form.portPlaceholder')" />
              </div>
            </div>
            <div class="edit-row">
              <div class="info-row">
                <label>{{ t('config.user') }}</label>
                <input v-model="editForm.user" class="edit-input" :placeholder="t('form.userPlaceholder')" />
              </div>
              <div class="info-row">
                <label>{{ t('config.password') }}</label>
                <input v-model="editForm.pass" type="password" class="edit-input" :placeholder="t('form.passwordPlaceholder')" />
              </div>
            </div>
          </template>

          <div class="info-row tags">
            <span class="tag tag-config">{{ t('tag.config') }}</span>
          </div>
        </div>

        <div class="card-footer">
          <button
            v-if="!item.mounted"
            class="btn btn-success"
            :disabled="store.isPending(item.id)"
            @click="handleMount(item)"
          >
            {{ store.isPending(item.id) ? '…' : t('mount.mount') }}
          </button>
          <button
            v-else
            class="btn btn-danger"
            :disabled="store.isPending(item.id)"
            @click="store.doUnmount(item)"
          >
            {{ store.isPending(item.id) ? '…' : t('mount.unmount') }}
          </button>

          <template v-if="!item.mounted">
            <button
              v-if="editingId !== item.id"
              class="btn"
              @click="startEdit(item)"
            >
              {{ t('mount.edit') }}
            </button>
            <template v-else>
              <button class="btn btn-primary" @click="saveEdit(item)">{{ t('mount.save') }}</button>
              <button class="btn" @click="cancelEdit">{{ t('mount.cancel') }}</button>
            </template>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useMountStore } from '../stores/mounts';
import type { MountItem } from '../types';

const { t } = useI18n();
const store = useMountStore();
const editingId = ref<string | null>(null);
const editForm = ref({ config_type: 'sftp', remote_dir: '/', mount_point: '', host: '', user: '', pass: '', port: '' });

/** Extract the directory part from a full rclone path like "WUXI:/dir" → "/dir" */
function getRemoteDir(remotePath: string, name: string): string {
  const prefix = name + ':';
  if (remotePath.startsWith(prefix)) {
    return remotePath.slice(prefix.length);
  }
  return remotePath;
}

function startEdit(item: MountItem) {
  editingId.value = item.id;
  editForm.value = {
    config_type: item.config_type,
    remote_dir: getRemoteDir(item.remote_path, item.name),
    mount_point: item.mount_point,
    host: item.host,
    user: item.user,
    pass: item.pass,
    port: item.port,
  };
}

function saveEdit(item: MountItem) {
  const dir = editForm.value.remote_dir.startsWith('/') ? editForm.value.remote_dir : '/' + editForm.value.remote_dir;
  const fullRemotePath = `${item.name}:${dir}`;
  store.updateMountConfig(item, editForm.value.config_type, fullRemotePath, editForm.value.mount_point, editForm.value.host, editForm.value.user, editForm.value.pass, editForm.value.port);
  editingId.value = null;
}

async function handleMount(item: MountItem) {
  if (editingId.value === item.id) {
    const dir = editForm.value.remote_dir.startsWith('/') ? editForm.value.remote_dir : '/' + editForm.value.remote_dir;
    const fullRemotePath = `${item.name}:${dir}`;
    await store.updateMountConfig(item, editForm.value.config_type, fullRemotePath, editForm.value.mount_point, editForm.value.host, editForm.value.user, editForm.value.pass, editForm.value.port);
    editingId.value = null;
  }
  await store.doMount(item);
}

function cancelEdit() {
  editingId.value = null;
}
</script>

<style scoped>
.mount-list {
  max-width: 800px;
  margin: 0 auto;
}

.empty {
  text-align: center;
  padding: 80px 24px;
  color: #999;
}

.empty p:first-child {
  font-size: 18px;
  margin-bottom: 8px;
}

.hint {
  font-size: 14px;
}

.cards {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card {
  background: #fff;
  border-radius: 12px;
  border: 1px solid #e0e0e0;
  padding: 20px;
  transition: all 0.2s;
}

.card.mounted {
  border-color: #28a745;
  box-shadow: 0 0 0 1px #28a745;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
}

.indicator {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #ccc;
}

.indicator.active {
  background: #28a745;
}

.card-header-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.move-buttons {
  display: flex;
  gap: 2px;
}

.btn-icon {
  background: #f0f0f0;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 2px 8px;
  cursor: pointer;
  font-size: 14px;
  color: #555;
}

.btn-icon:hover:not(:disabled) {
  background: #e0e0e0;
}

.btn-icon:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.badge {
  padding: 2px 10px;
  border-radius: 10px;
  font-size: 12px;
  background: #e8e8e8;
  color: #666;
}

.badge.active {
  background: #d4edda;
  color: #155724;
}

.card-body {
  margin-bottom: 16px;
}

.info-row {
  margin-bottom: 10px;
}

.info-row label {
  font-size: 12px;
  color: #999;
  margin-bottom: 2px;
}

.info-row code {
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 13px;
  color: #555;
  background: #f5f5f5;
  padding: 4px 8px;
  border-radius: 4px;
  display: block;
}

.edit-input {
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 13px;
  padding: 6px 10px;
  border: 1px solid #007bff;
  border-radius: 4px;
  width: 100%;
  background: #f8fbff;
}

.edit-select {
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 13px;
  padding: 6px 10px;
  border: 1px solid #007bff;
  border-radius: 4px;
  width: 100%;
  background: #f8fbff;
}

.path-input {
  display: flex;
  align-items: center;
  border: 1px solid #007bff;
  border-radius: 4px;
  overflow: hidden;
}

.path-prefix {
  background: #e8f0fe;
  padding: 6px 10px;
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 13px;
  color: #1976d2;
  border-right: 1px solid #007bff;
  white-space: nowrap;
}

.path-input .edit-input {
  border: none;
  border-radius: 0;
}

.tags {
  display: flex;
  gap: 8px;
}

.tag {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
}

.tag-config {
  background: #e3f2fd;
  color: #1976d2;
}

.edit-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.card-footer {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
</style>

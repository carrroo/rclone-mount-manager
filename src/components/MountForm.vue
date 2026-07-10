<template>
  <div class="form-container">
    <h2>{{ t('form.addCustom') }}</h2>

    <div class="form-group">
      <label>{{ t('form.name') }}</label>
      <input v-model="form.name" :placeholder="t('form.namePlaceholder')" />
    </div>

    <div class="form-row">
      <div class="form-group">
        <label>{{ t('config.type') }}</label>
        <select v-model="form.config_type">
          <option value="sftp">{{ t('form.typeSftp') }}</option>
          <option value="webdav">{{ t('form.typeWebdav') }}</option>
          <option value="http">{{ t('form.typeHttp') }}</option>
          <option value="ftp">{{ t('form.typeFtp') }}</option>
        </select>
      </div>
      <div class="form-group">
        <label>{{ t('form.selectRemote') }}</label>
        <select v-model="selectedRemote">
          <option value="">{{ t('form.selectRemotePlaceholder') }}</option>
          <option v-for="remote in configRemotes" :key="remote.name" :value="remote.name">
            {{ remote.name }} ({{ remote.config_type }})
          </option>
        </select>
      </div>
    </div>

    <div class="form-group">
      <label>{{ t('form.remoteDir') }}</label>
      <div class="path-input">
        <span class="path-prefix" v-if="selectedRemote">{{ selectedRemote }}:</span>
        <input v-model="remoteDir" :placeholder="t('form.remoteDirPlaceholder')" />
      </div>
    </div>

    <div class="form-group">
      <label>{{ t('config.mountPoint') }}</label>
      <input v-model="form.mount_point" :placeholder="t('form.mountPointPlaceholder')" />
    </div>

    <div class="form-row">
      <div class="form-group">
        <label>{{ t('config.host') }}</label>
        <input v-model="form.host" :placeholder="t('form.hostPlaceholder')" />
      </div>
      <div class="form-group">
        <label>{{ t('config.port') }}</label>
        <input v-model="form.port" :placeholder="t('form.portPlaceholder')" />
      </div>
    </div>

    <div class="form-row">
      <div class="form-group">
        <label>{{ t('config.user') }}</label>
        <input v-model="form.user" :placeholder="t('form.userPlaceholder')" />
      </div>
      <div class="form-group">
        <label>{{ t('config.password') }}</label>
        <input v-model="form.pass" type="password" :placeholder="t('form.passwordPlaceholder')" />
      </div>
    </div>

    <div class="form-group">
      <label>{{ t('config.extraArgs') }}</label>
      <input v-model="extraArgsInput" :placeholder="t('form.extraArgsPlaceholder')" />
    </div>

    <div class="form-actions">
      <button class="btn" @click="$emit('close')">{{ t('mount.cancel') }}</button>
      <button class="btn btn-primary" @click="save" :disabled="!isValid">
        {{ t('mount.save') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useMountStore } from '../stores/mounts';

const { t } = useI18n();
const store = useMountStore();
const emit = defineEmits(['close', 'saved']);

const form = ref({
  name: '',
  mount_point: '',
  config_type: 'sftp',
  host: '',
  user: '',
  pass: '',
  port: '',
});

const selectedRemote = ref('');
const remoteDir = ref('/');
const extraArgsInput = ref('');
const saving = ref(false);

const configRemotes = computed(() => store.items);

const fullRemotePath = computed(() => {
  const remote = selectedRemote.value || form.value.name;
  const dir = remoteDir.value.startsWith('/') ? remoteDir.value : '/' + remoteDir.value;
  return `${remote}:${dir}`;
});

const isValid = computed(() => {
  return form.value.name.trim() && remoteDir.value.trim() && form.value.mount_point.trim();
});

watch(selectedRemote, (val) => {
  if (val) {
    const remote = configRemotes.value.find((r) => r.name === val);
    if (remote) {
      if (!form.value.name) {
        form.value.name = val;
      }
      if (!form.value.mount_point) {
        form.value.mount_point = '/Volumes/' + val;
      }
      if (!form.value.host && remote.host) {
        form.value.host = remote.host;
      }
      if (!form.value.user && remote.user) {
        form.value.user = remote.user;
      }
      if (!form.value.port && remote.port) {
        form.value.port = remote.port;
      }
      form.value.config_type = remote.config_type;
    }
  }
});

async function save() {
  if (!isValid.value || saving.value) return;
  saving.value = true;

  const options: Record<string, string> = {};
  if (form.value.host) options.host = form.value.host;
  if (form.value.user) options.user = form.value.user;
  if (form.value.pass) options.pass = form.value.pass;
  if (form.value.port) options.port = form.value.port;

  const extraArgs = extraArgsInput.value.trim().split(/\s+/).filter(Boolean);

  await store.addAndMount(
    form.value.name,
    form.value.config_type,
    fullRemotePath.value,
    form.value.mount_point,
    options,
    extraArgs
  );

  saving.value = false;
  emit('saved');
}
</script>

<style scoped>
.form-container {
  max-width: 600px;
  margin: 0 auto;
  background: #fff;
  border-radius: 12px;
  padding: 32px;
}

.form-container h2 {
  margin-bottom: 24px;
  font-size: 20px;
}

.path-input {
  display: flex;
  align-items: center;
  border: 1px solid #ccc;
  border-radius: 4px;
  overflow: hidden;
}

.path-prefix {
  background: #f0f0f0;
  padding: 8px 12px;
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 13px;
  color: #555;
  border-right: 1px solid #ccc;
  white-space: nowrap;
}

.path-input input {
  flex: 1;
  border: none;
  padding: 8px 12px;
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 13px;
  outline: none;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}
</style>

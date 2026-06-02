<template>
  <div class="form-container">
    <h2>{{ t('form.addCustom') }}</h2>

    <div class="form-group">
      <label>{{ t('form.name') }}</label>
      <input v-model="form.name" :placeholder="t('form.namePlaceholder')" />
    </div>

    <div class="form-row">
      <div class="form-group">
        <label>{{ t('form.selectRemote') }}</label>
        <select v-model="selectedRemote">
          <option value="">{{ t('form.selectRemotePlaceholder') }}</option>
          <option v-for="remote in configRemotes" :key="remote.name" :value="remote.name">
            {{ remote.name }} ({{ remote.config_type }})
          </option>
        </select>
      </div>
      <div class="form-group">
        <label>{{ t('config.remotePath') }}</label>
        <input v-model="form.remote_path" :placeholder="t('form.remotePlaceholder')" />
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
  remote_path: '',
  mount_point: '',
  extra_args: [] as string[],
  source: 'custom',
  config_type: 'custom',
  host: '',
  user: '',
  pass: '',
  port: '',
});

const selectedRemote = ref('');
const extraArgsInput = ref('');

const configRemotes = computed(() => {
  return store.items.filter((item) => item.source === 'config');
});

const isValid = computed(() => {
  return form.value.name.trim() && form.value.remote_path.trim() && form.value.mount_point.trim();
});

watch(selectedRemote, (val) => {
  if (val) {
    const remote = configRemotes.value.find((r) => r.name === val);
    if (remote) {
      if (!form.value.remote_path || form.value.remote_path === selectedRemote.value + ':/') {
        form.value.remote_path = val + ':/';
      }
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
    }
  }
});

watch(extraArgsInput, (val) => {
  form.value.extra_args = val.trim().split(/\s+/).filter(Boolean);
});

function save() {
  if (!isValid.value) return;

  store.addCustomMount({ ...form.value });
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

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}
</style>

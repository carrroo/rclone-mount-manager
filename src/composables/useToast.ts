import { ref } from "vue";
import { useI18n } from "vue-i18n";

/**
 * Composable for toast notification state and error translation.
 * Extracted from App.vue so any component can display errors.
 */
export function useToast() {
  const message = ref<string | null>(null);
  const { t, te } = useI18n();

  function show(msg: string) {
    message.value = msg;
  }

  function dismiss() {
    message.value = null;
  }

  /** Translate backend error keys like "error.mount_failed:details" via i18n. */
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

  return { message, show, dismiss, translateError };
}

import { useI18n } from "vue-i18n";

/**
 * Composable for translating backend error strings.
 *
 * Backend errors look like "error.mount_failed:details" — the part before
 * the first colon is an i18n key, the rest is passed as the {msg} param.
 */
export function useToast() {
  const { t, te } = useI18n();

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

  return { translateError };
}

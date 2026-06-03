/**
 * i18n setup — creates the vue-i18n instance with system locale detection.
 *
 * Language persistence is managed by the Rust backend; the frontend
 * syncs via the settings store on startup. We default to the system
 * locale here and correct once the backend responds.
 */
import { createI18n } from 'vue-i18n';
import zh from './zh';
import en from './en';

/** Detect the system locale and map to our supported languages. */
function detectSystemLocale(): string {
  const lang = navigator.language || 'en';
  return lang.startsWith('zh') ? 'zh' : 'en';
}

/**
 * Initialize i18n with a given locale.
 * Used by settings store after reading language from the Rust backend.
 */
function createI18nWithLocale(locale: string) {
  return createI18n({
    legacy: false,
    locale,
    fallbackLocale: 'en',
    messages: { zh, en },
  });
}

// Start with system locale as default — will be corrected once Rust backend responds
const i18n = createI18nWithLocale(detectSystemLocale());

export { detectSystemLocale, createI18nWithLocale };
export default i18n;

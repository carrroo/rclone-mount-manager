import { createI18n } from 'vue-i18n';
import zh from './zh';
import en from './en';

function detectSystemLocale(): string {
  const lang = navigator.language || 'en';
  return lang.startsWith('zh') ? 'zh' : 'en';
}

const savedLanguage = localStorage.getItem('rclone-language');
const isSystemLang = !savedLanguage || savedLanguage === 'system';
const locale = isSystemLang ? detectSystemLocale() : savedLanguage;

const i18n = createI18n({
  legacy: false,
  locale,
  fallbackLocale: 'en',
  messages: { zh, en },
});

export { detectSystemLocale };
export default i18n;

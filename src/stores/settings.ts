/**
 * Settings store — manages language preference.
 *
 * The Rust backend is the single source of truth for the persisted
 * language setting. On init we read from Rust; language changes via
 * the native macOS menu are pushed to us via the `language-changed`
 * Tauri event.
 */
import { defineStore } from "pinia";
import { ref, computed, watch } from "vue";
import { listen } from "@tauri-apps/api/event";
import i18n, { detectSystemLocale } from "../locales";
import { getLanguage, setLanguage as apiSetLanguage } from "../api";

export type LanguageOption = "zh" | "en" | "system";

export const useSettingsStore = defineStore("settings", () => {
  const language = ref<LanguageOption>("system");
  const resolvedLocale = computed(() => {
    if (language.value === "system") {
      return detectSystemLocale();
    }
    return language.value;
  });

  /** Load language from backend on startup, then listen for native menu changes. */
  async function initLanguage() {
    try {
      const lang = await getLanguage();
      language.value = lang as LanguageOption;
    } catch {
      language.value = "system";
    }

    // Listen for language changes from native menu
    await listen<string>("language-changed", (event) => {
      const locale = event.payload;
      if (locale === "zh" || locale === "en") {
        language.value = locale;
      }
    });
  }

  /** Set language locally and persist to the Rust backend. */
  async function setLanguage(lang: LanguageOption) {
    language.value = lang;
    try {
      await apiSetLanguage(lang);
    } catch {
      // ignore
    }
  }

  watch(resolvedLocale, (val) => {
    i18n.global.locale.value = val as "zh" | "en";
  }, { immediate: true });

  return {
    language,
    resolvedLocale,
    initLanguage,
    setLanguage,
  };
});

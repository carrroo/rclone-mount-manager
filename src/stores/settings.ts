import { defineStore } from "pinia";
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import i18n, { detectSystemLocale } from "../locales";

export type LanguageOption = "zh" | "en" | "system";

export const useSettingsStore = defineStore("settings", () => {
  const language = ref<LanguageOption>("system");
  const resolvedLocale = computed(() => {
    if (language.value === "system") {
      return detectSystemLocale();
    }
    return language.value;
  });

  async function initLanguage() {
    try {
      const lang = await invoke<string>("get_language");
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

  async function setLanguage(lang: LanguageOption) {
    language.value = lang;
    try {
      await invoke("set_language", { lang });
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

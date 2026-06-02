import { ref, computed } from "vue";
import { zh } from "../locales/zh";
import { en } from "../locales/en";

export type Locale = "zh" | "en";

const localeStorageKey = "skillmaster-locale";

function getInitialLocale(): Locale {
  if (typeof localStorage === "undefined") return "zh";
  const stored = localStorage.getItem(localeStorageKey);
  if (stored === "zh" || stored === "en") return stored;
  
  if (typeof navigator !== "undefined") {
    return navigator.language.toLowerCase().startsWith("en") ? "en" : "zh";
  }
  return "zh";
}

const currentLocale = ref<Locale>(getInitialLocale());

const messages = { zh, en };

export function useI18n() {
  const locale = computed({
    get: () => currentLocale.value,
    set: (val: Locale) => {
      currentLocale.value = val;
      localStorage.setItem(localeStorageKey, val);
    }
  });

  function t(keyPath: string, params?: Record<string, string | number | undefined>): string {
    const dict = messages[currentLocale.value];
    const keys = keyPath.split(".");
    
    let current: any = dict;
    for (const key of keys) {
      if (current && typeof current === "object" && key in current) {
        current = current[key];
      } else {
        return keyPath;
      }
    }

    if (typeof current !== "string") {
      return keyPath;
    }

    let text = current;
    if (params) {
      Object.entries(params).forEach(([pKey, pVal]) => {
        text = text.replace(new RegExp(`{${pKey}}`, "g"), String(pVal ?? ""));
      });
    }
    return text;
  }

  return {
    locale,
    t
  };
}
export type UseI18nReturn = ReturnType<typeof useI18n>;

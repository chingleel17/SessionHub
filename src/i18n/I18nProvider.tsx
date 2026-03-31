import { createContext, useContext, useState, type PropsWithChildren } from "react";

import { zhTwMessages, type MessageKey } from "../locales/zh-TW";
import { enUsMessages } from "../locales/en-US";

export type Locale = "zh-TW" | "en-US";

type I18nContextValue = {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: MessageKey) => string;
};

const I18nContext = createContext<I18nContextValue | null>(null);

const STORAGE_KEY = "app-locale";

const messages: Record<Locale, Record<MessageKey, string>> = {
  "zh-TW": zhTwMessages,
  "en-US": enUsMessages,
};

function getInitialLocale(): Locale {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "zh-TW" || stored === "en-US") return stored;
  return navigator.language.startsWith("zh") ? "zh-TW" : "en-US";
}

export function I18nProvider({ children }: PropsWithChildren) {
  const [locale, setLocaleState] = useState<Locale>(getInitialLocale);

  const setLocale = (next: Locale) => {
    setLocaleState(next);
    localStorage.setItem(STORAGE_KEY, next);
  };

  const value: I18nContextValue = {
    locale,
    setLocale,
    t: (key) => messages[locale][key],
  };

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const context = useContext(I18nContext);

  if (!context) {
    throw new Error("useI18n must be used within I18nProvider");
  }

  return context;
}

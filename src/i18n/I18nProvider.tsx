import { createContext, useContext, type PropsWithChildren } from "react";

import { zhTwMessages, type MessageKey } from "../locales/zh-TW";

type Locale = "zh-TW";

type I18nContextValue = {
  locale: Locale;
  t: (key: MessageKey) => string;
};

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({ children }: PropsWithChildren) {
  const value: I18nContextValue = {
    locale: "zh-TW",
    t: (key) => zhTwMessages[key],
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

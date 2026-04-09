import { create } from 'zustand';
import i18n from '@/i18n';

export type Language = 'en' | 'zh' | 'ja';

interface I18nState {
  language: Language;
  setLanguage: (language: Language) => void;
  t: (key: string, options?: any) => string;
  changeLanguage: (language: Language) => Promise<void>;
}

export const useI18nStore = create<I18nState>((set, get) => ({
  language: 'en',
  setLanguage: (language) => set({ language }),
  t: (key, options) => {
    const result = i18n.t(key, options);
    return typeof result === 'string' ? result : String(result);
  },
  changeLanguage: async (language) => {
    await i18n.changeLanguage(language);
    set({ language });
  },
}));
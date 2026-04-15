import { useEffect } from 'react';
import { I18nextProvider } from 'react-i18next';
import { useI18nStore } from '@/store/i18n';
import { useSettingsStore } from '@/store/settings';
import i18n from '@/i18n';

interface I18nProviderProps {
  children: React.ReactNode;
}

export function I18nProvider({ children }: I18nProviderProps) {
  const language = useI18nStore((state) => state.language);
  const config = useSettingsStore((state) => state.config);
  const refreshSettings = useSettingsStore((state) => state.refresh);

  useEffect(() => {
    if (!config) {
      void refreshSettings();
    }
  }, [config, refreshSettings]);

  useEffect(() => {
    const configuredLanguage = config?.ui.language;
    if (configuredLanguage === 'en' || configuredLanguage === 'zh' || configuredLanguage === 'ja') {
      if (configuredLanguage !== language) {
        void useI18nStore.getState().changeLanguage(configuredLanguage);
      }
      return;
    }

    void useI18nStore.getState().changeLanguage(language);
  }, [config?.ui.language, language]);

  return (
    <I18nextProvider i18n={i18n}>
      {children}
    </I18nextProvider>
  );
}

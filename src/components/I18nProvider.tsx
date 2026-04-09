import { I18nextProvider } from 'react-i18next';
import { useI18nStore } from '@/store/i18n';
import { useEffect } from 'react';
import i18n from '@/i18n';

interface I18nProviderProps {
  children: React.ReactNode;
}

export function I18nProvider({ children }: I18nProviderProps) {
  const language = useI18nStore((state) => state.language);

  useEffect(() => {
    // Update i18n language when store changes
    const changeLanguage = async () => {
      const { changeLanguage: changeLanguageFn } = useI18nStore.getState();
      await changeLanguageFn(language);
    };
    changeLanguage();
  }, [language]);

  return (
    <I18nextProvider i18n={i18n}>
      {children}
    </I18nextProvider>
  );
}
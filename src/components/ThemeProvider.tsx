import { useEffect } from 'react';
import { useThemeStore, type Theme } from '@/store/theme';
import { useSettingsStore } from '@/store/settings';

interface ThemeProviderProps {
  children: React.ReactNode;
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const theme = useThemeStore((state) => state.theme);
  const setTheme = useThemeStore((state) => state.setTheme);
  const isDark = useThemeStore((state) => state.isDark);
  const config = useSettingsStore((state) => state.config);
  const refreshSettings = useSettingsStore((state) => state.refresh);

  useEffect(() => {
    if (!config) {
      void refreshSettings();
    }
  }, [config, refreshSettings]);

  useEffect(() => {
    const configuredTheme = config?.ui.theme;
    if (configuredTheme === 'light' || configuredTheme === 'dark' || configuredTheme === 'system') {
      setTheme(configuredTheme as Theme);
    }
  }, [config?.ui.theme, setTheme]);

  useEffect(() => {
    const root = window.document.documentElement;
    const dark = isDark();

    if (dark) {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [theme, isDark]);

  useEffect(() => {
    if (!config?.ui.fontSize) {
      return;
    }

    window.document.documentElement.style.fontSize = `${config.ui.fontSize}px`;
  }, [config?.ui.fontSize]);

  return <>{children}</>;
}

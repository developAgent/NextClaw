import { useEffect } from 'react';
import { useThemeStore } from '@/store/theme';

interface ThemeProviderProps {
  children: React.ReactNode;
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const { theme, isDark } = useThemeStore();

  useEffect(() => {
    const root = window.document.documentElement;
    const dark = isDark();

    if (dark) {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [isDark]);

  return <>{children}</>;
}
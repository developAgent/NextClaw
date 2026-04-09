import { create } from 'zustand';

export type Theme = 'light' | 'dark' | 'system';

interface ThemeState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
  isDark: () => boolean;
}

export const useThemeStore = create<ThemeState>((set, get) => ({
  theme: 'system',
  setTheme: (theme) => set({ theme }),
  toggleTheme: () => {
    const current = get().theme;
    let next: Theme;
    if (current === 'light') {
      next = 'dark';
    } else if (current === 'dark') {
      next = 'system';
    } else {
      next = 'light';
    }
    set({ theme: next });
  },
  isDark: () => {
    const theme = get().theme;
    if (theme !== 'system') {
      return theme === 'dark';
    }
    // Check system preference
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  },
}));
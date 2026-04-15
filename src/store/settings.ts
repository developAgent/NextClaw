import { create } from 'zustand';
import type { Config, ConfigUpdate } from '@/types';
import { deleteApiKey, getSettingsConfig, setApiKey, updateSettingsConfig } from '@/lib/settings';

interface SettingsStore {
  config: Config | null;
  loading: boolean;
  saving: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  saveConfig: (update: ConfigUpdate) => Promise<void>;
  saveApiKey: (apiKey: string) => Promise<void>;
  removeApiKey: () => Promise<void>;
}

function getErrorMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }

  if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') {
    return error.message;
  }

  return 'Unknown error';
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  config: null,
  loading: false,
  saving: false,
  error: null,
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const config = await getSettingsConfig();
      set({ config, loading: false });
    } catch (error) {
      set({ error: getErrorMessage(error), loading: false });
    }
  },
  saveConfig: async (update) => {
    set({ saving: true, error: null });
    try {
      await updateSettingsConfig(update);
      const config = await getSettingsConfig();
      set({ config, saving: false });
    } catch (error) {
      set({ error: getErrorMessage(error), saving: false });
    }
  },
  saveApiKey: async (apiKey) => {
    set({ saving: true, error: null });
    try {
      await setApiKey(apiKey);
      const config = await getSettingsConfig();
      set({ config, saving: false });
    } catch (error) {
      set({ error: getErrorMessage(error), saving: false });
    }
  },
  removeApiKey: async () => {
    set({ saving: true, error: null });
    try {
      await deleteApiKey();
      const config = await getSettingsConfig();
      set({ config, saving: false });
    } catch (error) {
      set({ error: getErrorMessage(error), saving: false });
    }
  },
}));

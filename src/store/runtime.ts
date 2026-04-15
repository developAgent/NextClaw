import { create } from 'zustand';
import type { RuntimeConfig, RuntimeStatus } from '@/types';
import {
  generateRuntimeToken,
  getRuntimeConfig,
  getRuntimeStatus,
  restartRuntime,
  startRuntime,
  stopRuntime,
  updateRuntimeConfig,
} from '@/lib/runtime';

interface RuntimeStore {
  status: RuntimeStatus | null;
  config: RuntimeConfig | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  start: () => Promise<void>;
  stop: () => Promise<void>;
  restart: () => Promise<void>;
  saveConfig: (config: RuntimeConfig) => Promise<void>;
  generateToken: () => Promise<void>;
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

export const useRuntimeStore = create<RuntimeStore>((set, get) => ({
  status: null,
  config: null,
  loading: false,
  error: null,
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const [status, config] = await Promise.all([getRuntimeStatus(), getRuntimeConfig()]);
      set({ status, config, loading: false });
    } catch (error) {
      set({ error: getErrorMessage(error), loading: false });
    }
  },
  start: async () => {
    set({ loading: true, error: null });
    try {
      await startRuntime();
      const [status, config] = await Promise.all([getRuntimeStatus(), getRuntimeConfig()]);
      set({ status, config, loading: false });
    } catch (error) {
      set({ error: getErrorMessage(error), loading: false });
    }
  },
  stop: async () => {
    set({ loading: true, error: null });
    try {
      await stopRuntime();
      const [status, config] = await Promise.all([getRuntimeStatus(), getRuntimeConfig()]);
      set({ status, config, loading: false });
    } catch (error) {
      set({ error: getErrorMessage(error), loading: false });
    }
  },
  restart: async () => {
    set({ loading: true, error: null });
    try {
      await restartRuntime();
      const [status, config] = await Promise.all([getRuntimeStatus(), getRuntimeConfig()]);
      set({ status, config, loading: false });
    } catch (error) {
      set({ error: getErrorMessage(error), loading: false });
    }
  },
  saveConfig: async (config) => {
    set({ loading: true, error: null });
    try {
      await updateRuntimeConfig(config);
      const [status, freshConfig] = await Promise.all([getRuntimeStatus(), getRuntimeConfig()]);
      set({ status, config: freshConfig, loading: false });
    } catch (error) {
      set({ error: getErrorMessage(error), loading: false });
    }
  },
  generateToken: async () => {
    set({ loading: true, error: null });
    try {
      const currentConfig = get().config ?? (await getRuntimeConfig());
      const token = await generateRuntimeToken();
      await updateRuntimeConfig({
        ...currentConfig,
        token,
      });
      const [status, config] = await Promise.all([getRuntimeStatus(), getRuntimeConfig()]);
      set({ status, config, loading: false });
    } catch (error) {
      set({ error: getErrorMessage(error), loading: false });
    }
  },
}));

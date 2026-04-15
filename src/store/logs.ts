import { create } from 'zustand';
import type { LogEntry } from '@/types';
import { listLogs } from '@/lib/logs';

interface LogStore {
  logs: LogEntry[];
  loading: boolean;
  error: string | null;
  refresh: (level?: string) => Promise<void>;
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

export const useLogStore = create<LogStore>((set) => ({
  logs: [],
  loading: false,
  error: null,
  refresh: async (level?: string) => {
    set({ loading: true, error: null });
    try {
      const logs = await listLogs({ level });
      set({ logs, loading: false });
    } catch (error) {
      set({ error: getErrorMessage(error), loading: false });
    }
  },
}));

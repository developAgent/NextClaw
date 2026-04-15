import { invoke } from '@tauri-apps/api/core';
import type { LogEntry } from '@/types';

export interface ListLogsParams {
  level?: string;
  limit?: number;
  offset?: number;
}

export async function listLogs(params: ListLogsParams = {}): Promise<LogEntry[]> {
  const { level, limit = 100, offset = 0 } = params;
  return invoke<LogEntry[]>('get_app_logs', {
    level: level?.trim() ? level : null,
    limit,
    offset,
  });
}

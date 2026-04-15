import { invoke } from '@tauri-apps/api/core';
import type { Config, ConfigUpdate } from '@/types';

export async function getSettingsConfig(): Promise<Config> {
  const raw = await invoke<string>('get_config');
  return JSON.parse(raw) as Config;
}

export async function updateSettingsConfig(update: ConfigUpdate): Promise<void> {
  await invoke('update_config', {
    configUpdate: JSON.stringify(update),
  });
}

export async function setApiKey(apiKey: string): Promise<void> {
  await invoke('set_api_key', { apiKey });
}

export async function deleteApiKey(): Promise<void> {
  await invoke('delete_api_key');
}

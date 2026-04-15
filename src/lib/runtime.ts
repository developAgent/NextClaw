import { invoke } from '@tauri-apps/api/core';
import type { RuntimeConfig, RuntimeState, RuntimeStatus } from '@/types';

interface GatewayStatusResponse {
  state: string;
  port: number;
  pid?: number;
  started_at?: string;
}

interface GatewayConfigResponse {
  auto_start: boolean;
  token?: string;
  port: number;
  proxy_enabled: boolean;
  proxy_server?: string;
  proxy_http_server?: string;
  proxy_https_server?: string;
  proxy_all_server?: string;
  proxy_bypass_rules?: string;
}

function normalizeState(state: string): RuntimeState {
  switch (state.toLowerCase()) {
    case 'starting':
      return 'starting';
    case 'running':
      return 'running';
    case 'stopping':
      return 'stopping';
    case 'error':
      return 'error';
    default:
      return 'stopped';
  }
}

function normalizeConfig(config: GatewayConfigResponse): RuntimeConfig {
  return {
    autoStart: config.auto_start,
    token: config.token,
    port: config.port,
    proxyEnabled: config.proxy_enabled,
    proxyServer: config.proxy_server,
    proxyHttpServer: config.proxy_http_server,
    proxyHttpsServer: config.proxy_https_server,
    proxyAllServer: config.proxy_all_server,
    proxyBypassRules: config.proxy_bypass_rules,
  };
}

export async function getRuntimeStatus(): Promise<RuntimeStatus> {
  const [status, healthy, controlUrl] = await Promise.all([
    invoke<GatewayStatusResponse>('get_gateway_status'),
    invoke<boolean>('check_gateway_health').catch(() => false),
    invoke<string>('get_control_ui_url').catch(() => undefined),
  ]);

  return {
    state: normalizeState(status.state),
    port: status.port,
    pid: status.pid,
    startedAt: status.started_at,
    healthy,
    controlUrl,
  };
}

export async function getRuntimeConfig(): Promise<RuntimeConfig> {
  const config = await invoke<GatewayConfigResponse>('get_gateway_config');
  return normalizeConfig(config);
}

export async function updateRuntimeConfig(config: RuntimeConfig): Promise<void> {
  await invoke('update_gateway_config', {
    config: {
      auto_start: config.autoStart,
      token: config.token,
      port: config.port,
      proxy_enabled: config.proxyEnabled,
      proxy_server: config.proxyServer,
      proxy_http_server: config.proxyHttpServer,
      proxy_https_server: config.proxyHttpsServer,
      proxy_all_server: config.proxyAllServer,
      proxy_bypass_rules: config.proxyBypassRules,
    },
  });
}

export async function generateRuntimeToken(): Promise<string> {
  return invoke<string>('generate_new_token');
}

export async function startRuntime(): Promise<void> {
  await invoke('start_gateway');
}

export async function stopRuntime(): Promise<void> {
  await invoke('stop_gateway');
}

export async function restartRuntime(): Promise<void> {
  await invoke('restart_gateway');
}

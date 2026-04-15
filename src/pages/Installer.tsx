import { useEffect, useMemo, useState } from 'react';
import {
  AlertCircle,
  ExternalLink,
  HardDriveDownload,
  HeartPulse,
  Loader2,
  Play,
  RefreshCw,
  RotateCcw,
  Save,
  ShieldCheck,
  SquareTerminal,
} from 'lucide-react';
import { useRuntimeStore } from '@/store/runtime';
import type { RuntimeConfig } from '@/types';

function formatStartedAt(value?: string): string {
  if (!value) {
    return 'Not started';
  }

  return new Date(value).toLocaleString();
}

function getStatusTone(state: string): string {
  switch (state) {
    case 'running':
      return 'border-green-500/30 bg-green-500/10 text-green-300';
    case 'starting':
    case 'stopping':
      return 'border-yellow-500/30 bg-yellow-500/10 text-yellow-300';
    case 'error':
      return 'border-red-500/30 bg-red-500/10 text-red-300';
    default:
      return 'border-zinc-700 bg-zinc-900 text-zinc-300';
  }
}

function defaultConfig(): RuntimeConfig {
  return {
    autoStart: true,
    token: '',
    port: 18789,
    proxyEnabled: false,
    proxyServer: '',
    proxyHttpServer: '',
    proxyHttpsServer: '',
    proxyAllServer: '',
    proxyBypassRules: '',
  };
}

export default function Installer() {
  const { status, config, loading, error, refresh, start, stop, restart, saveConfig, generateToken } = useRuntimeStore();
  const [draftConfig, setDraftConfig] = useState<RuntimeConfig>(defaultConfig());

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (config) {
      setDraftConfig({
        autoStart: config.autoStart,
        token: config.token ?? '',
        port: config.port,
        proxyEnabled: config.proxyEnabled,
        proxyServer: config.proxyServer ?? '',
        proxyHttpServer: config.proxyHttpServer ?? '',
        proxyHttpsServer: config.proxyHttpsServer ?? '',
        proxyAllServer: config.proxyAllServer ?? '',
        proxyBypassRules: config.proxyBypassRules ?? '',
      });
    }
  }, [config]);

  const runtimeState = status?.state ?? 'stopped';
  const isRunning = runtimeState === 'running';
  const isStarting = runtimeState === 'starting';
  const isStopping = runtimeState === 'stopping';
  const controlUrl = status?.controlUrl;
  const healthLabel = status?.healthy ? 'Healthy' : isRunning ? 'Unreachable' : 'Not running';
  const hasConfigChanges = useMemo(() => JSON.stringify(draftConfig) !== JSON.stringify({
    autoStart: config?.autoStart ?? true,
    token: config?.token ?? '',
    port: config?.port ?? 18789,
    proxyEnabled: config?.proxyEnabled ?? false,
    proxyServer: config?.proxyServer ?? '',
    proxyHttpServer: config?.proxyHttpServer ?? '',
    proxyHttpsServer: config?.proxyHttpsServer ?? '',
    proxyAllServer: config?.proxyAllServer ?? '',
    proxyBypassRules: config?.proxyBypassRules ?? '',
  }), [config, draftConfig]);

  const handleSaveConfig = async () => {
    await saveConfig({
      ...draftConfig,
      token: (draftConfig.token ?? '').trim() || undefined,
      proxyServer: (draftConfig.proxyServer ?? '').trim() || undefined,
      proxyHttpServer: (draftConfig.proxyHttpServer ?? '').trim() || undefined,
      proxyHttpsServer: (draftConfig.proxyHttpsServer ?? '').trim() || undefined,
      proxyAllServer: (draftConfig.proxyAllServer ?? '').trim() || undefined,
      proxyBypassRules: (draftConfig.proxyBypassRules ?? '').trim() || undefined,
    });
  };

  return (
    <div className="p-6">
      <div className="mx-auto max-w-6xl space-y-6">
        <div className="rounded-2xl border border-zinc-800 bg-zinc-900/80 p-6">
          <div className="flex items-start justify-between gap-4">
            <div>
              <div className="flex items-center gap-3">
                <div className="rounded-xl bg-blue-600/15 p-3 text-blue-400">
                  <HardDriveDownload className="h-6 w-6" />
                </div>
                <div>
                  <h1 className="text-2xl font-semibold">Installer & Runtime</h1>
                  <p className="mt-1 text-sm text-zinc-500">
                    Control the local OpenClaw runtime from the desktop shell.
                  </p>
                </div>
              </div>
            </div>
            <button
              onClick={() => void refresh()}
              disabled={loading}
              className="rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm text-zinc-200 transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-60"
            >
              {loading ? 'Refreshing…' : 'Refresh status'}
            </button>
          </div>
        </div>

        {error && (
          <div className="rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
            <div className="flex items-start gap-2">
              <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" />
              <span>{error}</span>
            </div>
          </div>
        )}

        <div className="grid gap-6 xl:grid-cols-[1.2fr_0.8fr]">
          <section className="space-y-6 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-6">
            <div className="flex items-center justify-between gap-4">
              <div>
                <h2 className="text-lg font-medium">Runtime status</h2>
                <p className="mt-1 text-sm text-zinc-500">
                  Current process state, health checks, and control endpoint.
                </p>
              </div>
              <div className={`rounded-full border px-3 py-1 text-xs font-medium uppercase tracking-[0.2em] ${getStatusTone(runtimeState)}`}>
                {runtimeState}
              </div>
            </div>

            <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
              <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
                <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">Port</div>
                <div className="mt-2 text-2xl font-semibold text-zinc-100">{status?.port ?? draftConfig.port}</div>
              </div>
              <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
                <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">PID</div>
                <div className="mt-2 text-2xl font-semibold text-zinc-100">{status?.pid ?? '—'}</div>
              </div>
              <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
                <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">Health</div>
                <div className="mt-2 inline-flex items-center gap-2 text-sm font-medium text-zinc-100">
                  <HeartPulse className={`h-4 w-4 ${status?.healthy ? 'text-green-400' : 'text-zinc-500'}`} />
                  {healthLabel}
                </div>
              </div>
              <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
                <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">Started</div>
                <div className="mt-2 text-sm font-medium text-zinc-100">{formatStartedAt(status?.startedAt)}</div>
              </div>
            </div>

            <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <div className="text-sm font-medium text-zinc-100">Control UI</div>
                  <div className="mt-1 break-all text-sm text-zinc-400">{controlUrl ?? 'Available after runtime starts'}</div>
                </div>
                {controlUrl ? (
                  <a
                    href={controlUrl}
                    target="_blank"
                    rel="noreferrer"
                    className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 transition-colors hover:bg-zinc-700"
                  >
                    <ExternalLink className="h-4 w-4" />
                    Open
                  </a>
                ) : null}
              </div>
            </div>

            <div className="flex flex-wrap gap-3">
              <button
                onClick={() => void start()}
                disabled={loading || isRunning || isStarting}
                className="inline-flex items-center gap-2 rounded-lg bg-green-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-green-500 disabled:cursor-not-allowed disabled:bg-zinc-700"
              >
                {loading && isStarting ? <Loader2 className="h-4 w-4 animate-spin" /> : <Play className="h-4 w-4" />}
                Start runtime
              </button>
              <button
                onClick={() => void stop()}
                disabled={loading || !isRunning || isStopping}
                className="inline-flex items-center gap-2 rounded-lg bg-zinc-800 px-4 py-2 text-sm font-medium text-zinc-100 transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:bg-zinc-800/60 disabled:text-zinc-500"
              >
                {loading && isStopping ? <Loader2 className="h-4 w-4 animate-spin" /> : <SquareTerminal className="h-4 w-4" />}
                Stop runtime
              </button>
              <button
                onClick={() => void restart()}
                disabled={loading || (!isRunning && runtimeState !== 'error')}
                className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:bg-zinc-700"
              >
                <RotateCcw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
                Restart runtime
              </button>
            </div>
          </section>

          <section className="space-y-6 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-6">
            <div>
              <h2 className="text-lg font-medium">Runtime configuration</h2>
              <p className="mt-1 text-sm text-zinc-500">
                Persist gateway port, token, and proxy behavior used during startup.
              </p>
            </div>

            <div className="space-y-4">
              <label className="flex items-center justify-between gap-4 rounded-xl border border-zinc-800 bg-zinc-950/60 px-4 py-3">
                <div>
                  <div className="text-sm font-medium text-zinc-100">Auto start</div>
                  <div className="mt-1 text-xs text-zinc-500">Start the local gateway automatically when supported.</div>
                </div>
                <input
                  type="checkbox"
                  checked={draftConfig.autoStart}
                  onChange={(event) => setDraftConfig((current) => ({ ...current, autoStart: event.target.checked }))}
                  className="h-4 w-4 rounded border-zinc-600 bg-zinc-900 text-blue-500 focus:ring-blue-500"
                />
              </label>

              <div>
                <label className="mb-1 block text-sm text-zinc-400">Port</label>
                <input
                  type="number"
                  min={1}
                  max={65535}
                  value={draftConfig.port}
                  onChange={(event) => setDraftConfig((current) => ({ ...current, port: Number(event.target.value) || 18789 }))}
                  className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition-colors focus:border-blue-500"
                />
              </div>

              <div>
                <div className="mb-1 flex items-center justify-between gap-3">
                  <label className="block text-sm text-zinc-400">Gateway token</label>
                  <button
                    onClick={() => void generateToken()}
                    disabled={loading}
                    className="inline-flex items-center gap-2 text-xs text-blue-300 transition-colors hover:text-blue-200 disabled:cursor-not-allowed disabled:text-zinc-500"
                  >
                    <RefreshCw className={`h-3.5 w-3.5 ${loading ? 'animate-spin' : ''}`} />
                    Generate token
                  </button>
                </div>
                <input
                  value={draftConfig.token}
                  onChange={(event) => setDraftConfig((current) => ({ ...current, token: event.target.value }))}
                  placeholder="Optional access token"
                  className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition-colors focus:border-blue-500"
                />
              </div>

              <label className="flex items-center justify-between gap-4 rounded-xl border border-zinc-800 bg-zinc-950/60 px-4 py-3">
                <div>
                  <div className="text-sm font-medium text-zinc-100">Enable proxy</div>
                  <div className="mt-1 text-xs text-zinc-500">Apply proxy variables to the spawned gateway process.</div>
                </div>
                <input
                  type="checkbox"
                  checked={draftConfig.proxyEnabled}
                  onChange={(event) => setDraftConfig((current) => ({ ...current, proxyEnabled: event.target.checked }))}
                  className="h-4 w-4 rounded border-zinc-600 bg-zinc-900 text-blue-500 focus:ring-blue-500"
                />
              </label>

              {draftConfig.proxyEnabled && (
                <div className="space-y-4 rounded-xl border border-zinc-800 bg-zinc-950/40 p-4">
                  <div>
                    <label className="mb-1 block text-sm text-zinc-400">Shared proxy</label>
                    <input
                      value={draftConfig.proxyServer}
                      onChange={(event) => setDraftConfig((current) => ({ ...current, proxyServer: event.target.value }))}
                      placeholder="http://127.0.0.1:7890"
                      className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition-colors focus:border-blue-500"
                    />
                  </div>
                  <div className="grid gap-4 sm:grid-cols-2">
                    <div>
                      <label className="mb-1 block text-sm text-zinc-400">HTTP proxy</label>
                      <input
                        value={draftConfig.proxyHttpServer}
                        onChange={(event) => setDraftConfig((current) => ({ ...current, proxyHttpServer: event.target.value }))}
                        placeholder="http://127.0.0.1:7890"
                        className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition-colors focus:border-blue-500"
                      />
                    </div>
                    <div>
                      <label className="mb-1 block text-sm text-zinc-400">HTTPS proxy</label>
                      <input
                        value={draftConfig.proxyHttpsServer}
                        onChange={(event) => setDraftConfig((current) => ({ ...current, proxyHttpsServer: event.target.value }))}
                        placeholder="http://127.0.0.1:7890"
                        className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition-colors focus:border-blue-500"
                      />
                    </div>
                  </div>
                  <div>
                    <label className="mb-1 block text-sm text-zinc-400">ALL_PROXY</label>
                    <input
                      value={draftConfig.proxyAllServer}
                      onChange={(event) => setDraftConfig((current) => ({ ...current, proxyAllServer: event.target.value }))}
                      placeholder="socks5://127.0.0.1:7891"
                      className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition-colors focus:border-blue-500"
                    />
                  </div>
                  <div>
                    <label className="mb-1 block text-sm text-zinc-400">NO_PROXY</label>
                    <input
                      value={draftConfig.proxyBypassRules}
                      onChange={(event) => setDraftConfig((current) => ({ ...current, proxyBypassRules: event.target.value }))}
                      placeholder="localhost,127.0.0.1"
                      className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition-colors focus:border-blue-500"
                    />
                  </div>
                </div>
              )}
            </div>

            <div className="flex flex-wrap gap-3">
              <button
                onClick={() => void handleSaveConfig()}
                disabled={loading || !hasConfigChanges}
                className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:bg-zinc-700"
              >
                <Save className="h-4 w-4" />
                Save configuration
              </button>
              <button
                onClick={() => setDraftConfig(config ? {
                  autoStart: config.autoStart,
                  token: config.token ?? '',
                  port: config.port,
                  proxyEnabled: config.proxyEnabled,
                  proxyServer: config.proxyServer ?? '',
                  proxyHttpServer: config.proxyHttpServer ?? '',
                  proxyHttpsServer: config.proxyHttpsServer ?? '',
                  proxyAllServer: config.proxyAllServer ?? '',
                  proxyBypassRules: config.proxyBypassRules ?? '',
                } : defaultConfig())}
                disabled={loading || !hasConfigChanges}
                className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm font-medium text-zinc-100 transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:bg-zinc-800/50 disabled:text-zinc-500"
              >
                Reset draft
              </button>
            </div>

            <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4 text-sm text-zinc-400">
              <div className="flex items-center gap-2 font-medium text-zinc-200">
                <ShieldCheck className="h-4 w-4 text-green-400" />
                MVP notes
              </div>
              <ul className="mt-3 space-y-2">
                <li>Runtime status now includes health probing and the control UI URL.</li>
                <li>Gateway configuration is persisted through the existing backend commands.</li>
                <li>Explicit runtime binary install/download flow is still the next backend milestone.</li>
              </ul>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}

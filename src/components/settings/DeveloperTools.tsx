import { useState, useEffect } from 'react';
import { Activity, CheckCircle, AlertTriangle, XCircle, Download, Trash2, RefreshCw, Terminal } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import type { DiagnosticInfo, TelemetryData, TokenUsageStats, LogEntry, SystemInfo } from '@/types';

export default function DeveloperTools() {
  const [diagnostics, setDiagnostics] = useState<DiagnosticInfo | null>(null);
  const [telemetry, setTelemetry] = useState<TelemetryData[]>([]);
  const [tokenStats, setTokenStats] = useState<TokenUsageStats | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [activeTab, setActiveTab] = useState<'diagnostics' | 'telemetry' | 'logs' | 'system'>('diagnostics');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadSystemInfo();
    loadDiagnostics();
    loadTelemetry();
    loadTokenStats();
    loadLogs();
  }, []);

  const loadDiagnostics = async () => {
    try {
      setLoading(true);
      const result = await invoke<DiagnosticInfo>('run_diagnostics');
      setDiagnostics(result);
    } catch (error) {
      console.error('Failed to run diagnostics:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadTelemetry = async () => {
    try {
      const result = await invoke<TelemetryData[]>('get_telemetry_data', { limit: 50 });
      setTelemetry(result);
    } catch (error) {
      console.error('Failed to load telemetry:', error);
    }
  };

  const loadTokenStats = async () => {
    try {
      const result = await invoke<TokenUsageStats>('get_token_usage_stats');
      setTokenStats(result);
    } catch (error) {
      console.error('Failed to load token stats:', error);
    }
  };

  const loadLogs = async () => {
    try {
      const result = await invoke<LogEntry[]>('get_app_logs', { limit: 50 });
      setLogs(result);
    } catch (error) {
      console.error('Failed to load logs:', error);
    }
  };

  const loadSystemInfo = async () => {
    try {
      const result = await invoke<SystemInfo>('get_system_info');
      setSystemInfo(result);
    } catch (error) {
      console.error('Failed to load system info:', error);
    }
  };

  const exportTelemetry = async (format: 'json' | 'csv') => {
    try {
      const data = await invoke<string>('export_telemetry_data', { format });
      const blob = new Blob([data], { type: format === 'json' ? 'application/json' : 'text/csv' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `telemetry_${Date.now()}.${format}`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export telemetry:', error);
      alert('Failed to export telemetry data');
    }
  };

  const clearTelemetry = async () => {
    if (confirm('Are you sure you want to clear all telemetry data? This action cannot be undone.')) {
      try {
        await invoke('clear_telemetry_data');
        await loadTelemetry();
        await loadTokenStats();
        alert('Telemetry data cleared successfully');
      } catch (error) {
        console.error('Failed to clear telemetry:', error);
        alert('Failed to clear telemetry data');
      }
    }
  };

  const getDiagnosticIcon = (status: string) => {
    switch (status) {
      case 'healthy':
        return <CheckCircle className="w-5 h-5 text-green-500" />;
      case 'warning':
        return <AlertTriangle className="w-5 h-5 text-yellow-500" />;
      case 'critical':
        return <XCircle className="w-5 h-5 text-red-500" />;
      default:
        return null;
    }
  };

  const getLogLevelColor = (level: string) => {
    switch (level.toLowerCase()) {
      case 'error':
        return 'text-red-400';
      case 'warn':
      case 'warning':
        return 'text-yellow-400';
      case 'info':
        return 'text-blue-400';
      case 'debug':
        return 'text-zinc-400';
      default:
        return 'text-zinc-300';
    }
  };

  return (
    <div className="space-y-6">
      {/* Tabs */}
      <div className="border-b border-zinc-700">
        <nav className="flex gap-6">
          {(['diagnostics', 'telemetry', 'logs', 'system'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`pb-4 px-2 text-sm font-medium transition-colors ${
                activeTab === tab
                  ? 'text-blue-500 border-b-2 border-blue-500'
                  : 'text-zinc-400 hover:text-zinc-300'
              }`}
            >
              {tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </nav>
      </div>

      {/* Diagnostics Tab */}
      {activeTab === 'diagnostics' && (
        <div>
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-xl font-semibold">System Diagnostics</h2>
            <button
              onClick={loadDiagnostics}
              disabled={loading}
              className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors disabled:opacity-50"
            >
              {loading ? (
                <RefreshCw className="w-4 h-4 animate-spin" />
              ) : (
                <RefreshCw className="w-4 h-4" />
              )}
              Run Diagnostics
            </button>
          </div>

          {diagnostics && (
            <div className="space-y-6">
              {/* Overall Status */}
              <div className={`p-6 rounded-lg border ${
                diagnostics.status === 'healthy' ? 'bg-green-500/20 border-green-500/50' :
                diagnostics.status === 'warning' ? 'bg-yellow-500/20 border-yellow-500/50' :
                'bg-red-500/20 border-red-500/50'
              }`}>
                <div className="flex items-center gap-3">
                  {getDiagnosticIcon(diagnostics.status)}
                  <div>
                    <p className="font-medium text-lg">{diagnostics.summary}</p>
                    <p className="text-sm text-zinc-400">
                      Status: <span className="uppercase">{diagnostics.status}</span>
                    </p>
                  </div>
                </div>
              </div>

              {/* Individual Checks */}
              <div className="space-y-3">
                <h3 className="font-semibold">Detailed Checks</h3>
                {diagnostics.checks.map((check, index) => (
                  <div key={index} className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                    <div className="flex items-start justify-between">
                      <div className="flex items-start gap-3">
                        {getDiagnosticIcon(check.status)}
                        <div>
                          <p className="font-medium">{check.name}</p>
                          <p className="text-sm text-zinc-400">{check.message}</p>
                          {check.details && (
                            <p className="text-xs text-zinc-500 mt-2">{check.details}</p>
                          )}
                        </div>
                      </div>
                      <span className={`px-2 py-1 rounded text-xs font-medium ${
                        check.status === 'healthy' ? 'bg-green-500/20 text-green-400' :
                        check.status === 'warning' ? 'bg-yellow-500/20 text-yellow-400' :
                        'bg-red-500/20 text-red-400'
                      }`}>
                        {check.status}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Telemetry Tab */}
      {activeTab === 'telemetry' && (
        <div>
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-xl font-semibold">Token Usage & Telemetry</h2>
            <div className="flex gap-2">
              <button
                onClick={() => exportTelemetry('json')}
                className="flex items-center gap-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
              >
                <Download className="w-4 h-4" />
                Export JSON
              </button>
              <button
                onClick={() => exportTelemetry('csv')}
                className="flex items-center gap-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
              >
                <Download className="w-4 h-4" />
                Export CSV
              </button>
              <button
                onClick={clearTelemetry}
                className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg transition-colors"
              >
                <Trash2 className="w-4 h-4" />
                Clear
              </button>
            </div>
          </div>

          {/* Token Stats */}
          {tokenStats && (
            <div className="grid grid-cols-4 gap-4 mb-6">
              <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                <p className="text-sm text-zinc-400">Total Requests</p>
                <p className="text-2xl font-bold">{tokenStats.totalRequests}</p>
              </div>
              <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                <p className="text-sm text-zinc-400">Total Tokens</p>
                <p className="text-2xl font-bold">{tokenStats.totalTokens.toLocaleString()}</p>
              </div>
              <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                <p className="text-sm text-zinc-400">Prompt Tokens</p>
                <p className="text-2xl font-bold">{tokenStats.totalPromptTokens.toLocaleString()}</p>
              </div>
              <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                <p className="text-sm text-zinc-400">Completion Tokens</p>
                <p className="text-2xl font-bold">{tokenStats.totalCompletionTokens.toLocaleString()}</p>
              </div>
            </div>
          )}

          {/* Telemetry Table */}
          <div className="bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
            <table className="w-full">
              <thead className="bg-zinc-800">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Timestamp</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Model</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Prompt</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Completion</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Total</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800">
                {telemetry.map((entry, index) => (
                  <tr key={index} className="hover:bg-zinc-800/50">
                    <td className="px-4 py-3 text-sm">{new Date(entry.timestamp).toLocaleString()}</td>
                    <td className="px-4 py-3 text-sm">{entry.modelId}</td>
                    <td className="px-4 py-3 text-sm">{entry.promptTokens}</td>
                    <td className="px-4 py-3 text-sm">{entry.completionTokens}</td>
                    <td className="px-4 py-3 text-sm">{entry.totalTokens}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Logs Tab */}
      {activeTab === 'logs' && (
        <div>
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-xl font-semibold">Application Logs</h2>
            <button
              onClick={loadLogs}
              className="flex items-center gap-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
            >
              <RefreshCw className="w-4 h-4" />
              Refresh
            </button>
          </div>

          <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4 font-mono text-sm space-y-2 max-h-96 overflow-y-auto">
            {logs.length === 0 ? (
              <p className="text-zinc-400">No logs available</p>
            ) : (
              logs.map((log, index) => (
                <div key={index} className="flex gap-3">
                  <span className="text-zinc-500 shrink-0">{new Date(log.timestamp).toLocaleTimeString()}</span>
                  <span className={`shrink-0 uppercase w-12 ${getLogLevelColor(log.level)}`}>{log.level}</span>
                  <span className="text-zinc-300">{log.message}</span>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {/* System Info Tab */}
      {activeTab === 'system' && (
        <div>
          <h2 className="text-xl font-semibold mb-6">System Information</h2>

          {systemInfo && (
            <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-6">
              <div className="flex items-center gap-3 mb-6">
                <Terminal className="w-6 h-6 text-blue-500" />
                <h3 className="font-medium">Environment</h3>
              </div>

              <div className="space-y-4">
                <div className="flex justify-between">
                  <span className="text-zinc-400">Operating System</span>
                  <span className="font-medium">{systemInfo.os}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-400">Architecture</span>
                  <span className="font-medium">{systemInfo.arch}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-400">Family</span>
                  <span className="font-medium">{systemInfo.family}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-400">Version</span>
                  <span className="font-medium">{systemInfo.version}</span>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
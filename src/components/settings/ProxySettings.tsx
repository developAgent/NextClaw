import { useState, useEffect } from 'react';
import { Network, Check, AlertCircle, RefreshCw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import type { ProxyConfig, ProxyType, TestResult } from '@/types';

export default function ProxySettings() {
  const [config, setConfig] = useState<ProxyConfig>({
    enabled: false,
    server: '',
    port: 8080,
    username: '',
    password: '',
    proxyType: 'http',
    bypassRules: ['localhost', '127.0.0.1', '::1', '*.local'],
  });
  const [loading, setLoading] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<TestResult | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setLoading(true);
      const result = await invoke<ProxyConfig | null>('get_proxy_config');
      if (result) {
        setConfig(result);
      }
    } catch (error) {
      console.error('Failed to load proxy config:', error);
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async () => {
    try {
      setLoading(true);
      await invoke('set_proxy_config', { config });
    } catch (error) {
      console.error('Failed to save proxy config:', error);
      alert('Failed to save proxy configuration');
    } finally {
      setLoading(false);
    }
  };

  const testConnection = async () => {
    try {
      setTesting(true);
      setTestResult(null);
      const result = await invoke<TestResult>('test_proxy_connection', { config });
      setTestResult(result);
    } catch (error) {
      console.error('Failed to test proxy:', error);
      setTestResult({
        success: false,
        latencyMs: 0,
        message: 'Connection failed',
      });
    } finally {
      setTesting(false);
    }
  };

  const addBypassRule = () => {
    setConfig(prev => ({
      ...prev,
      bypassRules: [...prev.bypassRules, ''],
    }));
  };

  const removeBypassRule = (index: number) => {
    setConfig(prev => ({
      ...prev,
      bypassRules: prev.bypassRules.filter((_, i) => i !== index),
    }));
  };

  const updateBypassRule = (index: number, value: string) => {
    setConfig(prev => ({
      ...prev,
      bypassRules: prev.bypassRules.map((rule, i) => i === index ? value : rule),
    }));
  };

  const loadDefaultBypassRules = async () => {
    try {
      const defaultRules = await invoke<string[]>('get_default_bypass_rules');
      setConfig(prev => ({
        ...prev,
        bypassRules: defaultRules,
      }));
    } catch (error) {
      console.error('Failed to load default bypass rules:', error);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-6">
        <div className="flex items-center gap-3 mb-6">
          <Network className="w-6 h-6 text-blue-500" />
          <h3 className="text-lg font-semibold">Proxy Configuration</h3>
        </div>

        <div className="space-y-6">
          {/* Enable/Disable */}
          <div className="flex items-center justify-between p-4 bg-zinc-800 rounded-lg">
            <div>
              <h4 className="font-medium">Enable Proxy</h4>
              <p className="text-sm text-zinc-400">Route all network traffic through proxy</p>
            </div>
            <button
              onClick={() => {
                setConfig(prev => ({ ...prev, enabled: !prev.enabled }));
              }}
              className={`w-12 h-6 rounded-full transition-colors ${
                config.enabled ? 'bg-blue-600' : 'bg-zinc-700'
              }`}
            >
              <div
                className={`w-5 h-5 bg-white rounded-full transition-transform ${
                  config.enabled ? 'translate-x-6' : 'translate-x-0.5'
                }`}
              />
            </button>
          </div>

          {/* Proxy Type */}
          <div>
            <label className="block text-sm font-medium mb-2">Proxy Type</label>
            <select
              value={config.proxyType}
              onChange={(e) => setConfig(prev => ({ ...prev, proxyType: e.target.value as ProxyType }))}
              disabled={!config.enabled}
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
            >
              <option value="http">HTTP</option>
              <option value="https">HTTPS</option>
              <option value="socks5">SOCKS5</option>
            </select>
          </div>

          {/* Server */}
          <div>
            <label className="block text-sm font-medium mb-2">Server Address</label>
            <input
              type="text"
              value={config.server}
              onChange={(e) => setConfig(prev => ({ ...prev, server: e.target.value }))}
              placeholder="proxy.example.com"
              disabled={!config.enabled}
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
            />
          </div>

          {/* Port */}
          <div>
            <label className="block text-sm font-medium mb-2">Port</label>
            <input
              type="number"
              value={config.port}
              onChange={(e) => setConfig(prev => ({ ...prev, port: parseInt(e.target.value) }))}
              disabled={!config.enabled}
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
            />
          </div>

          {/* Authentication */}
          <div className="border-t border-zinc-700 pt-6">
            <h4 className="font-medium mb-4">Authentication (Optional)</h4>
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Username</label>
                <input
                  type="text"
                  value={config.username}
                  onChange={(e) => setConfig(prev => ({ ...prev, username: e.target.value }))}
                  placeholder="Optional username"
                  disabled={!config.enabled}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
                />
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Password</label>
                <input
                  type="password"
                  value={config.password}
                  onChange={(e) => setConfig(prev => ({ ...prev, password: e.target.value }))}
                  placeholder="Optional password"
                  disabled={!config.enabled}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
                />
              </div>
            </div>
          </div>

          {/* Bypass Rules */}
          <div className="border-t border-zinc-700 pt-6">
            <div className="flex items-center justify-between mb-4">
              <h4 className="font-medium">Bypass Rules</h4>
              <button
                onClick={loadDefaultBypassRules}
                className="text-sm text-blue-500 hover:text-blue-400"
              >
                Load Defaults
              </button>
            </div>
            <div className="space-y-2">
              {config.bypassRules.map((rule, index) => (
                <div key={index} className="flex gap-2">
                  <input
                    type="text"
                    value={rule}
                    onChange={(e) => updateBypassRule(index, e.target.value)}
                    disabled={!config.enabled}
                    placeholder="hostname or pattern"
                    className="flex-1 bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
                  />
                  <button
                    onClick={() => removeBypassRule(index)}
                    disabled={!config.enabled}
                    className="px-3 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors disabled:opacity-50"
                  >
                    ×
                  </button>
                </div>
              ))}
              <button
                onClick={addBypassRule}
                disabled={!config.enabled}
                className="text-sm text-blue-500 hover:text-blue-400 disabled:opacity-50"
              >
                + Add Rule
              </button>
            </div>
          </div>

          {/* Test Connection */}
          <div className="border-t border-zinc-700 pt-6">
            <button
              onClick={testConnection}
              disabled={!config.enabled || testing}
              className="flex items-center gap-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors disabled:opacity-50"
            >
              {testing ? (
                <RefreshCw className="w-4 h-4 animate-spin" />
              ) : (
                <Network className="w-4 h-4" />
              )}
              Test Connection
            </button>

            {testResult && (
              <div className={`mt-4 p-4 rounded-lg flex items-start gap-3 ${
                testResult.success ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'
              }`}>
                {testResult.success ? (
                  <Check className="w-5 h-5 mt-0.5" />
                ) : (
                  <AlertCircle className="w-5 h-5 mt-0.5" />
                )}
                <div>
                  <p className="font-medium">{testResult.message}</p>
                  {testResult.success && (
                    <p className="text-sm mt-1">Latency: {testResult.latencyMs}ms</p>
                  )}
                </div>
              </div>
            )}
          </div>

          {/* Actions */}
          <div className="border-t border-zinc-700 pt-6 flex gap-3">
            <button
              onClick={saveConfig}
              disabled={loading}
              className="px-6 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors disabled:opacity-50"
            >
              {loading ? 'Saving...' : 'Save Configuration'}
            </button>
            <button
              onClick={() => {
                setConfig({
                  enabled: false,
                  server: '',
                  port: 8080,
                  username: '',
                  password: '',
                  proxyType: 'http',
                  bypassRules: ['localhost', '127.0.0.1', '::1', '*.local'],
                });
                setTestResult(null);
              }}
              className="px-6 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
            >
              Reset
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
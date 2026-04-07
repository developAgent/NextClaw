import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X } from 'lucide-react';
import { Config, ConfigUpdate } from '@/types';

export default function SettingsModal() {
  const [isOpen, setIsOpen] = useState(false);
  const [config, setConfig] = useState<Config | null>(null);
  const [apiKey, setApiKey] = useState('');
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === ',' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setIsOpen(true);
        loadConfig();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const loadConfig = async () => {
    try {
      const loadedConfig = await invoke<Config>('get_config');
      setConfig(loadedConfig);
      const storedKey = localStorage.getItem('api_key');
      if (storedKey) {
        setApiKey(storedKey);
      }
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  };

  const handleSaveApiKey = async () => {
    setIsSaving(true);
    try {
      await invoke('set_api_key', { apiKey });
      localStorage.setItem('api_key', apiKey);
      if (config) {
        setConfig({
          ...config,
          api: { ...config.api, apiKeyConfigured: true },
        });
      }
      setIsOpen(false);
    } catch (error) {
      console.error('Failed to save API key:', error);
      alert('Failed to save API key');
    } finally {
      setIsSaving(false);
    }
  };

  const handleUpdateConfig = async (update: ConfigUpdate) => {
    try {
      await invoke('update_config', { config: update });
      await loadConfig();
    } catch (error) {
      console.error('Failed to update config:', error);
    }
  };

  if (!isOpen) return null;

  if (!config) {
    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div className="bg-zinc-900 rounded-lg p-6 max-w-md w-full mx-4">
          <div className="animate-pulse text-center">Loading...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-zinc-900 rounded-lg max-w-2xl w-full max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-zinc-800">
          <h2 className="text-xl font-semibold">Settings</h2>
          <button
            onClick={() => setIsOpen(false)}
            className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto max-h-[calc(90vh-140px)] space-y-6">
          {/* API Key Section */}
          <section>
            <h3 className="text-lg font-medium mb-3">API Configuration</h3>
            <div className="space-y-3">
              <div>
                <label className="block text-sm text-zinc-400 mb-1">
                  Anthropic API Key
                </label>
                <input
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="sk-ant-..."
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 text-zinc-100"
                />
              </div>
              <button
                onClick={handleSaveApiKey}
                disabled={isSaving || !apiKey}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-zinc-700 disabled:cursor-not-allowed rounded-lg transition-colors"
              >
                {isSaving ? 'Saving...' : 'Save API Key'}
              </button>
              {config.api.apiKeyConfigured && (
                <p className="text-sm text-green-400">
                  ✓ API key is configured
                </p>
              )}
            </div>
          </section>

          {/* Model Settings */}
          <section>
            <h3 className="text-lg font-medium mb-3">Model Settings</h3>
            <div className="space-y-3">
              <div>
                <label className="block text-sm text-zinc-400 mb-1">
                  Claude Model
                </label>
                <select
                  value={config.api.claudeModel}
                  onChange={(e) =>
                    handleUpdateConfig({ claudeModel: e.target.value })
                  }
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 text-zinc-100"
                >
                  <option value="claude-3-sonnet-20240229">Claude 3 Sonnet</option>
                  <option value="claude-3-opus-20240229">Claude 3 Opus</option>
                  <option value="claude-3-haiku-20240307">Claude 3 Haiku</option>
                </select>
              </div>
            </div>
          </section>

          {/* Command Execution Settings */}
          <section>
            <h3 className="text-lg font-medium mb-3">Command Execution</h3>
            <div className="space-y-3">
              <div>
                <label className="block text-sm text-zinc-400 mb-1">
                  Command Timeout (seconds)
                </label>
                <input
                  type="number"
                  value={config.commands.timeoutSecs}
                  onChange={(e) =>
                    handleUpdateConfig({
                      timeoutSecs: parseInt(e.target.value) || 60,
                    })
                  }
                  min="1"
                  max="3600"
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 text-zinc-100"
                />
              </div>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={config.commands.requireConfirmation}
                  onChange={(e) =>
                    handleUpdateConfig({ requireConfirmation: e.target.checked })
                  }
                  className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-zinc-900"
                />
                <span className="text-sm text-zinc-300">
                  Require confirmation for dangerous commands
                </span>
              </label>
            </div>
          </section>
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-zinc-800 flex justify-end">
          <button
            onClick={() => setIsOpen(false)}
            className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
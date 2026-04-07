import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, Plus, Trash2, Check, AlertCircle, Key, Settings, Layers, Keyboard, Palette } from 'lucide-react';
import { Config, Channel, Plugin, Hotkey } from '@/types';

type TabType = 'general' | 'channels' | 'plugins' | 'hotkeys' | 'themes';

interface ChannelFormData {
  name: string;
  provider: 'claude' | 'openai' | 'gemini';
  model: string;
  apiKey: string;
  apiBase: string;
  priority: number;
}

export default function SettingsModal() {
  const [isOpen, setIsOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>('general');
  const [config, setConfig] = useState<Config | null>(null);
  const [apiKey, setApiKey] = useState('');
  const [isSaving, setIsSaving] = useState(false);

  // Channel management
  const [channels, setChannels] = useState<Channel[]>([]);
  const [isAddingChannel, setIsAddingChannel] = useState(false);
  const [channelForm, setChannelForm] = useState<ChannelFormData>({
    name: '',
    provider: 'claude',
    model: 'claude-3-sonnet-20240229',
    apiKey: '',
    apiBase: '',
    priority: 0,
  });

  // Plugin management
  const [plugins, setPlugins] = useState<Plugin[]>([]);

  // Hotkey management
  const [hotkeys, setHotkeys] = useState<Hotkey[]>([]);
  const [isRecordingHotkey, setIsRecordingHotkey] = useState<string | null>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === ',' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setIsOpen(true);
        loadAllData();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const loadAllData = async () => {
    try {
      const [loadedConfig, channelsData, pluginsData, hotkeysData] = await Promise.all([
        invoke<Config>('get_config'),
        invoke<Channel[]>('get_all_channels'),
        invoke<Plugin[]>('get_all_plugins'),
        invoke<Hotkey[]>('get_all_hotkeys'),
      ]);
      setConfig(loadedConfig);
      setChannels(channelsData || []);
      setPlugins(pluginsData || []);
      setHotkeys(hotkeysData || []);
      const storedKey = localStorage.getItem('api_key');
      if (storedKey) setApiKey(storedKey);
    } catch (error) {
      console.error('Failed to load data:', error);
    }
  };

  const handleSaveApiKey = async () => {
    setIsSaving(true);
    try {
      await invoke('set_api_key', { apiKey });
      localStorage.setItem('api_key', apiKey);
      await loadAllData();
      setIsOpen(false);
    } catch (error) {
      alert('Failed to save API key');
    } finally {
      setIsSaving(false);
    }
  };

  const handleAddChannel = async () => {
    try {
      const newChannel: Channel = {
        id: crypto.randomUUID(),
        ...channelForm,
        enabled: true,
        healthStatus: 'unknown',
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };
      await invoke('add_channel', { channel: newChannel });
      setChannels([...channels, newChannel]);
      setIsAddingChannel(false);
      setChannelForm({
        name: '',
        provider: 'claude',
        model: 'claude-3-sonnet-20240229',
        apiKey: '',
        apiBase: '',
        priority: 0,
      });
    } catch (error) {
      alert('Failed to add channel');
    }
  };

  const handleDeleteChannel = async (id: string) => {
    if (!confirm('Are you sure you want to delete this channel?')) return;
    try {
      await invoke('delete_channel', { id });
      setChannels(channels.filter((c) => c.id !== id));
    } catch (error) {
      alert('Failed to delete channel');
    }
  };

  const handleTogglePlugin = async (id: string, enabled: boolean) => {
    try {
      await invoke(enabled ? 'enable_plugin' : 'disable_plugin', { id });
      setPlugins(plugins.map((p) => (p.id === id ? { ...p, enabled } : p)));
    } catch (error) {
      alert('Failed to toggle plugin');
    }
  };

  const handleRecordHotkey = async (hotkeyId: string) => {
    setIsRecordingHotkey(hotkeyId);
    // Record hotkey logic would go here
  };

  const handleUpdateConfig = async (update: any) => {
    try {
      await invoke('update_config', { config: update });
      await loadAllData();
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

  const tabs = [
    { id: 'general' as TabType, label: 'General', icon: Settings },
    { id: 'channels' as TabType, label: 'Channels', icon: Layers },
    { id: 'plugins' as TabType, label: 'Plugins', icon: Key },
    { id: 'hotkeys' as TabType, label: 'Hotkeys', icon: Keyboard },
    { id: 'themes' as TabType, label: 'Themes', icon: Palette },
  ];

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-zinc-900 rounded-lg max-w-4xl w-full max-h-[90vh] overflow-hidden">
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

        <div className="flex">
          {/* Sidebar Tabs */}
          <div className="w-48 border-r border-zinc-800 p-2 space-y-1">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left transition-colors ${
                  activeTab === tab.id
                    ? 'bg-blue-600 text-white'
                    : 'text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100'
                }`}
              >
                <tab.icon className="w-4 h-4" />
                <span className="text-sm">{tab.label}</span>
              </button>
            ))}
          </div>

          {/* Content */}
          <div className="flex-1 p-6 overflow-y-auto max-h-[calc(90vh-140px)]">
            {/* General Tab */}
            {activeTab === 'general' && (
              <div className="space-y-6">
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
                  </div>
                </section>

                <section>
                  <h3 className="text-lg font-medium mb-3">Model Settings</h3>
                  <div className="space-y-3">
                    <div>
                      <label className="block text-sm text-zinc-400 mb-1">
                        Claude Model
                      </label>
                      <select
                        value={config.api.claudeModel}
                        onChange={(e) => handleUpdateConfig({ claudeModel: e.target.value })}
                        className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 text-zinc-100"
                      >
                        <option value="claude-3-sonnet-20240229">Claude 3 Sonnet</option>
                        <option value="claude-3-opus-20240229">Claude 3 Opus</option>
                        <option value="claude-3-haiku-20240307">Claude 3 Haiku</option>
                      </select>
                    </div>
                  </div>
                </section>

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
                        onChange={(e) => handleUpdateConfig({ timeoutSecs: parseInt(e.target.value) || 60 })}
                        min="1"
                        max="3600"
                        className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 text-zinc-100"
                      />
                    </div>
                    <label className="flex items-center gap-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={config.commands.requireConfirmation}
                        onChange={(e) => handleUpdateConfig({ requireConfirmation: e.target.checked })}
                        className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-zinc-900"
                      />
                      <span className="text-sm text-zinc-300">
                        Require confirmation for dangerous commands
                      </span>
                    </label>
                  </div>
                </section>
              </div>
            )}

            {/* Channels Tab */}
            {activeTab === 'channels' && (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-medium">AI Channels</h3>
                  <button
                    onClick={() => setIsAddingChannel(true)}
                    className="flex items-center gap-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors text-sm"
                  >
                    <Plus className="w-4 h-4" />
                    Add Channel
                  </button>
                </div>

                {isAddingChannel && (
                  <div className="bg-zinc-800 rounded-lg p-4 space-y-3">
                    <div>
                      <label className="block text-sm text-zinc-400 mb-1">Channel Name</label>
                      <input
                        type="text"
                        value={channelForm.name}
                        onChange={(e) => setChannelForm({ ...channelForm, name: e.target.value })}
                        placeholder="My Claude Channel"
                        className="w-full bg-zinc-700 border border-zinc-600 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 text-zinc-100"
                      />
                    </div>
                    <div>
                      <label className="block text-sm text-zinc-400 mb-1">Provider</label>
                      <select
                        value={channelForm.provider}
                        onChange={(e) => setChannelForm({ ...channelForm, provider: e.target.value as any })}
                        className="w-full bg-zinc-700 border border-zinc-600 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 text-zinc-100"
                      >
                        <option value="claude">Claude</option>
                        <option value="openai">OpenAI</option>
                        <option value="gemini">Gemini</option>
                      </select>
                    </div>
                    <div>
                      <label className="block text-sm text-zinc-400 mb-1">Model</label>
                      <input
                        type="text"
                        value={channelForm.model}
                        onChange={(e) => setChannelForm({ ...channelForm, model: e.target.value })}
                        placeholder="claude-3-sonnet-20240229"
                        className="w-full bg-zinc-700 border border-zinc-600 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 text-zinc-100"
                      />
                    </div>
                    <div>
                      <label className="block text-sm text-zinc-400 mb-1">API Key</label>
                      <input
                        type="password"
                        value={channelForm.apiKey}
                        onChange={(e) => setChannelForm({ ...channelForm, apiKey: e.target.value })}
                        placeholder="Enter API key"
                        className="w-full bg-zinc-700 border border-zinc-600 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 text-zinc-100"
                      />
                    </div>
                    <div>
                      <label className="block text-sm text-zinc-400 mb-1">API Base (optional)</label>
                      <input
                        type="text"
                        value={channelForm.apiBase}
                        onChange={(e) => setChannelForm({ ...channelForm, apiBase: e.target.value })}
                        placeholder="https://api.anthropic.com"
                        className="w-full bg-zinc-700 border border-zinc-600 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 text-zinc-100"
                      />
                    </div>
                    <div>
                      <label className="block text-sm text-zinc-400 mb-1">Priority</label>
                      <input
                        type="number"
                        value={channelForm.priority}
                        onChange={(e) => setChannelForm({ ...channelForm, priority: parseInt(e.target.value) || 0 })}
                        min="0"
                        className="w-full bg-zinc-700 border border-zinc-600 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 text-zinc-100"
                      />
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={handleAddChannel}
                        disabled={!channelForm.name || !channelForm.apiKey}
                        className="flex-1 px-3 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-zinc-600 disabled:cursor-not-allowed rounded-lg transition-colors text-sm"
                      >
                        Add Channel
                      </button>
                      <button
                        onClick={() => setIsAddingChannel(false)}
                        className="px-3 py-2 bg-zinc-700 hover:bg-zinc-600 rounded-lg transition-colors text-sm"
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                )}

                <div className="space-y-2">
                  {channels.map((channel) => (
                    <div key={channel.id} className="bg-zinc-800 rounded-lg p-4">
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-2">
                            <h4 className="font-medium">{channel.name}</h4>
                            <span className={`px-2 py-0.5 rounded text-xs ${
                              channel.enabled ? 'bg-green-600/20 text-green-400' : 'bg-zinc-600/20 text-zinc-400'
                            }`}>
                              {channel.enabled ? 'Enabled' : 'Disabled'}
                            </span>
                            <span className={`px-2 py-0.5 rounded text-xs ${
                              channel.healthStatus === 'healthy' ? 'bg-green-600/20 text-green-400' :
                              channel.healthStatus === 'degraded' ? 'bg-yellow-600/20 text-yellow-400' :
                              'bg-zinc-600/20 text-zinc-400'
                            }`}>
                              {channel.healthStatus}
                            </span>
                          </div>
                          <div className="mt-2 text-sm text-zinc-400 space-y-1">
                            <div>Provider: <span className="text-zinc-200">{channel.provider}</span></div>
                            <div>Model: <span className="text-zinc-200">{channel.model}</span></div>
                            <div>Priority: <span className="text-zinc-200">{channel.priority}</span></div>
                          </div>
                        </div>
                        <button
                          onClick={() => handleDeleteChannel(channel.id)}
                          className="p-2 hover:bg-red-600/20 hover:text-red-400 rounded-lg transition-colors"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  ))}
                  {channels.length === 0 && (
                    <div className="text-center text-zinc-500 py-8">
                      No channels configured. Add a channel to get started.
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Plugins Tab */}
            {activeTab === 'plugins' && (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-medium">Installed Plugins</h3>
                  <button
                    onClick={() => alert('Plugin marketplace coming soon!')}
                    className="flex items-center gap-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors text-sm"
                  >
                    <Plus className="w-4 h-4" />
                    Browse Plugins
                  </button>
                </div>

                <div className="space-y-2">
                  {plugins.length === 0 ? (
                    <div className="text-center text-zinc-500 py-8">
                      No plugins installed. Browse the plugin marketplace to add functionality.
                    </div>
                  ) : (
                    plugins.map((plugin) => (
                      <div key={plugin.id} className="bg-zinc-800 rounded-lg p-4">
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-2">
                              <h4 className="font-medium">{plugin.name}</h4>
                              <span className="text-sm text-zinc-500">v{plugin.version}</span>
                              <span className={`px-2 py-0.5 rounded text-xs ${
                                plugin.enabled ? 'bg-green-600/20 text-green-400' : 'bg-zinc-600/20 text-zinc-400'
                              }`}>
                                {plugin.enabled ? 'Enabled' : 'Disabled'}
                              </span>
                            </div>
                            <div className="mt-2 text-sm text-zinc-400 space-y-1">
                              <div>Author: <span className="text-zinc-200">{plugin.author || 'Unknown'}</span></div>
                              {plugin.description && (
                                <div>{plugin.description}</div>
                              )}
                            </div>
                          </div>
                          <button
                            onClick={() => handleTogglePlugin(plugin.id, !plugin.enabled)}
                            className={`p-2 rounded-lg transition-colors ${
                              plugin.enabled
                                ? 'hover:bg-yellow-600/20 hover:text-yellow-400'
                                : 'hover:bg-green-600/20 hover:text-green-400'
                            }`}
                          >
                            {plugin.enabled ? (
                              <AlertCircle className="w-4 h-4" />
                            ) : (
                              <Check className="w-4 h-4" />
                            )}
                          </button>
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </div>
            )}

            {/* Hotkeys Tab */}
            {activeTab === 'hotkeys' && (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-medium">Keyboard Shortcuts</h3>
                  <button
                    onClick={() => alert('Add hotkey functionality coming soon!')}
                    className="flex items-center gap-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors text-sm"
                  >
                    <Plus className="w-4 h-4" />
                    Add Hotkey
                  </button>
                </div>

                <div className="space-y-2">
                  {hotkeys.length === 0 ? (
                    <div className="text-center text-zinc-500 py-8">
                      No hotkeys configured. Add custom keyboard shortcuts to quickly access features.
                    </div>
                  ) : (
                    hotkeys.map((hotkey) => (
                      <div key={hotkey.id} className="bg-zinc-800 rounded-lg p-4">
                        <div className="flex items-center justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-2">
                              <h4 className="font-medium">{hotkey.action}</h4>
                              {!hotkey.enabled && (
                                <span className="px-2 py-0.5 rounded text-xs bg-zinc-600/20 text-zinc-400">
                                  Disabled
                                </span>
                              )}
                            </div>
                            <div className="mt-2 text-sm text-zinc-400">
                              <span className="font-mono bg-zinc-700 px-2 py-1 rounded">{hotkey.keyCombination}</span>
                            </div>
                          </div>
                          <button
                            onClick={() => setIsRecordingHotkey(hotkey.id)}
                            disabled={isRecordingHotkey !== null}
                            className="px-3 py-1.5 bg-zinc-700 hover:bg-zinc-600 disabled:bg-zinc-800 disabled:cursor-not-allowed rounded-lg transition-colors text-sm"
                          >
                            {isRecordingHotkey === hotkey.id ? 'Recording...' : 'Change'}
                          </button>
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </div>
            )}

            {/* Themes Tab */}
            {activeTab === 'themes' && (
              <div className="space-y-4">
                <h3 className="text-lg font-medium">Appearance</h3>

                <div>
                  <label className="block text-sm text-zinc-400 mb-3">Theme</label>
                  <div className="grid grid-cols-3 gap-3">
                    {['light', 'dark', 'auto'].map((theme) => (
                      <button
                        key={theme}
                        onClick={() => handleUpdateConfig({ theme })}
                        className={`p-4 rounded-lg border-2 transition-colors ${
                          config.ui.theme === theme
                            ? 'border-blue-500 bg-blue-500/10'
                            : 'border-zinc-700 hover:border-zinc-600'
                        }`}
                      >
                        <div className="text-center">
                          <div className={`w-8 h-8 mx-auto rounded-full mb-2 ${
                            theme === 'light' ? 'bg-white border border-zinc-300' :
                            theme === 'dark' ? 'bg-zinc-900' :
                            'bg-gradient-to-r from-white to-zinc-900'
                          }`} />
                          <div className="text-sm capitalize">{theme}</div>
                        </div>
                      </button>
                    ))}
                  </div>
                </div>

                <div>
                  <label className="block text-sm text-zinc-400 mb-1">Font Size</label>
                  <input
                    type="range"
                    min="12"
                    max="20"
                    value={config.ui.fontSize}
                    onChange={(e) => handleUpdateConfig({ fontSize: parseInt(e.target.value) })}
                    className="w-full"
                  />
                  <div className="flex justify-between text-sm text-zinc-500 mt-1">
                    <span>12px</span>
                    <span>{config.ui.fontSize}px</span>
                    <span>20px</span>
                  </div>
                </div>

                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={config.ui.showTimestamps}
                    onChange={(e) => handleUpdateConfig({ showTimestamps: e.target.checked })}
                    className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-zinc-900"
                  />
                  <span className="text-sm text-zinc-300">
                    Show timestamps in messages
                  </span>
                </label>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-zinc-800 flex justify-between items-center">
          <div className="text-sm text-zinc-500">
            Press <kbd className="px-2 py-1 bg-zinc-800 rounded text-xs">Ctrl</kbd> + <kbd className="px-2 py-1 bg-zinc-800 rounded text-xs">,</kbd> to open settings
          </div>
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
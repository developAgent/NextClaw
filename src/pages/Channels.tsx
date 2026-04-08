import { useState, useEffect } from 'react';
import { Plus, Link2, Check, X, Edit, Trash2, Key, RefreshCw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface Channel {
  id: string;
  provider_type: string;
  name: string;
  enabled: boolean;
  health_status: string;
  created_at: string;
}

interface ChannelAccount {
  id: string;
  channel_id: string;
  name: string;
  credentials: string;
  is_default: boolean;
  created_at: string;
}

interface AccountFormData {
  name: string;
  credentials: string;
  is_default: boolean;
}

interface ChannelFormData {
  provider_type: string;
  name: string;
  config: string;
}

export default function Channels() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [accounts, setAccounts] = useState<Record<string, ChannelAccount[]>>({});
  const [selectedChannel, setSelectedChannel] = useState<Channel | null>(null);
  const [showAddChannel, setShowAddChannel] = useState(false);
  const [showAddAccount, setShowAddAccount] = useState(false);
  const [editingChannel, setEditingChannel] = useState<Channel | null>(null);
  const [loading, setLoading] = useState(false);

  const [channelForm, setChannelForm] = useState<ChannelFormData>({
    provider_type: 'openai',
    name: '',
    config: '{}',
  });

  const [accountForm, setAccountForm] = useState<AccountFormData>({
    name: '',
    credentials: '',
    is_default: false,
  });

  useEffect(() => {
    loadChannels();
  }, []);

  const loadChannels = async () => {
    try {
      const data = await invoke<Channel[]>('get_all_channels');
      setChannels(data);
      if (data.length > 0 && !selectedChannel) {
        await loadAccountsForChannel(data[0].id);
        setSelectedChannel(data[0]);
      }
    } catch (error) {
      console.error('Failed to load channels:', error);
    }
  };

  const loadAccountsForChannel = async (channelId: string) => {
    try {
      const data = await invoke<ChannelAccount[]>('get_channel_accounts', {
        channelId,
      });
      setAccounts((prev) => ({ ...prev, [channelId]: data }));
    } catch (error) {
      console.error('Failed to load accounts:', error);
    }
  };

  const selectChannel = async (channel: Channel) => {
    setSelectedChannel(channel);
    await loadAccountsForChannel(channel.id);
  };

  const handleCreateChannel = async () => {
    setLoading(true);
    try {
      await invoke('add_channel', {
        channel: {
          provider_type: channelForm.provider_type,
          name: channelForm.name,
          config: JSON.parse(channelForm.config),
          enabled: true,
          priority: 0,
        },
      });
      await loadChannels();
      setShowAddChannel(false);
      setChannelForm({ provider_type: 'openai', name: '', config: '{}' });
    } catch (error) {
      console.error('Failed to create channel:', error);
      alert('Failed to create channel');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateAccount = async () => {
    if (!selectedChannel) return;

    setLoading(true);
    try {
      await invoke('create_channel_account', {
        request: {
          channel_id: selectedChannel.id,
          name: accountForm.name,
          credentials: JSON.parse(accountForm.credentials),
          is_default: accountForm.is_default,
        },
      });
      await loadAccountsForChannel(selectedChannel.id);
      setShowAddAccount(false);
      setAccountForm({ name: '', credentials: '', is_default: false });
    } catch (error) {
      console.error('Failed to create account:', error);
      alert('Failed to create account');
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteChannel = async (id: string) => {
    if (!confirm('Are you sure you want to delete this channel?')) return;

    try {
      await invoke('delete_channel', { id });
      await loadChannels();
      if (selectedChannel?.id === id) {
        setSelectedChannel(null);
        setAccounts((prev) => {
          const updated = { ...prev };
          delete updated[id];
          return updated;
        });
      }
    } catch (error) {
      console.error('Failed to delete channel:', error);
      alert('Failed to delete channel');
    }
  };

  const handleDeleteAccount = async (id: string) => {
    if (!confirm('Are you sure you want to delete this account?')) return;

    try {
      await invoke('delete_channel_account', { id });
      await loadAccountsForChannel(selectedChannel!.id);
    } catch (error) {
      console.error('Failed to delete account:', error);
      alert('Failed to delete account');
    }
  };

  const handleSetDefaultAccount = async (accountId: string) => {
    try {
      await invoke('set_default_channel_account', { accountId });
      await loadAccountsForChannel(selectedChannel!.id);
    } catch (error) {
      console.error('Failed to set default account:', error);
      alert('Failed to set default account');
    }
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Channels</h1>
        <button
          onClick={() => setShowAddChannel(true)}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" />
          Add Channel
        </button>
      </div>

      {channels.length === 0 ? (
        <div className="flex items-center justify-center h-64 text-zinc-500">
          <div className="text-center">
            <Link2 className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p className="text-lg">No channels configured</p>
            <p className="text-sm">Connect AI providers and messaging platforms</p>
          </div>
        </div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Channels List */}
          <div className="space-y-4">
            <h2 className="text-lg font-medium">Providers</h2>
            {channels.map((channel) => (
              <div
                key={channel.id}
                onClick={() => selectChannel(channel)}
                className={`bg-zinc-900 border rounded-lg p-4 cursor-pointer transition-colors ${
                  selectedChannel?.id === channel.id
                    ? 'border-blue-500'
                    : 'border-zinc-800 hover:border-zinc-700'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="font-medium">{channel.name}</h3>
                    <p className="text-sm text-zinc-400">{channel.provider_type}</p>
                  </div>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteChannel(channel.id);
                    }}
                    className="p-2 hover:bg-red-600/20 hover:text-red-400 rounded-lg transition-colors"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
                <div className="flex items-center gap-4 mt-3">
                  <div className={`flex items-center gap-2 px-2 py-1 rounded text-xs ${
                    channel.enabled ? 'bg-green-600/20 text-green-400' : 'bg-zinc-700 text-zinc-400'
                  }`}>
                    {channel.enabled ? <Check className="w-3 h-3" /> : <X className="w-3 h-3" />}
                    <span>{channel.enabled ? 'Enabled' : 'Disabled'}</span>
                  </div>
                  <div className={`px-2 py-1 rounded text-xs ${
                    channel.health_status === 'healthy' ? 'bg-green-600/20 text-green-400' :
                    channel.health_status === 'degraded' ? 'bg-yellow-600/20 text-yellow-400' :
                    'bg-zinc-700 text-zinc-400'
                  }`}>
                    {channel.health_status}
                  </div>
                  {accounts[channel.id] && accounts[channel.id].length > 0 && (
                    <span className="text-xs text-zinc-500">
                      {accounts[channel.id].length} accounts
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>

          {/* Channel Details / Accounts */}
          {selectedChannel && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h2 className="text-lg font-medium">
                  {selectedChannel.name} - Accounts
                </h2>
                <button
                  onClick={() => setShowAddAccount(true)}
                  className="flex items-center gap-2 px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors text-sm"
                >
                  <Plus className="w-4 h-4" />
                  Add Account
                </button>
              </div>

              {accounts[selectedChannel.id]?.length === 0 ? (
                <div className="flex items-center justify-center h-32 text-zinc-500">
                  <div className="text-center">
                    <Key className="w-8 h-8 mx-auto mb-2 opacity-50" />
                    <p className="text-sm">No accounts yet</p>
                  </div>
                </div>
              ) : (
                <div className="space-y-3">
                  {accounts[selectedChannel.id].map((account) => (
                    <div
                      key={account.id}
                      className="bg-zinc-900 border border-zinc-800 rounded-lg p-4"
                    >
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                          <h4 className="font-medium">{account.name}</h4>
                          {account.is_default && (
                            <span className="px-2 py-0.5 bg-blue-600/20 text-blue-400 rounded text-xs">
                              Default
                            </span>
                          )}
                        </div>
                        <div className="flex items-center gap-1">
                          <button
                            onClick={() => handleSetDefaultAccount(account.id)}
                            className="p-1.5 hover:bg-zinc-800 rounded transition-colors"
                            title="Set as default"
                            disabled={account.is_default}
                          >
                            <Check className="w-3 h-3" />
                          </button>
                          <button
                            onClick={() => handleDeleteAccount(account.id)}
                            className="p-1.5 hover:bg-red-600/20 hover:text-red-400 rounded transition-colors"
                            title="Delete"
                          >
                            <Trash2 className="w-3 h-3" />
                          </button>
                        </div>
                      </div>
                      <p className="text-xs text-zinc-500">
                        Created: {new Date(account.created_at).toLocaleDateString()}
                      </p>
                    </div>
                  ))}
                </div>
              )}

              {/* Health Check */}
              <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                <h3 className="font-medium mb-3">Health Check</h3>
                <button
                  onClick={async () => {
                    try {
                      await invoke('check_channel_health', { id: selectedChannel.id });
                      // Refresh channels after health check
                      await loadChannels();
                    } catch (error) {
                      console.error('Health check failed:', error);
                    }
                  }}
                  disabled={loading}
                  className="flex items-center gap-2 px-3 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors text-sm"
                >
                  <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
                  Check Health
                </button>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Add Channel Modal */}
      {showAddChannel && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-zinc-900 rounded-lg p-6 max-w-md w-full">
            <h2 className="text-xl font-semibold mb-4">Add Channel</h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Provider Type</label>
                <select
                  value={channelForm.provider_type}
                  onChange={(e) => setChannelForm({ ...channelForm, provider_type: e.target.value })}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="openai">OpenAI</option>
                  <option value="anthropic">Anthropic</option>
                  <option value="moonshot">Moonshot</option>
                  <option value="custom">Custom</option>
                </select>
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Name</label>
                <input
                  type="text"
                  value={channelForm.name}
                  onChange={(e) => setChannelForm({ ...channelForm, name: e.target.value })}
                  placeholder="My Channel"
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Configuration (JSON)</label>
                <textarea
                  value={channelForm.config}
                  onChange={(e) => setChannelForm({ ...channelForm, config: e.target.value })}
                  placeholder='{"base_url": "https://api.example.com"}'
                  rows={4}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none font-mono text-sm"
                />
              </div>
              <div className="flex justify-end gap-2">
                <button
                  onClick={() => setShowAddChannel(false)}
                  className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleCreateChannel}
                  disabled={loading || !channelForm.name.trim()}
                  className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-zinc-700 disabled:cursor-not-allowed rounded-lg transition-colors"
                >
                  {loading ? 'Creating...' : 'Create'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Add Account Modal */}
      {showAddAccount && selectedChannel && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-zinc-900 rounded-lg p-6 max-w-md w-full">
            <h2 className="text-xl font-semibold mb-4">Add Account to {selectedChannel.name}</h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Account Name</label>
                <input
                  type="text"
                  value={accountForm.name}
                  onChange={(e) => setAccountForm({ ...accountForm, name: e.target.value })}
                  placeholder="My Account"
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Credentials (JSON)</label>
                <textarea
                  value={accountForm.credentials}
                  onChange={(e) => setAccountForm({ ...accountForm, credentials: e.target.value })}
                  placeholder='{"api_key": "sk-..."}'
                  rows={4}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none font-mono text-sm"
                />
              </div>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={accountForm.is_default}
                  onChange={(e) => setAccountForm({ ...accountForm, is_default: e.target.checked })}
                  className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-zinc-900"
                />
                <span className="text-sm text-zinc-300">Set as default</span>
              </label>
              <div className="flex justify-end gap-2">
                <button
                  onClick={() => setShowAddAccount(false)}
                  className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleCreateAccount}
                  disabled={loading || !accountForm.name.trim()}
                  className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-zinc-700 disabled:cursor-not-allowed rounded-lg transition-colors"
                >
                  {loading ? 'Creating...' : 'Create'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
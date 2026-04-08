import { useState, useEffect } from 'react';
import { Plus, Edit, Trash2, Copy, Zap, Check, X, Save, Settings as SettingsIcon } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface Agent {
  id: string;
  name: string;
  description?: string;
  provider_id?: string;
  model_id?: string;
  system_prompt?: string;
  temperature?: number;
  max_tokens?: number;
  created_at: string;
}

interface AgentForm {
  name: string;
  description: string;
  provider_id: string;
  model_id: string;
  system_prompt: string;
  temperature: number;
  max_tokens: number;
}

const defaultForm: AgentForm = {
  name: '',
  description: '',
  provider_id: 'openai',
  model_id: 'gpt-4o-mini',
  system_prompt: 'You are a helpful AI assistant.',
  temperature: 0.7,
  max_tokens: 4096,
};

export default function Agents() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isEditing, setIsEditing] = useState(false);
  const [editingAgent, setEditingAgent] = useState<Agent | null>(null);
  const [form, setForm] = useState<AgentForm>(defaultForm);
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadAgents();
  }, []);

  const loadAgents = async () => {
    try {
      const data = await invoke<Agent[]>('get_all_agents');
      setAgents(data);
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  };

  const handleCreate = async () => {
    setSaving(true);
    try {
      await invoke('create_agent', {
        request: form,
      });
      await loadAgents();
      handleCancel();
    } catch (error) {
      console.error('Failed to create agent:', error);
      alert('Failed to create agent');
    } finally {
      setSaving(false);
    }
  };

  const handleUpdate = async () => {
    if (!editingAgent) return;

    setSaving(true);
    try {
      await invoke('update_agent', {
        request: {
          id: editingAgent.id,
          name: form.name || undefined,
          description: form.description || undefined,
          provider_id: form.provider_id || undefined,
          model_id: form.model_id || undefined,
          system_prompt: form.system_prompt || undefined,
          temperature: form.temperature,
          max_tokens: form.max_tokens,
        },
      });
      await loadAgents();
      handleCancel();
    } catch (error) {
      console.error('Failed to update agent:', error);
      alert('Failed to update agent');
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this agent?')) return;

    try {
      await invoke('delete_agent', { id });
      await loadAgents();
    } catch (error) {
      console.error('Failed to delete agent:', error);
      alert('Failed to delete agent');
    }
  };

  const handleClone = async (agent: Agent) => {
    try {
      await invoke('clone_agent', {
        id: agent.id,
      });
      await loadAgents();
    } catch (error) {
      console.error('Failed to clone agent:', error);
      alert('Failed to clone agent');
    }
  };

  const handleEdit = (agent: Agent) => {
    setEditingAgent(agent);
    setForm({
      name: agent.name,
      description: agent.description || '',
      provider_id: agent.provider_id || 'openai',
      model_id: agent.model_id || 'gpt-4o-mini',
      system_prompt: agent.system_prompt || 'You are a helpful AI assistant.',
      temperature: agent.temperature ?? 0.7,
      max_tokens: agent.max_tokens ?? 4096,
    });
    setIsEditing(true);
    setShowForm(true);
  };

  const handleCancel = () => {
    setShowForm(false);
    setIsEditing(false);
    setEditingAgent(null);
    setForm(defaultForm);
  };

  const providerModels: Record<string, string[]> = {
    openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-3.5-turbo'],
    anthropic: ['claude-3-opus-20240229', 'claude-3-sonnet-20240229', 'claude-3-haiku-20240307'],
    custom: ['custom-model-1', 'custom-model-2'],
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Agents</h1>
        <button
          onClick={() => {
            setEditingAgent(null);
            setIsEditing(false);
            setShowForm(true);
          }}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" />
          New Agent
        </button>
      </div>

      {showForm && (
        <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-6 mb-6">
          <h2 className="text-lg font-semibold mb-4">
            {isEditing ? 'Edit Agent' : 'New Agent'}
          </h2>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Name *</label>
                <input
                  type="text"
                  value={form.name}
                  onChange={(e) => setForm({ ...form, name: e.target.value })}
                  placeholder="My Agent"
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Provider</label>
                <select
                  value={form.provider_id}
                  onChange={(e) => setForm({ ...form, provider_id: e.target.value, model_id: providerModels[e.target.value][0] })}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="openai">OpenAI</option>
                  <option value="anthropic">Anthropic</option>
                  <option value="custom">Custom</option>
                </select>
              </div>
            </div>

            <div>
              <label className="block text-sm text-zinc-400 mb-1">Model</label>
              <select
                value={form.model_id}
                onChange={(e) => setForm({ ...form, model_id: e.target.value })}
                className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                {providerModels[form.provider_id]?.map((model) => (
                  <option key={model} value={model}>{model}</option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm text-zinc-400 mb-1">Description</label>
              <textarea
                value={form.description}
                onChange={(e) => setForm({ ...form, description: e.target.value })}
                placeholder="Describe what this agent does"
                rows={2}
                className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
              />
            </div>

            <div>
              <label className="block text-sm text-zinc-400 mb-1">System Prompt</label>
              <textarea
                value={form.system_prompt}
                onChange={(e) => setForm({ ...form, system_prompt: e.target.value })}
                placeholder="You are a helpful AI assistant..."
                rows={3}
                className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Temperature ({form.temperature.toFixed(2)})</label>
                <input
                  type="range"
                  min="0"
                  max="2"
                  step="0.1"
                  value={form.temperature}
                  onChange={(e) => setForm({ ...form, temperature: parseFloat(e.target.value) })}
                  className="w-full"
                />
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Max Tokens</label>
                <input
                  type="number"
                  min="1"
                  max="128000"
                  value={form.max_tokens}
                  onChange={(e) => setForm({ ...form, max_tokens: parseInt(e.target.value) || 4096 })}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>

            <div className="flex justify-end gap-2">
              <button
                onClick={handleCancel}
                className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={isEditing ? handleUpdate : handleCreate}
                disabled={saving || !form.name.trim()}
                className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-zinc-700 disabled:cursor-not-allowed rounded-lg transition-colors"
              >
                {saving ? 'Saving...' : <><Save className="w-4 h-4" /> {isEditing ? 'Save' : 'Create'}</>}
              </button>
            </div>
          </div>
        </div>
      )}

      {agents.length === 0 ? (
        <div className="flex items-center justify-center h-64 text-zinc-500">
          <div className="text-center">
            <Zap className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p className="text-lg">No agents yet</p>
            <p className="text-sm">Create your first AI agent</p>
          </div>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {agents.map((agent) => (
            <div key={agent.id} className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
              <div className="flex items-start justify-between mb-3">
                <div className="flex-1">
                  <h3 className="font-medium">{agent.name}</h3>
                  <p className="text-sm text-zinc-400">{agent.description || 'No description'}</p>
                  <div className="flex items-center gap-2 mt-2">
                    <span className="px-2 py-0.5 bg-zinc-800 text-zinc-400 rounded text-xs">
                      {agent.provider_id}
                    </span>
                    <span className="px-2 py-0.5 bg-blue-600/20 text-blue-400 rounded text-xs">
                      {agent.model_id}
                    </span>
                  </div>
                </div>
              </div>
              {agent.system_prompt && (
                <div className="mb-3 p-2 bg-zinc-800 rounded">
                  <p className="text-xs text-zinc-400 line-clamp-2">{agent.system_prompt}</p>
                </div>
              )}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-xs text-zinc-500">
                  <span>Temp: {agent.temperature?.toFixed(2)}</span>
                  <span>Tokens: {agent.max_tokens}</span>
                </div>
                <div className="flex items-center gap-1">
                  <button
                    onClick={() => handleEdit(agent)}
                    className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
                    title="Edit"
                  >
                    <Edit className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => handleClone(agent)}
                    className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
                    title="Clone"
                  >
                    <Copy className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => handleDelete(agent.id)}
                    className="p-2 hover:bg-red-600/20 hover:text-red-400 rounded-lg transition-colors"
                    title="Delete"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
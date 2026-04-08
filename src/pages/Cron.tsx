import { useState, useEffect } from 'react';
import { Plus, Play, Clock, Trash2, Edit, Check, X } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface CronJob {
  id: string;
  name: string;
  description?: string;
  agent_id: string;
  channel_account_id?: string;
  cron_expression: string;
  message: string;
  target_config?: string;
  enabled: boolean;
  last_run?: string;
  next_run?: string;
  created_at: string;
  updated_at: string;
}

interface Agent {
  id: string;
  name: string;
}

interface ChannelAccount {
  id: string;
  name: string;
  channel_id: string;
}

interface CronFormData {
  name: string;
  description: string;
  agent_id: string;
  channel_account_id: string;
  cron_expression: string;
  message: string;
  target_config: string;
  enabled: boolean;
}

export default function Cron() {
  const [jobs, setJobs] = useState<CronJob[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [channelAccounts, setChannelAccounts] = useState<ChannelAccount[]>([]);
  const [showAddJob, setShowAddJob] = useState(false);
  const [editingJob, setEditingJob] = useState<CronJob | null>(null);
  const [loading, setLoading] = useState(false);
  const [executions, setExecutions] = useState<Record<string, any[]>>({});
  const [showExecutions, setShowExecutions] = useState<string | null>(null);

  const [form, setForm] = useState<CronFormData>({
    name: '',
    description: '',
    agent_id: '',
    channel_account_id: '',
    cron_expression: '0 */5 * * * *',
    message: '',
    target_config: '',
    enabled: true,
  });

  useEffect(() => {
    loadJobs();
    loadAgents();
    loadChannelAccounts();
  }, []);

  const loadJobs = async () => {
    try {
      const data = await invoke<CronJob[]>('get_all_cron_jobs');
      setJobs(data);
    } catch (error) {
      console.error('Failed to load jobs:', error);
    }
  };

  const loadAgents = async () => {
    try {
      const data = await invoke<Agent[]>('get_all_agents');
      setAgents(data);
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  };

  const loadChannelAccounts = async () => {
    try {
      const data = await invoke<ChannelAccount[]>('get_all_channel_accounts');
      setChannelAccounts(data);
    } catch (error) {
      console.error('Failed to load channel accounts:', error);
    }
  };

  const loadExecutions = async (jobId: string) => {
    try {
      const data = await invoke<any[]>('get_cron_executions', {
        jobId,
        limit: 10,
      });
      setExecutions((prev) => ({ ...prev, [jobId]: data }));
    } catch (error) {
      console.error('Failed to load executions:', error);
    }
  };

  const handleCreateJob = async () => {
    setLoading(true);
    try {
      await invoke('create_cron_job', {
        request: {
          name: form.name,
          description: form.description || undefined,
          agent_id: form.agent_id,
          channel_account_id: form.channel_account_id || undefined,
          cron_expression: form.cron_expression,
          message: form.message,
          target_config: form.target_config || undefined,
          enabled: form.enabled,
        },
      });
      await loadJobs();
      setShowAddJob(false);
      resetForm();
    } catch (error) {
      console.error('Failed to create job:', error);
      alert('Failed to create cron job');
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateJob = async () => {
    if (!editingJob) return;

    setLoading(true);
    try {
      await invoke('update_cron_job', {
        request: {
          id: editingJob.id,
          name: form.name || undefined,
          description: form.description || undefined,
          agent_id: form.agent_id || undefined,
          channel_account_id: form.channel_account_id || undefined,
          cron_expression: form.cron_expression || undefined,
          message: form.message || undefined,
          target_config: form.target_config || undefined,
          enabled: form.enabled,
        },
      });
      await loadJobs();
      setEditingJob(null);
      setShowAddJob(false);
      resetForm();
    } catch (error) {
      console.error('Failed to update job:', error);
      alert('Failed to update cron job');
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteJob = async (id: string) => {
    if (!confirm('Are you sure you want to delete this job?')) return;

    try {
      await invoke('delete_cron_job', { id });
      await loadJobs();
    } catch (error) {
      console.error('Failed to delete job:', error);
      alert('Failed to delete cron job');
    }
  };

  const handleRunJob = async (id: string) => {
    try {
      await invoke('run_cron_job', { id });
      await loadJobs();
      await loadExecutions(id);
    } catch (error) {
      console.error('Failed to run job:', error);
      alert('Failed to run cron job');
    }
  };

  const handleToggleEnabled = async (job: CronJob) => {
    try {
      await invoke('update_cron_job', {
        request: {
          id: job.id,
          enabled: !job.enabled,
        },
      });
      await loadJobs();
    } catch (error) {
      console.error('Failed to toggle job:', error);
      alert('Failed to toggle cron job');
    }
  };

  const handleEditJob = (job: CronJob) => {
    setEditingJob(job);
    setForm({
      name: job.name,
      description: job.description || '',
      agent_id: job.agent_id,
      channel_account_id: job.channel_account_id || '',
      cron_expression: job.cron_expression,
      message: job.message,
      target_config: job.target_config || '',
      enabled: job.enabled,
    });
    setShowAddJob(true);
  };

  const resetForm = () => {
    setForm({
      name: '',
      description: '',
      agent_id: '',
      channel_account_id: '',
      cron_expression: '0 */5 * * * *',
      message: '',
      target_config: '',
      enabled: true,
    });
    setEditingJob(null);
  };

  const getAgentName = (agentId: string) => {
    const agent = agents.find((a) => a.id === agentId);
    return agent?.name || 'Unknown Agent';
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Cron Jobs</h1>
        <button
          onClick={() => {
            resetForm();
            setShowAddJob(true);
          }}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" />
          New Job
        </button>
      </div>

      {jobs.length === 0 ? (
        <div className="flex items-center justify-center h-64 text-zinc-500">
          <div className="text-center">
            <Clock className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p className="text-lg">No scheduled tasks</p>
            <p className="text-sm">Create automated AI tasks with cron expressions</p>
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          {jobs.map((job) => (
            <div
              key={job.id}
              className="bg-zinc-900 border border-zinc-800 rounded-lg p-4"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h3 className="font-medium">{job.name}</h3>
                    {job.enabled ? (
                      <span className="flex items-center gap-1 px-2 py-0.5 bg-green-600/20 text-green-400 rounded text-xs">
                        <Check className="w-3 h-3" />
                        Enabled
                      </span>
                    ) : (
                      <span className="flex items-center gap-1 px-2 py-0.5 bg-zinc-700 text-zinc-400 rounded text-xs">
                        <X className="w-3 h-3" />
                        Disabled
                      </span>
                    )}
                  </div>
                  {job.description && (
                    <p className="text-sm text-zinc-400 mt-1">{job.description}</p>
                  )}
                  <div className="flex items-center gap-4 mt-2 text-sm text-zinc-500">
                    <span>Agent: {getAgentName(job.agent_id)}</span>
                    <code className="bg-zinc-800 px-2 py-0.5 rounded text-xs">
                      {job.cron_expression}
                    </code>
                  </div>
                  {job.next_run && (
                    <p className="text-xs text-zinc-500 mt-2">
                      Next run: {new Date(job.next_run).toLocaleString()}
                    </p>
                  )}
                  {job.last_run && (
                    <p className="text-xs text-zinc-500">
                      Last run: {new Date(job.last_run).toLocaleString()}
                    </p>
                  )}
                </div>
                <div className="flex items-center gap-1 ml-4">
                  <button
                    onClick={() => handleRunJob(job.id)}
                    className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
                    title="Run now"
                  >
                    <Play className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => {
                      setShowExecutions(job.id);
                      loadExecutions(job.id);
                    }}
                    className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
                    title="View executions"
                  >
                    <Clock className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => handleToggleEnabled(job)}
                    className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
                    title={job.enabled ? 'Disable' : 'Enable'}
                  >
                    {job.enabled ? <X className="w-4 h-4" /> : <Check className="w-4 h-4" />}
                  </button>
                  <button
                    onClick={() => handleEditJob(job)}
                    className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
                    title="Edit"
                  >
                    <Edit className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => handleDeleteJob(job.id)}
                    className="p-2 hover:bg-red-600/20 hover:text-red-400 rounded-lg transition-colors"
                    title="Delete"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>

              {showExecutions === job.id && (
                <div className="mt-4 pt-4 border-t border-zinc-800">
                  <h4 className="text-sm font-medium mb-2">Recent Executions</h4>
                  {executions[job.id]?.length === 0 ? (
                    <p className="text-sm text-zinc-500">No executions yet</p>
                  ) : (
                    <div className="space-y-2">
                      {executions[job.id].map((exec) => (
                        <div
                          key={exec.id}
                          className="bg-zinc-800 rounded px-3 py-2 text-sm"
                        >
                          <div className="flex items-center justify-between">
                            <span className="flex items-center gap-2">
                              {exec.status === 'success' && (
                                <Check className="w-3 h-3 text-green-400" />
                              )}
                              {exec.status === 'failed' && (
                                <X className="w-3 h-3 text-red-400" />
                              )}
                              {exec.status === 'running' && (
                                <Clock className="w-3 h-3 text-blue-400" />
                              )}
                              {new Date(exec.started_at).toLocaleString()}
                            </span>
                          </div>
                          {exec.output && (
                            <p className="text-xs text-zinc-400 mt-1">{exec.output}</p>
                          )}
                          {exec.error && (
                            <p className="text-xs text-red-400 mt-1">{exec.error}</p>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Add/Edit Job Modal */}
      {showAddJob && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-zinc-900 rounded-lg p-6 max-w-lg w-full max-h-[90vh] overflow-y-auto">
            <h2 className="text-xl font-semibold mb-4">
              {editingJob ? 'Edit Cron Job' : 'Create Cron Job'}
            </h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Name</label>
                <input
                  type="text"
                  value={form.name}
                  onChange={(e) => setForm({ ...form, name: e.target.value })}
                  placeholder="My Scheduled Task"
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Description</label>
                <textarea
                  value={form.description}
                  onChange={(e) => setForm({ ...form, description: e.target.value })}
                  placeholder="Optional description"
                  rows={2}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
                />
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Agent</label>
                <select
                  value={form.agent_id}
                  onChange={(e) => setForm({ ...form, agent_id: e.target.value })}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="">Select an agent</option>
                  {agents.map((agent) => (
                    <option key={agent.id} value={agent.id}>
                      {agent.name}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Channel Account (Optional)</label>
                <select
                  value={form.channel_account_id}
                  onChange={(e) => setForm({ ...form, channel_account_id: e.target.value })}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="">Use default account</option>
                  {channelAccounts.map((account) => (
                    <option key={account.id} value={account.id}>
                      {account.name}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Cron Expression</label>
                <input
                  type="text"
                  value={form.cron_expression}
                  onChange={(e) => setForm({ ...form, cron_expression: e.target.value })}
                  placeholder="0 */5 * * * *"
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                />
                <p className="text-xs text-zinc-500 mt-1">
                  Format: second minute hour day month weekday
                </p>
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Message</label>
                <textarea
                  value={form.message}
                  onChange={(e) => setForm({ ...form, message: e.target.value })}
                  placeholder="Message to send to the agent"
                  rows={3}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
                />
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Target Config (Optional, JSON)</label>
                <textarea
                  value={form.target_config}
                  onChange={(e) => setForm({ ...form, target_config: e.target.value })}
                  placeholder='{"webhook_url": "https://..."}'
                  rows={2}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none font-mono text-sm"
                />
              </div>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={form.enabled}
                  onChange={(e) => setForm({ ...form, enabled: e.target.checked })}
                  className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-zinc-900"
                />
                <span className="text-sm text-zinc-300">Enabled</span>
              </label>
              <div className="flex justify-end gap-2">
                <button
                  onClick={() => {
                    setShowAddJob(false);
                    resetForm();
                  }}
                  className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={editingJob ? handleUpdateJob : handleCreateJob}
                  disabled={loading || !form.name.trim() || !form.agent_id.trim()}
                  className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-zinc-700 disabled:cursor-not-allowed rounded-lg transition-colors"
                >
                  {loading ? 'Saving...' : editingJob ? 'Update' : 'Create'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
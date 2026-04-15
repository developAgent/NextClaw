import { useEffect, useMemo, useState } from 'react';
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

interface CronExecution {
  id: string;
  status: string;
  started_at: string;
  completed_at?: string;
  output?: string;
  error?: string;
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

type TargetConfig = Record<string, unknown>;

interface TargetConfigValidation {
  error: string | null;
  formatted: string;
  parsed: TargetConfig | null;
}

const defaultForm: CronFormData = {
  name: '',
  description: '',
  agent_id: '',
  channel_account_id: '',
  cron_expression: '0 */5 * * * *',
  message: '',
  target_config: '',
  enabled: true,
};

const targetConfigPresets = {
  webhook: JSON.stringify(
    {
      provider: 'webhook',
      webhook_url: 'https://example.com/webhooks/cron',
      method: 'POST',
      headers: {
        'X-Source': 'nextclaw-cron',
      },
    },
    null,
    2,
  ),
  slack: JSON.stringify(
    {
      provider: 'slack',
      webhook_url: 'https://hooks.slack.com/services/XXX/YYY/ZZZ',
    },
    null,
    2,
  ),
  authenticatedWebhook: JSON.stringify(
    {
      provider: 'webhook',
      webhook_url: 'https://example.com/notify',
      bearer_token: 'replace-me',
      content_type: 'application/json',
    },
    null,
    2,
  ),
};

function normalizeOptionalString(value: string): string | undefined {
  const trimmed = value.trim();
  return trimmed ? trimmed : undefined;
}

function getErrorMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }

  if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') {
    return error.message;
  }

  return 'Unknown error';
}

function validateTargetConfig(raw: string): TargetConfigValidation {
  const trimmed = raw.trim();
  if (!trimmed) {
    return { error: null, formatted: '', parsed: null };
  }

  try {
    const parsed = JSON.parse(trimmed) as unknown;
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      return {
        error: 'Target config must be a JSON object.',
        formatted: raw,
        parsed: null,
      };
    }

    return {
      error: null,
      formatted: JSON.stringify(parsed, null, 2),
      parsed: parsed as TargetConfig,
    };
  } catch (error) {
    return {
      error: `Invalid JSON: ${getErrorMessage(error)}`,
      formatted: raw,
      parsed: null,
    };
  }
}

function getStringValue(config: TargetConfig | null, keys: string[]): string | null {
  if (!config) {
    return null;
  }

  for (const key of keys) {
    const value = config[key];
    if (typeof value === 'string' && value.trim()) {
      return value.trim();
    }
  }

  return null;
}

function describeDelivery(job: CronJob, accounts: ChannelAccount[]): string {
  const validation = validateTargetConfig(job.target_config ?? '');
  const provider = getStringValue(validation.parsed, ['provider']);
  const endpoint = getStringValue(validation.parsed, ['webhook_url', 'url', 'endpoint', 'base_url']);
  const account = accounts.find((item) => item.id === job.channel_account_id);

  const parts: string[] = [];
  if (provider) {
    parts.push(provider);
  }
  if (account) {
    parts.push(`account ${account.name}`);
  } else if (job.channel_account_id) {
    parts.push('selected account');
  }
  if (endpoint) {
    parts.push(endpoint);
  }

  if (parts.length > 0) {
    return parts.join(' · ');
  }

  if (job.channel_account_id) {
    return 'Uses selected channel account settings';
  }

  return 'No delivery target override';
}

export default function Cron() {
  const [jobs, setJobs] = useState<CronJob[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [channelAccounts, setChannelAccounts] = useState<ChannelAccount[]>([]);
  const [showAddJob, setShowAddJob] = useState(false);
  const [editingJob, setEditingJob] = useState<CronJob | null>(null);
  const [loading, setLoading] = useState(false);
  const [pageError, setPageError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [executions, setExecutions] = useState<Record<string, CronExecution[]>>({});
  const [showExecutions, setShowExecutions] = useState<string | null>(null);

  const [form, setForm] = useState<CronFormData>(defaultForm);

  const targetConfigValidation = useMemo(
    () => validateTargetConfig(form.target_config),
    [form.target_config],
  );

  useEffect(() => {
    void loadJobs();
    void loadAgents();
    void loadChannelAccounts();
  }, []);

  const loadJobs = async () => {
    try {
      const data = await invoke<CronJob[]>('get_all_cron_jobs');
      setJobs(data);
      setPageError(null);
    } catch (error) {
      console.error('Failed to load jobs:', error);
      setPageError(`Failed to load cron jobs: ${getErrorMessage(error)}`);
    }
  };

  const loadAgents = async () => {
    try {
      const data = await invoke<Agent[]>('get_all_agents');
      setAgents(data);
    } catch (error) {
      console.error('Failed to load agents:', error);
      setPageError(`Failed to load agents: ${getErrorMessage(error)}`);
    }
  };

  const loadChannelAccounts = async () => {
    try {
      const data = await invoke<ChannelAccount[]>('get_all_channel_accounts');
      setChannelAccounts(data);
    } catch (error) {
      console.error('Failed to load channel accounts:', error);
      setPageError(`Failed to load channel accounts: ${getErrorMessage(error)}`);
    }
  };

  const loadExecutions = async (jobId: string) => {
    try {
      const data = await invoke<CronExecution[]>('get_cron_executions', {
        job_id: jobId,
        limit: 10,
      });
      setExecutions((prev) => ({ ...prev, [jobId]: data }));
      setPageError(null);
    } catch (error) {
      console.error('Failed to load executions:', error);
      setPageError(`Failed to load executions: ${getErrorMessage(error)}`);
    }
  };

  const getRequestPayload = () => {
    if (targetConfigValidation.error) {
      throw new Error(targetConfigValidation.error);
    }

    return {
      name: normalizeOptionalString(form.name),
      description: normalizeOptionalString(form.description),
      agent_id: normalizeOptionalString(form.agent_id),
      channel_account_id: normalizeOptionalString(form.channel_account_id),
      cron_expression: normalizeOptionalString(form.cron_expression),
      message: normalizeOptionalString(form.message),
      target_config: targetConfigValidation.formatted || undefined,
      enabled: form.enabled,
    };
  };

  const handleCreateJob = async () => {
    setLoading(true);
    setFormError(null);
    try {
      const request = getRequestPayload();
      await invoke('create_cron_job', { request });
      await loadJobs();
      setShowAddJob(false);
      resetForm();
    } catch (error) {
      const message = getErrorMessage(error);
      console.error('Failed to create job:', error);
      setFormError(`Failed to create cron job: ${message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateJob = async () => {
    if (!editingJob) return;

    setLoading(true);
    setFormError(null);
    try {
      const payload = getRequestPayload();
      await invoke('update_cron_job', {
        request: {
          id: editingJob.id,
          name: payload.name,
          description: payload.description,
          agent_id: payload.agent_id,
          channel_account_id: payload.channel_account_id,
          cron_expression: payload.cron_expression,
          message: payload.message,
          target_config: payload.target_config,
          enabled: payload.enabled,
        },
      });
      await loadJobs();
      setEditingJob(null);
      setShowAddJob(false);
      resetForm();
    } catch (error) {
      const message = getErrorMessage(error);
      console.error('Failed to update job:', error);
      setFormError(`Failed to update cron job: ${message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteJob = async (id: string) => {
    if (!confirm('Are you sure you want to delete this job?')) return;

    try {
      await invoke('delete_cron_job', { id });
      await loadJobs();
      setPageError(null);
    } catch (error) {
      console.error('Failed to delete job:', error);
      setPageError(`Failed to delete cron job: ${getErrorMessage(error)}`);
    }
  };

  const handleRunJob = async (id: string) => {
    try {
      await invoke('run_cron_job', { id });
      await loadJobs();
      await loadExecutions(id);
      setShowExecutions(id);
      setPageError(null);
    } catch (error) {
      console.error('Failed to run job:', error);
      setPageError(`Failed to run cron job: ${getErrorMessage(error)}`);
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
      setPageError(null);
    } catch (error) {
      console.error('Failed to toggle job:', error);
      setPageError(`Failed to toggle cron job: ${getErrorMessage(error)}`);
    }
  };

  const handleEditJob = (job: CronJob) => {
    setEditingJob(job);
    setFormError(null);
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
    setForm(defaultForm);
    setFormError(null);
    setEditingJob(null);
  };

  const applyTargetPreset = (preset: keyof typeof targetConfigPresets) => {
    setForm((current) => ({
      ...current,
      target_config: targetConfigPresets[preset],
    }));
    setFormError(null);
  };

  const getAgentName = (agentId: string) => {
    const agent = agents.find((a) => a.id === agentId);
    return agent?.name || 'Unknown Agent';
  };

  const getExecutionStatus = (status: string) => status.toLowerCase();

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold">Cron Jobs</h1>
          <p className="text-sm text-zinc-500 mt-1">
            Schedule an agent run, optionally deliver the result through a channel account, and keep execution history.
          </p>
        </div>
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

      {pageError && (
        <div className="mb-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
          {pageError}
        </div>
      )}

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
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
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
                  <div className="flex items-center gap-4 mt-2 text-sm text-zinc-500 flex-wrap">
                    <span>Agent: {getAgentName(job.agent_id)}</span>
                    <code className="bg-zinc-800 px-2 py-0.5 rounded text-xs">
                      {job.cron_expression}
                    </code>
                  </div>
                  <p className="text-xs text-zinc-500 mt-2">
                    Delivery: {describeDelivery(job, channelAccounts)}
                  </p>
                  {job.target_config && (
                    <details className="mt-2">
                      <summary className="cursor-pointer text-xs text-zinc-400 hover:text-zinc-300">
                        View target config override
                      </summary>
                      <pre className="mt-2 overflow-x-auto rounded bg-zinc-950 p-3 text-xs text-zinc-300 whitespace-pre-wrap break-words">
                        {job.target_config}
                      </pre>
                    </details>
                  )}
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
                <div className="flex items-center gap-1 ml-4 shrink-0">
                  <button
                    onClick={() => void handleRunJob(job.id)}
                    className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
                    title="Run now"
                  >
                    <Play className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => {
                      if (showExecutions === job.id) {
                        setShowExecutions(null);
                        return;
                      }
                      setShowExecutions(job.id);
                      void loadExecutions(job.id);
                    }}
                    className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
                    title="View executions"
                  >
                    <Clock className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => void handleToggleEnabled(job)}
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
                    onClick={() => void handleDeleteJob(job.id)}
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
                      {executions[job.id]?.map((exec) => {
                        const status = getExecutionStatus(exec.status);

                        return (
                          <div
                            key={exec.id}
                            className="bg-zinc-800 rounded px-3 py-2 text-sm"
                          >
                            <div className="flex items-center justify-between gap-4">
                              <span className="flex items-center gap-2">
                                {status === 'success' && (
                                  <Check className="w-3 h-3 text-green-400" />
                                )}
                                {status === 'failed' && (
                                  <X className="w-3 h-3 text-red-400" />
                                )}
                                {status === 'running' && (
                                  <Clock className="w-3 h-3 text-blue-400" />
                                )}
                                {new Date(exec.started_at).toLocaleString()}
                              </span>
                              {exec.completed_at && (
                                <span className="text-xs text-zinc-500">
                                  Completed {new Date(exec.completed_at).toLocaleString()}
                                </span>
                              )}
                            </div>
                            {exec.output && (
                              <pre className="text-xs text-zinc-400 mt-2 whitespace-pre-wrap break-words">
                                {exec.output}
                              </pre>
                            )}
                            {exec.error && (
                              <p className="text-xs text-red-400 mt-2 whitespace-pre-wrap break-words">{exec.error}</p>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {showAddJob && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-zinc-900 rounded-lg p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto">
            <h2 className="text-xl font-semibold mb-4">
              {editingJob ? 'Edit Cron Job' : 'Create Cron Job'}
            </h2>

            {formError && (
              <div className="mb-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
                {formError}
              </div>
            )}

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
                  <option value="">No explicit account</option>
                  {channelAccounts.map((account) => (
                    <option key={account.id} value={account.id}>
                      {account.name}
                    </option>
                  ))}
                </select>
                <p className="text-xs text-zinc-500 mt-1">
                  If selected, the scheduler uses this account and its channel settings for delivery. You can still override fields below.
                </p>
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
                  rows={4}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
                />
              </div>
              <div className="rounded-lg border border-zinc-800 bg-zinc-950/60 p-4">
                <div className="flex items-center justify-between gap-3 flex-wrap">
                  <div>
                    <label className="block text-sm text-zinc-400 mb-1">Target Config Override (Optional JSON)</label>
                    <p className="text-xs text-zinc-500">
                      Priority order is target config → channel account credentials → channel config.
                    </p>
                  </div>
                  <div className="flex gap-2 flex-wrap text-xs">
                    <button
                      type="button"
                      onClick={() => applyTargetPreset('webhook')}
                      className="px-2 py-1 rounded bg-zinc-800 hover:bg-zinc-700 transition-colors"
                    >
                      Webhook preset
                    </button>
                    <button
                      type="button"
                      onClick={() => applyTargetPreset('slack')}
                      className="px-2 py-1 rounded bg-zinc-800 hover:bg-zinc-700 transition-colors"
                    >
                      Slack preset
                    </button>
                    <button
                      type="button"
                      onClick={() => applyTargetPreset('authenticatedWebhook')}
                      className="px-2 py-1 rounded bg-zinc-800 hover:bg-zinc-700 transition-colors"
                    >
                      Auth preset
                    </button>
                  </div>
                </div>
                <textarea
                  value={form.target_config}
                  onChange={(e) => setForm({ ...form, target_config: e.target.value })}
                  placeholder={'{\n  "provider": "webhook",\n  "webhook_url": "https://..."\n}'}
                  rows={10}
                  className="w-full mt-3 bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-y font-mono text-sm"
                />
                {targetConfigValidation.error ? (
                  <p className="text-xs text-red-400 mt-2">{targetConfigValidation.error}</p>
                ) : (
                  <div className="mt-2 space-y-1 text-xs text-zinc-500">
                    <p>Supported keys include provider, webhook_url/url/endpoint/base_url, method, headers, bearer_token, authorization, and content_type.</p>
                    <p>Use provider "slack" for Slack webhooks, or provider "webhook" for generic HTTP delivery.</p>
                  </div>
                )}
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
                  onClick={() => void (editingJob ? handleUpdateJob() : handleCreateJob())}
                  disabled={
                    loading
                    || !form.name.trim()
                    || !form.agent_id.trim()
                    || !form.cron_expression.trim()
                    || !form.message.trim()
                    || Boolean(targetConfigValidation.error)
                  }
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

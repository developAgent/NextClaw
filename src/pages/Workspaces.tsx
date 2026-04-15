import { useEffect, useMemo, useState } from 'react';
import { AlertCircle, Check, FolderKanban, Plus } from 'lucide-react';
import { useWorkspaceStore } from '@/store/workspaces';

export default function Workspaces() {
  const { workspaces, currentWorkspace, loading, error, refresh, create, select } = useWorkspaceStore();
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const canCreate = useMemo(() => name.trim().length > 0 && !loading, [loading, name]);

  const handleCreate = async () => {
    if (!name.trim()) {
      return;
    }

    await create({
      name: name.trim(),
      description: description.trim() || undefined,
    });

    setName('');
    setDescription('');
  };

  return (
    <div className="p-6">
      <div className="mx-auto max-w-6xl space-y-6">
        <div className="rounded-2xl border border-zinc-800 bg-zinc-900/80 p-6">
          <div className="flex items-start justify-between gap-4">
            <div>
              <h1 className="text-2xl font-semibold">Workspaces</h1>
              <p className="mt-1 text-sm text-zinc-500">
                Create isolated product contexts and choose the active workspace.
              </p>
            </div>
            <button
              onClick={() => void refresh()}
              disabled={loading}
              className="rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm text-zinc-100 transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-60"
            >
              {loading ? 'Refreshing…' : 'Refresh'}
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

        <div className="grid gap-6 lg:grid-cols-[1.2fr_0.8fr]">
          <section className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-6">
            <div className="flex items-center justify-between gap-4">
              <div>
                <h2 className="text-lg font-medium">Available workspaces</h2>
                <p className="mt-1 text-sm text-zinc-500">
                  The selected workspace is persisted in the settings table.
                </p>
              </div>
              <div className="text-sm text-zinc-500">{workspaces.length} total</div>
            </div>

            {workspaces.length === 0 ? (
              <div className="flex h-72 items-center justify-center text-zinc-500">
                <div className="text-center">
                  <FolderKanban className="mx-auto mb-4 h-12 w-12 opacity-50" />
                  <p className="text-lg">No workspaces yet</p>
                  <p className="text-sm">Create one to start organizing runtime state.</p>
                </div>
              </div>
            ) : (
              <div className="mt-6 space-y-3">
                {workspaces.map((workspace) => {
                  const isCurrent = workspace.isCurrent;
                  return (
                    <article
                      key={workspace.id}
                      className={`rounded-xl border p-4 transition-colors ${
                        isCurrent
                          ? 'border-blue-500/40 bg-blue-600/10'
                          : 'border-zinc-800 bg-zinc-950/60'
                      }`}
                    >
                      <div className="flex items-start justify-between gap-4">
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2">
                            <h3 className="truncate text-base font-medium text-zinc-100">{workspace.name}</h3>
                            {isCurrent && (
                              <span className="rounded-full border border-blue-500/30 bg-blue-600/15 px-2 py-0.5 text-[11px] font-medium uppercase tracking-[0.2em] text-blue-300">
                                Current
                              </span>
                            )}
                          </div>
                          <p className="mt-2 text-sm text-zinc-400">
                            {workspace.description || 'No description provided.'}
                          </p>
                          <div className="mt-3 text-xs text-zinc-500">
                            Created {new Date(workspace.createdAt).toLocaleString()}
                          </div>
                        </div>
                        <button
                          onClick={() => void select(workspace.id)}
                          disabled={loading || isCurrent}
                          className="inline-flex items-center gap-2 rounded-lg bg-zinc-800 px-3 py-2 text-sm text-zinc-100 transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:bg-zinc-800/50 disabled:text-zinc-500"
                        >
                          {isCurrent ? <Check className="h-4 w-4 text-green-400" /> : null}
                          {isCurrent ? 'Selected' : 'Switch'}
                        </button>
                      </div>
                    </article>
                  );
                })}
              </div>
            )}
          </section>

          <section className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-6">
            <h2 className="text-lg font-medium">Create workspace</h2>
            <p className="mt-1 text-sm text-zinc-500">
              Start with lightweight isolation for logs, runtime settings, and future skill state.
            </p>

            <div className="mt-6 space-y-4">
              <div>
                <label className="mb-1 block text-sm text-zinc-400">Name</label>
                <input
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder="Workspace name"
                  className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition-colors focus:border-blue-500"
                />
              </div>
              <div>
                <label className="mb-1 block text-sm text-zinc-400">Description</label>
                <textarea
                  value={description}
                  onChange={(event) => setDescription(event.target.value)}
                  placeholder="Optional description"
                  rows={4}
                  className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition-colors focus:border-blue-500 resize-none"
                />
              </div>
              <button
                onClick={() => void handleCreate()}
                disabled={!canCreate}
                className="inline-flex w-full items-center justify-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:bg-zinc-700"
              >
                <Plus className="h-4 w-4" />
                Create workspace
              </button>
            </div>

            <div className="mt-6 rounded-xl border border-zinc-800 bg-zinc-950/60 p-4 text-sm text-zinc-400">
              <div className="font-medium text-zinc-200">Current workspace</div>
              <div className="mt-2">{currentWorkspace?.name ?? 'None selected'}</div>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}

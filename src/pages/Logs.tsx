import { useEffect, useMemo, useState } from 'react';
import { AlertCircle, FileSearch, Filter, RefreshCw } from 'lucide-react';
import { useLogStore } from '@/store/logs';

const logLevels = ['all', 'error', 'warn', 'info', 'debug'] as const;

type LogLevelFilter = (typeof logLevels)[number];

export default function LogsPage() {
  const { logs, loading, error, refresh } = useLogStore();
  const [level, setLevel] = useState<LogLevelFilter>('all');
  const [query, setQuery] = useState('');

  useEffect(() => {
    void refresh(level === 'all' ? undefined : level);
  }, [level, refresh]);

  const filteredLogs = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    if (!normalizedQuery) {
      return logs;
    }

    return logs.filter((log) => {
      const haystacks = [log.level, log.message, log.timestamp, log.context ?? ''];
      return haystacks.some((value) => value.toLowerCase().includes(normalizedQuery));
    });
  }, [logs, query]);

  return (
    <div className="p-6">
      <div className="mx-auto max-w-6xl space-y-6">
        <div className="flex items-start justify-between gap-4 rounded-2xl border border-zinc-800 bg-zinc-900/80 p-6">
          <div>
            <h1 className="text-2xl font-semibold">Logs</h1>
            <p className="mt-1 text-sm text-zinc-500">
              Inspect diagnostic events stored in the app log table.
            </p>
          </div>
          <button
            onClick={() => void refresh(level === 'all' ? undefined : level)}
            disabled={loading}
            className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm text-zinc-100 transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-60"
          >
            <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </button>
        </div>

        {error && (
          <div className="rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
            <div className="flex items-start gap-2">
              <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" />
              <span>{error}</span>
            </div>
          </div>
        )}

        <div className="grid gap-4 lg:grid-cols-[220px_1fr]">
          <aside className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4">
            <div className="flex items-center gap-2 text-sm font-medium text-zinc-200">
              <Filter className="h-4 w-4" />
              Filter level
            </div>
            <div className="mt-4 space-y-2">
              {logLevels.map((item) => {
                const active = item === level;
                return (
                  <button
                    key={item}
                    onClick={() => setLevel(item)}
                    className={`w-full rounded-lg border px-3 py-2 text-left text-sm transition-colors ${
                      active
                        ? 'border-blue-500/40 bg-blue-600/15 text-white'
                        : 'border-zinc-800 bg-zinc-950/50 text-zinc-400 hover:bg-zinc-800/80 hover:text-zinc-100'
                    }`}
                  >
                    {item.toUpperCase()}
                  </button>
                );
              })}
            </div>
          </aside>

          <section className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4">
            <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
              <div className="relative w-full sm:max-w-md">
                <FileSearch className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-500" />
                <input
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                  placeholder="Search message or context"
                  className="w-full rounded-lg border border-zinc-700 bg-zinc-950 pl-10 pr-4 py-2 text-sm text-zinc-100 outline-none transition-colors focus:border-blue-500"
                />
              </div>
              <div className="text-sm text-zinc-500">
                {filteredLogs.length} of {logs.length} entries
              </div>
            </div>

            {filteredLogs.length === 0 ? (
              <div className="flex h-72 items-center justify-center text-zinc-500">
                <div className="text-center">
                  <FileSearch className="mx-auto mb-4 h-12 w-12 opacity-50" />
                  <p className="text-lg">No matching logs</p>
                  <p className="text-sm">Adjust the level filter or search query.</p>
                </div>
              </div>
            ) : (
              <div className="space-y-3">
                {filteredLogs.map((log) => (
                  <article key={log.id} className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
                    <div className="flex flex-wrap items-center gap-3">
                      <span className="rounded-full border border-zinc-700 bg-zinc-900 px-2 py-1 text-[11px] font-medium uppercase tracking-[0.2em] text-zinc-300">
                        {log.level}
                      </span>
                      <span className="text-xs text-zinc-500">{new Date(log.timestamp).toLocaleString()}</span>
                    </div>
                    <p className="mt-3 text-sm text-zinc-100 whitespace-pre-wrap break-words">{log.message}</p>
                    {log.context && (
                      <details className="mt-3">
                        <summary className="cursor-pointer text-xs text-zinc-400 hover:text-zinc-300">
                          View context
                        </summary>
                        <pre className="mt-2 overflow-x-auto rounded-lg bg-zinc-900 p-3 text-xs text-zinc-300 whitespace-pre-wrap break-words">
                          {log.context}
                        </pre>
                      </details>
                    )}
                  </article>
                ))}
              </div>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}

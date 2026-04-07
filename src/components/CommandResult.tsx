import { CommandExecution } from '@/types';
import { CheckCircle, XCircle, Clock, Terminal } from 'lucide-react';
import { formatFileSize } from '@/utils/helpers';

interface CommandResultProps {
  execution: CommandExecution;
}

export default function CommandResult({ execution }: CommandResultProps) {
  const isSuccess = execution.exitCode === 0;
  const isRunning = execution.exitCode === undefined;

  return (
    <div className="my-4 bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 bg-zinc-800 border-b border-zinc-700">
        <div className="flex items-center gap-2">
          <Terminal className="w-4 h-4 text-zinc-400" />
          <span className="font-mono text-sm text-zinc-200">{execution.command}</span>
        </div>
        <div className="flex items-center gap-2">
          {isRunning ? (
            <div className="flex items-center gap-1 text-zinc-400">
              <Clock className="w-4 h-4" />
              <span className="text-xs">Running...</span>
            </div>
          ) : isSuccess ? (
            <div className="flex items-center gap-1 text-green-400">
              <CheckCircle className="w-4 h-4" />
              <span className="text-xs">Exit: {execution.exitCode}</span>
            </div>
          ) : (
            <div className="flex items-center gap-1 text-red-400">
              <XCircle className="w-4 h-4" />
              <span className="text-xs">Exit: {execution.exitCode}</span>
            </div>
          )}
          {execution.durationMs && (
            <span className="text-xs text-zinc-500">
              {execution.durationMs}ms
            </span>
          )}
        </div>
      </div>

      {/* Output */}
      <div className="px-4 py-3">
        {execution.stdout && (
          <div className="mb-2">
            <div className="text-xs text-zinc-500 mb-1">stdout</div>
            <pre className="font-mono text-sm text-green-300 whitespace-pre-wrap break-all bg-zinc-950 p-2 rounded overflow-x-auto">
              {execution.stdout}
            </pre>
          </div>
        )}

        {execution.stderr && (
          <div>
            <div className="text-xs text-zinc-500 mb-1">stderr</div>
            <pre className="font-mono text-sm text-red-300 whitespace-pre-wrap break-all bg-zinc-950 p-2 rounded overflow-x-auto">
              {execution.stderr}
            </pre>
          </div>
        )}

        {!execution.stdout && !execution.stderr && !isRunning && (
          <div className="text-sm text-zinc-500 italic">No output</div>
        )}
      </div>
    </div>
  );
}

export { CommandResult };
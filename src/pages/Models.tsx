import { useState } from 'react';
import { Cpu, Plus, Key } from 'lucide-react';

interface Model {
  id: string;
  name: string;
  provider: string;
  context_window?: number;
}

export default function Models() {
  const [models, setModels] = useState<Model[]>([]);
  const [hasApiKey, setHasApiKey] = useState(false);

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Models</h1>
        <button className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors">
          <Key className="w-4 h-4" />
          Configure API Keys
        </button>
      </div>

      {!hasApiKey ? (
        <div className="flex items-center justify-center h-64 text-zinc-500">
          <div className="text-center">
            <Key className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p className="text-lg">No API keys configured</p>
            <p className="text-sm">Add an API key to see available models</p>
          </div>
        </div>
      ) : (
        <>
          <div className="mb-6">
            <h2 className="text-lg font-medium mb-4">Available Models</h2>
            {models.length === 0 ? (
              <div className="flex items-center justify-center h-32 text-zinc-500">
                <div className="text-center">
                  <Cpu className="w-8 h-8 mx-auto mb-2 opacity-50" />
                  <p className="text-sm">No models available</p>
                </div>
              </div>
            ) : (
              <div className="space-y-4">
                {models.map((model) => (
                  <div key={model.id} className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <h3 className="font-medium">{model.name}</h3>
                        <p className="text-sm text-zinc-400">{model.provider}</p>
                      </div>
                      {model.context_window && (
                        <p className="text-xs text-zinc-500">{model.context_window.toLocaleString()} tokens</p>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );
}
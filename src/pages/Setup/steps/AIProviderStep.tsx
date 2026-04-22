import { useState, useEffect } from 'react';
import { ArrowRight, Loader2, Check, X, Shield } from 'lucide-react';
import { AI_PROVIDERS, checkOllamaConnection, saveApiKey } from '@/lib/wizard';
import type { AIProvider } from '@/lib/wizard';

interface AIProviderStepProps {
  onNext: () => void;
  onBack: () => void;
  selectedProvider: string | null;
  onProviderSelect: (provider: string | null) => void;
  apiKey: string;
  onApiKeyChange: (key: string) => void;
  t: (key: string) => string;
}

export default function AIProviderStep({
  onNext,
  onBack,
  selectedProvider,
  onProviderSelect,
  apiKey,
  onApiKeyChange,
  t,
}: AIProviderStepProps) {
  const [loading, setLoading] = useState(false);
  const [ollamaStatus, setOllamaStatus] = useState<{ available: boolean; models: string[] } | null>(null);
  const [skipMode, setSkipMode] = useState(false);

  useEffect(() => {
    // 检测 Ollama 连接状态
    setLoading(true);
    checkOllamaConnection()
      .then((status) => {
        setOllamaStatus(status);
        if (status.available) {
          onProviderSelect('ollama');
        }
      })
      .catch(() => {
        setOllamaStatus({ available: false, models: [] });
      })
      .finally(() => setLoading(false));
  }, []);

  const handleProviderClick = (provider: AIProvider) => {
    if (provider.id !== 'ollama') {
      onProviderSelect(null); // 需要输入 API Key
    } else {
      onProviderSelect(provider.id);
    }
    setSkipMode(false);
  };

  const handleSkip = () => {
    setSkipMode(true);
    onProviderSelect(null);
  };

  const handleNext = async () => {
    if (skipMode) {
      onNext();
      return;
    }

    if (selectedProvider && selectedProvider !== 'ollama' && apiKey) {
      try {
        await saveApiKey(selectedProvider, apiKey);
      } catch (e) {
        console.error('Failed to save API key:', e);
      }
    }
    onNext();
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="mb-2 text-2xl font-bold">{t('setup.views.provider.title')}</h2>
        <p className="text-zinc-400">{t('setup.views.provider.description')}</p>
      </div>

      {/* 提供商列表 */}
      <div className="space-y-3">
        {AI_PROVIDERS.map((provider) => {
          const isSelected = selectedProvider === provider.id;
          const needsApiKey = provider.id !== 'ollama' && isSelected;
          const isOllamaUnavailable = provider.id === 'ollama' && ollamaStatus !== null && !ollamaStatus.available;

          return (
            <button
              key={provider.id}
              onClick={() => handleProviderClick(provider)}
              disabled={provider.id === 'ollama' && isOllamaUnavailable}
              className={`w-full rounded-xl border p-4 text-left transition-all ${
                isSelected
                  ? 'border-blue-500 bg-blue-500/10'
                  : 'border-zinc-800 hover:border-zinc-700'
              } ${provider.id === 'ollama' && isOllamaUnavailable ? 'opacity-50 cursor-not-allowed' : ''}`}
            >
              <div className="flex items-center gap-4">
                <span className="text-3xl">{provider.icon}</span>
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <p className="font-medium">{provider.name}</p>
                    {isSelected && <Check className="h-4 w-4 text-blue-500" />}
                    {provider.id === 'ollama' && loading && <Loader2 className="h-4 w-4 animate-spin" />}
                    {provider.id === 'ollama' && ollamaStatus?.available && (
                      <span className="rounded bg-green-500/20 px-2 py-0.5 text-xs text-green-400">
                        已连接 ({ollamaStatus.models.length} 个模型)
                      </span>
                    )}
                    {provider.id === 'ollama' && ollamaStatus && !ollamaStatus.available && (
                      <span className="rounded bg-red-500/20 px-2 py-0.5 text-xs text-red-400">未检测到</span>
                    )}
                  </div>
                  <p className="text-sm text-zinc-500">{provider.description}</p>
                </div>
              </div>

              {/* API Key 输入框 */}
              {needsApiKey && (
                <div className="mt-4">
                  <label className="mb-2 block text-sm font-medium">{provider.name} API Key</label>
                  <input
                    type="password"
                    value={apiKey}
                    onChange={(e) => onApiKeyChange(e.target.value)}
                    placeholder={`输入 ${provider.name} API Key`}
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm placeholder:text-zinc-500 focus:border-blue-500 focus:outline-none"
                  />
                  <p className="mt-2 flex items-center gap-1 text-xs text-zinc-500">
                    <Shield className="h-3 w-3" />
                    {t('setup.views.provider.keySecurity')}
                  </p>
                </div>
              )}
            </button>
          );
        })}

        {/* 跳过选项 */}
        <button
          onClick={handleSkip}
          className={`w-full rounded-xl border p-4 text-left transition-all ${
            skipMode ? 'border-blue-500 bg-blue-500/10' : 'border-zinc-800 hover:border-zinc-700'
          }`}
        >
          <div className="flex items-center gap-4">
            <span className="text-3xl">⏭️</span>
            <div>
              <p className="font-medium">{t('setup.views.provider.skip')}</p>
              <p className="text-sm text-zinc-500">{t('setup.views.provider.skipDescription')}</p>
            </div>
            {skipMode && <Check className="ml-auto h-4 w-4 text-blue-500" />}
          </div>
        </button>
      </div>

      <div className="flex justify-between">
        <button
          onClick={onBack}
          className="rounded-lg bg-zinc-800 px-6 py-3 font-medium hover:bg-zinc-700"
        >
          {t('setup.actions.back')}
        </button>
        <button
          onClick={handleNext}
          disabled={!skipMode && !selectedProvider}
          className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700 disabled:opacity-50"
        >
          {t('setup.actions.continue')}
          <ArrowRight className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}

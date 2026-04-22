import { useState } from 'react';
import { ArrowRight, Check } from 'lucide-react';
import { FEATURES } from '@/lib/wizard';

interface FeaturesStepProps {
  onNext: () => void;
  onBack: () => void;
  selectedFeatures: string[];
  onFeaturesChange: (features: string[]) => void;
  t: (key: string) => string;
}

export default function FeaturesStep({
  onNext,
  onBack,
  selectedFeatures,
  onFeaturesChange,
  t,
}: FeaturesStepProps) {
  // 默认全选
  const [localSelected, setLocalSelected] = useState<string[]>(
    selectedFeatures.length > 0 ? selectedFeatures : FEATURES.map((f) => f.id)
  );

  const toggleFeature = (featureId: string) => {
    setLocalSelected((current) => {
      if (current.includes(featureId)) {
        return current.filter((id) => id !== featureId);
      }
      return [...current, featureId];
    });
  };

  const selectAll = () => {
    setLocalSelected(FEATURES.map((f) => f.id));
  };

  const deselectAll = () => {
    setLocalSelected([]);
  };

  const handleNext = () => {
    onFeaturesChange(localSelected);
    onNext();
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="mb-2 text-2xl font-bold">{t('setup.views.features.title')}</h2>
        <p className="text-zinc-400">{t('setup.views.features.description')}</p>
      </div>

      {/* 全选/取消全选 */}
      <div className="flex gap-2">
        <button
          onClick={selectAll}
          className="rounded-lg border border-zinc-700 px-3 py-1.5 text-sm text-zinc-400 hover:bg-zinc-800"
        >
          {t('setup.views.features.selectAll')}
        </button>
        <button
          onClick={deselectAll}
          className="rounded-lg border border-zinc-700 px-3 py-1.5 text-sm text-zinc-400 hover:bg-zinc-800"
        >
          {t('setup.views.features.deselectAll')}
        </button>
      </div>

      {/* 功能列表 */}
      <div className="grid grid-cols-2 gap-3">
        {FEATURES.map((feature) => {
          const isSelected = localSelected.includes(feature.id);
          return (
            <button
              key={feature.id}
              onClick={() => toggleFeature(feature.id)}
              className={`flex items-start gap-3 rounded-xl border p-4 text-left transition-all ${
                isSelected
                  ? 'border-blue-500 bg-blue-500/10'
                  : 'border-zinc-800 hover:border-zinc-700'
              }`}
            >
              <span className="text-2xl">{feature.icon}</span>
              <div className="flex-1">
                <p className="font-medium">{feature.name}</p>
                <p className="mt-0.5 text-xs text-zinc-500">{feature.description}</p>
              </div>
              <div
                className={`flex h-5 w-5 shrink-0 items-center justify-center rounded border ${
                  isSelected
                    ? 'border-blue-500 bg-blue-500 text-white'
                    : 'border-zinc-600 bg-zinc-800'
                }`}
              >
                {isSelected && <Check className="h-3 w-3" />}
              </div>
            </button>
          );
        })}
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
          disabled={localSelected.length === 0}
          className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700 disabled:opacity-50"
        >
          {t('setup.actions.continue')}
          <ArrowRight className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}

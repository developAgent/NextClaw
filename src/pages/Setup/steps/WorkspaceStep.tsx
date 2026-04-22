import { useState } from 'react';
import { ArrowRight, Check, Folder } from 'lucide-react';

interface WorkspaceStepProps {
  onNext: () => void;
  onBack: () => void;
  workspaceName: string;
  onWorkspaceNameChange: (name: string) => void;
  t: (key: string) => string;
}

export default function WorkspaceStep({
  onNext,
  onBack,
  workspaceName,
  onWorkspaceNameChange,
  t,
}: WorkspaceStepProps) {
  const presets = [
    { name: '默认工作区', emoji: '🏠' },
    { name: '开发环境', emoji: '💻' },
    { name: '办公环境', emoji: '📊' },
    { name: '研究项目', emoji: '🔬' },
  ];

  const handlePresetClick = (name: string) => {
    onWorkspaceNameChange(name);
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="mb-2 text-2xl font-bold">{t('setup.views.workspace.title')}</h2>
        <p className="text-zinc-400">{t('setup.views.workspace.description')}</p>
      </div>

      {/* 预设选项 */}
      <div className="grid grid-cols-2 gap-3">
        {presets.map((preset) => {
          const isSelected = workspaceName === preset.name;
          return (
            <button
              key={preset.name}
              onClick={() => handlePresetClick(preset.name)}
              className={`flex items-center gap-3 rounded-xl border p-4 text-left transition-all ${
                isSelected
                  ? 'border-blue-500 bg-blue-500/10'
                  : 'border-zinc-800 hover:border-zinc-700'
              }`}
            >
              <span className="text-2xl">{preset.emoji}</span>
              <div className="flex-1">
                <p className="font-medium">{preset.name}</p>
              </div>
              {isSelected && <Check className="h-4 w-4 text-blue-500" />}
            </button>
          );
        })}
      </div>

      {/* 自定义输入 */}
      <div>
        <label className="mb-2 block text-sm font-medium">{t('setup.views.workspace.customLabel')}</label>
        <div className="relative">
          <Folder className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-500" />
          <input
            type="text"
            value={workspaceName}
            onChange={(e) => onWorkspaceNameChange(e.target.value)}
            placeholder={t('setup.views.workspace.placeholder')}
            className="w-full rounded-lg border border-zinc-700 bg-zinc-800 py-2 pl-10 pr-4 text-sm placeholder:text-zinc-500 focus:border-blue-500 focus:outline-none"
          />
        </div>
      </div>

      <div className="flex justify-between">
        <button
          onClick={onBack}
          className="rounded-lg bg-zinc-800 px-6 py-3 font-medium hover:bg-zinc-700"
        >
          {t('setup.actions.back')}
        </button>
        <button
          onClick={onNext}
          disabled={!workspaceName.trim()}
          className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700 disabled:opacity-50"
        >
          {t('setup.actions.continue')}
          <ArrowRight className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}

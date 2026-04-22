import { useEffect, useMemo, useState } from 'react';
import { ChevronRight, Check, Zap, Languages, Key, Settings, Play, ArrowRight, Sparkles } from 'lucide-react';
import { useI18nStore, type Language } from '@/store/i18n';
import { useSettingsStore } from '@/store/settings';
import { AIProviderStep, WorkspaceStep, FeaturesStep } from './steps';
import {
  saveWizardState,
  completeWizard,
  type FeatureId,
} from '@/lib/wizard';

type SetupStep = 'welcome' | 'language' | 'provider' | 'workspace' | 'features' | 'verification' | 'complete';

interface StepConfig {
  id: SetupStep;
  titleKey: string;
  descriptionKey: string;
  icon: React.ElementType;
}

interface SetupWizardProps {
  onComplete?: () => void;
  onSkip?: () => void;
}

const languages = [
  { code: 'en', name: 'English', flag: '🇺🇸' },
  { code: 'zh', name: '中文', flag: '🇨🇳' },
  { code: 'ja', name: '日本語', flag: '🇯🇵' },
] as const;

export default function SetupWizard({ onComplete, onSkip }: SetupWizardProps) {
  const [currentStep, setCurrentStep] = useState<SetupStep>('welcome');
  const [completedSteps, setCompletedSteps] = useState<Set<SetupStep>>(new Set());

  // 向导状态
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null);
  const [apiKey, setApiKey] = useState('');
  const [workspaceName, setWorkspaceName] = useState('默认工作区');
  const [selectedFeatures, setSelectedFeatures] = useState<string[]>([]);

  const { language, t, changeLanguage } = useI18nStore();
  const saveConfig = useSettingsStore((state) => state.saveConfig);

  const steps: StepConfig[] = [
    { id: 'welcome', titleKey: 'setup.steps.welcome.title', descriptionKey: 'setup.steps.welcome.description', icon: Zap },
    { id: 'language', titleKey: 'setup.steps.language.title', descriptionKey: 'setup.steps.language.description', icon: Languages },
    { id: 'provider', titleKey: 'setup.steps.provider.title', descriptionKey: 'setup.steps.provider.description', icon: Key },
    { id: 'workspace', titleKey: 'setup.steps.workspace.title', descriptionKey: 'setup.steps.workspace.description', icon: Settings },
    { id: 'features', titleKey: 'setup.steps.features.title', descriptionKey: 'setup.steps.features.description', icon: Sparkles },
    { id: 'verification', titleKey: 'setup.steps.verification.title', descriptionKey: 'setup.steps.verification.description', icon: Play },
  ];

  const selectedLanguage = useMemo(
    () => languages.find((lang) => lang.code === language) ?? languages[0],
    [language]
  );

  // 保存语言更改
  useEffect(() => {
    void saveConfig({ language });
  }, [language, saveConfig]);

  // 保存向导状态到后端
  const persistWizardState = async () => {
    try {
      await saveWizardState({
        language,
        aiProvider: selectedProvider,
        apiKey: apiKey || null,
        apiKeyProvider: selectedProvider && selectedProvider !== 'ollama' ? selectedProvider : null,
        workspaceName,
        enabledFeatures: selectedFeatures,
      });
    } catch (e) {
      console.error('Failed to persist wizard state:', e);
    }
  };

  const handleNext = async () => {
    // 在切换步骤前保存状态
    await persistWizardState();

    setCompletedSteps((prev) => new Set(prev).add(currentStep));

    const currentIndex = steps.findIndex((step) => step.id === currentStep);
    if (currentIndex < steps.length - 1) {
      setCurrentStep(steps[currentIndex + 1].id);
      return;
    }

    // 完成向导
    try {
      await completeWizard();
    } catch (e) {
      console.error('Failed to complete wizard:', e);
    }
    onComplete?.();
  };

  const handleBack = () => {
    const currentIndex = steps.findIndex((step) => step.id === currentStep);
    if (currentIndex > 0) {
      setCurrentStep(steps[currentIndex - 1].id);
      setCompletedSteps((prev) => {
        const next = new Set(prev);
        next.delete(currentStep);
        return next;
      });
    }
  };

  const handleLanguageSelect = (code: Language) => {
    changeLanguage(code);
  };

  const getProgress = () => {
    const completedCount = completedSteps.size;
    return ((completedCount - (completedSteps.has(currentStep) ? 1 : 0)) / steps.length) * 100;
  };

  const StepIndicator = ({ step, isCompleted, isActive }: { step: StepConfig; isCompleted: boolean; isActive: boolean }) => {
    const Icon = step.icon;
    return (
      <div
        className={`flex items-center gap-3 rounded-xl border px-4 py-3 transition-all ${
          isActive
            ? 'border-blue-500 bg-blue-600 text-white'
            : 'border-zinc-700 bg-zinc-800 hover:bg-zinc-700'
        }`}
      >
        <div
          className={`flex h-8 w-8 items-center justify-center rounded-full ${
            isCompleted
              ? 'bg-green-500 text-white'
              : isActive
                ? 'bg-blue-500 text-white'
                : 'bg-zinc-700 text-zinc-400'
          }`}
        >
          {isCompleted ? <Check className="h-4 w-4" /> : <Icon className="h-4 w-4" />}
        </div>
        <div className="flex-1">
          <p className="text-sm font-medium">{t(step.titleKey)}</p>
          <p className="text-xs text-zinc-500">{t(step.descriptionKey)}</p>
        </div>
        {isActive ? <ChevronRight className="h-4 w-4 text-zinc-200" /> : null}
      </div>
    );
  };

  return (
    <div className="flex min-h-screen bg-zinc-950 text-zinc-100">
      {/* 侧边栏 */}
      <div className="flex w-72 shrink-0 flex-col border-r border-zinc-800 p-6">
        <div className="mb-8">
          <h1 className="mb-2 text-xl font-bold text-blue-500">NextClaw</h1>
          <p className="text-xs text-zinc-500">{t('setup.sidebar.caption')}</p>
        </div>

        <div className="flex-1 space-y-2">
          {steps.map((step) => (
            <StepIndicator
              key={step.id}
              step={step}
              isCompleted={completedSteps.has(step.id)}
              isActive={currentStep === step.id}
            />
          ))}
        </div>

        {/* 进度条 */}
        <div className="mt-6">
          <div className="mb-2 flex items-center justify-between text-xs">
            <span className="text-zinc-500">{t('setup.sidebar.progress')}</span>
            <span className="text-zinc-400">{Math.round(getProgress())}%</span>
          </div>
          <div className="h-2 overflow-hidden rounded-full bg-zinc-800">
            <div
              className="h-full bg-blue-500 transition-all duration-300"
              style={{ width: `${getProgress()}%` }}
            />
          </div>
        </div>

        {onSkip ? (
          <button
            onClick={onSkip}
            className="mt-6 rounded-lg border border-zinc-700 px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-800"
          >
            {t('setup.actions.skip')}
          </button>
        ) : null}
      </div>

      {/* 主内容区 */}
      <div className="flex flex-1 items-center justify-center p-8">
        <div className="w-full max-w-2xl">
          {/* Welcome */}
          {currentStep === 'welcome' && (
            <div className="space-y-6 text-center">
              <div className="mx-auto flex h-20 w-20 items-center justify-center rounded-2xl bg-blue-600">
                <Zap className="h-10 w-10 text-white" />
              </div>
              <div>
                <h2 className="mb-2 text-3xl font-bold">{t('setup.welcome.title')}</h2>
                <p className="text-lg text-zinc-400">{t('setup.welcome.description')}</p>
              </div>
              <p className="text-zinc-500">{t('setup.welcome.helper')}</p>
              <button
                onClick={handleNext}
                className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700"
              >
                {t('setup.actions.getStarted')}
                <ArrowRight className="h-4 w-4" />
              </button>
            </div>
          )}

          {/* Language */}
          {currentStep === 'language' && (
            <div className="space-y-6">
              <div>
                <h2 className="mb-2 text-2xl font-bold">{t('setup.views.language.title')}</h2>
                <p className="text-zinc-400">{t('setup.views.language.description')}</p>
              </div>

              <div className="space-y-3">
                {languages.map((lang) => (
                  <button
                    key={lang.code}
                    onClick={() => handleLanguageSelect(lang.code)}
                    className={`w-full rounded-xl border p-4 text-left transition-colors ${
                      language === lang.code
                        ? 'border-blue-500 bg-blue-500/10'
                        : 'border-zinc-800 hover:border-zinc-700'
                    }`}
                  >
                    <div className="flex items-center gap-4">
                      <span className="text-2xl">{lang.flag}</span>
                      <div>
                        <p className="font-medium">{lang.name}</p>
                        <p className="text-xs text-zinc-500">{lang.code.toUpperCase()}</p>
                      </div>
                      {selectedLanguage.code === lang.code ? (
                        <span className="ml-auto rounded-full bg-blue-500 px-2 py-1 text-xs font-medium text-white">
                          {t('setup.views.language.selected')}
                        </span>
                      ) : null}
                    </div>
                  </button>
                ))}
              </div>

              <div className="flex justify-end">
                <button
                  onClick={handleNext}
                  className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700"
                >
                  {t('setup.actions.continue')}
                  <ArrowRight className="h-4 w-4" />
                </button>
              </div>
            </div>
          )}

          {/* Provider */}
          {currentStep === 'provider' && (
            <AIProviderStep
              onNext={handleNext}
              onBack={handleBack}
              selectedProvider={selectedProvider}
              onProviderSelect={setSelectedProvider}
              apiKey={apiKey}
              onApiKeyChange={setApiKey}
              t={t}
            />
          )}

          {/* Workspace */}
          {currentStep === 'workspace' && (
            <WorkspaceStep
              onNext={handleNext}
              onBack={handleBack}
              workspaceName={workspaceName}
              onWorkspaceNameChange={setWorkspaceName}
              t={t}
            />
          )}

          {/* Features */}
          {currentStep === 'features' && (
            <FeaturesStep
              onNext={handleNext}
              onBack={handleBack}
              selectedFeatures={selectedFeatures}
              onFeaturesChange={setSelectedFeatures}
              t={t}
            />
          )}

          {/* Verification */}
          {currentStep === 'verification' && (
            <div className="space-y-6">
              <div>
                <h2 className="mb-2 text-2xl font-bold">{t('setup.views.verification.title')}</h2>
                <p className="text-zinc-400">{t('setup.views.verification.description')}</p>
              </div>

              <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-6">
                <div className="space-y-4">
                  <div className="flex items-center gap-3">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-green-500">
                      <Check className="h-5 w-5 text-white" />
                    </div>
                    <p>{t('setup.views.verification.languageSet', { language: selectedLanguage.name })}</p>
                  </div>
                  <div className="flex items-center gap-3">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-green-500">
                      <Check className="h-5 w-5 text-white" />
                    </div>
                    <p>
                      {selectedProvider
                        ? `${t('setup.views.verification.providerSet')}: ${selectedProvider}`
                        : t('setup.views.verification.providerDeferred')}
                    </p>
                  </div>
                  <div className="flex items-center gap-3">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-green-500">
                      <Check className="h-5 w-5 text-white" />
                    </div>
                    <p>{t('setup.views.verification.workspaceSet')}: {workspaceName}</p>
                  </div>
                  <div className="flex items-center gap-3">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-green-500">
                      <Check className="h-5 w-5 text-white" />
                    </div>
                    <p>
                      {selectedFeatures.length > 0
                        ? t('setup.views.verification.featuresSelected', { count: selectedFeatures.length })
                        : t('setup.views.verification.noFeaturesSelected')}
                    </p>
                  </div>
                </div>
              </div>

              <div className="flex justify-between">
                <button
                  onClick={handleBack}
                  className="rounded-lg bg-zinc-800 px-6 py-3 font-medium hover:bg-zinc-700"
                >
                  {t('setup.actions.back')}
                </button>
                <button
                  onClick={handleNext}
                  className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700"
                >
                  {t('setup.actions.openDashboard')}
                  <ArrowRight className="h-4 w-4" />
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

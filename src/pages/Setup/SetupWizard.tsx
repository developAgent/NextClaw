import { useState } from 'react';
import { ChevronRight, Check, Zap, Languages, Key, Database, Play, ArrowRight } from 'lucide-react';
import { useI18nStore } from '@/store/i18n';

type SetupStep = 'welcome' | 'language' | 'provider' | 'skills' | 'verification' | 'complete';

interface StepConfig {
  id: SetupStep;
  title: string;
  description: string;
  icon: React.ElementType;
}

const steps: StepConfig[] = [
  {
    id: 'welcome',
    title: 'Welcome',
    description: 'Get started with CEOClaw',
    icon: Zap,
  },
  {
    id: 'language',
    title: 'Language',
    description: 'Choose your language',
    icon: Languages,
  },
  {
    id: 'provider',
    title: 'AI Provider',
    description: 'Configure your AI provider',
    icon: Key,
  },
  {
    id: 'skills',
    title: 'Skills',
    description: 'Select skill bundles',
    icon: Database,
  },
  {
    id: 'verification',
    title: 'Verification',
    description: 'Test your configuration',
    icon: Play,
  },
];

export default function SetupWizard() {
  const [currentStep, setCurrentStep] = useState<SetupStep>('welcome');
  const [completedSteps, setCompletedSteps] = useState<Set<SetupStep>>(new Set());
  const { language } = useI18nStore();

  const languages = [
    { code: 'en', name: 'English', flag: '🇺🇸' },
    { code: 'zh', name: '中文', flag: '🇨🇳' },
    { code: 'ja', name: '日本語', flag: '🇯🇵' },
  ];

  const skillBundles = [
    { id: 'document', name: 'Document Processing', description: 'Process PDF, Excel, Word files', installed: false },
    { id: 'search', name: 'Search Skills', description: 'Brave and Tavily web search', installed: false },
    { id: 'agent', name: 'Agent Tools', description: 'Self-improving and find skills', installed: false },
  ];

  const handleNext = () => {
    setCompletedSteps(prev => new Set(prev).add(currentStep));

    const currentIndex = steps.findIndex(s => s.id === currentStep);
    if (currentIndex < steps.length - 1) {
      setCurrentStep(steps[currentIndex + 1].id);
    } else {
      // Setup complete - you can navigate to dashboard or show completion message
      console.log('Setup complete!');
    }
  };

  const handleBack = () => {
    const currentIndex = steps.findIndex(s => s.id === currentStep);
    if (currentIndex > 0) {
      setCurrentStep(steps[currentIndex - 1].id);
      setCompletedSteps(prev => {
        const next = new Set(prev);
        next.delete(currentStep);
        return next;
      });
    }
  };

  const getProgress = () => {
    const completedCount = completedSteps.size;
    return ((completedCount - (completedSteps.has(currentStep) ? 1 : 0)) / steps.length) * 100;
  };

  const Step = ({ step, isCompleted, isActive }: { step: StepConfig; isCompleted: boolean; isActive: boolean }) => {
    const Icon = step.icon;
    return (
      <div
        className={`flex items-center gap-3 px-4 py-3 rounded-xl border transition-all cursor-pointer ${
          isActive
            ? 'bg-blue-600 border-blue-500 text-white'
            : 'bg-zinc-800 border-zinc-700 hover:bg-zinc-700'
        }`}
      >
        <div
          className={`w-8 h-8 rounded-full flex items-center justify-center ${
            isCompleted
              ? 'bg-green-500 text-white'
              : isActive
              ? 'bg-blue-500 text-white'
              : 'bg-zinc-700 text-zinc-400'
          }`}
        >
          {isCompleted ? <Check className="w-4 h-4" /> : <Icon className="w-4 h-4" />}
        </div>
        <div className="flex-1">
          <p className="font-medium text-sm">{step.title}</p>
          <p className="text-xs text-zinc-500">{step.description}</p>
        </div>
        {isActive && <ChevronRight className="w-4 h-4 text-zinc-500" />}
      </div>
    );
  };

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 flex">
      {/* Sidebar */}
      <div className="w-64 border-r border-zinc-800 p-6 flex flex-col">
        <div className="mb-8">
          <h1 className="text-xl font-bold text-blue-500 mb-2">CEOClaw</h1>
          <p className="text-xs text-zinc-500">Setup Wizard</p>
        </div>

        <div className="flex-1 space-y-2">
          {steps.map((step) => (
            <Step
              key={step.id}
              step={step}
              isCompleted={completedSteps.has(step.id)}
              isActive={currentStep === step.id}
            />
          ))}
        </div>

        {/* Progress */}
        <div className="mt-6">
          <div className="flex items-center justify-between text-xs mb-2">
            <span className="text-zinc-500">Progress</span>
            <span className="text-zinc-400">{Math.round(getProgress())}%</span>
          </div>
          <div className="h-2 bg-zinc-800 rounded-full overflow-hidden">
            <div
              className="h-full bg-blue-500 transition-all duration-300"
              style={{ width: `${getProgress()}%` }}
            />
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex items-center justify-center p-8">
        <div className="max-w-2xl w-full">
          {currentStep === 'welcome' && (
            <div className="text-center space-y-6">
              <div className="w-20 h-20 bg-blue-600 rounded-2xl flex items-center justify-center mx-auto">
                <Zap className="w-10 h-10 text-white" />
              </div>
              <div>
                <h2 className="text-3xl font-bold mb-2">Welcome to CEOClaw</h2>
                <p className="text-zinc-400 text-lg">
                  Your desktop AI assistant powered by Claude
                </p>
              </div>
              <p className="text-zinc-500">
                Let's get you set up in just a few minutes
              </p>
              <button
                onClick={handleNext}
                className="flex items-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg text-white font-medium"
              >
                Get Started
                <ArrowRight className="w-4 h-4" />
              </button>
            </div>
          )}

          {currentStep === 'language' && (
            <div className="space-y-6">
              <div>
                <h2 className="text-2xl font-bold mb-2">Choose Your Language</h2>
                <p className="text-zinc-400">
                  Select the language you'd like to use for the interface
                </p>
              </div>

              <div className="space-y-3">
                {languages.map((lang) => (
                  <button
                    key={lang.code}
                    onClick={() => useI18nStore.getState().changeLanguage(lang.code as any)}
                    className={`w-full flex items-center gap-4 p-4 rounded-xl border transition-colors text-left ${
                      language === lang.code
                        ? 'border-blue-500 bg-blue-500/10'
                        : 'border-zinc-800 hover:border-zinc-700'
                    }`}
                  >
                    <span className="text-2xl">{lang.flag}</span>
                    <div>
                      <p className="font-medium">{lang.name}</p>
                      <p className="text-xs text-zinc-500">{lang.code.toUpperCase()}</p>
                    </div>
                  </button>
                ))}
              </div>

              <div className="flex justify-end">
                <button
                  onClick={handleNext}
                  className="flex items-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg text-white font-medium"
                >
                  Continue
                  <ArrowRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          )}

          {currentStep === 'provider' && (
            <div className="space-y-6">
              <div>
                <h2 className="text-2xl font-bold mb-2">Configure AI Provider</h2>
                <p className="text-zinc-400">
                  Add your Anthropic API key to start using Claude
                </p>
              </div>

              <div className="bg-zinc-900 rounded-xl p-6 border border-zinc-800">
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">API Key</label>
                    <input
                      type="password"
                      placeholder="sk-ant-..."
                      className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">Base URL (optional)</label>
                    <input
                      type="text"
                      placeholder="https://api.anthropic.com"
                      className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>
                </div>
              </div>

              <div className="flex justify-between">
                <button
                  onClick={handleBack}
                  className="px-6 py-3 bg-zinc-800 hover:bg-zinc-700 rounded-lg font-medium"
                >
                  Back
                </button>
                <button
                  onClick={handleNext}
                  className="flex items-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg text-white font-medium"
                >
                  Continue
                  <ArrowRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          )}

          {currentStep === 'skills' && (
            <div className="space-y-6">
              <div>
                <h2 className="text-2xl font-bold mb-2">Select Skill Bundles</h2>
                <p className="text-zinc-400">
                  Choose which skill packages you'd like to install
                </p>
              </div>

              <div className="space-y-3">
                {skillBundles.map((bundle) => (
                  <button
                    key={bundle.id}
                    onClick={() => {
                      bundle.installed = !bundle.installed;
                    }}
                    className={`w-full flex items-center gap-4 p-4 rounded-xl border transition-colors text-left ${
                      bundle.installed
                        ? 'border-blue-500 bg-blue-500/10'
                        : 'border-zinc-800 hover:border-zinc-700'
                    }`}
                  >
                    <div
                      className={`w-12 h-12 rounded-lg flex items-center justify-center ${
                        bundle.installed ? 'bg-blue-600 text-white' : 'bg-zinc-800'
                      }`}
                    >
                      {bundle.installed ? <Check className="w-6 h-6" /> : <Database className="w-6 h-6" />}
                    </div>
                    <div className="flex-1 text-left">
                      <p className="font-medium">{bundle.name}</p>
                      <p className="text-sm text-zinc-500">{bundle.description}</p>
                    </div>
                  </button>
                ))}
              </div>

              <div className="flex justify-between">
                <button
                  onClick={handleBack}
                  className="px-6 py-3 bg-zinc-800 hover:bg-zinc-700 rounded-lg font-medium"
                >
                  Back
                </button>
                <button
                  onClick={handleNext}
                  className="flex items-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg text-white font-medium"
                >
                  Continue
                  <ArrowRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          )}

          {currentStep === 'verification' && (
            <div className="space-y-6">
              <div>
                <h2 className="text-2xl font-bold mb-2">Verify Configuration</h2>
                <p className="text-zinc-400">
                  Let's test your setup before you start using CEOClaw
                </p>
              </div>

              <div className="bg-zinc-900 rounded-xl p-6 border border-zinc-800">
                <div className="space-y-4">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-full bg-green-500 flex items-center justify-center">
                      <Check className="w-5 h-5 text-white" />
                    </div>
                    <p>Language configured</p>
                  </div>
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-full bg-green-500 flex items-center justify-center">
                      <Check className="w-5 h-5 text-white" />
                    </div>
                    <p>API key added</p>
                  </div>
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-full bg-green-500 flex items-center justify-center">
                      <Check className="w-5 h-5 text-white" />
                    </div>
                    <p>Skills selected</p>
                  </div>
                </div>
              </div>

              <div className="flex justify-between">
                <button
                  onClick={handleBack}
                  className="px-6 py-3 bg-zinc-800 hover:bg-zinc-700 rounded-lg font-medium"
                >
                  Back
                </button>
                <button
                  onClick={handleNext}
                  className="flex items-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg text-white font-medium"
                >
                  Complete Setup
                  <ArrowRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
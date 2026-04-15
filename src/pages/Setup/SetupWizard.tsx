import { useMemo, useState } from 'react';
import { ChevronRight, Check, Zap, Languages, Key, Database, Play, ArrowRight } from 'lucide-react';
import { useI18nStore } from '@/store/i18n';

type SetupStep = 'welcome' | 'language' | 'provider' | 'skills' | 'verification' | 'complete';

interface StepConfig {
  id: SetupStep;
  title: string;
  description: string;
  icon: React.ElementType;
}

interface SkillBundle {
  id: string;
  name: string;
  description: string;
}

interface SetupWizardProps {
  onComplete?: () => void;
  onSkip?: () => void;
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

const languages = [
  { code: 'en', name: 'English', flag: '🇺🇸' },
  { code: 'zh', name: '中文', flag: '🇨🇳' },
  { code: 'ja', name: '日本語', flag: '🇯🇵' },
] as const;

const skillBundles: SkillBundle[] = [
  { id: 'document', name: 'Document Processing', description: 'Process PDF, Excel, Word files' },
  { id: 'search', name: 'Search Skills', description: 'Brave and Tavily web search' },
  { id: 'agent', name: 'Agent Tools', description: 'Self-improving and find skills' },
];

export default function SetupWizard({ onComplete, onSkip }: SetupWizardProps) {
  const [currentStep, setCurrentStep] = useState<SetupStep>('welcome');
  const [completedSteps, setCompletedSteps] = useState<Set<SetupStep>>(new Set());
  const [selectedSkills, setSelectedSkills] = useState<string[]>([]);
  const { language } = useI18nStore();

  const selectedLanguage = useMemo(
    () => languages.find((lang) => lang.code === language) ?? languages[0],
    [language]
  );

  const handleNext = () => {
    setCompletedSteps((prev) => new Set(prev).add(currentStep));

    const currentIndex = steps.findIndex((step) => step.id === currentStep);
    if (currentIndex < steps.length - 1) {
      setCurrentStep(steps[currentIndex + 1].id);
      return;
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

  const toggleSkill = (skillId: string) => {
    setSelectedSkills((current) => (
      current.includes(skillId)
        ? current.filter((item) => item !== skillId)
        : [...current, skillId]
    ));
  };

  const getProgress = () => {
    const completedCount = completedSteps.size;
    return ((completedCount - (completedSteps.has(currentStep) ? 1 : 0)) / steps.length) * 100;
  };

  const Step = ({ step, isCompleted, isActive }: { step: StepConfig; isCompleted: boolean; isActive: boolean }) => {
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
          <p className="text-sm font-medium">{step.title}</p>
          <p className="text-xs text-zinc-500">{step.description}</p>
        </div>
        {isActive ? <ChevronRight className="h-4 w-4 text-zinc-200" /> : null}
      </div>
    );
  };

  return (
    <div className="flex min-h-screen bg-zinc-950 text-zinc-100">
      <div className="flex w-72 shrink-0 flex-col border-r border-zinc-800 p-6">
        <div className="mb-8">
          <h1 className="mb-2 text-xl font-bold text-blue-500">NextClaw</h1>
          <p className="text-xs text-zinc-500">Guided first-run setup</p>
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

        <div className="mt-6">
          <div className="mb-2 flex items-center justify-between text-xs">
            <span className="text-zinc-500">Progress</span>
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
            Skip for now
          </button>
        ) : null}
      </div>

      <div className="flex flex-1 items-center justify-center p-8">
        <div className="w-full max-w-2xl">
          {currentStep === 'welcome' && (
            <div className="space-y-6 text-center">
              <div className="mx-auto flex h-20 w-20 items-center justify-center rounded-2xl bg-blue-600">
                <Zap className="h-10 w-10 text-white" />
              </div>
              <div>
                <h2 className="mb-2 text-3xl font-bold">Welcome to NextClaw</h2>
                <p className="text-lg text-zinc-400">
                  A ClawX-compatible desktop workspace for local runtime, skills, and automation.
                </p>
              </div>
              <p className="text-zinc-500">Start with a few core preferences, then jump into the installer and product shell.</p>
              <button
                onClick={handleNext}
                className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700"
              >
                Get Started
                <ArrowRight className="h-4 w-4" />
              </button>
            </div>
          )}

          {currentStep === 'language' && (
            <div className="space-y-6">
              <div>
                <h2 className="mb-2 text-2xl font-bold">Choose Your Language</h2>
                <p className="text-zinc-400">Select the language you'd like to use for the interface.</p>
              </div>

              <div className="space-y-3">
                {languages.map((lang) => (
                  <button
                    key={lang.code}
                    onClick={() => useI18nStore.getState().changeLanguage(lang.code)}
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
                    </div>
                  </button>
                ))}
              </div>

              <div className="flex justify-end">
                <button
                  onClick={handleNext}
                  className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700"
                >
                  Continue
                  <ArrowRight className="h-4 w-4" />
                </button>
              </div>
            </div>
          )}

          {currentStep === 'provider' && (
            <div className="space-y-6">
              <div>
                <h2 className="mb-2 text-2xl font-bold">Configure AI Provider</h2>
                <p className="text-zinc-400">Add your primary provider later in Channels or Settings. This step keeps the launch flow lightweight.</p>
              </div>

              <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-6">
                <div className="space-y-4">
                  <div>
                    <label className="mb-2 block text-sm font-medium">Preferred provider</label>
                    <div className="rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-3 text-sm text-zinc-300">
                      Anthropic Claude is recommended for first-run setup.
                    </div>
                  </div>
                  <div className="rounded-lg border border-blue-500/20 bg-blue-500/10 px-4 py-3 text-sm text-blue-200">
                    Provider keys and account bindings can be configured safely after onboarding from the main shell.
                  </div>
                </div>
              </div>

              <div className="flex justify-between">
                <button
                  onClick={handleBack}
                  className="rounded-lg bg-zinc-800 px-6 py-3 font-medium hover:bg-zinc-700"
                >
                  Back
                </button>
                <button
                  onClick={handleNext}
                  className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700"
                >
                  Continue
                  <ArrowRight className="h-4 w-4" />
                </button>
              </div>
            </div>
          )}

          {currentStep === 'skills' && (
            <div className="space-y-6">
              <div>
                <h2 className="mb-2 text-2xl font-bold">Select Skill Bundles</h2>
                <p className="text-zinc-400">Choose which skill categories you want to explore first.</p>
              </div>

              <div className="space-y-3">
                {skillBundles.map((bundle) => {
                  const isSelected = selectedSkills.includes(bundle.id);

                  return (
                    <button
                      key={bundle.id}
                      onClick={() => toggleSkill(bundle.id)}
                      className={`w-full rounded-xl border p-4 text-left transition-colors ${
                        isSelected
                          ? 'border-blue-500 bg-blue-500/10'
                          : 'border-zinc-800 hover:border-zinc-700'
                      }`}
                    >
                      <div className="flex items-center gap-4">
                        <div
                          className={`flex h-12 w-12 items-center justify-center rounded-lg ${
                            isSelected ? 'bg-blue-600 text-white' : 'bg-zinc-800'
                          }`}
                        >
                          {isSelected ? <Check className="h-6 w-6" /> : <Database className="h-6 w-6" />}
                        </div>
                        <div className="flex-1 text-left">
                          <p className="font-medium">{bundle.name}</p>
                          <p className="text-sm text-zinc-500">{bundle.description}</p>
                        </div>
                      </div>
                    </button>
                  );
                })}
              </div>

              <div className="flex justify-between">
                <button
                  onClick={handleBack}
                  className="rounded-lg bg-zinc-800 px-6 py-3 font-medium hover:bg-zinc-700"
                >
                  Back
                </button>
                <button
                  onClick={handleNext}
                  className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700"
                >
                  Continue
                  <ArrowRight className="h-4 w-4" />
                </button>
              </div>
            </div>
          )}

          {currentStep === 'verification' && (
            <div className="space-y-6">
              <div>
                <h2 className="mb-2 text-2xl font-bold">Ready to Open the Product Shell</h2>
                <p className="text-zinc-400">Your first-run preferences are captured. The next recommended stop is the Installer page to start the local runtime.</p>
              </div>

              <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-6">
                <div className="space-y-4">
                  <div className="flex items-center gap-3">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-green-500">
                      <Check className="h-5 w-5 text-white" />
                    </div>
                    <p>Language set to {selectedLanguage.name}</p>
                  </div>
                  <div className="flex items-center gap-3">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-green-500">
                      <Check className="h-5 w-5 text-white" />
                    </div>
                    <p>Provider setup deferred to the full Settings and Channels pages</p>
                  </div>
                  <div className="flex items-center gap-3">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-green-500">
                      <Check className="h-5 w-5 text-white" />
                    </div>
                    <p>{selectedSkills.length > 0 ? `${selectedSkills.length} skill bundle(s) selected` : 'No skill bundles selected yet'}</p>
                  </div>
                </div>
              </div>

              <div className="flex justify-between">
                <button
                  onClick={handleBack}
                  className="rounded-lg bg-zinc-800 px-6 py-3 font-medium hover:bg-zinc-700"
                >
                  Back
                </button>
                <button
                  onClick={handleNext}
                  className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-3 font-medium text-white hover:bg-blue-700"
                >
                  Open Installer
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

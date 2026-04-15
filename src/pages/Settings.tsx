import { useEffect, useMemo, useState } from 'react';
import { Settings as SettingsIcon, Globe, Zap, Network, Palette, Code, Save, RefreshCw, Key, Trash2 } from 'lucide-react';
import ProxySettings from '@/components/settings/ProxySettings';
import DeveloperTools from '@/components/settings/DeveloperTools';
import { useSettingsStore } from '@/store/settings';
import { useI18nStore, type Language } from '@/store/i18n';
import type { ConfigUpdate } from '@/types';

type TabType = 'general' | 'providers' | 'skills' | 'network' | 'appearance' | 'developer';

function parseListInput(value: string): string[] {
  return value
    .split(/[\n,]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

export default function SettingsPage() {
  const [activeTab, setActiveTab] = useState<TabType>('general');
  const { config, loading, saving, error, refresh, saveConfig, saveApiKey, removeApiKey } = useSettingsStore();
  const { t } = useI18nStore();
  const changeLanguage = useI18nStore((state) => state.changeLanguage);

  const [apiKey, setApiKey] = useState('');
  const [generalDraft, setGeneralDraft] = useState({
    timeoutSecs: 60,
    allowShell: false,
    requireConfirmation: true,
    sandboxPath: '',
    whitelist: '',
    blacklist: '',
  });
  const [providerDraft, setProviderDraft] = useState({
    claudeModel: 'claude-3-5-sonnet-latest',
    requestTimeoutSecs: 120,
    maxRetries: 3,
  });
  const [appearanceDraft, setAppearanceDraft] = useState({
    theme: 'dark',
    language: 'en' as Language,
    fontSize: 14,
    showTimestamps: true,
    maxHistory: 100,
  });

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (!config) {
      return;
    }

    setGeneralDraft({
      timeoutSecs: config.commands.timeoutSecs,
      allowShell: config.commands.allowShell,
      requireConfirmation: config.commands.requireConfirmation,
      sandboxPath: config.commands.sandboxPath,
      whitelist: config.commands.whitelist.join('\n'),
      blacklist: config.commands.blacklist.join('\n'),
    });
    setProviderDraft({
      claudeModel: config.api.claudeModel,
      requestTimeoutSecs: config.api.requestTimeoutSecs,
      maxRetries: config.api.maxRetries,
    });
    setAppearanceDraft({
      theme: config.ui.theme,
      language: config.ui.language,
      fontSize: config.ui.fontSize,
      showTimestamps: config.ui.showTimestamps,
      maxHistory: config.ui.maxHistory,
    });
    setApiKey('');
  }, [config]);

  const tabs = [
    { id: 'general' as TabType, label: t('settingsPage.tabs.general'), icon: SettingsIcon },
    { id: 'providers' as TabType, label: t('settingsPage.tabs.providers'), icon: Globe },
    { id: 'skills' as TabType, label: t('settingsPage.tabs.skills'), icon: Zap },
    { id: 'network' as TabType, label: t('settingsPage.tabs.network'), icon: Network },
    { id: 'appearance' as TabType, label: t('settingsPage.tabs.appearance'), icon: Palette },
    { id: 'developer' as TabType, label: t('settingsPage.tabs.developer'), icon: Code },
  ];

  const languageOptions = [
    { value: 'en' as Language, label: 'English', helper: t('settingsPage.appearance.languages.en') },
    { value: 'zh' as Language, label: '中文', helper: t('settingsPage.appearance.languages.zh') },
    { value: 'ja' as Language, label: '日本語', helper: t('settingsPage.appearance.languages.ja') },
  ];

  const hasConfig = config !== null;
  const apiKeyStatus = useMemo(() => {
    if (!config?.api.apiKeyConfigured) {
      return t('settingsPage.providers.apiKey.status.none');
    }

    return apiKey.trim()
      ? t('settingsPage.providers.apiKey.status.ready')
      : t('settingsPage.providers.apiKey.status.configured');
  }, [apiKey, config?.api.apiKeyConfigured, t]);

  const handleSaveGeneral = async () => {
    const update: ConfigUpdate = {
      timeoutSecs: generalDraft.timeoutSecs,
      allowShell: generalDraft.allowShell,
      requireConfirmation: generalDraft.requireConfirmation,
      sandboxPath: generalDraft.sandboxPath.trim(),
      whitelist: parseListInput(generalDraft.whitelist),
      blacklist: parseListInput(generalDraft.blacklist),
    };

    await saveConfig(update);
  };

  const handleSaveProvider = async () => {
    const update: ConfigUpdate = {
      claudeModel: providerDraft.claudeModel,
      requestTimeoutSecs: providerDraft.requestTimeoutSecs,
      maxRetries: providerDraft.maxRetries,
    };

    await saveConfig(update);

    const trimmedApiKey = apiKey.trim();
    if (trimmedApiKey) {
      await saveApiKey(trimmedApiKey);
      setApiKey('');
    }
  };

  const handleSaveAppearance = async () => {
    await saveConfig({
      theme: appearanceDraft.theme,
      language: appearanceDraft.language,
      fontSize: appearanceDraft.fontSize,
      showTimestamps: appearanceDraft.showTimestamps,
      maxHistory: appearanceDraft.maxHistory,
    });

    await changeLanguage(appearanceDraft.language);
  };

  const renderSaveButton = (onClick: () => Promise<void>, label = t('settingsPage.actions.saveChanges')) => (
    <button
      onClick={() => void onClick()}
      disabled={!hasConfig || saving}
      className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
    >
      {saving ? <RefreshCw className="h-4 w-4 animate-spin" /> : <Save className="h-4 w-4" />}
      {saving ? t('settingsPage.actions.saving') : label}
    </button>
  );

  return (
    <div className="flex h-full">
      <div className="w-64 border-r border-zinc-800 bg-zinc-900">
        <div className="border-b border-zinc-800 p-4">
          <h2 className="text-xl font-bold">{t('settings')}</h2>
        </div>
        <nav className="p-2">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`mb-1 flex w-full items-center gap-3 rounded-lg px-3 py-2 transition-colors ${
                activeTab === tab.id ? 'bg-blue-600 text-white' : 'text-zinc-400 hover:bg-zinc-800'
              }`}
            >
              <tab.icon className="h-4 w-4" />
              <span className="text-sm">{tab.label}</span>
            </button>
          ))}
        </nav>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="mb-6 flex items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold">{t('settingsPage.title')}</h1>
            <p className="mt-1 text-sm text-zinc-400">{t('settingsPage.subtitle')}</p>
          </div>
          <button
            onClick={() => void refresh()}
            disabled={loading || saving}
            className="inline-flex items-center gap-2 rounded-lg bg-zinc-800 px-4 py-2 text-sm transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            {t('settingsPage.actions.refresh')}
          </button>
        </div>

        {error && (
          <div className="mb-6 rounded-lg border border-red-500/40 bg-red-500/10 px-4 py-3 text-sm text-red-300">
            {error}
          </div>
        )}

        {loading && !config ? (
          <div className="flex h-64 items-center justify-center">
            <div className="h-8 w-8 animate-spin rounded-full border-b-2 border-blue-500" />
          </div>
        ) : null}

        {!loading && !config ? (
          <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-6 text-sm text-zinc-400">
            {t('settingsPage.states.unableToLoad')}
          </div>
        ) : null}

        {activeTab === 'general' && config && (
          <div className="space-y-6">
            <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
              <h2 className="mb-4 text-xl font-semibold">{t('settingsPage.general.execution.title')}</h2>
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-2 block text-sm font-medium text-zinc-300">{t('settingsPage.general.execution.timeout')}</label>
                  <input
                    type="number"
                    min={1}
                    value={generalDraft.timeoutSecs}
                    onChange={(e) => setGeneralDraft((current) => ({ ...current, timeoutSecs: Number(e.target.value) || 1 }))}
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="mb-2 block text-sm font-medium text-zinc-300">{t('settingsPage.general.execution.sandboxPath')}</label>
                  <input
                    type="text"
                    value={generalDraft.sandboxPath}
                    onChange={(e) => setGeneralDraft((current) => ({ ...current, sandboxPath: e.target.value }))}
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>

              <div className="mt-4 space-y-3">
                <label className="flex items-center gap-3 text-sm text-zinc-200">
                  <input
                    type="checkbox"
                    checked={generalDraft.allowShell}
                    onChange={(e) => setGeneralDraft((current) => ({ ...current, allowShell: e.target.checked }))}
                    className="h-4 w-4"
                  />
                  {t('settingsPage.general.execution.allowShell')}
                </label>
                <label className="flex items-center gap-3 text-sm text-zinc-200">
                  <input
                    type="checkbox"
                    checked={generalDraft.requireConfirmation}
                    onChange={(e) => setGeneralDraft((current) => ({ ...current, requireConfirmation: e.target.checked }))}
                    className="h-4 w-4"
                  />
                  {t('settingsPage.general.execution.requireConfirmation')}
                </label>
              </div>
            </div>

            <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
              <h2 className="mb-4 text-xl font-semibold">{t('settingsPage.general.rules.title')}</h2>
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-2 block text-sm font-medium text-zinc-300">{t('settingsPage.general.rules.whitelist')}</label>
                  <textarea
                    value={generalDraft.whitelist}
                    onChange={(e) => setGeneralDraft((current) => ({ ...current, whitelist: e.target.value }))}
                    rows={8}
                    placeholder={t('settingsPage.general.rules.placeholder')}
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="mb-2 block text-sm font-medium text-zinc-300">{t('settingsPage.general.rules.blacklist')}</label>
                  <textarea
                    value={generalDraft.blacklist}
                    onChange={(e) => setGeneralDraft((current) => ({ ...current, blacklist: e.target.value }))}
                    rows={8}
                    placeholder={t('settingsPage.general.rules.placeholder')}
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>
            </div>

            {renderSaveButton(handleSaveGeneral)}
          </div>
        )}

        {activeTab === 'providers' && config && (
          <div className="space-y-6">
            <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
              <div className="mb-4 flex items-center gap-3">
                <Globe className="h-5 w-5 text-blue-400" />
                <h2 className="text-xl font-semibold">{t('settingsPage.providers.title')}</h2>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-2 block text-sm font-medium text-zinc-300">{t('settingsPage.providers.fields.model')}</label>
                  <input
                    type="text"
                    value={providerDraft.claudeModel}
                    onChange={(e) => setProviderDraft((current) => ({ ...current, claudeModel: e.target.value }))}
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="mb-2 block text-sm font-medium text-zinc-300">{t('settingsPage.providers.fields.requestTimeout')}</label>
                  <input
                    type="number"
                    min={1}
                    value={providerDraft.requestTimeoutSecs}
                    onChange={(e) => setProviderDraft((current) => ({ ...current, requestTimeoutSecs: Number(e.target.value) || 1 }))}
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>

              <div className="mt-4 max-w-sm">
                <label className="mb-2 block text-sm font-medium text-zinc-300">{t('settingsPage.providers.fields.maxRetries')}</label>
                <input
                  type="number"
                  min={0}
                  value={providerDraft.maxRetries}
                  onChange={(e) => setProviderDraft((current) => ({ ...current, maxRetries: Number(e.target.value) || 0 }))}
                  className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>

            <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
              <div className="mb-4 flex items-center gap-3">
                <Key className="h-5 w-5 text-blue-400" />
                <div>
                  <h2 className="text-xl font-semibold">{t('settingsPage.providers.apiKey.title')}</h2>
                  <p className="mt-1 text-sm text-zinc-400">{apiKeyStatus}</p>
                </div>
              </div>

              <div className="max-w-xl space-y-4">
                <div>
                  <label className="mb-2 block text-sm font-medium text-zinc-300">{t('settingsPage.providers.apiKey.label')}</label>
                  <input
                    type="password"
                    value={apiKey}
                    onChange={(e) => setApiKey(e.target.value)}
                    placeholder={config.api.apiKeyConfigured ? t('settingsPage.providers.apiKey.placeholder.replace') : t('settingsPage.providers.apiKey.placeholder.empty')}
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>

                <div className="flex flex-wrap items-center gap-3">
                  {renderSaveButton(handleSaveProvider, t('settingsPage.providers.actions.save'))}
                  <button
                    onClick={() => void removeApiKey()}
                    disabled={!config.api.apiKeyConfigured || saving}
                    className="inline-flex items-center gap-2 rounded-lg bg-red-600 px-4 py-2 text-sm font-medium transition-colors hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    <Trash2 className="h-4 w-4" />
                    {t('settingsPage.providers.actions.removeApiKey')}
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'skills' && (
          <div className="space-y-6">
            <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
              <div className="mb-3 flex items-center gap-3">
                <Zap className="h-5 w-5 text-blue-400" />
                <h2 className="text-xl font-semibold">{t('skills')}</h2>
              </div>
              <p className="text-sm leading-6 text-zinc-400">
                {t('settingsPage.skills.description')}
              </p>
            </div>
          </div>
        )}

        {activeTab === 'network' && (
          <div>
            <h2 className="mb-6 text-2xl font-bold">{t('settingsPage.network.title')}</h2>
            <ProxySettings />
          </div>
        )}

        {activeTab === 'appearance' && config && (
          <div className="space-y-6">
            <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
              <h2 className="mb-4 text-xl font-semibold">{t('settingsPage.appearance.title')}</h2>
              <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
                {['light', 'dark', 'system'].map((theme) => {
                  const selected = appearanceDraft.theme === theme;
                  return (
                    <button
                      key={theme}
                      onClick={() => setAppearanceDraft((current) => ({ ...current, theme }))}
                      className={`rounded-lg border-2 p-4 text-left transition-colors ${
                        selected ? 'border-blue-500 bg-blue-500/10' : 'border-zinc-700 bg-zinc-800 hover:border-zinc-500'
                      }`}
                    >
                      <div className="text-sm font-medium capitalize">{t(`settingsPage.appearance.themes.${theme}.label`)}</div>
                      <div className="mt-2 text-xs text-zinc-400">
                        {t(`settingsPage.appearance.themes.${theme}.description`)}
                      </div>
                    </button>
                  );
                })}
              </div>
            </div>

            <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
              <h2 className="mb-4 text-xl font-semibold">{t('language')}</h2>
              <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
                {languageOptions.map((option) => {
                  const selected = appearanceDraft.language === option.value;
                  return (
                    <button
                      key={option.value}
                      onClick={() => setAppearanceDraft((current) => ({ ...current, language: option.value }))}
                      className={`rounded-lg border-2 p-4 text-left transition-colors ${
                        selected ? 'border-blue-500 bg-blue-500/10' : 'border-zinc-700 bg-zinc-800 hover:border-zinc-500'
                      }`}
                    >
                      <div className="text-sm font-medium">{option.label}</div>
                      <div className="mt-2 text-xs text-zinc-400">{option.helper}</div>
                    </button>
                  );
                })}
              </div>
            </div>

            <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
              <h2 className="mb-4 text-xl font-semibold">{t('settingsPage.appearance.display.title')}</h2>
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <label className="mb-2 block text-sm font-medium text-zinc-300">{t('settingsPage.appearance.display.fontSize')}</label>
                  <input
                    type="number"
                    min={10}
                    max={24}
                    value={appearanceDraft.fontSize}
                    onChange={(e) => setAppearanceDraft((current) => ({ ...current, fontSize: Number(e.target.value) || 10 }))}
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="mb-2 block text-sm font-medium text-zinc-300">{t('settingsPage.appearance.display.maxHistory')}</label>
                  <input
                    type="number"
                    min={1}
                    value={appearanceDraft.maxHistory}
                    onChange={(e) => setAppearanceDraft((current) => ({ ...current, maxHistory: Number(e.target.value) || 1 }))}
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>

              <label className="mt-4 flex items-center gap-3 text-sm text-zinc-200">
                <input
                  type="checkbox"
                  checked={appearanceDraft.showTimestamps}
                  onChange={(e) => setAppearanceDraft((current) => ({ ...current, showTimestamps: e.target.checked }))}
                  className="h-4 w-4"
                />
                {t('settingsPage.appearance.display.showTimestamps')}
              </label>
            </div>

            {renderSaveButton(handleSaveAppearance)}
          </div>
        )}

        {activeTab === 'developer' && (
          <div>
            <h2 className="mb-6 text-2xl font-bold">{t('settingsPage.developer.title')}</h2>
            <DeveloperTools />
          </div>
        )}
      </div>
    </div>
  );
}

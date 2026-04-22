import { invoke } from '@tauri-apps/api/core';
import type { Config, ConfigUpdate } from '@/types';

export interface WizardState {
  currentStep: number;
  completed: boolean;
  language: string;
  aiProvider: string | null;
  apiKey: string | null;
  apiKeyProvider: string | null;
  workspaceName: string;
  enabledFeatures: string[];
}

export interface AIProvider {
  id: string;
  name: string;
  description: string;
  icon: string;
}

export const AI_PROVIDERS: AIProvider[] = [
  {
    id: 'ollama',
    name: 'Ollama (本地)',
    description: '连接本地 Ollama 服务器，无需 API Key',
    icon: '🏠',
  },
  {
    id: 'anthropic',
    name: 'Anthropic Claude',
    description: '使用 Claude 系列模型，需要 API Key',
    icon: '🤖',
  },
  {
    id: 'openai',
    name: 'OpenAI',
    description: '使用 GPT 系列模型，需要 API Key',
    icon: '🔷',
  },
  {
    id: 'moonshot',
    name: 'Moonshot (月之暗面)',
    description: '使用 Kimi 系列模型，需要 API Key',
    icon: '🌙',
  },
];

export const FEATURES = [
  { id: 'chat', name: 'AI 对话', description: '与 AI 进行自然语言对话', icon: '💬' },
  { id: 'automation', name: '自动化录制', description: '录制和回放用户操作', icon: '🎬' },
  { id: 'cron', name: '定时任务', description: '设置定时执行的自动化任务', icon: '📅' },
  { id: 'skills', name: '技能市场', description: '浏览和安装 WASM 技能', icon: '✨' },
  { id: 'hotkeys', name: '热键管理', description: '配置全局快捷键', icon: '⌨️' },
  { id: 'workflow', name: '工作流', description: '创建自动化工作流图表', icon: '🔀' },
] as const;

export type FeatureId = (typeof FEATURES)[number]['id'];

// 获取向导状态
export async function getWizardState(): Promise<WizardState> {
  return await invoke<WizardState>('wizard_get_state');
}

// 保存向导状态
export async function saveWizardState(state: Partial<WizardState>): Promise<void> {
  await invoke('wizard_save_state', { state: JSON.stringify(state) });
}

// 完成向导
export async function completeWizard(): Promise<void> {
  await invoke('wizard_complete');
}

// 检查是否需要显示向导
export async function shouldShowWizard(): Promise<boolean> {
  const state = await getWizardState();
  return !state.completed;
}

// 检测 Ollama 是否可用
export async function checkOllamaConnection(): Promise<{ available: boolean; models: string[] }> {
  return await invoke<{ available: boolean; models: string[] }>('ollama_check_connection');
}

// 保存 API Key
export async function saveApiKey(provider: string, apiKey: string): Promise<void> {
  await invoke('wizard_save_api_key', { provider, apiKey });
}

// 创建工作区
export async function createWorkspace(name: string): Promise<void> {
  await invoke('create_workspace', { name, description: '' });
}

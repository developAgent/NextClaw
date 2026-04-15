export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
}

export interface Session {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
  messageCount?: number;
}

export interface CommandExecution {
  id: string;
  sessionId: string;
  command: string;
  exitCode?: number;
  stdout?: string;
  stderr?: string;
  durationMs?: number;
  createdAt: string;
}

export interface FileMetadata {
  path: string;
  size: number;
  isFile: boolean;
  isDir: boolean;
  isReadonly: boolean;
  modified?: number;
}

export interface Config {
  api: {
    claudeModel: string;
    requestTimeoutSecs: number;
    maxRetries: number;
    apiKeyConfigured: boolean;
  };
  commands: {
    timeoutSecs: number;
    allowShell: boolean;
    whitelist: string[];
    blacklist: string[];
    sandboxPath: string;
    requireConfirmation: boolean;
  };
  ui: {
    theme: string;
    fontSize: number;
    showTimestamps: boolean;
    maxHistory: number;
  };
}

export interface ConfigUpdate {
  claudeModel?: string;
  requestTimeoutSecs?: number;
  maxRetries?: number;
  timeoutSecs?: number;
  whitelist?: string[];
  blacklist?: string[];
  sandboxPath?: string;
  requireConfirmation?: boolean;
  theme?: string;
  fontSize?: number;
  showTimestamps?: boolean;
}

export interface Channel {
  id: string;
  name: string;
  provider: 'claude' | 'openai' | 'gemini';
  model: string;
  apiKey?: string;
  apiBase?: string;
  priority: number;
  enabled: boolean;
  healthStatus: 'healthy' | 'degraded' | 'unhealthy' | 'unknown';
  lastUsed?: number;
  createdAt: number;
  updatedAt: number;
}

export interface Plugin {
  id: string;
  name: string;
  version: string;
  author?: string;
  description?: string;
  enabled: boolean;
  config?: string;
  installedAt: number;
  updatedAt: number;
}

export interface Hotkey {
  id: string;
  action: string;
  keyCombination: string;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export type MessageType = 'user' | 'assistant' | 'system';

// Proxy types
export type ProxyType = 'http' | 'https' | 'socks5';

export interface ProxyConfig {
  enabled: boolean;
  server: string;
  port: number;
  username?: string;
  password?: string;
  proxyType: ProxyType;
  bypassRules: string[];
}

export interface TestResult {
  success: boolean;
  latencyMs: number;
  message: string;
}

// Update types
export interface UpdateInfo {
  version: string;
  body: string;
  date: string;
}

export interface DownloadProgress {
  totalLength?: number;
  currentLength?: number;
  chunkLength: number;
}

export interface UpdateStatus {
  isChecking: boolean;
  isDownloading: boolean;
}

// Gateway types
export interface GatewayStatus {
  running: boolean;
  pid?: number;
  port: number;
  uptime: number;
  token?: string;
  controlUrl: string;
}

export interface GatewayConfig {
  autoStart: boolean;
  token?: string;
  port: number;
  proxyEnabled: boolean;
  proxyServer?: string;
  proxyBypassRules: string[];
}

// Marketplace types
export interface SkillMetadata {
  slug: string;
  name: string;
  version: string;
  description: string;
  author: string;
  icon?: string;
  tags: string[];
  dependencies: string[];
}

export interface InstalledSkill {
  slug: string;
  installedAt: string;
  installedPath: string;
}

// Developer tools types
export type DiagnosticStatus = 'healthy' | 'warning' | 'critical';

export interface DiagnosticCheck {
  name: string;
  status: DiagnosticStatus;
  message: string;
  details?: string;
}

export interface DiagnosticInfo {
  status: DiagnosticStatus;
  checks: DiagnosticCheck[];
  summary: string;
}

export interface TelemetryData {
  sessionId: string;
  agentId?: string;
  modelId: string;
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
  timestamp: string;
}

export interface TokenUsageStats {
  totalPromptTokens: number;
  totalCompletionTokens: number;
  totalTokens: number;
  totalRequests: number;
}

export interface LogEntry {
  id: string;
  level: string;
  message: string;
  timestamp: string;
  context?: string;
}

export type RuntimeState = 'stopped' | 'starting' | 'running' | 'stopping' | 'error';

export interface RuntimeStatus {
  state: RuntimeState;
  port: number;
  pid?: number;
  startedAt?: string;
  healthy?: boolean;
  controlUrl?: string;
}

export interface RuntimeConfig {
  autoStart: boolean;
  token?: string;
  port: number;
  proxyEnabled: boolean;
  proxyServer?: string;
  proxyHttpServer?: string;
  proxyHttpsServer?: string;
  proxyAllServer?: string;
  proxyBypassRules?: string;
}

export interface Workspace {
  id: string;
  name: string;
  description?: string;
  isCurrent: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CreateWorkspaceInput {
  name: string;
  description?: string;
}

export interface SystemInfo {
  os: string;
  arch: string;
  family: string;
  version: string;
}
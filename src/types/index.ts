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
}

export type MessageType = 'user' | 'assistant' | 'system';
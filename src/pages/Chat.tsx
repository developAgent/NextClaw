import { useState, useEffect, useRef } from 'react';
import { Send, Plus, Trash2, Bot, X } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  isStreaming?: boolean;
}

interface Session {
  id: string;
  agent_id?: string | null;
  title: string;
  created_at: string;
  updated_at?: string;
}

interface Agent {
  id: string;
  name: string;
  description?: string;
  provider_id?: string;
  model_id?: string;
  system_prompt?: string;
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'string') {
    return error;
  }

  return 'Unknown error';
}

export default function Chat() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [currentSession, setCurrentSession] = useState<Session | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [selectedAgentId, setSelectedAgentId] = useState('');

  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadInitialData();

    const unlisten = listen<string>('chat:stream', (event) => {
      setMessages((prev) => {
        const lastMessage = prev[prev.length - 1];
        if (lastMessage && lastMessage.isStreaming) {
          const updated = [...prev];
          updated[updated.length - 1] = {
            ...lastMessage,
            content: lastMessage.content + event.payload,
          };
          return updated;
        }
        return prev;
      });
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const loadInitialData = async () => {
    await Promise.all([loadAgents(), loadSessions()]);
  };

  const loadAgents = async () => {
    try {
      const data = await invoke<Agent[]>('get_all_agents');
      setAgents(data);
      setSelectedAgentId((current) => {
        if (current && data.some((agent) => agent.id === current)) {
          return current;
        }
        return data[0]?.id ?? '';
      });
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  };

  const loadSessions = async () => {
    try {
      const sessionsData = await invoke<Session[]>('list_sessions');
      setSessions(sessionsData);

      if (sessionsData.length > 0) {
        setCurrentSession((current) => {
          const nextSession = current
            ? sessionsData.find((session) => session.id === current.id) ?? sessionsData[0]
            : sessionsData[0];

          void loadMessages(nextSession.id);
          if (nextSession.agent_id) {
            setSelectedAgentId(nextSession.agent_id);
          }
          return nextSession;
        });
      } else {
        setCurrentSession(null);
        setMessages([]);
      }
    } catch (error) {
      console.error('Failed to load sessions:', error);
    }
  };

  const loadMessages = async (sessionId: string) => {
    try {
      const messagesData = await invoke<Message[]>('get_chat_history', {
        sessionId,
      });
      setMessages(messagesData);
    } catch (error) {
      console.error('Failed to load messages:', error);
    }
  };

  const createNewSession = async () => {
    try {
      const session = await invoke<Session>('create_session', {
        title: 'New Chat',
        agentId: selectedAgentId || undefined,
      });
      setSessions((prev) => [session, ...prev]);
      setCurrentSession(session);
      setMessages([]);
      if (session.agent_id) {
        setSelectedAgentId(session.agent_id);
      }
    } catch (error) {
      console.error('Failed to create session:', error);
    }
  };

  const deleteSession = async (sessionId: string) => {
    try {
      await invoke('delete_session', { sessionId });
      setSessions((prev) => prev.filter((session) => session.id !== sessionId));
      if (currentSession?.id === sessionId) {
        setCurrentSession(null);
        setMessages([]);
      }
    } catch (error) {
      console.error('Failed to delete session:', error);
    }
  };

  const handleSend = async () => {
    if (!inputValue.trim() || isLoading) return;
    if (!selectedAgentId && !currentSession?.agent_id) {
      alert('Please select an agent first.');
      return;
    }

    const userContent = inputValue.trim();
    const userMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: userContent,
      timestamp: new Date().toISOString(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInputValue('');
    setIsLoading(true);

    const assistantMessageId = (Date.now() + 1).toString();
    const assistantMessage: Message = {
      id: assistantMessageId,
      role: 'assistant',
      content: '',
      timestamp: new Date().toISOString(),
      isStreaming: true,
    };
    setMessages((prev) => [...prev, assistantMessage]);

    try {
      let session = currentSession;
      if (!session) {
        session = await invoke<Session>('create_session', {
          title: userContent.slice(0, 50) + (userContent.length > 50 ? '...' : ''),
          agentId: selectedAgentId || undefined,
        });
        setSessions((prev) => [session!, ...prev]);
        setCurrentSession(session);
      }

      const response = await invoke<string>('send_message', {
        message: userContent,
        sessionId: session.id,
        agentId: (session.agent_id ?? selectedAgentId) || undefined,
      });

      const updatedSession: Session = {
        ...session,
        agent_id: (session.agent_id ?? selectedAgentId) || undefined,
      };

      setCurrentSession(updatedSession);
      setSessions((prev) => {
        const others = prev.filter((item) => item.id !== updatedSession.id);
        return [updatedSession, ...others];
      });

      setMessages((prev) =>
        prev.map((message) =>
          message.id === assistantMessageId
            ? {
                ...message,
                content: response,
                isStreaming: false,
              }
            : message
        )
      );
    } catch (error) {
      console.error('Failed to send message:', error);
      setMessages((prev) => prev.filter((message) => message.id !== assistantMessageId));

      const errorMessage: Message = {
        id: (Date.now() + 2).toString(),
        role: 'assistant',
        content: `Sorry, there was an error processing your request. ${getErrorMessage(error)}`,
        timestamp: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      void handleSend();
    }
  };

  const selectSession = (session: Session) => {
    setCurrentSession(session);
    setSelectedAgentId(session.agent_id ?? selectedAgentId);
    void loadMessages(session.id);
  };

  const selectedAgent = agents.find((agent) => agent.id === (currentSession?.agent_id ?? selectedAgentId));

  return (
    <div className="flex h-full">
      {sidebarOpen && (
        <div className="w-72 bg-zinc-900 border-r border-zinc-800 flex flex-col">
          <div className="p-4 border-b border-zinc-800 space-y-3">
            <button
              onClick={createNewSession}
              className="w-full flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
            >
              <Plus className="w-4 h-4" />
              <span className="text-sm">New Chat</span>
            </button>

            <div>
              <label className="block text-xs text-zinc-500 mb-1">Agent</label>
              <div className="relative">
                <Bot className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" />
                <select
                  value={currentSession?.agent_id ?? selectedAgentId}
                  onChange={(e) => {
                    if (currentSession?.agent_id) {
                      return;
                    }
                    setSelectedAgentId(e.target.value);
                  }}
                  disabled={Boolean(currentSession?.agent_id) || agents.length === 0}
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg pl-9 pr-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-60"
                >
                  <option value="">Select agent</option>
                  {agents.map((agent) => (
                    <option key={agent.id} value={agent.id}>
                      {agent.name}
                    </option>
                  ))}
                </select>
              </div>
              {currentSession?.agent_id && (
                <p className="mt-1 text-xs text-zinc-500">This session is bound to its selected agent.</p>
              )}
            </div>

            {selectedAgent && (
              <div className="rounded-lg border border-zinc-800 bg-zinc-950/60 p-3 text-xs text-zinc-400 space-y-1">
                <div className="font-medium text-zinc-200">{selectedAgent.name}</div>
                <div>{selectedAgent.provider_id || 'anthropic'} · {selectedAgent.model_id || 'default model'}</div>
                {selectedAgent.description && <div className="line-clamp-2">{selectedAgent.description}</div>}
              </div>
            )}
          </div>

          <div className="flex-1 overflow-y-auto p-2">
            {sessions.map((session) => {
              const sessionAgent = agents.find((agent) => agent.id === session.agent_id);
              return (
                <div
                  key={session.id}
                  onClick={() => selectSession(session)}
                  className={`group px-3 py-2 rounded-lg cursor-pointer transition-colors ${
                    currentSession?.id === session.id
                      ? 'bg-blue-600 text-white'
                      : 'text-zinc-400 hover:bg-zinc-800'
                  }`}
                >
                  <div className="flex items-start gap-2">
                    <div className="flex-1 min-w-0">
                      <div className="truncate text-sm">{session.title}</div>
                      <div className={`truncate text-xs ${currentSession?.id === session.id ? 'text-blue-100' : 'text-zinc-500'}`}>
                        {sessionAgent?.name || 'No agent'}
                      </div>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        void deleteSession(session.id);
                      }}
                      className="p-1 hover:bg-red-600/20 rounded opacity-0 group-hover:opacity-100 transition-opacity"
                    >
                      <Trash2 className="w-3 h-3" />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      <div className="flex-1 flex flex-col">
        <div className="border-b border-zinc-800 px-6 py-4 flex items-center justify-between gap-4">
          <div>
            <h1 className="text-lg font-semibold">{currentSession?.title || 'Chat'}</h1>
            <p className="text-sm text-zinc-500">
              {selectedAgent
                ? `${selectedAgent.name} · ${selectedAgent.provider_id || 'anthropic'} · ${selectedAgent.model_id || 'default model'}`
                : 'Select an agent to start chatting'}
            </p>
          </div>
          <button
            onClick={() => setSidebarOpen((open) => !open)}
            className="px-3 py-2 text-zinc-400 hover:bg-zinc-800 rounded-lg transition-colors"
          >
            {sidebarOpen ? <X className="w-5 h-5" /> : <Plus className="w-5 h-5" />}
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-6">
          {messages.length === 0 && (
            <div className="flex items-center justify-center h-full text-zinc-500">
              <div className="text-center max-w-md">
                <p className="text-lg mb-2">Start a new conversation</p>
                <p className="text-sm">
                  {selectedAgent
                    ? `Your messages will be sent through ${selectedAgent.name}.`
                    : 'Create an agent first, then choose it here to start chatting.'}
                </p>
              </div>
            </div>
          )}

          {messages.map((message) => (
            <div
              key={message.id}
              className={`flex mb-4 ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
            >
              <div
                className={`max-w-[70%] rounded-lg px-4 py-2 ${
                  message.role === 'user'
                    ? 'bg-blue-600 text-white'
                    : message.role === 'system'
                      ? 'bg-zinc-900 border border-zinc-800 text-zinc-300'
                      : 'bg-zinc-800 text-zinc-100'
                }`}
              >
                <p className="text-sm whitespace-pre-wrap">
                  {message.content}
                  {message.isStreaming && <span className="animate-pulse">▊</span>}
                </p>
              </div>
            </div>
          ))}

          {isLoading && !messages.find((message) => message.isStreaming) && (
            <div className="flex justify-start mb-4">
              <div className="bg-zinc-800 rounded-lg px-4 py-2">
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 bg-zinc-500 rounded-full animate-bounce" />
                  <div className="w-2 h-2 bg-zinc-500 rounded-full animate-bounce" style={{ animationDelay: '0.1s' }} />
                  <div className="w-2 h-2 bg-zinc-500 rounded-full animate-bounce" style={{ animationDelay: '0.2s' }} />
                </div>
              </div>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        <div className="border-t border-zinc-800 p-4">
          <div className="flex gap-2">
            <textarea
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyDown={handleKeyPress}
              placeholder={selectedAgent ? `Message ${selectedAgent.name}...` : 'Select an agent before typing...'}
              className="flex-1 bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
              rows={1}
              style={{ minHeight: '40px', maxHeight: '120px' }}
              disabled={isLoading || (!selectedAgent && !currentSession?.agent_id)}
            />
            <button
              onClick={() => void handleSend()}
              disabled={isLoading || !inputValue.trim() || (!selectedAgent && !currentSession?.agent_id)}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-zinc-700 disabled:cursor-not-allowed rounded-lg transition-colors"
            >
              <Send className="w-5 h-5" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

import { useState, useEffect, useRef } from 'react';
import { Send, Plus, Trash2, Settings, X, Bot, User, FileText, Image as ImageIcon, Copy, Eye, Paperclip } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import ReactMarkdown from 'react-markdown';

interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  isStreaming?: boolean;
  agentId?: string;
  agentName?: string;
  attachments?: Attachment[];
  contextId?: string;
}

interface Attachment {
  id: string;
  name: string;
  type: 'image' | 'document' | 'code';
  url: string;
  size: number;
}

interface Context {
  id: string;
  name: string;
  messages: string[];
}

interface Agent {
  id: string;
  name: string;
  description?: string;
}

interface Session {
  id: string;
  title: string;
  created_at: string;
  agentId?: string;
  contextId?: string;
}

interface ChatCompletionRequest {
  model: string;
  messages: Array<{ role: string; content: string }>;
  temperature?: number;
  max_tokens?: number;
  agentId?: string;
  contextId?: string;
}

export default function ChatEnhanced() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [currentSession, setCurrentSession] = useState<Session | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [selectedProvider, setSelectedProvider] = useState<'openai' | 'anthropic'>('anthropic');
  const [selectedModel, setSelectedModel] = useState<string>('claude-3-opus-20240229');
  const [agents, setAgents] = useState<Agent[]>([]);
  const [contexts, setContexts] = useState<Context[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<string | undefined>();
  const [selectedContext, setSelectedContext] = useState<string | undefined>();
  const [attachments, setAttachments] = useState<Attachment[]>([]);
  const [selectedMessage, setSelectedMessage] = useState<Message | null>(null);
  const [showMessageDetails, setShowMessageDetails] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    loadSessions();
    loadAgents();
    loadContexts();

    // Listen for streaming events
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
      unlisten.then(fn => fn());
    };
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const loadSessions = async () => {
    try {
      const sessionsData = await invoke<Session[]>('list_sessions');
      setSessions(sessionsData);
      if (sessionsData.length > 0 && !currentSession) {
        setCurrentSession(sessionsData[0]);
        await loadMessages(sessionsData[0].id);
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

  const loadAgents = async () => {
    try {
      const agentsData = await invoke<Agent[]>('get_all_agents');
      setAgents(agentsData);
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  };

  const loadContexts = async () => {
    try {
      // This would be implemented in the backend
      const contextsData: Context[] = [
        { id: 'default', name: 'Default', messages: [] },
        { id: 'coding', name: 'Coding Assistant', messages: [] },
      ];
      setContexts(contextsData);
    } catch (error) {
      console.error('Failed to load contexts:', error);
    }
  };

  const createNewSession = async () => {
    try {
      const session = await invoke<Session>('create_session', {
        title: 'New Chat',
      });
      setSessions([...sessions, session]);
      setCurrentSession(session);
      setMessages([]);
      setSelectedAgent(undefined);
      setSelectedContext(undefined);
    } catch (error) {
      console.error('Failed to create session:', error);
    }
  };

  const deleteSession = async (sessionId: string) => {
    try {
      await invoke('delete_session', { sessionId });
      setSessions(sessions.filter((s) => s.id !== sessionId));
      if (currentSession?.id === sessionId) {
        setCurrentSession(null);
        setMessages([]);
      }
    } catch (error) {
      console.error('Failed to delete session:', error);
    }
  };

  const handleAttachFile = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: 'Documents',
            extensions: ['txt', 'md', 'pdf', 'doc', 'docx'],
          },
          {
            name: 'Images',
            extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'],
          },
          {
            name: 'Code',
            extensions: ['js', 'ts', 'py', 'java', 'cpp', 'c', 'rs', 'go'],
          },
        ],
      }) as string[] | null;

      if (selected) {
        // In a real implementation, you would upload the files
        // For now, we'll just create mock attachments
        const newAttachments: Attachment[] = await Promise.all(
          selected.map(async (path: string) => {
            const fileName = path.split(/[/\\]/).pop() || 'unknown';
            const ext = fileName.split('.').pop()?.toLowerCase() || '';
            let type: 'image' | 'document' | 'code' = 'document';
            if (['png', 'jpg', 'jpeg', 'gif', 'webp'].includes(ext)) {
              type = 'image';
            } else if (['js', 'ts', 'py', 'java', 'cpp', 'c', 'rs', 'go'].includes(ext)) {
              type = 'code';
            }

            return {
              id: Math.random().toString(),
              name: fileName,
              type,
              url: path,
              size: 0, // Would get actual file size
            };
          })
        );

        setAttachments([...attachments, ...newAttachments]);
      }
    } catch (error) {
      console.error('Failed to attach file:', error);
    }
  };

  const removeAttachment = (attachmentId: string) => {
    setAttachments(attachments.filter(a => a.id !== attachmentId));
  };

  const parseAgentMention = (content: string): { content: string; agentId?: string } => {
    const agentMatch = content.match(/@(\w+)/);
    if (agentMatch) {
      const agentName = agentMatch[1];
      const agent = agents.find(a => a.name.toLowerCase() === agentName.toLowerCase());
      if (agent) {
        return {
          content: content.replace(agentMatch[0], ''),
          agentId: agent.id,
        };
      }
    }
    return { content, agentId: undefined };
  };

  const handleSend = async () => {
    if (!inputValue.trim() && attachments.length === 0 || isLoading) return;

    const { content: cleanContent, agentId: mentionedAgentId } = parseAgentMention(inputValue);

    const userMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: cleanContent,
      timestamp: new Date().toISOString(),
      attachments: attachments.length > 0 ? [...attachments] : undefined,
      agentId: mentionedAgentId || selectedAgent,
      agentName: mentionedAgentId
        ? agents.find(a => a.id === mentionedAgentId)?.name
        : selectedAgent
          ? agents.find(a => a.id === selectedAgent)?.name
          : undefined,
    };

    setMessages([...messages, userMessage]);
    const userContent = cleanContent;
    const userAttachments = [...attachments];
    setInputValue('');
    setAttachments([]);
    setIsLoading(true);

    try {
      // Create session if needed
      let sessionId = currentSession?.id;
      if (!currentSession) {
        const session = await invoke<Session>('create_session', {
          title: userContent.slice(0, 50) + (userContent.length > 50 ? '...' : 'New Chat'),
        });
        setSessions([...sessions, session]);
        setCurrentSession(session);
        sessionId = session.id;
      }

      // Prepare conversation history
      const conversationHistory = [
        ...messages.filter(m => !m.isStreaming).map(m => ({
          role: m.role,
          content: m.content,
        })),
        {
          role: 'user',
          content: userContent,
        },
      ];

      // Add assistant message for streaming
      const assistantMessageId = (Date.now() + 1).toString();
      const assistantMessage: Message = {
        id: assistantMessageId,
        role: 'assistant',
        content: '',
        timestamp: new Date().toISOString(),
        isStreaming: true,
        agentId: mentionedAgentId || selectedAgent,
        agentName: mentionedAgentId
          ? agents.find(a => a.id === mentionedAgentId)?.name
          : selectedAgent
            ? agents.find(a => a.id === selectedAgent)?.name
            : undefined,
      };
      setMessages((prev) => [...prev, assistantMessage]);

      // Send request based on selected provider
      if (selectedProvider === 'openai') {
        const request: ChatCompletionRequest = {
          model: selectedModel,
          messages: conversationHistory,
          temperature: 0.7,
          max_tokens: 4096,
          agentId: mentionedAgentId || selectedAgent,
          contextId: selectedContext,
        };

        const response = await invoke<{ content: string }>('create_chat_completion', {
          request,
        });

        // Update with final response
        setMessages((prev) =>
          prev.map((m) =>
            m.id === assistantMessageId
              ? {
                  ...m,
                  content: response.content,
                  isStreaming: false,
                }
              : m
          )
        );
      } else {
        // Anthropic
        const request = {
          model: selectedModel,
          messages: conversationHistory,
          temperature: 0.7,
          max_tokens: 4096,
          agentId: mentionedAgentId || selectedAgent,
          contextId: selectedContext,
        };

        const response = await invoke<{ content: string }>('create_anthropic_message', {
          request,
        });

        // Update with final response
        setMessages((prev) =>
          prev.map((m) =>
            m.id === assistantMessageId
              ? {
                  ...m,
                  content: response.content,
                  isStreaming: false,
                }
              : m
          )
        );
      }
    } catch (error) {
      console.error('Failed to send message:', error);
      // Remove streaming message and show error
      setMessages((prev) => prev.filter(m => !m.isStreaming));

      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: 'Sorry, there was an error processing your request.',
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
      handleSend();
    }
  };

  const selectSession = (session: Session) => {
    setCurrentSession(session);
    loadMessages(session.id);
    setSelectedAgent(session.agentId);
    setSelectedContext(session.contextId);
  };

  const copyMessageContent = (content: string) => {
    navigator.clipboard.writeText(content);
  };

  const showMessageDetailsModal = (message: Message) => {
    setSelectedMessage(message);
    setShowMessageDetails(true);
  };

  const openAIModels = ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-3.5-turbo'];
  const anthropicModels = ['claude-3-opus-20240229', 'claude-3-sonnet-20240229', 'claude-3-haiku-20240307'];

  const currentModels = selectedProvider === 'openai' ? openAIModels : anthropicModels;

  return (
    <div className="flex h-full">
      {/* Sidebar */}
      {sidebarOpen && (
        <div className="w-72 bg-zinc-900 border-r border-zinc-800 flex flex-col">
          <div className="p-4 border-b border-zinc-800">
            <button
              onClick={createNewSession}
              className="w-full flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
            >
              <Plus className="w-4 h-4" />
              <span className="text-sm">New Chat</span>
            </button>
          </div>

          {/* Agent Selection */}
          {agents.length > 0 && (
            <div className="p-4 border-b border-zinc-800">
              <label className="block text-xs text-zinc-400 mb-2">Agent</label>
              <select
                value={selectedAgent || ''}
                onChange={(e) => setSelectedAgent(e.target.value || undefined)}
                className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="">No Agent</option>
                {agents.map((agent) => (
                  <option key={agent.id} value={agent.id}>{agent.name}</option>
                ))}
              </select>
            </div>
          )}

          {/* Context Selection */}
          {contexts.length > 0 && (
            <div className="p-4 border-b border-zinc-800">
              <label className="block text-xs text-zinc-400 mb-2">Context</label>
              <select
                value={selectedContext || ''}
                onChange={(e) => setSelectedContext(e.target.value || undefined)}
                className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                {contexts.map((context) => (
                  <option key={context.id} value={context.id}>{context.name}</option>
                ))}
              </select>
            </div>
          )}

          <div className="flex-1 overflow-y-auto p-2">
            {sessions.map((session) => (
              <div
                key={session.id}
                onClick={() => selectSession(session)}
                className={`group flex items-start gap-2 px-3 py-2 rounded-lg cursor-pointer transition-colors ${
                  currentSession?.id === session.id
                    ? 'bg-blue-600 text-white'
                    : 'text-zinc-400 hover:bg-zinc-800'
                }`}
              >
                <div className="flex-1 min-w-0">
                  <p className="text-sm truncate">{session.title}</p>
                  {session.agentId && (
                    <p className="text-xs opacity-70 truncate">
                      {agents.find(a => a.id === session.agentId)?.name}
                    </p>
                  )}
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    deleteSession(session.id);
                  }}
                  className="p-1 hover:bg-red-600/20 rounded opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>

          <div className="p-4 border-t border-zinc-800 space-y-2">
            {/* Provider Selection */}
            <select
              value={selectedProvider}
              onChange={(e) => {
                setSelectedProvider(e.target.value as 'openai' | 'anthropic');
                setSelectedModel(currentModels[0]);
              }}
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="openai">OpenAI</option>
              <option value="anthropic">Anthropic</option>
            </select>

            {/* Model Selection */}
            <select
              value={selectedModel}
              onChange={(e) => setSelectedModel(e.target.value)}
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {currentModels.map((model) => (
                <option key={model} value={model}>{model}</option>
              ))}
            </select>
          </div>
        </div>
      )}

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col">
        {/* Messages */}
        <div className="flex-1 overflow-y-auto p-6">
          {messages.length === 0 && (
            <div className="flex items-center justify-center h-full text-zinc-500">
              <div className="text-center">
                <p className="text-lg mb-2">Start a new conversation</p>
                <p className="text-sm">Type a message below or use @agent to route to a specific agent</p>
              </div>
            </div>
          )}

          {messages.map((message) => (
            <div
              key={message.id}
              className={`flex mb-4 group ${
                message.role === 'user' ? 'justify-end' : 'justify-start'
              }`}
            >
              <div
                className={`max-w-[80%] rounded-lg ${
                  message.role === 'user'
                    ? 'bg-blue-600 text-white'
                    : 'bg-zinc-800 text-zinc-100'
                }`}
              >
                {/* Agent Badge */}
                {message.agentName && (
                  <div className={`flex items-center gap-2 px-4 py-2 border-b ${
                    message.role === 'user' ? 'border-blue-500' : 'border-zinc-700'
                  }`}>
                    <Bot className="w-4 h-4" />
                    <span className="text-xs font-medium">{message.agentName}</span>
                  </div>
                )}

                {/* Attachments */}
                {message.attachments && message.attachments.length > 0 && (
                  <div className="px-4 py-2 border-b border-zinc-700">
                    {message.attachments.map((attachment) => (
                      <div
                        key={attachment.id}
                        className="flex items-center gap-2 bg-zinc-900/50 rounded px-3 py-2 mb-1"
                      >
                        {attachment.type === 'image' ? (
                          <ImageIcon className="w-4 h-4" />
                        ) : (
                          <FileText className="w-4 h-4" />
                        )}
                        <span className="text-sm">{attachment.name}</span>
                      </div>
                    ))}
                  </div>
                )}

                {/* Message Content */}
                <div className="px-4 py-3">
                  {message.role === 'assistant' ? (
                    <div className="prose prose-invert max-w-none">
                      <ReactMarkdown>{message.content}</ReactMarkdown>
                      {message.isStreaming && <span className="animate-pulse">▊</span>}
                    </div>
                  ) : (
                    <p className="text-sm whitespace-pre-wrap">
                      {message.content}
                      {message.isStreaming && <span className="animate-pulse">▊</span>}
                    </p>
                  )}
                </div>

                {/* Message Actions */}
                <div className={`flex items-center gap-2 px-4 py-2 ${
                  message.role === 'user' ? 'bg-blue-700/50' : 'bg-zinc-700/50'
                } opacity-0 group-hover:opacity-100 transition-opacity`}>
                  <button
                    onClick={() => copyMessageContent(message.content)}
                    className="p-1 hover:bg-zinc-600 rounded transition-colors"
                    title="Copy"
                  >
                    <Copy className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => showMessageDetailsModal(message)}
                    className="p-1 hover:bg-zinc-600 rounded transition-colors"
                    title="View Details"
                  >
                    <Eye className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          ))}

          {isLoading && !messages.find(m => m.isStreaming) && (
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

        {/* Attachments Preview */}
        {attachments.length > 0 && (
          <div className="px-6 py-2 bg-zinc-900 border-t border-zinc-800">
            <div className="flex flex-wrap gap-2">
              {attachments.map((attachment) => (
                <div
                  key={attachment.id}
                  className="flex items-center gap-2 bg-zinc-800 rounded-lg px-3 py-1"
                >
                  {attachment.type === 'image' ? (
                    <ImageIcon className="w-4 h-4 text-blue-400" />
                  ) : (
                    <FileText className="w-4 h-4 text-zinc-400" />
                  )}
                  <span className="text-sm">{attachment.name}</span>
                  <button
                    onClick={() => removeAttachment(attachment.id)}
                    className="p-1 hover:bg-zinc-700 rounded"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Input Area */}
        <div className="border-t border-zinc-800 p-4">
          <div className="flex gap-2">
            <button
              onClick={() => setSidebarOpen(!sidebarOpen)}
              className="px-3 py-2 text-zinc-400 hover:bg-zinc-800 rounded-lg transition-colors"
            >
              {sidebarOpen ? <X className="w-5 h-5" /> : <Plus className="w-5 h-5" />}
            </button>
            <button
              onClick={handleAttachFile}
              className="px-3 py-2 text-zinc-400 hover:bg-zinc-800 rounded-lg transition-colors"
              title="Attach File"
            >
              <Paperclip className="w-5 h-5" />
            </button>
            <textarea
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="Type a message... (use @agent to route to specific agent)"
              className="flex-1 bg-zinc-800 border border-zinc-700 rounded-lg px-4 py-2 resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
              rows={1}
              style={{ minHeight: '40px', maxHeight: '120px' }}
              disabled={isLoading}
            />
            <button
              onClick={handleSend}
              disabled={isLoading || (!inputValue.trim() && attachments.length === 0)}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-zinc-700 disabled:cursor-not-allowed rounded-lg transition-colors"
            >
              <Send className="w-5 h-5" />
            </button>
          </div>
        </div>
      </div>

      {/* Message Details Modal */}
      {showMessageDetails && selectedMessage && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowMessageDetails(false)}>
          <div className="bg-zinc-900 rounded-lg max-w-2xl w-full max-h-[80vh] overflow-auto" onClick={(e) => e.stopPropagation()}>
            <div className="p-4 border-b border-zinc-800 flex items-center justify-between">
              <h3 className="font-semibold">Message Details</h3>
              <button onClick={() => setShowMessageDetails(false)} className="p-1 hover:bg-zinc-800 rounded">
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="p-4 space-y-4">
              <div>
                <p className="text-sm text-zinc-400 mb-1">Role</p>
                <p className="font-medium capitalize">{selectedMessage.role}</p>
              </div>
              <div>
                <p className="text-sm text-zinc-400 mb-1">Timestamp</p>
                <p>{new Date(selectedMessage.timestamp).toLocaleString()}</p>
              </div>
              {selectedMessage.agentId && (
                <div>
                  <p className="text-sm text-zinc-400 mb-1">Agent ID</p>
                  <p className="font-medium">{selectedMessage.agentId}</p>
                </div>
              )}
              {selectedMessage.agentName && (
                <div>
                  <p className="text-sm text-zinc-400 mb-1">Agent Name</p>
                  <p className="font-medium">{selectedMessage.agentName}</p>
                </div>
              )}
              {selectedMessage.attachments && selectedMessage.attachments.length > 0 && (
                <div>
                  <p className="text-sm text-zinc-400 mb-1">Attachments</p>
                  <div className="space-y-1">
                    {selectedMessage.attachments.map((attachment) => (
                      <div key={attachment.id} className="text-sm bg-zinc-800 rounded px-3 py-2">
                        {attachment.name} ({attachment.type})
                      </div>
                    ))}
                  </div>
                </div>
              )}
              <div>
                <p className="text-sm text-zinc-400 mb-1">Content</p>
                <pre className="bg-zinc-800 rounded p-3 text-sm overflow-auto max-h-64 whitespace-pre-wrap">
                  {selectedMessage.content}
                </pre>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
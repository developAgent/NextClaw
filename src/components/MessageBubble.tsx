import ReactMarkdown from 'react-markdown';
import { Message } from '@/types';
import { formatTimestamp } from '@/utils/helpers';

interface MessageBubbleProps {
  message: Message;
}

export default function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === 'user';
  const isSystem = message.role === 'system';

  if (isSystem) {
    return (
      <div className="flex justify-center">
        <div className="bg-zinc-800/50 px-4 py-2 rounded-full text-sm text-zinc-400">
          {message.content}
        </div>
      </div>
    );
  }

  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}>
      <div className={`max-w-3xl ${isUser ? 'ml-auto' : ''}`}>
        <div className="flex items-center gap-2 mb-1">
          <span className="text-xs text-zinc-500">
            {isUser ? 'You' : 'CEOClaw'}
          </span>
          <span className="text-xs text-zinc-600">
            {formatTimestamp(message.timestamp)}
          </span>
        </div>
        <div
          className={`rounded-lg px-4 py-3 ${
            isUser
              ? 'bg-blue-600 text-white'
              : 'bg-zinc-800 text-zinc-100'
          }`}
        >
          {message.role === 'assistant' ? (
            <ReactMarkdown
              components={{
                code({ node, inline, className, children, ...props }) {
                  const match = /language-(\w+)/.exec(className || '');
                  return !inline && match ? (
                    <code className={className} {...props}>
                      {children}
                    </code>
                  ) : (
                    <code className="bg-zinc-700 px-1 py-0.5 rounded text-sm" {...props}>
                      {children}
                    </code>
                  );
                },
                pre({ children }) {
                  return (
                    <pre className="bg-zinc-900 p-3 rounded-lg overflow-x-auto">
                      {children}
                    </pre>
                  );
                },
              }}
            >
              {message.content}
            </ReactMarkdown>
          ) : (
            <p className="whitespace-pre-wrap">{message.content}</p>
          )}
        </div>
      </div>
    </div>
  );
}
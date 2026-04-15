import { useState } from 'react';
import { MessageSquare, Cog } from 'lucide-react';

export default function DashboardSimple() {
  const [activeTab, setActiveTab] = useState<'chat' | 'settings'>('chat');

  return (
    <div className="flex h-screen bg-zinc-950 text-zinc-100">
      {/* Sidebar */}
      <div className="w-64 bg-zinc-900 border-r border-zinc-800 flex flex-col">
        {/* Logo */}
        <div className="p-4 border-b border-zinc-800">
          <h1 className="text-xl font-bold text-blue-500">CEOClaw</h1>
          <p className="text-xs text-zinc-500">OpenClaw AI Assistant</p>
        </div>

        {/* Navigation */}
        <nav className="flex-1 p-2 space-y-1">
          <button
            onClick={() => setActiveTab('chat')}
            className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
              activeTab === 'chat'
                ? 'bg-blue-600 text-white'
                : 'text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100'
            }`}
          >
            <MessageSquare className="w-4 h-4" />
            <span className="text-sm">Chat</span>
          </button>
        </nav>

        {/* Bottom Actions */}
        <div className="p-2 border-t border-zinc-800">
          <button
            onClick={() => setActiveTab('settings')}
            className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
              activeTab === 'settings'
                ? 'bg-blue-600 text-white'
                : 'text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100'
            }`}
          >
            <Cog className="w-4 h-4" />
            <span className="text-sm">Settings</span>
          </button>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-hidden">
        {activeTab === 'chat' && (
          <div className="h-full flex items-center justify-center">
            <div className="text-center">
              <h2 className="text-xl font-bold mb-2">Chat</h2>
              <p className="text-zinc-400">Select a session or create a new chat</p>
            </div>
          </div>
        )}
        {activeTab === 'settings' && (
          <div className="h-full flex items-center justify-center">
            <div className="text-center">
              <h2 className="text-xl font-bold mb-2">Settings</h2>
              <p className="text-zinc-400">Configure your preferences</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
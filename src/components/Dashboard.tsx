import { useState } from 'react';
import { MessageSquare, Users, Clock, Zap, Database, Cog, Plus } from 'lucide-react';
import Chat from '@/pages/Chat';
import Agents from '@/pages/Agents';
import Channels from '@/pages/Channels';
import Cron from '@/pages/Cron';
import Skills from '@/pages/Skills';
import Models from '@/pages/Models';
import Settings from '@/pages/Settings';

type TabType = 'chat' | 'agents' | 'channels' | 'cron' | 'skills' | 'models' | 'settings';

export default function Dashboard() {
  const [activeTab, setActiveTab] = useState<TabType>('chat');

  const tabs = [
    { id: 'chat' as TabType, label: 'Chat', icon: MessageSquare },
    { id: 'agents' as TabType, label: 'Agents', icon: Users },
    { id: 'channels' as TabType, label: 'Channels', icon: Zap },
    { id: 'cron' as TabType, label: 'Cron', icon: Clock },
    { id: 'skills' as TabType, label: 'Skills', icon: Database },
    { id: 'models' as TabType, label: 'Models', icon: Cog },
  ];

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
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                activeTab === tab.id
                  ? 'bg-blue-600 text-white'
                  : 'text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100'
              }`}
            >
              <tab.icon className="w-4 h-4" />
              <span className="text-sm">{tab.label}</span>
            </button>
          ))}
        </nav>

        {/* Bottom Actions */}
        <div className="p-2 border-t border-zinc-800 space-y-1">
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
        {activeTab === 'chat' && <Chat />}
        {activeTab === 'agents' && <Agents />}
        {activeTab === 'channels' && <Channels />}
        {activeTab === 'cron' && <Cron />}
        {activeTab === 'skills' && <Skills />}
        {activeTab === 'models' && <Models />}
        {activeTab === 'settings' && <Settings />}
      </div>
    </div>
  );
}
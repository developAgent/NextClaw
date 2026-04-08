import { useState } from 'react';
import { Settings as SettingsIcon, Globe, Zap, Shield, Palette } from 'lucide-react';

type TabType = 'general' | 'providers' | 'skills' | 'appearance';

export default function SettingsPage() {
  const [activeTab, setActiveTab] = useState<TabType>('general');

  const tabs = [
    { id: 'general' as TabType, label: 'General', icon: SettingsIcon },
    { id: 'providers' as TabType, label: 'Providers', icon: Globe },
    { id: 'skills' as TabType, label: 'Skills', icon: Zap },
    { id: 'appearance' as TabType, label: 'Appearance', icon: Palette },
  ];

  return (
    <div className="flex h-full">
      {/* Sidebar */}
      <div className="w-64 bg-zinc-900 border-r border-zinc-800">
        <div className="p-4 border-b border-zinc-800">
          <h2 className="text-xl font-bold">Settings</h2>
        </div>
        <nav className="p-2">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg mb-1 transition-colors ${
                activeTab === tab.id
                  ? 'bg-blue-600 text-white'
                  : 'text-zinc-400 hover:bg-zinc-800'
              }`}
            >
              <tab.icon className="w-4 h-4" />
              <span className="text-sm">{tab.label}</span>
            </button>
          ))}
        </nav>
      </div>

      {/* Content */}
      <div className="flex-1 p-6">
        {activeTab === 'general' && (
          <div>
            <h2 className="text-2xl font-bold mb-6">General Settings</h2>
            <div className="space-y-6">
              <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                <h3 className="font-medium mb-4">Application</h3>
                <div className="space-y-4">
                  <label className="flex items-center gap-3 cursor-pointer">
                    <input type="checkbox" className="w-4 h-4" />
                    <span className="text-sm">Start on system startup</span>
                  </label>
                  <label className="flex items-center gap-3 cursor-pointer">
                    <input type="checkbox" className="w-4 h-4" />
                    <span className="text-sm">Minimize to tray on close</span>
                  </label>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'providers' && (
          <div>
            <h2 className="text-2xl font-bold mb-6">AI Providers</h2>
            <div className="space-y-4">
              <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                <h3 className="font-medium mb-4">OpenAI</h3>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm text-zinc-400 mb-1">API Key</label>
                    <input
                      type="password"
                      placeholder="sk-..."
                      className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>
                  <div>
                    <label className="block text-sm text-zinc-400 mb-1">Base URL (optional)</label>
                    <input
                      type="text"
                      placeholder="https://api.openai.com/v1"
                      className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>
                  <button className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors">
                    Save Configuration
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'skills' && (
          <div>
            <h2 className="text-2xl font-bold mb-6">Skills Settings</h2>
            <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
              <p className="text-zinc-400">Configure skill directories and preferences</p>
            </div>
          </div>
        )}

        {activeTab === 'appearance' && (
          <div>
            <h2 className="text-2xl font-bold mb-6">Appearance</h2>
            <div className="space-y-4">
              <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                <h3 className="font-medium mb-4">Theme</h3>
                <div className="grid grid-cols-3 gap-4">
                  <button className="p-4 bg-zinc-800 border-2 border-transparent hover:border-blue-500 rounded-lg transition-colors">
                    Light
                  </button>
                  <button className="p-4 bg-zinc-950 border-2 border-blue-500 rounded-lg">
                    Dark
                  </button>
                  <button className="p-4 bg-zinc-800 border-2 border-transparent hover:border-blue-500 rounded-lg transition-colors">
                    System
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
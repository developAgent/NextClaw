import { useEffect, useMemo, useState } from 'react';
import {
  Bot,
  CalendarClock,
  Cog,
  Cpu,
  FolderKanban,
  HardDriveDownload,
  Link2,
  Logs,
  MessageSquare,
  Sparkles,
} from 'lucide-react';
import Chat from '@/pages/Chat';
import Agents from '@/pages/Agents';
import Channels from '@/pages/Channels';
import Cron from '@/pages/Cron';
import Skills from '@/pages/Skills';
import Models from '@/pages/Models';
import Settings from '@/pages/Settings';
import Installer from '@/pages/Installer';
import LogsPage from '@/pages/Logs';
import Workspaces from '@/pages/Workspaces';
import SetupWizard from '@/pages/Setup/SetupWizard';

type AppSection =
  | 'chat'
  | 'agents'
  | 'channels'
  | 'cron'
  | 'skills'
  | 'models'
  | 'workspaces'
  | 'logs'
  | 'installer'
  | 'settings';

const SETUP_STORAGE_KEY = 'nextclaw.setup.completed';

interface NavItem {
  id: AppSection;
  label: string;
  description: string;
  icon: typeof MessageSquare;
}

const navItems: NavItem[] = [
  {
    id: 'chat',
    label: 'Chat',
    description: 'Sessions and conversations',
    icon: MessageSquare,
  },
  {
    id: 'agents',
    label: 'Agents',
    description: 'Create and configure agents',
    icon: Bot,
  },
  {
    id: 'channels',
    label: 'Channels',
    description: 'Provider and account bindings',
    icon: Link2,
  },
  {
    id: 'cron',
    label: 'Cron',
    description: 'Scheduled automations',
    icon: CalendarClock,
  },
  {
    id: 'skills',
    label: 'Skills',
    description: 'Installed skills and marketplace',
    icon: Sparkles,
  },
  {
    id: 'models',
    label: 'Models',
    description: 'Providers and model availability',
    icon: Cpu,
  },
  {
    id: 'workspaces',
    label: 'Workspaces',
    description: 'Switch isolated product contexts',
    icon: FolderKanban,
  },
  {
    id: 'logs',
    label: 'Logs',
    description: 'Inspect runtime and diagnostic events',
    icon: Logs,
  },
  {
    id: 'installer',
    label: 'Installer',
    description: 'Install and control the local runtime',
    icon: HardDriveDownload,
  },
  {
    id: 'settings',
    label: 'Settings',
    description: 'Application configuration',
    icon: Cog,
  },
];

export default function App() {
  const [activeSection, setActiveSection] = useState<AppSection>('installer');
  const [setupComplete, setSetupComplete] = useState<boolean>(() => {
    if (typeof window === 'undefined') {
      return true;
    }

    return window.localStorage.getItem(SETUP_STORAGE_KEY) === 'true';
  });

  useEffect(() => {
    if (typeof window === 'undefined') {
      return;
    }

    if (setupComplete) {
      window.localStorage.setItem(SETUP_STORAGE_KEY, 'true');
    } else {
      window.localStorage.removeItem(SETUP_STORAGE_KEY);
    }
  }, [setupComplete]);

  const activeItem = useMemo(
    () => navItems.find((item) => item.id === activeSection) ?? navItems[0],
    [activeSection]
  );

  const handleSetupComplete = () => {
    setSetupComplete(true);
    setActiveSection('installer');
  };

  const renderContent = () => {
    switch (activeSection) {
      case 'chat':
        return <Chat />;
      case 'agents':
        return <Agents />;
      case 'channels':
        return <Channels />;
      case 'cron':
        return <Cron />;
      case 'skills':
        return <Skills />;
      case 'models':
        return <Models />;
      case 'workspaces':
        return <Workspaces />;
      case 'logs':
        return <LogsPage />;
      case 'installer':
        return <Installer />;
      case 'settings':
        return <Settings />;
      default:
        return <Chat />;
    }
  };

  if (!setupComplete) {
    return <SetupWizard onComplete={handleSetupComplete} onSkip={handleSetupComplete} />;
  }

  return (
    <div className="flex h-screen bg-zinc-950 text-zinc-100">
      <aside className="flex w-72 shrink-0 flex-col border-r border-zinc-800 bg-zinc-900/95">
        <div className="border-b border-zinc-800 px-5 py-5">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-blue-600/15 text-blue-400">
              <Sparkles className="h-5 w-5" />
            </div>
            <div>
              <h1 className="text-lg font-semibold tracking-tight">NextClaw</h1>
              <p className="text-xs text-zinc-500">ClawX-compatible desktop runtime</p>
            </div>
          </div>
        </div>

        <nav className="flex-1 space-y-1 overflow-y-auto p-3">
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = item.id === activeSection;

            return (
              <button
                key={item.id}
                onClick={() => setActiveSection(item.id)}
                className={`w-full rounded-xl border px-4 py-3 text-left transition-colors ${
                  isActive
                    ? 'border-blue-500/40 bg-blue-600/15 text-white'
                    : 'border-transparent text-zinc-400 hover:border-zinc-800 hover:bg-zinc-800/80 hover:text-zinc-100'
                }`}
              >
                <div className="flex items-start gap-3">
                  <Icon className={`mt-0.5 h-4 w-4 shrink-0 ${isActive ? 'text-blue-400' : ''}`} />
                  <div className="min-w-0">
                    <div className="text-sm font-medium">{item.label}</div>
                    <div className="mt-1 text-xs text-zinc-500">{item.description}</div>
                  </div>
                </div>
              </button>
            );
          })}
        </nav>

        <div className="border-t border-zinc-800 px-4 py-4">
          <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 px-4 py-3">
            <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">Current surface</div>
            <div className="mt-2 text-sm font-medium text-zinc-200">{activeItem.label}</div>
            <div className="mt-1 text-xs text-zinc-500">{activeItem.description}</div>
          </div>
        </div>
      </aside>

      <main className="flex min-w-0 flex-1 flex-col overflow-hidden">
        <header className="border-b border-zinc-800 bg-zinc-950/80 px-6 py-4 backdrop-blur">
          <div className="flex items-center justify-between gap-4">
            <div>
              <h2 className="text-xl font-semibold tracking-tight">{activeItem.label}</h2>
              <p className="mt-1 text-sm text-zinc-500">{activeItem.description}</p>
            </div>
          </div>
        </header>

        <section className="min-h-0 flex-1 overflow-auto">{renderContent()}</section>
      </main>
    </div>
  );
}

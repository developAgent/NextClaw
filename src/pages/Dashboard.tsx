import { useEffect, useState } from 'react';
import { useI18nStore } from '@/store/i18n';
import { getWizardState, FEATURES } from '@/lib/wizard';

const FEATURE_ICONS: Record<string, string> = {
  chat: '💬',
  automation: '🎬',
  cron: '📅',
  skills: '✨',
  hotkeys: '⌨️',
  workflow: '🔀',
};

type AppSection =
  | 'chat'
  | 'agents'
  | 'channels'
  | 'cron'
  | 'recorder'
  | 'skills'
  | 'hotkeys'
  | 'workflows'
  | 'models'
  | 'workspaces'
  | 'logs'
  | 'installer'
  | 'settings'
  | 'dashboard';

interface DashboardProps {
  onNavigate: (section: AppSection) => void;
}

interface RecentItem {
  id: string;
  type: string;
  title: string;
  time: string;
}

export default function Dashboard({ onNavigate }: DashboardProps) {
  const { t } = useI18nStore();
  const [enabledFeatures, setEnabledFeatures] = useState<string[]>([]);
  const [recentItems, setRecentItems] = useState<RecentItem[]>([]);

  useEffect(() => {
    // 从向导状态获取启用的功能
    getWizardState()
      .then((state) => {
        setEnabledFeatures(state.enabledFeatures.length > 0 ? state.enabledFeatures : ['chat']);
      })
      .catch(() => {
        setEnabledFeatures(['chat']);
      });
  }, []);

  // 模拟最近使用数据（实际应从后端获取）
  useEffect(() => {
    setRecentItems([
      { id: '1', type: 'chat', title: 'Rust 异步编程讨论', time: '2分钟前' },
      { id: '2', type: 'automation', title: '自动化录制 - 登录', time: '昨天' },
      { id: '3', type: 'cron', title: '定时任务: 每日早报', time: '3天前' },
    ]);
  }, []);

  const handleFeatureClick = (featureId: string) => {
    onNavigate(featureId as AppSection);
  };

  const displayFeatures = enabledFeatures.length > 0 ? enabledFeatures : ['chat'];

  return (
    <div className="space-y-8 p-6">
      {/* 功能入口 */}
      <div>
        <h2 className="mb-4 text-lg font-medium text-zinc-400">功能</h2>
        <div className="grid grid-cols-3 gap-4">
          {displayFeatures.map((featureId) => {
            const feature = FEATURES.find((f) => f.id === featureId);
            if (!feature) return null;

            return (
              <button
                key={featureId}
                onClick={() => handleFeatureClick(featureId)}
                className="flex flex-col items-center gap-3 rounded-xl border border-zinc-800 bg-zinc-900/50 p-6 transition-all hover:border-zinc-700 hover:bg-zinc-900"
              >
                <span className="text-4xl">{FEATURE_ICONS[featureId] || '⚡'}</span>
                <div className="text-center">
                  <p className="font-medium">{feature.name}</p>
                  <p className="mt-1 text-sm text-zinc-500">{feature.description}</p>
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* 最近使用 */}
      <div>
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-lg font-medium text-zinc-400">最近使用</h2>
          <button
            onClick={() => onNavigate('logs')}
            className="text-sm text-blue-500 hover:text-blue-400"
          >
            查看全部
          </button>
        </div>
        <div className="space-y-2">
          {recentItems.map((item) => (
            <div
              key={item.id}
              className="flex items-center gap-3 rounded-lg border border-zinc-800 bg-zinc-900/50 p-4 transition-all hover:border-zinc-700"
            >
              <span className="text-xl">{FEATURE_ICONS[item.type] || '⚡'}</span>
              <div className="flex-1">
                <p className="font-medium">{item.title}</p>
              </div>
              <span className="text-sm text-zinc-500">{item.time}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

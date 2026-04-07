import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '@/store';
import ChatInterface from '@/components/ChatInterface';
import Sidebar from '@/components/Sidebar';
import SettingsModal from '@/components/SettingsModal';

function App() {
  const { sessions, setSessions, currentSessionId, setCurrentSession } = useAppStore();

  useEffect(() => {
    // Load sessions on mount
    const loadSessions = async () => {
      try {
        const sessionList = await invoke<any[]>('list_sessions');
        setSessions(sessionList);
      } catch (error) {
        console.error('Failed to load sessions:', error);
      }
    };

    loadSessions();
  }, [setSessions]);

  const handleNewChat = async () => {
    try {
      const newSession = await invoke<any>('create_session', {
        title: `Chat ${sessions.length + 1}`,
      });
      setSessions([...sessions, newSession]);
      setCurrentSession(newSession.id);
    } catch (error) {
      console.error('Failed to create session:', error);
    }
  };

  const handleSessionSelect = (sessionId: string) => {
    setCurrentSession(sessionId);
  };

  const handleSessionDelete = async (sessionId: string) => {
    try {
      await invoke('delete_session', { sessionId });
      setSessions(sessions.filter((s) => s.id !== sessionId));
      if (currentSessionId === sessionId) {
        setCurrentSession(null);
      }
    } catch (error) {
      console.error('Failed to delete session:', error);
    }
  };

  return (
    <div className="flex h-screen bg-zinc-950 text-zinc-100">
      <Sidebar
        sessions={sessions}
        onNewChat={handleNewChat}
        onSessionSelect={handleSessionSelect}
        onSessionDelete={handleSessionDelete}
        currentSessionId={currentSessionId}
      />
      <main className="flex-1 flex flex-col overflow-hidden">
        <ChatInterface sessionId={currentSessionId} />
      </main>
      <SettingsModal />
    </div>
  );
}

export default App;
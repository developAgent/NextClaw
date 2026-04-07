import { create } from 'zustand';
import { Message, Session } from '@/types';
import { v4 as uuidv4 } from 'uuid';

interface AppState {
  // Messages
  messages: Message[];
  currentSessionId: string | null;
  sessions: Session[];
  isLoading: boolean;

  // Actions
  addMessage: (message: Omit<Message, 'id' | 'timestamp'>) => void;
  setMessages: (messages: Message[]) => void;
  setCurrentSession: (sessionId: string | null) => void;
  setSessions: (sessions: Session[]) => void;
  addSession: (session: Session) => void;
  removeSession: (sessionId: string) => void;
  setLoading: (loading: boolean) => void;
  clearCurrentSession: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  messages: [],
  currentSessionId: null,
  sessions: [],
  isLoading: false,

  addMessage: (message) =>
    set((state) => ({
      messages: [
        ...state.messages,
        {
          ...message,
          id: uuidv4(),
          timestamp: new Date().toISOString(),
        },
      ],
    })),

  setMessages: (messages) => set({ messages }),

  setCurrentSession: (sessionId) => set({ currentSessionId: sessionId }),

  setSessions: (sessions) => set({ sessions }),

  addSession: (session) =>
    set((state) => ({ sessions: [...state.sessions, session] })),

  removeSession: (sessionId) =>
    set((state) => ({
      sessions: state.sessions.filter((s) => s.id !== sessionId),
      currentSessionId:
        state.currentSessionId === sessionId ? null : state.currentSessionId,
    })),

  setLoading: (isLoading) => set({ isLoading }),

  clearCurrentSession: () => set({ messages: [], currentSessionId: null }),
}));
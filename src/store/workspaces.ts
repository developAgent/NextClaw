import { create } from 'zustand';
import type { CreateWorkspaceInput, Workspace } from '@/types';
import {
  createWorkspace,
  getCurrentWorkspace,
  listWorkspaces,
  setCurrentWorkspace,
} from '@/lib/workspaces';

interface WorkspaceStore {
  workspaces: Workspace[];
  currentWorkspace: Workspace | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  create: (input: CreateWorkspaceInput) => Promise<void>;
  select: (workspaceId: string) => Promise<void>;
}

function getErrorMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }

  if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') {
    return error.message;
  }

  return 'Unknown error';
}

export const useWorkspaceStore = create<WorkspaceStore>((set) => ({
  workspaces: [],
  currentWorkspace: null,
  loading: false,
  error: null,
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const [workspaces, currentWorkspace] = await Promise.all([
        listWorkspaces(),
        getCurrentWorkspace(),
      ]);
      set({ workspaces, currentWorkspace, loading: false });
    } catch (error) {
      set({ error: getErrorMessage(error), loading: false });
    }
  },
  create: async (input) => {
    set({ loading: true, error: null });
    try {
      await createWorkspace(input);
      const [workspaces, currentWorkspace] = await Promise.all([
        listWorkspaces(),
        getCurrentWorkspace(),
      ]);
      set({ workspaces, currentWorkspace, loading: false });
    } catch (error) {
      set({ error: getErrorMessage(error), loading: false });
    }
  },
  select: async (workspaceId) => {
    set({ loading: true, error: null });
    try {
      const currentWorkspace = await setCurrentWorkspace(workspaceId);
      const workspaces = await listWorkspaces();
      set({ workspaces, currentWorkspace, loading: false });
    } catch (error) {
      set({ error: getErrorMessage(error), loading: false });
    }
  },
}));

import { invoke } from '@tauri-apps/api/core';
import type { CreateWorkspaceInput, Workspace } from '@/types';

export async function listWorkspaces(): Promise<Workspace[]> {
  return invoke<Workspace[]>('list_workspaces');
}

export async function getCurrentWorkspace(): Promise<Workspace | null> {
  return invoke<Workspace | null>('get_current_workspace');
}

export async function createWorkspace(input: CreateWorkspaceInput): Promise<Workspace> {
  return invoke<Workspace>('create_workspace', { input });
}

export async function setCurrentWorkspace(workspaceId: string): Promise<Workspace> {
  return invoke<Workspace>('set_current_workspace', { workspaceId });
}

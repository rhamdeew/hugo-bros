import { writable } from 'svelte/store';
import type { HugoConfig } from '$lib/types';

export interface ProjectState {
  projectPath: string | null;
  config: HugoConfig | null;
  isLoading: boolean;
}

function createProjectStore() {
  const { subscribe, set, update } = writable<ProjectState>({
    projectPath: null,
    config: null,
    isLoading: false,
  });

  return {
    subscribe,
    setProject: (path: string, config: HugoConfig) => {
      set({ projectPath: path, config, isLoading: false });
    },
    clearProject: () => {
      set({ projectPath: null, config: null, isLoading: false });
    },
    setLoading: (isLoading: boolean) => {
      update((state) => ({ ...state, isLoading }));
    },
  };
}

export const project = createProjectStore();

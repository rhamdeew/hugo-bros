<script lang="ts">
  import { onMount } from 'svelte';
  import { CheckCircle, FolderOpen, Clock } from 'lucide-svelte';
  import { backend } from '$lib/services/backend';
  import type { AppConfig } from '$lib/types';
  import { goto } from '$app/navigation';

  let appConfig = $state<AppConfig | null>(null);
  let loading = $state(true);

  onMount(async () => {
    try {
      appConfig = await backend.getAppConfig();
    } catch (err) {
      console.error('Failed to load app config:', err);
    } finally {
      loading = false;
    }
  });

  async function selectProject() {
    try {
      await backend.selectProjectFolder();
      goto('/posts');
    } catch (err) {
      console.error('Failed to select project:', err);
    }
  }

  async function openRecentProject(projectPath: string) {
    try {
      backend.setProjectPath(projectPath);
      goto('/posts');
    } catch (err) {
      console.error('Failed to open project:', err);
    }
  }
</script>

<div class="min-h-screen bg-background dark:bg-dark-background p-8">
  <div class="max-w-4xl mx-auto">
    <header class="text-center mb-12">
      <h1 class="text-4xl font-bold text-text-primary dark:text-dark-text-primary mb-4">
        Hex Tool
      </h1>
      <p class="text-text-secondary dark:text-dark-text-secondary">
        Hexo Blog Editor - Built with Tauri + Svelte
      </p>
    </header>

    <main class="space-y-8">
      <!-- Quick Actions Card -->
      <div class="bg-surface dark:bg-dark-surface rounded-lg p-6 shadow-sm">
        <h2 class="text-2xl font-semibold text-text-primary dark:text-dark-text-primary mb-4">
          Get Started
        </h2>
        <button
          onclick={selectProject}
          class="inline-flex items-center gap-2 px-6 py-3 bg-accent hover:bg-blue-600 text-white rounded-lg font-medium transition-colors"
        >
          <FolderOpen size={20} />
          <span>Select Hexo Project</span>
        </button>
      </div>

      <!-- Recent Projects Card -->
      {#if !loading && appConfig && appConfig.recentProjects.length > 0}
        <div class="bg-surface dark:bg-dark-surface rounded-lg p-6 shadow-sm">
          <div class="flex items-center gap-2 mb-4">
            <Clock size={20} class="text-text-secondary dark:text-dark-text-secondary" />
            <h2 class="text-2xl font-semibold text-text-primary dark:text-dark-text-primary">
              Recent Projects
            </h2>
          </div>
          <div class="space-y-2">
            {#each appConfig.recentProjects as project (project)}
              <button
                onclick={() => openRecentProject(project)}
                class="w-full text-left px-4 py-3 bg-gray-50 dark:bg-gray-800 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
              >
                <div class="flex items-center gap-2">
                  <FolderOpen size={16} class="text-text-secondary dark:text-dark-text-secondary flex-shrink-0" />
                  <span class="text-sm text-text-primary dark:text-dark-text-primary truncate">
                    {project}
                  </span>
                </div>
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Posts Page Card -->
      <div class="bg-surface dark:bg-dark-surface rounded-lg p-6 shadow-sm">
        <h2 class="text-2xl font-semibold text-text-primary dark:text-dark-text-primary mb-4">
          Posts
        </h2>

        <a
          href="/posts"
          class="inline-flex items-center gap-2 px-6 py-3 bg-accent hover:bg-blue-600 text-white rounded-lg font-medium transition-colors"
        >
          Manage Posts
        </a>
      </div>

    </main>
  </div>
</div>

<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, FolderOpen, X, ArrowLeft } from 'lucide-svelte';
  import { goto } from '$app/navigation';
  import { confirm, message } from '@tauri-apps/plugin-dialog';
  import { backend } from '$lib/services/backend';
  import { PostList, ImageGallery, HugoControls } from '$lib/components';
  import type { Post, Page, Draft, StaticEntry, FrontmatterConfig } from '$lib/types';

  let posts: Post[] = $state([]);
  let pages: Page[] = $state([]);
  let drafts: Draft[] = $state([]);
  let frontmatterConfig = $state<FrontmatterConfig | null>(null);
  let activeTab = $state<'posts' | 'pages' | 'drafts'>('posts');
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showImageGallery = $state(false);
  let showCreateDialog = $state(false);
  let newPostTitle = $state('');
  let createError = $state<string | null>(null);
  let pendingImageField:
    | { fieldName: string; post: Post }
    | null = null;
  let isGeneratingFrontmatterConfig = $state(false);
  let createKind = $derived(
    activeTab === 'pages' ? 'Page' :
    activeTab === 'drafts' ? 'Draft' :
    'Post'
  );
  let previewImageWarning = $derived(
    frontmatterConfig?.previewImageField
      ? frontmatterConfig.customFields?.some(
          (field) => field.name === frontmatterConfig?.previewImageField
        )
        ? ''
        : `Preview image field "${frontmatterConfig.previewImageField}" not found in customFields.`
      : ''
  );

  // Get current items based on active tab
  let currentItems = $derived(
    activeTab === 'posts' ? posts :
    activeTab === 'pages' ? pages :
    drafts
  ) as Post[];

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    loading = true;
    error = null;

    try {
      // Check if project is selected
      const projectPath = backend.getProjectPath();
      if (!projectPath) {
        error = 'No project selected. Please select a project folder first.';
        return;
      }

      const [postsData, pagesData, draftsData, frontmatterConfigData] = await Promise.all([
        backend.listPosts(),
        backend.listPages(),
        backend.listDrafts(),
        backend.getFrontmatterConfig(),
      ]);

      posts = postsData;
      pages = pagesData;
      drafts = draftsData;
      frontmatterConfig = frontmatterConfigData;
    } catch (err) {
      console.error('Failed to load data:', err);
      error = err instanceof Error ? err.message : 'Failed to load data';
    } finally {
      loading = false;
    }
  }

  function handleTabChange(tab: 'posts' | 'pages' | 'drafts') {
    activeTab = tab;
  }

  function handleCreatePost() {
    createError = null;
    newPostTitle = '';
    showCreateDialog = true;
  }

  async function confirmCreatePost() {
    try {
      const title = newPostTitle.trim();
      if (!title) {
        createError = 'Title is required.';
        return;
      }

      if (activeTab === 'posts') {
        const newPost = await backend.createPost(title);
        window.location.href = `/editor?id=${encodeURIComponent(newPost.id)}`;
        return;
      }

      if (activeTab === 'pages') {
        await backend.createPage(title);
      } else {
        await backend.createDraft(title);
      }

      await loadData();
      showCreateDialog = false;
    } catch (err) {
      console.error('Failed to create item:', err);
      createError = err instanceof Error ? err.message : 'Unknown error';
      await message(`Failed to create ${createKind.toLowerCase()}: ` + createError, {
        title: 'Hugo Bros',
        kind: 'error'
      });
    }
  }

  function closeCreateDialog() {
    showCreateDialog = false;
  }

  function handleEdit(item: Post | Page | Draft, type: 'post' | 'page' | 'draft') {
    window.location.href = `/editor?id=${encodeURIComponent(item.id)}&type=${type}`;
  }

  async function handleDeletePost(post: Post) {
    try {
      const shouldDelete = await confirm(`Delete "${post.title}"?`, {
        title: 'Hugo Bros',
        kind: 'warning'
      });
      if (!shouldDelete) return;

      await backend.deletePost(post.id);
      // Reload posts
      posts = await backend.listPosts();
    } catch (err) {
      console.error('Failed to delete post:', err);
      await message('Failed to delete post: ' + (err instanceof Error ? err.message : 'Unknown error'), {
        title: 'Hugo Bros',
        kind: 'error'
      });
    }
  }

  async function handleDeletePage(page: Page) {
    try {
      const shouldDelete = await confirm(`Delete "${page.title}"?`, {
        title: 'Hugo Bros',
        kind: 'warning'
      });
      if (!shouldDelete) return;

      await backend.deletePage(page.id);
      pages = await backend.listPages();
    } catch (err) {
      console.error('Failed to delete page:', err);
      await message('Failed to delete page: ' + (err instanceof Error ? err.message : 'Unknown error'), {
        title: 'Hugo Bros',
        kind: 'error'
      });
    }
  }

  async function handleDeleteDraft(draft: Draft) {
    try {
      const shouldDelete = await confirm(`Delete "${draft.title}"?`, {
        title: 'Hugo Bros',
        kind: 'warning'
      });
      if (!shouldDelete) return;

      await backend.deleteDraft(draft.id);
      drafts = await backend.listDrafts();
    } catch (err) {
      console.error('Failed to delete draft:', err);
      await message('Failed to delete draft: ' + (err instanceof Error ? err.message : 'Unknown error'), {
        title: 'Hugo Bros',
        kind: 'error'
      });
    }
  }

  async function handleSelectProject() {
    try {
      await backend.selectProjectFolder();
      await loadData();
    } catch (err) {
      console.error('Failed to select project:', err);
      alert('Failed to select project: ' + (err instanceof Error ? err.message : 'Unknown error'));
    }
  }

  function goToStart() {
    goto('/');
  }

  function handleImageSelect(entry: StaticEntry) {
    // This will be called when an image is selected from gallery
    if (pendingImageField) {
      if (!entry.url) return;
      const { fieldName, post } = pendingImageField;
      // Update the post's frontmatter
      post.frontmatter.customFields = post.frontmatter.customFields || {};
      post.frontmatter.customFields[fieldName] = entry.url;
      pendingImageField = null;
    }
  }

  function openImageGalleryForPost(fieldName: string, post: Post) {
    pendingImageField = { fieldName, post };
    showImageGallery = true;
  }

  async function handleGenerateFrontmatterConfig() {
    if (isGeneratingFrontmatterConfig) return;
    isGeneratingFrontmatterConfig = true;

    try {
      frontmatterConfig = await backend.generateFrontmatterConfig();
      await message('Frontmatter config generated successfully.', {
        title: 'Hugo Bros'
      });
    } catch (err) {
      console.error('Failed to generate frontmatter config:', err);
      await message(
        'Failed to generate frontmatter config: ' +
          (err instanceof Error ? err.message : 'Unknown error'),
        { title: 'Hugo Bros', kind: 'error' }
      );
    } finally {
      isGeneratingFrontmatterConfig = false;
    }
  }
</script>

<div class="posts-page">
  <!-- Header -->
  <header class="page-header">
    <div class="header-left">
      <h1 class="page-title">Posts</h1>
      <p class="page-subtitle">Manage your Hugo site content</p>
    </div>

    <div class="header-right">
      {#if !backend.getProjectPath()}
        <button class="select-project-btn" onclick={handleSelectProject} type="button">
          <FolderOpen size={18} />
          <span>Select Project</span>
        </button>
      {:else}
        <button class="back-btn" onclick={goToStart} type="button">
          <ArrowLeft size={18} />
          <span>Back to Start Screen</span>
        </button>
        <button class="create-btn" onclick={handleCreatePost} type="button">
          <Plus size={18} />
          <span>New Post</span>
        </button>
      {/if}
    </div>
  </header>

  <!-- Hugo Controls (shown when project is selected) -->
  {#if backend.getProjectPath()}
    <div class="hugo-controls-wrapper">
      <HugoControls />
    </div>
  {/if}

  <!-- Content -->
  <main class="page-content">
    {#if loading}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>Loading posts...</p>
      </div>
    {:else if error}
      <div class="error-state">
        <div class="error-icon">⚠️</div>
        <h3>Error</h3>
        <p>{error}</p>
        <button class="retry-btn" onclick={handleSelectProject} type="button">
          <FolderOpen size={18} />
          Select Project Folder
        </button>
      </div>
    {:else if posts.length === 0}
      {#if frontmatterConfig?.isDefault}
        <div class="info-banner">
          <div class="info-banner-content">
            <span>
              No frontmatter config found. Generate one from existing posts to enable custom fields.
            </span>
            <button
              class="info-banner-btn"
              onclick={handleGenerateFrontmatterConfig}
              disabled={isGeneratingFrontmatterConfig}
              type="button"
            >
              {isGeneratingFrontmatterConfig ? 'Generating...' : 'Generate Config'}
            </button>
          </div>
        </div>
      {/if}
      <div class="empty-state">
        <div class="empty-icon">📝</div>
        <h3>No posts yet</h3>
        <p>Create your first post to get started</p>
        <button class="create-btn-empty" onclick={handleCreatePost} type="button">
          <Plus size={18} />
          Create Post
        </button>
      </div>
    {:else}
      {#if frontmatterConfig?.isDefault}
        <div class="info-banner">
          <div class="info-banner-content">
            <span>
              No frontmatter config found. Generate one from existing posts to enable custom fields.
            </span>
            <button
              class="info-banner-btn"
              onclick={handleGenerateFrontmatterConfig}
              disabled={isGeneratingFrontmatterConfig}
              type="button"
            >
              {isGeneratingFrontmatterConfig ? 'Generating...' : 'Generate Config'}
            </button>
          </div>
        </div>
      {/if}
      {#if previewImageWarning}
        <div class="warning-banner">
          {previewImageWarning}
        </div>
      {/if}
      <PostList
        posts={currentItems}
        activeTab={activeTab}
        postsCount={posts.length}
        pagesCount={pages.length}
        draftsCount={drafts.length}
        frontmatterConfig={frontmatterConfig ?? undefined}
        onCreate={handleCreatePost}
        onEdit={(item) => handleEdit(
          item,
          activeTab === 'pages' ? 'page' : activeTab === 'drafts' ? 'draft' : 'post'
        )}
        onDelete={(item) => {
          if (activeTab === 'pages') {
            handleDeletePage(item as Page);
          } else if (activeTab === 'drafts') {
            handleDeleteDraft(item as Draft);
          } else {
            handleDeletePost(item as Post);
          }
        }}
        onTabChange={handleTabChange}
      />
    {/if}
  </main>

  <!-- Image Gallery Modal -->
  <ImageGallery
    bind:open={showImageGallery}
    onSelect={handleImageSelect}
  />

  <!-- New Post Modal -->
  {#if showCreateDialog}
    <div
      class="modal-overlay"
      role="presentation"
      onclick={closeCreateDialog}
      onkeydown={(e) => {
        if (e.key === 'Escape') {
          closeCreateDialog();
        }
      }}
    >
      <div
        class="modal-content"
        role="dialog"
        aria-modal="true"
        aria-labelledby="create-dialog-title"
        tabindex="-1"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
      >
        <div class="modal-header">
          <h3 id="create-dialog-title">Create {createKind}</h3>
          <button class="close-btn" onclick={closeCreateDialog} type="button" aria-label="Close">
            <X size={20} />
          </button>
        </div>
        <div class="modal-body">
          <label class="modal-label" for="new-post-title">Title</label>
          <input
            id="new-post-title"
            class="modal-input"
            type="text"
            placeholder={"Enter " + createKind.toLowerCase() + " title"}
            bind:value={newPostTitle}
            onkeydown={(e) => {
              if (e.key === 'Enter') {
                confirmCreatePost();
              }
            }}
          />
          {#if createError}
            <p class="modal-error">{createError}</p>
          {/if}
        </div>
        <div class="modal-footer">
          <button class="modal-btn" onclick={closeCreateDialog} type="button">Cancel</button>
          <button
            class="modal-btn primary"
            onclick={confirmCreatePost}
            type="button"
            disabled={!newPostTitle.trim()}
          >
            Create {createKind}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .posts-page {
    min-height: 100vh;
    background-color: #ffffff;
    display: flex;
    flex-direction: column;
  }

  :global(.dark .posts-page) {
    background-color: #1a1a1a;
  }

  /* Header */
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.5rem 2rem;
    border-bottom: 1px solid #e5e5e5;
    background-color: #ffffff;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1100;
    padding: 1rem;
  }

  .modal-content {
    width: 100%;
    max-width: 420px;
    background-color: #ffffff;
    border-radius: 0.75rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  :global(.dark .modal-content) {
    background-color: #2d2d2d;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid #e5e7eb;
  }

  :global(.dark .modal-header) {
    border-bottom-color: #404040;
  }

  .close-btn {
    background: none;
    border: none;
    color: #6b7280;
    cursor: pointer;
  }

  :global(.dark .close-btn) {
    color: #d1d5db;
  }

  .modal-body {
    padding: 1rem 1.25rem 0.5rem;
  }

  .modal-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: #374151;
  }

  :global(.dark .modal-label) {
    color: #e5e7eb;
  }

  .modal-input {
    width: 100%;
    margin-top: 0.5rem;
    padding: 0.625rem 0.75rem;
    border-radius: 0.5rem;
    border: 1px solid #e5e7eb;
    font-size: 0.95rem;
  }

  :global(.dark .modal-input) {
    background-color: #1f1f1f;
    border-color: #404040;
    color: #e5e7eb;
  }

  .modal-error {
    margin-top: 0.5rem;
    color: #dc2626;
    font-size: 0.85rem;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding: 0.75rem 1.25rem 1.25rem;
  }

  .modal-btn {
    padding: 0.5rem 0.9rem;
    border-radius: 0.5rem;
    border: 1px solid #d1d5db;
    background-color: #ffffff;
    color: #111827;
    cursor: pointer;
  }

  .modal-btn.primary {
    background-color: #2563eb;
    border-color: #2563eb;
    color: #ffffff;
  }

  .modal-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  :global(.dark .modal-btn) {
    background-color: #1f1f1f;
    border-color: #404040;
    color: #e5e7eb;
  }

  :global(.dark .page-header) {
    background-color: #2d2d2d;
    border-bottom-color: #404040;
  }

  .page-title {
    font-size: 1.875rem;
    font-weight: 700;
    color: #1a1a1a;
    margin: 0;
  }

  :global(.dark .page-title) {
    color: #f5f5f5;
  }

  .page-subtitle {
    font-size: 0.875rem;
    color: #666666;
    margin: 0.25rem 0 0 0;
  }

  :global(.dark .page-subtitle) {
    color: #a3a3a3;
  }

  .header-right {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    background-color: #ffffff;
    color: #1f2937;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .back-btn:hover {
    background-color: #f3f4f6;
    border-color: #9ca3af;
  }

  :global(.dark .back-btn) {
    background-color: #2d2d2d;
    color: #e5e7eb;
    border-color: #404040;
  }

  :global(.dark .back-btn:hover) {
    background-color: #3d3d3d;
    border-color: #525252;
  }

  .hugo-controls-wrapper {
    padding: 1rem 2rem;
    background-color: #ffffff;
    border-bottom: 1px solid #e5e5e5;
  }

  :global(.dark .hugo-controls-wrapper) {
    background-color: #2d2d2d;
    border-bottom-color: #404040;
  }

  .create-btn,
  .select-project-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1.25rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .create-btn:hover,
  .select-project-btn:hover {
    background-color: #2563eb;
  }

  .create-btn-empty {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.5rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
  }

  .create-btn-empty:hover {
    background-color: #2563eb;
  }

  /* Content */
  .page-content {
    flex: 1;
    padding: 2rem;
    overflow-y: auto;
  }

  .warning-banner {
    padding: 0.75rem 1rem;
    margin-bottom: 1rem;
    border-radius: 0.5rem;
    border: 1px solid #facc15;
    background-color: #fef9c3;
    color: #92400e;
    font-size: 0.875rem;
  }

  :global(.dark .warning-banner) {
    border-color: #eab308;
    background-color: #422006;
    color: #fef08a;
  }

  .info-banner {
    padding: 0.75rem 1rem;
    margin-bottom: 1rem;
    border-radius: 0.5rem;
    border: 1px solid #93c5fd;
    background-color: #eff6ff;
    color: #1e3a8a;
    font-size: 0.875rem;
  }

  :global(.dark .info-banner) {
    border-color: #2563eb;
    background-color: #0f172a;
    color: #bfdbfe;
  }

  .info-banner-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .info-banner-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.375rem 0.75rem;
    border-radius: 0.375rem;
    border: 1px solid #2563eb;
    background-color: #2563eb;
    color: #ffffff;
    font-size: 0.8125rem;
    font-weight: 600;
    cursor: pointer;
  }

  .info-banner-btn:disabled {
    cursor: not-allowed;
    opacity: 0.7;
  }

  /* Loading State */
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    gap: 1rem;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid #e5e5e5;
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .loading-state p {
    font-size: 0.875rem;
    color: #666666;
    margin: 0;
  }

  :global(.dark .loading-state p) {
    color: #a3a3a3;
  }

  /* Error State */
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    text-align: center;
  }

  .error-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .error-state h3 {
    font-size: 1.25rem;
    font-weight: 600;
    color: #1a1a1a;
    margin: 0 0 0.5rem 0;
  }

  :global(.dark .error-state h3) {
    color: #f5f5f5;
  }

  .error-state p {
    font-size: 0.875rem;
    color: #666666;
    margin: 0 0 1rem 0;
  }

  :global(.dark .error-state p) {
    color: #a3a3a3;
  }

  .retry-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1.25rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
  }

  .retry-btn:hover {
    background-color: #2563eb;
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    text-align: center;
  }

  .empty-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
  }

  .empty-state h3 {
    font-size: 1.25rem;
    font-weight: 600;
    color: #1a1a1a;
    margin: 0 0 0.5rem 0;
  }

  :global(.dark .empty-state h3) {
    color: #f5f5f5;
  }

  .empty-state p {
    font-size: 0.875rem;
    color: #666666;
    margin: 0;
  }

  :global(.dark .empty-state p) {
    color: #a3a3a3;
  }
</style>

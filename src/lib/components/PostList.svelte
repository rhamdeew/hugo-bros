<script lang="ts">
  import { Search, SlidersHorizontal, Plus, FileText, File, FileCode } from 'lucide-svelte';
  import PostCard from './PostCard.svelte';
  import type { Post } from '$lib/types';

  interface Props {
    posts: Post[];
    activeTab?: 'posts' | 'pages' | 'drafts';
    postsCount?: number;
    pagesCount?: number;
    draftsCount?: number;
    onCreate?: () => void;
    onEdit?: (post: Post) => void;
    onDelete?: (post: Post) => void;
    onTabChange?: (tab: 'posts' | 'pages' | 'drafts') => void;
  }

  let {
    posts,
    activeTab = 'posts',
    postsCount = 0,
    pagesCount = 0,
    draftsCount = 0,
    onCreate,
    onEdit,
    onDelete,
    onTabChange
  }: Props = $props();

  function handleTabClick(tab: 'posts' | 'pages' | 'drafts') {
    onTabChange?.(tab);
  }

  let searchQuery = $state('');
  let filterTag = $state<string | null>(null);
  let sortBy = $state<'date' | 'modified' | 'title'>('date');
  let sortOrder = $state<'desc' | 'asc'>('desc');
  let showFilters = $state(false);

  // Get unique tags from all posts
  let allTags = $derived(Array.from(
    new Set(posts.flatMap((post) => post.frontmatter.tags || []))
  ).sort());

  // Filter and sort posts
  const getPostDateTimestamp = (post: Post) => {
    const dateValue = post.date || post.frontmatter.date;
    const parsed = dateValue ? Date.parse(dateValue) : NaN;
    return Number.isNaN(parsed) ? 0 : parsed;
  };

  let filteredPosts = $derived(posts
    .filter((post) => {
      // Search filter
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        const matchesSearch =
          post.title.toLowerCase().includes(query) ||
          post.content.toLowerCase().includes(query) ||
          post.frontmatter.tags?.some((tag) => tag.toLowerCase().includes(query));

        if (!matchesSearch) return false;
      }

      // Tag filter
      if (filterTag) {
        if (!post.frontmatter.tags?.includes(filterTag)) {
          return false;
        }
      }

      return true;
    })
    .sort((a, b) => {
      let comparison = 0;

      switch (sortBy) {
        case 'date':
          comparison = getPostDateTimestamp(a) - getPostDateTimestamp(b);
          break;
        case 'modified':
          comparison = a.modifiedAt - b.modifiedAt;
          break;
        case 'title':
          comparison = a.title.localeCompare(b.title);
          break;
      }

      return sortOrder === 'asc' ? comparison : -comparison;
    }));

  function handleEdit(post: Post) {
    onEdit?.(post);
  }

  function handleDelete(post: Post) {
    onDelete?.(post);
  }

  function clearFilters() {
    searchQuery = '';
    filterTag = null;
  }

  let hasActiveFilters = $derived(!!searchQuery || !!filterTag);
</script>

<div class="post-list">
  <!-- Header with type tabs -->
  <div class="list-header">
    <div class="type-tabs">
      <button
        class="tab"
        class:active={activeTab === 'posts'}
        onclick={() => handleTabClick('posts')}
        type="button"
      >
        <FileText size={16} />
        Posts ({postsCount})
      </button>
      <button
        class="tab"
        class:active={activeTab === 'pages'}
        onclick={() => handleTabClick('pages')}
        type="button"
      >
        <File size={16} />
        Pages ({pagesCount})
      </button>
      <button
        class="tab"
        class:active={activeTab === 'drafts'}
        onclick={() => handleTabClick('drafts')}
        type="button"
      >
        <FileCode size={16} />
        Drafts ({draftsCount})
      </button>
    </div>

    {#if onCreate}
      <button class="create-btn" onclick={onCreate} type="button">
        <Plus size={18} />
        <span>New</span>
      </button>
    {/if}
  </div>

  <!-- Search and Filters -->
  <div class="search-bar">
    <div class="search-input-wrapper">
      <Search size={18} class="search-icon" />
      <input
        type="text"
        placeholder="Search posts..."
        class="search-input"
        bind:value={searchQuery}
      />
      {#if hasActiveFilters}
        <button class="clear-btn" onclick={clearFilters} type="button">
          Clear
        </button>
      {/if}
    </div>

    <button
      class="filter-toggle"
      class:active={showFilters}
      onclick={() => (showFilters = !showFilters)}
      type="button"
      title="Toggle filters"
    >
      <SlidersHorizontal size={18} />
    </button>
  </div>

  <!-- Expanded Filters -->
  {#if showFilters}
    <div class="filters-panel">
      <!-- Tag Filter -->
      {#if allTags.length > 0}
        <div class="filter-group">
          <label class="filter-label">Filter by tag:</label>
          <div class="tags-list">
            <button
              class="tag-filter"
              class:active={!filterTag}
              onclick={() => (filterTag = null)}
              type="button"
            >
              All
            </button>
            {#each allTags as tag}
              <button
                class="tag-filter"
                class:active={filterTag === tag}
                onclick={() => (filterTag = tag)}
                type="button"
              >
                #{tag}
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Sort Options -->
      <div class="filter-group">
        <label class="filter-label">Sort by:</label>
        <select class="sort-select" bind:value={sortBy}>
          <option value="modified">Last modified</option>
          <option value="date">Publish date</option>
          <option value="title">Title</option>
        </select>

        <button
          class="sort-order-btn"
          onclick={() => (sortOrder = sortOrder === 'desc' ? 'asc' : 'desc')}
          type="button"
          title={sortOrder === 'desc' ? 'Descending' : 'Ascending'}
        >
          {sortOrder === 'desc' ? '↓' : '↑'}
        </button>
      </div>
    </div>
  {/if}

  <!-- Posts Grid -->
  {#if filteredPosts.length > 0}
    <div class="posts-grid">
      {#each filteredPosts as post (post.id)}
        <PostCard
          post={post}
          onClick={() => handleEdit(post)}
          onDelete={() => handleDelete(post)}
        />
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <div class="empty-icon">📝</div>
      <h3>No posts found</h3>
      <p>
        {#if hasActiveFilters}
          Try adjusting your search or filters
        {:else}
          Create your first post to get started
        {/if}
      </p>
      {#if onCreate && !hasActiveFilters}
        <button class="create-btn-empty" onclick={onCreate} type="button">
          <Plus size={18} />
          Create Post
        </button>
      {/if}
    </div>
  {/if}

  <!-- Results Count -->
  {#if filteredPosts.length > 0}
    <div class="results-count">
      Showing {filteredPosts.length} of {posts.length} posts
    </div>
  {/if}
</div>

<style>
  .post-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  /* Header */
  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.5rem 0;
  }

  .type-tabs {
    display: flex;
    gap: 0.5rem;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background-color: transparent;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    color: #666666;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  :global(.dark .tab) {
    background-color: transparent;
    border-color: #404040;
    color: #a3a3a3;
  }

  .tab:hover {
    background-color: #f7f7f7;
  }

  :global(.dark .tab:hover) {
    background-color: #404040;
  }

  .tab.active {
    background-color: #eff6ff;
    border-color: #3b82f6;
    color: #3b82f6;
  }

  :global(.dark .tab.active) {
    background-color: #1e3a8a;
    border-color: #3b82f6;
    color: #93c5fd;
  }

  .create-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .create-btn:hover {
    background-color: #2563eb;
  }

  .create-btn-empty {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.5rem;
    margin-top: 1rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .create-btn-empty:hover {
    background-color: #2563eb;
  }

  /* Search Bar */
  .search-bar {
    display: flex;
    gap: 0.5rem;
  }

  .search-input-wrapper {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 0.75rem;
    color: #9ca3af;
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 0.625rem 1rem 0.625rem 2.5rem;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: #1a1a1a;
  }

  :global(.dark .search-input) {
    background-color: #2d2d2d;
    border-color: #404040;
    color: #f5f5f5;
  }

  .search-input:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .clear-btn {
    position: absolute;
    right: 0.5rem;
    padding: 0.25rem 0.5rem;
    background-color: #f7f7f7;
    border: 1px solid #e5e5e5;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    color: #666666;
    cursor: pointer;
  }

  :global(.dark .clear-btn) {
    background-color: #404040;
    border-color: #525252;
    color: #a3a3a3;
  }

  .clear-btn:hover {
    background-color: #e5e5e5;
  }

  :global(.dark .clear-btn:hover) {
    background-color: #525252;
  }

  .filter-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.625rem;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    color: #666666;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  :global(.dark .filter-toggle) {
    background-color: #2d2d2d;
    border-color: #404040;
    color: #a3a3a3;
  }

  .filter-toggle:hover {
    background-color: #f7f7f7;
  }

  :global(.dark .filter-toggle:hover) {
    background-color: #404040;
  }

  .filter-toggle.active {
    background-color: #eff6ff;
    border-color: #3b82f6;
    color: #3b82f6;
  }

  :global(.dark .filter-toggle.active) {
    background-color: #1e3a8a;
    border-color: #3b82f6;
    color: #93c5fd;
  }

  /* Filters Panel */
  .filters-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem;
    background-color: #f7f7f7;
    border: 1px solid #e5e5e5;
    border-radius: 0.5rem;
  }

  :global(.dark .filters-panel) {
    background-color: #2d2d2d;
    border-color: #404040;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .filter-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #666666;
  }

  :global(.dark .filter-label) {
    color: #a3a3a3;
  }

  .tags-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
  }

  .tag-filter {
    padding: 0.375rem 0.75rem;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 9999px;
    font-size: 0.75rem;
    color: #666666;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  :global(.dark .tag-filter) {
    background-color: #404040;
    border-color: #525252;
    color: #a3a3a3;
  }

  .tag-filter:hover {
    background-color: #f7f7f7;
  }

  :global(.dark .tag-filter:hover) {
    background-color: #525252;
  }

  .tag-filter.active {
    background-color: #3b82f6;
    border-color: #3b82f6;
    color: white;
  }

  .sort-select {
    padding: 0.5rem;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: #1a1a1a;
  }

  :global(.dark .sort-select) {
    background-color: #404040;
    border-color: #525252;
    color: #f5f5f5;
  }

  .sort-order-btn {
    padding: 0.5rem 0.75rem;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: #1a1a1a;
    cursor: pointer;
  }

  :global(.dark .sort-order-btn) {
    background-color: #404040;
    border-color: #525252;
    color: #f5f5f5;
  }

  /* Posts Grid */
  .posts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(500px, 1fr));
    gap: 1rem;
  }

  @media (max-width: 640px) {
    .posts-grid {
      grid-template-columns: 1fr;
    }
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

  /* Results Count */
  .results-count {
    padding: 0.5rem;
    text-align: center;
    font-size: 0.875rem;
    color: #666666;
  }

  :global(.dark .results-count) {
    color: #a3a3a3;
  }
</style>

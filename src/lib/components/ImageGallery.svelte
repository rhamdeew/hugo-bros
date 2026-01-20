<script lang="ts">
  import { X, Search, Upload as UploadIcon, Trash2, Image as ImageIcon, Folder, FolderPlus, ArrowUp } from 'lucide-svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { backend } from '$lib/services/backend';
  import type { StaticEntry } from '$lib/types';

  interface ImageWithSrc extends StaticEntry {
    displaySrc: string;
  }

  interface Props {
    open: boolean;
    onSelect?: (entry: StaticEntry) => void;
  }

  let {
    open = $bindable(false),
    onSelect
  }: Props = $props();

  let searchQuery = $state('');
  let sortBy = $state<'name' | 'date' | 'size'>('date');
  let selectedEntry = $state<StaticEntry | null>(null);
  let entries = $state<StaticEntry[]>([]);
  let loading = $state(false);
  let loadError = $state<string | null>(null);
  let currentDir = $state('');
  let showNewFolder = $state(false);
  let newFolderName = $state('');

  const resolveEntrySrc = (entry: StaticEntry) => {
    if (entry.fullPath) return convertFileSrc(entry.fullPath);

    const projectPath = backend.getProjectPath();
    if (projectPath) {
      if (entry.path) {
        return convertFileSrc(`${projectPath}/static/${entry.path}`);
      }
      if (entry.url && entry.url.startsWith('/')) {
        return convertFileSrc(`${projectPath}/static${entry.url}`);
      }
    }

    return entry.url || '';
  };

  $effect(() => {
    if (!open) return;
    currentDir;
    void loadEntries();
  });

  // Filter and sort entries
  let filteredEntries = $derived(entries
    .filter((entry) => {
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        return (
          entry.name.toLowerCase().includes(query) ||
          entry.path.toLowerCase().includes(query)
        );
      }
      return true;
    }));

  let directoryEntries = $derived(
    filteredEntries
      .filter((entry) => entry.kind === 'dir')
      .sort((a, b) => a.name.localeCompare(b.name))
  );

  let fileEntries = $derived(
    filteredEntries
      .filter((entry) => entry.kind === 'file')
      .sort((a, b) => {
        switch (sortBy) {
          case 'name':
            return a.name.localeCompare(b.name);
          case 'date':
            return b.createdAt - a.createdAt;
          case 'size':
            return b.size - a.size;
          default:
            return 0;
        }
      })
  );

  let filesWithSrc = $derived(fileEntries.map(entry => ({
    ...entry,
    displaySrc: resolveEntrySrc(entry)
  })) as ImageWithSrc[]);

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
  }

  async function loadEntries() {
    if (loading) return;
    loading = true;
    loadError = null;
    try {
      entries = await backend.listStaticEntries(currentDir);
    } catch (err) {
      loadError = err instanceof Error ? err.message : 'Failed to load entries';
    } finally {
      loading = false;
    }
  }

  function navigateToDir(path: string) {
    currentDir = path;
    selectedEntry = null;
  }

  function handleSelect(entry: StaticEntry) {
    if (entry.kind !== 'file') {
      return;
    }
    selectedEntry = entry;
    onSelect?.(entry);
    open = false;
  }

  function handleEntryClick(entry: StaticEntry) {
    selectedEntry = entry;
  }

  function handleEntryDoubleClick(entry: StaticEntry) {
    if (entry.kind === 'dir') {
      navigateToDir(entry.path);
      return;
    }
    handleSelect(entry);
  }

  async function handleDelete(entry: StaticEntry) {
    const label = entry.kind === 'dir' ? 'folder' : 'file';
    const prompt = entry.kind === 'dir'
      ? `Delete folder "${entry.name}" and all of its contents?`
      : `Delete "${entry.name}"?`;
    if (!confirm(prompt)) return;
    try {
      await backend.deleteStaticEntry(entry.path);
      await loadEntries();
      if (selectedEntry?.path === entry.path) {
        selectedEntry = null;
      }
    } catch (err) {
      alert(err instanceof Error ? err.message : `Failed to delete ${label}`);
    }
  }

  async function handleCreateFolder() {
    const name = newFolderName.trim();
    if (!name) return;
    try {
      await backend.createStaticFolder(currentDir, name);
      newFolderName = '';
      showNewFolder = false;
      await loadEntries();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to create folder');
    }
  }

  async function handleUpload() {
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [{
          name: 'Images',
          extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg']
        }]
      });

      if (!selected) return;
      const sourcePath = typeof selected === 'string' ? selected : selected[0];
      if (!sourcePath) return;
      await backend.copyImageToProject(sourcePath, currentDir);
      await loadEntries();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to upload image');
    }
  }

  let breadcrumbSegments = $derived((() => {
    if (!currentDir) return [];
    const parts = currentDir.split('/').filter(Boolean);
    return parts.map((segment, index) => ({
      name: segment,
      path: parts.slice(0, index + 1).join('/')
    }));
  })());

  function getParentDir() {
    if (!breadcrumbSegments.length) return '';
    return breadcrumbSegments.slice(0, -1).map((seg) => seg.name).join('/');
  }
</script>

{#if open}
  <div
    class="modal-overlay"
    role="presentation"
    onclick={() => (open = false)}
    onkeydown={(e) => {
      if (e.key === 'Escape') {
        open = false;
      }
    }}
  >
    <div
      class="modal-content"
      role="dialog"
      aria-modal="true"
      aria-labelledby="gallery-title"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="modal-header">
        <h2 id="gallery-title" class="modal-title">Image Gallery</h2>
        <button
          class="close-btn"
          onclick={() => (open = false)}
          type="button"
          aria-label="Close"
        >
          <X size={24} />
        </button>
      </div>

      <!-- Search and Controls -->
      <div class="modal-controls">
        <div class="search-wrapper">
          <Search size={18} class="search-icon" />
          <input
            type="text"
            class="search-input"
            bind:value={searchQuery}
            placeholder="Search images..."
          />
        </div>

        <div class="controls-right">
          <select class="sort-select" bind:value={sortBy}>
            <option value="date">Sort by date</option>
            <option value="name">Sort by name</option>
            <option value="size">Sort by size</option>
          </select>

          <button class="folder-btn" onclick={() => (showNewFolder = !showNewFolder)} type="button">
            <FolderPlus size={18} />
            <span>New Folder</span>
          </button>

          <button class="upload-btn" onclick={handleUpload} type="button">
            <UploadIcon size={18} />
            <span>Upload</span>
          </button>
        </div>
      </div>

      {#if showNewFolder}
        <div class="folder-row">
          <input
            class="folder-input"
            type="text"
            placeholder="Folder name"
            bind:value={newFolderName}
            onkeydown={(e) => {
              if (e.key === 'Enter') handleCreateFolder();
              if (e.key === 'Escape') showNewFolder = false;
            }}
          />
          <button class="folder-create-btn" onclick={handleCreateFolder} type="button">
            Create
          </button>
          <button class="folder-cancel-btn" onclick={() => (showNewFolder = false)} type="button">
            Cancel
          </button>
        </div>
      {/if}

      <div class="breadcrumb-row">
        <button
          class="breadcrumb-root"
          onclick={() => navigateToDir('')}
          type="button"
        >
          static
        </button>
        {#if currentDir}
          <span class="breadcrumb-separator">/</span>
        {/if}
        {#each breadcrumbSegments as segment}
          <button
            class="breadcrumb-link"
            onclick={() => navigateToDir(segment.path)}
            type="button"
          >
            {segment.name}
          </button>
          <span class="breadcrumb-separator">/</span>
        {/each}
      </div>

      <!-- Entries Grid -->
      <div class="images-grid">
        {#if currentDir}
          <button class="folder-up" onclick={() => navigateToDir(getParentDir())} type="button">
            <ArrowUp size={20} />
            <span>Up</span>
          </button>
        {/if}

        {#each directoryEntries as entry (entry.path)}
          <div
            class="image-card folder-card"
            class:selected={selectedEntry?.path === entry.path}
            onclick={() => handleEntryClick(entry)}
            ondblclick={() => handleEntryDoubleClick(entry)}
            role="button"
            tabindex="0"
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                handleEntryDoubleClick(entry);
              }
            }}
          >
            <div class="image-thumb folder-thumb">
              <Folder size={36} />
              <span class="folder-label" title={entry.name}>{entry.name}</span>
            </div>
            <div class="image-info">
              <p class="image-meta">Folder</p>
              <button
                class="delete-btn"
                onclick={(e) => {
                  e.stopPropagation();
                  handleDelete(entry);
                }}
                type="button"
                aria-label="Delete folder"
              >
                <Trash2 size={14} />
              </button>
            </div>
          </div>
        {/each}

        {#each filesWithSrc as entry (entry.fullPath || entry.path || entry.name)}
          <div
            class="image-card"
            class:selected={selectedEntry?.path === entry.path}
            onclick={() => handleEntryClick(entry)}
            ondblclick={() => handleEntryDoubleClick(entry)}
            role="button"
            tabindex="0"
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                handleEntryDoubleClick(entry);
              }
            }}
          >
            <div class="image-thumb">
              {#if entry.displaySrc}
                <img
                  src={entry.displaySrc}
                  alt={entry.name}
                  loading="lazy"
                  onerror={(e) => {
                    // Hide broken image and show placeholder
                    const img = e.target as HTMLImageElement;
                    img.style.display = 'none';
                    const placeholder = img.nextElementSibling as HTMLElement | null;
                    if (placeholder) placeholder.style.display = 'flex';
                  }}
                />
                <div class="no-image error-placeholder" style="display: none;">
                  <ImageIcon size={32} />
                </div>
              {:else}
                <div class="no-image">
                  <ImageIcon size={32} />
                </div>
              {/if}
            </div>
            <div class="image-info">
              <p class="image-name" title={entry.name}>{entry.name}</p>
              <p class="image-meta">{formatBytes(entry.size)}</p>
              <button
                class="delete-btn"
                onclick={(e) => {
                  e.stopPropagation();
                  handleDelete(entry);
                }}
                type="button"
                aria-label="Delete image"
              >
                <Trash2 size={14} />
              </button>
            </div>
          </div>
        {/each}

        {#if loading && directoryEntries.length + filesWithSrc.length === 0}
          <div class="empty-gallery">
            <ImageIcon size={48} class="empty-icon" />
            <h3>Loading...</h3>
          </div>
        {:else if loadError}
          <div class="empty-gallery">
            <ImageIcon size={48} class="empty-icon" />
            <h3>Failed to load entries</h3>
            <p>{loadError}</p>
          </div>
        {:else if directoryEntries.length + filesWithSrc.length === 0}
          <div class="empty-gallery">
            <ImageIcon size={48} class="empty-icon" />
            <h3>No images found</h3>
            <p>
              {#if searchQuery}
                Try a different search term
              {:else}
                Upload an image to get started
              {/if}
            </p>
            {#if !searchQuery}
              <button class="upload-btn-empty" onclick={handleUpload} type="button">
                <UploadIcon size={18} />
                Upload Image
              </button>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Selected Image Info -->
      {#if selectedEntry && selectedEntry.kind === 'file'}
        <div class="selected-info">
          <p class="info-text">
            Selected: <strong>{selectedEntry.name}</strong> ({formatBytes(selectedEntry.size)})
          </p>
          <button
            class="select-btn"
            onclick={() => selectedEntry && handleSelect(selectedEntry)}
            type="button"
          >
            Insert
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
  }

  .modal-content {
    width: 100%;
    max-width: 900px;
    max-height: 80vh;
    background-color: #ffffff;
    border-radius: 0.5rem;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  :global(.dark .modal-content) {
    background-color: #2d2d2d;
  }

  /* Header */
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid #e5e5e5;
  }

  :global(.dark .modal-header) {
    border-bottom-color: #404040;
  }

  .modal-title {
    font-size: 1.25rem;
    font-weight: 600;
    color: #1a1a1a;
    margin: 0;
  }

  :global(.dark .modal-title) {
    color: #f5f5f5;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.25rem;
    background-color: transparent;
    border: none;
    color: #666666;
    cursor: pointer;
    border-radius: 0.25rem;
  }

  :global(.dark .close-btn) {
    color: #a3a3a3;
  }

  .close-btn:hover {
    background-color: #f7f7f7;
  }

  :global(.dark .close-btn:hover) {
    background-color: #404040;
  }

  /* Controls */
  .modal-controls {
    display: flex;
    gap: 1rem;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid #e5e5e5;
  }

  :global(.dark .modal-controls) {
    border-bottom-color: #404040;
  }

  .search-wrapper {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .search-input {
    flex: 1;
    width: 100%;
    padding: 0.625rem 0.75rem;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    font-size: 0.875rem;
  }

  :global(.dark .search-input) {
    background-color: #404040;
    border-color: #525252;
  }

  .search-input:focus {
    outline: none;
    border-color: #3b82f6;
  }

  .controls-right {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .sort-select {
    padding: 0.625rem 0.75rem;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    font-size: 0.875rem;
  }

  :global(.dark .sort-select) {
    background-color: #404040;
    border-color: #525252;
  }

  .upload-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
  }

  .upload-btn:hover {
    background-color: #2563eb;
  }

  .folder-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 0.9rem;
    background-color: #ffffff;
    color: #374151;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
  }

  .folder-btn:hover {
    background-color: #f3f4f6;
  }

  :global(.dark .folder-btn) {
    background-color: #404040;
    color: #e5e7eb;
    border-color: #525252;
  }

  .folder-row {
    display: flex;
    gap: 0.5rem;
    padding: 0 1.5rem 1rem;
  }

  .folder-input {
    flex: 1;
    padding: 0.625rem 0.75rem;
    border-radius: 0.375rem;
    border: 1px solid #e5e5e5;
    font-size: 0.875rem;
  }

  :global(.dark .folder-input) {
    background-color: #404040;
    border-color: #525252;
    color: #e5e7eb;
  }

  .folder-create-btn,
  .folder-cancel-btn {
    padding: 0.625rem 0.9rem;
    border-radius: 0.375rem;
    border: 1px solid #d1d5db;
    background-color: #ffffff;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .folder-create-btn {
    background-color: #3b82f6;
    border-color: #3b82f6;
    color: #ffffff;
  }

  .folder-cancel-btn {
    color: #374151;
  }

  :global(.dark .folder-create-btn) {
    border-color: #2563eb;
  }

  :global(.dark .folder-cancel-btn) {
    background-color: #404040;
    color: #e5e7eb;
    border-color: #525252;
  }

  .breadcrumb-row {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0 1.5rem 0.75rem;
    font-size: 0.85rem;
    color: #6b7280;
    flex-wrap: wrap;
  }

  .breadcrumb-root,
  .breadcrumb-link {
    background: none;
    border: none;
    color: #2563eb;
    cursor: pointer;
    padding: 0;
    font-size: 0.85rem;
  }

  :global(.dark .breadcrumb-root),
  :global(.dark .breadcrumb-link) {
    color: #93c5fd;
  }

  .breadcrumb-separator {
    color: #9ca3af;
  }

  .folder-up {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    border: 2px dashed #cbd5f5;
    border-radius: 0.5rem;
    background-color: #f8fafc;
    color: #1f2937;
    cursor: pointer;
  }

  :global(.dark .folder-up) {
    border-color: #334155;
    background-color: #1f2937;
    color: #e5e7eb;
  }

  /* Images Grid */
  .images-grid {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.5rem;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 1rem;
    align-items: start;
  }

  .image-card {
    position: relative;
    cursor: pointer;
    border: 2px solid transparent;
    border-radius: 0.5rem;
    overflow: hidden;
    transition: all 0.15s ease;
    height: 150px;
  }

  .image-card:hover {
    border-color: #bfdbfe;
  }

  :global(.dark .image-card:hover) {
    border-color: #1e40af;
  }

  .image-card.selected {
    border-color: #3b82f6;
  }

  :global(.dark .image-card.selected) {
    border-color: #3b82f6;
  }

  .image-thumb {
    aspect-ratio: 1;
    min-height: 110px;
    height: 110px;
    background-color: #f7f7f7;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    position: relative;
  }

  .folder-card {
    height: 150px;
  }

  .folder-thumb {
    color: #2563eb;
    background: linear-gradient(135deg, #e0e7ff, #f8fafc);
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.5rem;
    height: 100%;
    width: 100%;
  }

  :global(.dark .image-thumb) {
    background-color: #404040;
  }

  :global(.dark .folder-thumb) {
    background: linear-gradient(135deg, #1e293b, #0f172a);
    color: #93c5fd;
  }

  .image-thumb img {
    height: 100%;
    width: 100%;
    object-fit: cover;
  }

  .no-image {
    color: #d1d5db;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    position: absolute;
    top: 0;
    left: 0;
  }

  .no-image.error-placeholder {
    background-color: #f7f7f7;
  }

  :global(.dark .no-image) {
    color: #525252;
  }

  :global(.dark .no-image.error-placeholder) {
    background-color: #404040;
  }

  .image-info {
    padding: 0.5rem;
    background-color: #ffffff;
    border-top: 1px solid #e5e5e5;
    min-height: 40px;
  }

  :global(.dark .image-info) {
    background-color: #2d2d2d;
    border-top-color: #404040;
  }

  .image-name {
    font-size: 0.75rem;
    font-weight: 500;
    color: #1a1a1a;
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(.dark .image-name) {
    color: #f5f5f5;
  }

  .image-meta {
    font-size: 0.714px;
    color: #666666;
    margin: 0;
  }

  .folder-label {
    font-size: 0.75rem;
    color: #1e3a8a;
    text-align: center;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 0 0.25rem;
  }

  :global(.dark .folder-label) {
    color: #bfdbfe;
  }

  :global(.dark .image-meta) {
    color: #a3a3a3;
  }

  .delete-btn {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    padding: 0.25rem;
    background-color: rgba(255, 255, 255, 0.9);
    border: none;
    border-radius: 0.25rem;
    color: #dc2626;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .image-card:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background-color: #ffffff;
  }

  :global(.dark .delete-btn) {
    background-color: rgba(0, 0, 0, 0.8);
  }

  /* Empty Gallery */
  .empty-gallery {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    text-align: center;
  }

  .empty-gallery h3 {
    font-size: 1.125rem;
    font-weight: 600;
    color: #1a1a1a;
    margin: 0 0 0.5rem 0;
  }

  :global(.dark .empty-gallery h3) {
    color: #f5f5f5;
  }

  .empty-gallery p {
    font-size: 0.875rem;
    color: #666666;
    margin: 0 0 1rem 0;
  }

  :global(.dark .empty-gallery p) {
    color: #a3a3a3;
  }

  .upload-btn-empty {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
  }

  .upload-btn-empty:hover {
    background-color: #2563eb;
  }

  /* Selected Info */
  .selected-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-top: 1px solid #e5e5e5;
  }

  :global(.dark .selected-info) {
    border-top-color: #404040;
  }

  .info-text {
    font-size: 0.875rem;
    color: #1a1a1a;
    margin: 0;
  }

  :global(.dark .info-text) {
    color: #f5f5f5;
  }

  .select-btn {
    padding: 0.625rem 1.5rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
  }

  .select-btn:hover {
    background-color: #2563eb;
  }
</style>

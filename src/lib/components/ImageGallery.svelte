<script lang="ts">
  import { X, Search, Upload as UploadIcon, Trash2, Image as ImageIcon } from 'lucide-svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { backend } from '$lib/services/backend';
  import type { ImageInfo } from '$lib/types';

  interface ImageWithSrc extends ImageInfo {
    displaySrc: string;
  }

  interface Props {
    open: boolean;
    images: ImageInfo[];
    onSelect?: (image: ImageInfo) => void;
    onDelete?: (image: ImageInfo) => void;
    onUpload?: () => void;
  }

  let {
    open = $bindable(false),
    images,
    onSelect,
    onDelete,
    onUpload
  }: Props = $props();

  let searchQuery = $state('');
  let sortBy = $state<'name' | 'date' | 'size'>('date');
  let selectedImage = $state<ImageInfo | null>(null);

  const resolveImageSrc = (image: ImageInfo) => {
    if (image.fullPath) return convertFileSrc(image.fullPath);

    const projectPath = backend.getProjectPath();
    if (projectPath) {
      if (image.path) {
        return convertFileSrc(`${projectPath}/source/images/${image.path}`);
      }
      if (image.url && image.url.startsWith('/')) {
        return convertFileSrc(`${projectPath}/source${image.url}`);
      }
    }

    return image.url || '';
  };

  // Pre-compute display URLs for all images
  let imagesWithSrc = $derived(images.map(img => ({
    ...img,
    displaySrc: resolveImageSrc(img)
  })) as ImageWithSrc[]);

  // Filter and sort images
  let filteredImages = $derived(imagesWithSrc
    .filter((img) => {
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        return (
          img.filename.toLowerCase().includes(query) ||
          img.path.toLowerCase().includes(query)
        );
      }
      return true;
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'name':
          return a.filename.localeCompare(b.filename);
        case 'date':
          return b.createdAt - a.createdAt;
        case 'size':
          return b.size - a.size;
        default:
          return 0;
      }
    }));

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
  }

  function handleSelect(image: ImageInfo) {
    selectedImage = image;
    onSelect?.(image);
    open = false;
  }

  function handleDelete(image: ImageInfo) {
    if (confirm(`Delete "${image.filename}"?`)) {
      onDelete?.(image);
    }
  }

  function handleImageClick(image: ImageInfo) {
    selectedImage = image;
  }
</script>

{#if open}
  <div class="modal-overlay" onclick={() => (open = false)}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <!-- Header -->
      <div class="modal-header">
        <h2 class="modal-title">Image Gallery</h2>
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

          {#if onUpload}
            <button class="upload-btn" onclick={onUpload} type="button">
              <UploadIcon size={18} />
              <span>Upload</span>
            </button>
          {/if}
        </div>
      </div>

      <!-- Images Grid -->
      <div class="images-grid">
        {#each filteredImages as image (image.fullPath || image.path || image.filename)}
          <div
            class="image-card"
            class:selected={selectedImage?.fullPath === image.fullPath}
            onclick={() => handleImageClick(image)}
            ondblclick={() => handleSelect(image)}
            role="button"
            tabindex="0"
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                handleSelect(image);
              }
            }}
          >
            <div class="image-thumb">
              {#if image.displaySrc}
                <img
                  src={image.displaySrc}
                  alt={image.filename}
                  loading="lazy"
                  onerror={(e) => {
                    // Hide broken image and show placeholder
                    const img = e.target as HTMLImageElement;
                    img.style.display = 'none';
                    const placeholder = img.nextElementSibling;
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
              <p class="image-name" title={image.filename}>{image.filename}</p>
              <p class="image-meta">{formatBytes(image.size)}</p>
              {#if onDelete}
                <button
                  class="delete-btn"
                  onclick={(e) => {
                    e.stopPropagation();
                    handleDelete(image);
                  }}
                  type="button"
                  aria-label="Delete image"
                >
                  <Trash2 size={14} />
                </button>
              {/if}
            </div>
          </div>
        {/each}

        {#if filteredImages.length === 0}
          <div class="empty-gallery">
            <ImageIcon size={48} class="empty-icon" />
            <h3>No images found</h3>
            <p>
              {#if searchQuery}
                Try a different search term
              {:else if onUpload}
                Upload an image to get started
              {:else}
                No images available
              {/if}
            </p>
            {#if onUpload && !searchQuery}
              <button class="upload-btn-empty" onclick={onUpload} type="button">
                <UploadIcon size={18} />
                Upload Image
              </button>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Selected Image Info -->
      {#if selectedImage}
        <div class="selected-info">
          <p class="info-text">
            Selected: <strong>{selectedImage.filename}</strong> ({formatBytes(selectedImage.size)})
          </p>
          <button
            class="select-btn"
            onclick={() => selectedImage && handleSelect(selectedImage)}
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

  /* Images Grid */
  .images-grid {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.5rem;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 1rem;
  }

  .image-card {
    position: relative;
    cursor: pointer;
    border: 2px solid transparent;
    border-radius: 0.5rem;
    overflow: hidden;
    transition: all 0.15s ease;
    height: 100px;
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
    min-height: 120px;
    height: 100px;
    background-color: #f7f7f7;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    position: relative;
  }

  :global(.dark .image-thumb) {
    background-color: #404040;
  }

  .image-thumb img {
    height: 100%;
    width: auto;
    max-width: 100%;
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

  .empty-icon {
    color: #d1d5db;
    margin-bottom: 1rem;
  }

  :global(.dark .empty-icon) {
    color: #525252;
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

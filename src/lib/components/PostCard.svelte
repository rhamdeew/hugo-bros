<script lang="ts">
  import { Calendar, Tag, Trash2, Copy } from 'lucide-svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { backend } from '$lib/services/backend';
  import type { FrontmatterConfig, Post } from '$lib/types';

  interface Props {
    post: Post;
    frontmatterConfig?: FrontmatterConfig;
    onDelete?: () => void;
    onDuplicate?: () => void;
    onClick?: () => void;
  }

  let {
    post,
    frontmatterConfig,
    onDelete,
    onDuplicate,
    onClick
  }: Props = $props();

  const truncateText = (text: string, maxLength: number) => {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength).trim() + '...';
  };

  const formatDate = (dateInput?: string | number) => {
    if (!dateInput) return '';
    try {
      const date = typeof dateInput === 'number'
        ? new Date(dateInput * 1000)
        : new Date(dateInput);
      return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric'
      });
    } catch {
      return typeof dateInput === 'string' ? dateInput : '';
    }
  };

  const resolveImageUrl = (url?: string) => {
    if (!url) return '';
    const projectPath = backend.getProjectPath();
    if (!projectPath || !url.startsWith('/')) return url;
    return convertFileSrc(`${projectPath}/static${url}`);
  };

  const getCustomFieldString = (name: string) => {
    const value = post.frontmatter.customFields?.[name];
    return typeof value === 'string' ? value : '';
  };

  const getCustomImageAlt = (imageFieldName: string) => {
    const altCandidates = [`${imageFieldName}_alt`, `${imageFieldName}Alt`];
    for (const candidate of altCandidates) {
      const value = getCustomFieldString(candidate);
      if (value) return value;
    }
    return post.title;
  };

  let previewText = $derived(truncateText(post.content.replace(/<[^>]*>/g, ''), 150));
  let imageFieldNames = $derived(
    (frontmatterConfig?.customFields || [])
      .filter((field) => field.type === 'image')
      .map((field) => field.name)
  );
  let preferredImageField = $derived(frontmatterConfig?.previewImageField || '');
  let preferredImageValue = $derived(
    preferredImageField ? getCustomFieldString(preferredImageField) : ''
  );
  let selectedImageField = $derived(
    preferredImageValue
      ? preferredImageField
      : imageFieldNames.find((name) => getCustomFieldString(name)) || ''
  );
  let imageUrl = $derived(
    resolveImageUrl(selectedImageField ? getCustomFieldString(selectedImageField) : '')
  );
  let imageAlt = $derived(
    selectedImageField ? getCustomImageAlt(selectedImageField) : post.title
  );
  let displayTags = $derived(post.frontmatter.tags?.slice(0, 3) || []);
</script>

<div
  class="post-card"
  class:clickable={!!onClick}
  onclick={() => onClick?.()}
  role={onClick ? 'button' : 'article'}
  tabindex={onClick ? 0 : undefined}
  onkeydown={(e) => {
    if (onClick && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault();
      onClick();
    }
  }}
>
  <!-- Image -->
  {#if imageUrl}
    <div class="post-image">
      <img src={imageUrl} alt={imageAlt} />
    </div>
  {:else}
    <div class="post-image placeholder">
      <div class="placeholder-icon">📝</div>
    </div>
  {/if}

  <!-- Content -->
  <div class="post-content">
    <h3 class="post-title">{post.title}</h3>
  <p class="post-date">
    <Calendar size={14} />
    {formatDate(post.date || post.frontmatter.date || post.modifiedAt)}
  </p>
    <p class="post-preview">{previewText}</p>

    {#if displayTags.length > 0}
      <div class="post-tags">
        {#each displayTags as tag}
          <span class="tag">#{tag}</span>
        {/each}
        {#if post.frontmatter.tags && post.frontmatter.tags.length > 3}
          <span class="tag more">+{post.frontmatter.tags.length - 3}</span>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Actions -->
  {#if onDelete || onDuplicate}
    <div class="post-actions" onclick={(e) => e.stopPropagation()}>
      {#if onDuplicate}
        <button
          class="action-btn"
          onclick={onDuplicate}
          title="Duplicate post"
          type="button"
        >
          <Copy size={16} />
        </button>
      {/if}
      {#if onDelete}
        <button
          class="action-btn danger"
          onclick={onDelete}
          title="Delete post"
          type="button"
        >
          <Trash2 size={16} />
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .post-card {
    position: relative;
    display: flex;
    gap: 1rem;
    padding: 1rem;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.5rem;
    transition: all 0.15s ease;
  }

  :global(.dark .post-card) {
    background-color: #2d2d2d;
    border-color: #404040;
  }

  .post-card.clickable {
    cursor: pointer;
  }

  .post-card.clickable:hover {
    border-color: #3b82f6;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  :global(.dark .post-card.clickable:hover) {
    border-color: #60a5fa;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .post-card.clickable:focus-visible {
    outline: 2px solid #3b82f6;
    outline-offset: 2px;
  }

  /* Image */
  .post-image {
    flex-shrink: 0;
    width: 120px;
    height: 120px;
    border-radius: 0.375rem;
    overflow: hidden;
    background-color: #f7f7f7;
  }

  .post-image img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .post-image.placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  }

  .placeholder-icon {
    font-size: 2rem;
  }

  /* Content */
  .post-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .post-title {
    font-size: 1.125rem;
    font-weight: 600;
    color: #1a1a1a;
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(.dark .post-title) {
    color: #f5f5f5;
  }

  .post-date {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.875rem;
    color: #666666;
    margin: 0;
  }

  :global(.dark .post-date) {
    color: #a3a3a3;
  }

  .post-preview {
    font-size: 0.875rem;
    color: #666666;
    line-height: 1.5;
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  :global(.dark .post-preview) {
    color: #a3a3a3;
  }

  /* Tags */
  .post-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
    margin-top: auto;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    padding: 0.125rem 0.5rem;
    background-color: #eff6ff;
    color: #3b82f6;
    font-size: 0.75rem;
    font-weight: 500;
    border-radius: 9999px;
  }

  :global(.dark .tag) {
    background-color: #1e3a8a;
    color: #93c5fd;
  }

  .tag.more {
    background-color: #f7f7f7;
    color: #666666;
  }

  :global(.dark .tag.more) {
    background-color: #404040;
    color: #a3a3a3;
  }

  /* Actions */
  .post-actions {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    display: flex;
    gap: 0.25rem;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .post-card:hover .post-actions {
    opacity: 1;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    color: #666666;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  :global(.dark .action-btn) {
    background-color: #2d2d2d;
    border-color: #404040;
    color: #a3a3a3;
  }

  .action-btn:hover {
    background-color: #f7f7f7;
    border-color: #d1d5db;
  }

  :global(.dark .action-btn:hover) {
    background-color: #404040;
    border-color: #525252;
  }

  .action-btn.danger:hover {
    background-color: #fef2f2;
    border-color: #fecaca;
    color: #dc2626;
  }

  :global(.dark .action-btn.danger:hover) {
    background-color: #450a0a;
    border-color: #7f1d1d;
    color: #fca5a5;
  }

  /* Responsive */
  @media (max-width: 640px) {
    .post-card {
      flex-direction: column;
    }

    .post-image {
      width: 100%;
      height: 160px;
    }

    .post-actions {
      opacity: 1;
    }
  }
</style>

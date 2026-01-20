<script lang="ts">
  import { ChevronDown, ChevronUp, Plus, X } from 'lucide-svelte';
  import type { Frontmatter } from '$lib/types';

  interface Props {
    frontmatter: Frontmatter;
  }

  let {
    frontmatter = $bindable()
  }: Props = $props();
  let isOpen = $state(true);

  let newTagName = $state('');
  let newCategoryName = $state('');
  let customFieldName = $state('');
  let customFieldValue = $state('');

  function addTag() {
    if (newTagName.trim()) {
      frontmatter.tags = [...(frontmatter.tags || []), newTagName.trim()];
      newTagName = '';
    }
  }

  function removeTag(index: number) {
    frontmatter.tags = frontmatter.tags?.filter((_, i) => i !== index) || [];
  }

  function addCategory() {
    if (newCategoryName.trim()) {
      frontmatter.categories = [...(frontmatter.categories || []), newCategoryName.trim()];
      newCategoryName = '';
    }
  }

  function removeCategory(index: number) {
    frontmatter.categories = frontmatter.categories?.filter((_, i) => i !== index) || [];
  }

  function addCustomField() {
    if (customFieldName.trim()) {
      frontmatter.customFields = frontmatter.customFields || {};
      frontmatter.customFields[customFieldName.trim()] = customFieldValue;
      customFieldName = '';
      customFieldValue = '';
    }
  }

  function removeCustomField(name: string) {
    if (frontmatter.customFields) {
      delete frontmatter.customFields[name];
    }
  }

  function formatDate(dateString: string): string {
    try {
      const date = new Date(dateString);
      return date.toISOString().slice(0, 16); // YYYY-MM-DDTHH:mm
    } catch {
      return dateString;
    }
  }
</script>

<div class="frontmatter-editor">
  <div
    class="editor-header"
    role="button"
    tabindex="0"
    onclick={() => (isOpen = !isOpen)}
    onkeydown={(e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        isOpen = !isOpen;
      }
    }}
  >
    <h2 class="editor-title">Frontmatter</h2>
    <button class="toggle-btn" type="button" tabindex="-1">
      {#if isOpen}
        <ChevronUp size={20} />
      {:else}
        <ChevronDown size={20} />
      {/if}
    </button>
  </div>

  {#if isOpen}
    <div class="editor-content">
      <!-- Title -->
      <div class="field-group">
        <label for="title" class="field-label">Title *</label>
        <input
          id="title"
          type="text"
          class="field-input"
          bind:value={frontmatter.title}
          placeholder="Post title"
          required
        />
      </div>

      <!-- Date -->
      <div class="field-group">
        <label for="date" class="field-label">Date *</label>
        <input
          id="date"
          type="datetime-local"
          class="field-input"
          bind:value={frontmatter.date}
        />
      </div>

      <!-- Updated -->
      <div class="field-group">
        <label for="updated" class="field-label">Updated</label>
        <input
          id="updated"
          type="datetime-local"
          class="field-input"
          bind:value={frontmatter.updated}
        />
      </div>

      <!-- Description -->
      <div class="field-group">
        <label for="description" class="field-label">Description</label>
        <textarea
          id="description"
          class="field-textarea"
          bind:value={frontmatter.description}
          placeholder="Brief description of the post for SEO and previews"
          rows="3"
        ></textarea>
        <span class="field-hint">Used in meta tags and post previews</span>
      </div>

      <!-- Tags -->
      <div class="field-group">
        <span class="field-label">Tags</span>
        <div class="tags-input-group">
          <div class="tags-list">
            {#each (frontmatter.tags || []) as tag, index (index)}
              <span class="tag-chip">
                #{tag}
                <button
                  class="tag-remove"
                  onclick={() => removeTag(index)}
                  type="button"
                  aria-label="Remove tag"
                >
                  <X size={14} />
                </button>
              </span>
            {/each}
          </div>
          <div class="tag-add-row">
            <input
              type="text"
              class="field-input"
              bind:value={newTagName}
              placeholder="Add tag..."
              onkeypress={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  addTag();
                }
              }}
            />
            <button class="add-btn" onclick={addTag} type="button">
              <Plus size={16} />
            </button>
          </div>
        </div>
      </div>

      <!-- Categories -->
      <div class="field-group">
        <span class="field-label">Categories</span>
        <div class="categories-input-group">
          <div class="categories-list">
            {#each (frontmatter.categories || []) as category, index (index)}
              <span class="category-chip">
                {category}
                <button
                  class="category-remove"
                  onclick={() => removeCategory(index)}
                  type="button"
                  aria-label="Remove category"
                >
                  <X size={14} />
                </button>
              </span>
            {/each}
          </div>
          <div class="category-add-row">
            <input
              type="text"
              class="field-input"
              bind:value={newCategoryName}
              placeholder="Add category..."
              onkeypress={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  addCategory();
                }
              }}
            />
            <button class="add-btn" onclick={addCategory} type="button">
              <Plus size={16} />
            </button>
          </div>
        </div>
      </div>

      <!-- Permalink -->
      <div class="field-group">
        <label for="permalink" class="field-label">Permalink</label>
        <input
          id="permalink"
          type="text"
          class="field-input"
          bind:value={frontmatter.permalink}
          placeholder="/2024/my-post"
        />
        <span class="field-hint">Custom URL path for this post</span>
      </div>

      <!-- Layout -->
      <div class="field-group">
        <label for="layout" class="field-label">Layout</label>
        <input
          id="layout"
          type="text"
          class="field-input"
          bind:value={frontmatter.layout}
          placeholder="post"
        />
      </div>

      <!-- Comments -->
      <div class="field-group">
        <span class="field-label">Comments</span>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={frontmatter.comments} />
          <span>Allow comments</span>
        </label>
      </div>

      <!-- Draft -->
      <div class="field-group">
        <span class="field-label">Draft</span>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={frontmatter.draft} />
          <span>Mark as draft</span>
        </label>
      </div>

      <!-- Custom Fields -->
      <div class="field-group">
        <span class="field-label">Custom Fields</span>
        <div class="custom-fields-list">
          {#each Object.entries(frontmatter.customFields || {}) as [key, value]}
            <div class="custom-field-row">
              <input type="text" class="field-input" value={key} disabled />
              <input
                type="text"
                class="field-input"
                bind:value={frontmatter.customFields![key]}
                placeholder="Value"
              />
              <button
                class="remove-btn"
                onclick={() => removeCustomField(key)}
                type="button"
                aria-label="Remove field"
              >
                <X size={16} />
              </button>
            </div>
          {/each}
        </div>
        <div class="custom-field-add-row">
          <input
            type="text"
            class="field-input"
            bind:value={customFieldName}
            placeholder="Field name"
          />
          <input
            type="text"
            class="field-input"
            bind:value={customFieldValue}
            placeholder="Value"
          />
          <button class="add-btn" onclick={addCustomField} type="button">
            <Plus size={16} />
            <span>Add</span>
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .frontmatter-editor {
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.5rem;
    overflow: hidden;
  }

  :global(.dark .frontmatter-editor) {
    background-color: #2d2d2d;
    border-color: #404040;
  }

  .editor-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    border-bottom: 1px solid #e5e5e5;
    cursor: pointer;
    user-select: none;
  }

  :global(.dark .editor-header) {
    border-bottom-color: #404040;
  }

  .editor-title {
    font-size: 1.125rem;
    font-weight: 600;
    color: #1a1a1a;
    margin: 0;
  }

  :global(.dark .editor-title) {
    color: #f5f5f5;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.25rem;
    background-color: transparent;
    border: none;
    color: #666666;
    cursor: pointer;
  }

  :global(.dark .toggle-btn) {
    color: #a3a3a3;
  }

  .editor-content {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .field-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #1a1a1a;
  }

  :global(.dark .field-label) {
    color: #f5f5f5;
  }

  .field-input {
    padding: 0.625rem 0.75rem;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: #1a1a1a;
  }

  :global(.dark .field-input) {
    background-color: #404040;
    border-color: #525252;
    color: #f5f5f5;
  }

  .field-input:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .field-input:disabled {
    background-color: #f7f7f7;
    color: #666666;
    cursor: not-allowed;
  }

  :global(.dark .field-input:disabled) {
    background-color: #2d2d2d;
    color: #666666;
  }

  .field-textarea {
    padding: 0.625rem 0.75rem;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: #1a1a1a;
    font-family: inherit;
    resize: vertical;
    min-height: 4rem;
  }

  :global(.dark .field-textarea) {
    background-color: #404040;
    border-color: #525252;
    color: #f5f5f5;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: #1a1a1a;
  }

  :global(.dark .checkbox-label) {
    color: #f5f5f5;
  }

  .field-textarea:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .field-hint {
    font-size: 0.75rem;
    color: #666666;
  }

  :global(.dark .field-hint) {
    color: #a3a3a3;
  }

  /* Tags */
  .tags-input-group,
  .categories-input-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .tags-list,
  .categories-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
  }

  .tag-chip,
  .category-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.5rem 0.375rem 0.75rem;
    background-color: #eff6ff;
    border: 1px solid #bfdbfe;
    border-radius: 9999px;
    font-size: 0.75rem;
    color: #3b82f6;
  }

  :global(.dark .tag-chip),
  :global(.dark .category-chip) {
    background-color: #1e3a8a;
    border-color: #3b82f6;
    color: #93c5fd;
  }

  .tag-remove,
  .category-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    background-color: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    opacity: 0.7;
  }

  .tag-remove:hover,
  .category-remove:hover {
    opacity: 1;
  }

  .tag-add-row,
  .category-add-row,
  .custom-field-add-row {
    display: flex;
    gap: 0.5rem;
  }

  .add-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    padding: 0.5rem 0.75rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
  }

  .add-btn:hover {
    background-color: #2563eb;
  }

  /* Custom Fields */
  .custom-fields-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .custom-field-row {
    display: flex;
    gap: 0.5rem;
  }

  .remove-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.5rem;
    background-color: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 0.375rem;
    color: #dc2626;
    cursor: pointer;
    flex-shrink: 0;
  }

  :global(.dark .remove-btn) {
    background-color: #450a0a;
    border-color: #7f1d1d;
    color: #fca5a5;
  }

  .remove-btn:hover {
    background-color: #fee2e2;
  }

  :global(.dark .remove-btn:hover) {
    background-color: #7f1d1d;
  }
</style>

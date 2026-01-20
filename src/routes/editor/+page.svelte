<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto, beforeNavigate } from '$app/navigation';
  import { marked } from 'marked';
  import {
    ArrowLeft,
    Save,
    Settings,
    Eye,
    EyeOff,
    Bold,
    Italic,
    Strikethrough,
    Code,
    List,
    ListOrdered,
    Quote,
    Minus,
    Heading1,
    Heading2,
    Heading3,
    Link as LinkIcon,
    Image as ImageIcon,
    FileText,
    AlertCircle,
    CheckCircle,
    Loader2,
    ChevronDown,
    ChevronUp,
    ChevronLeft,
    ChevronRight,
    Plus,
    X,
    GripVertical
  } from 'lucide-svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { confirm, open } from '@tauri-apps/plugin-dialog';
  import ImageGallery from '$lib/components/ImageGallery.svelte';
  import { backend } from '$lib/services/backend';
  import type { Post, Page, Draft, StaticEntry, FrontmatterConfig } from '$lib/types';

  // State
  let post = $state<Post | Page | Draft | null>(null);
  let markdownContent = $state('');
  let originalContent = $state('');
  let hasUnsavedChanges = $state(false);
  let saveStatus = $state<'saved' | 'saving' | 'unsaved' | 'error'>('saved');
  let saveMessage = $state('');
  let lastSavedAt = $state<Date | null>(null);

  // UI State
  let showPreview = $state(true);
  let showFrontmatter = $state(true);
  let showImageGallery = $state(false);
  let pendingImageField = $state<
    | { kind: 'content' }
    | { kind: 'frontmatter'; fieldName: string }
    | null
  >(null);
  let isLoading = $state(true);
  let loadError = $state<string | null>(null);
  let entryType = $state<'post' | 'page' | 'draft'>('post');
  let shortcutsCleanup: (() => void) | null = null;
  let frontmatterConfig = $state<FrontmatterConfig | null>(null);
  let customGroupCollapsed = $state<Record<string, boolean>>({});

  // Editor refs
  let textareaRef = $state<HTMLTextAreaElement | null>(null);
  let autoSaveTimer: ReturnType<typeof setInterval> | null = null;

  // Resizable panel
  let editorWidth = $state(50); // percentage
  let isResizing = $state(false);

  // Frontmatter editing
  let newTagName = $state('');
  let newCategoryName = $state('');

  // Helper to convert project-relative image URLs to Tauri asset URLs
  function getPreviewImageSrc(relativeUrl: string | undefined): string {
    if (!relativeUrl) return '';
    const projectPath = backend.getProjectPath();
    if (!projectPath) return relativeUrl;
    if (!relativeUrl.startsWith('/')) return relativeUrl;
    // Project-relative URLs start with /, we need to prepend static/ path
    const fullPath = `${projectPath}/static${relativeUrl}`;
    return convertFileSrc(fullPath);
  }

  function formatDatetimeLocal(value?: string | null): string {
    if (!value) return '';
    const parsed = new Date(value);
    if (Number.isNaN(parsed.getTime())) return value;
    const pad = (num: number) => String(num).padStart(2, '0');
    return `${parsed.getFullYear()}-${pad(parsed.getMonth() + 1)}-${pad(parsed.getDate())}T${pad(parsed.getHours())}:${pad(parsed.getMinutes())}`;
  }

  function toRfc3339Local(value: string): string {
    if (!value) return '';
    const parsed = new Date(value);
    if (Number.isNaN(parsed.getTime())) return value;
    const pad = (num: number) => String(num).padStart(2, '0');
    const offsetMinutes = -parsed.getTimezoneOffset();
    const sign = offsetMinutes >= 0 ? '+' : '-';
    const absOffset = Math.abs(offsetMinutes);
    const offsetHours = pad(Math.floor(absOffset / 60));
    const offsetMins = pad(absOffset % 60);
    return `${parsed.getFullYear()}-${pad(parsed.getMonth() + 1)}-${pad(parsed.getDate())}T${pad(parsed.getHours())}:${pad(parsed.getMinutes())}:${pad(parsed.getSeconds())}${sign}${offsetHours}:${offsetMins}`;
  }

  function normalizeOptionalDatetimeLocal(value: string): string | undefined {
    if (!value) return undefined;
    return toRfc3339Local(value);
  }

  function formatFieldLabel(name: string): string {
    return name
      .replace(/[_-]+/g, ' ')
      .replace(/([a-z])([A-Z])/g, '$1 $2')
      .replace(/\b\w/g, (char) => char.toUpperCase());
  }

  function buildCustomFieldGroups(config: FrontmatterConfig | null) {
    if (!config || config.customFields.length === 0) {
      return [];
    }

    const fieldMap = new Map(config.customFields.map((field) => [field.name, field]));
    const groupedFields = new Set<string>();
    const groups = (config.fieldGroups || []).map((group) => {
      const fields = group.fields
        .map((fieldName) => fieldMap.get(fieldName))
        .filter((field): field is NonNullable<typeof field> => !!field);
      fields.forEach((field) => groupedFields.add(field.name));
      return {
        name: group.name,
        label: group.label || formatFieldLabel(group.name),
        collapsed: group.collapsed ?? false,
        fields
      };
    });

    const ungrouped = config.customFields.filter((field) => !groupedFields.has(field.name));
    if (ungrouped.length > 0) {
      groups.push({
        name: 'custom',
        label: 'Custom Fields',
        collapsed: false,
        fields: ungrouped
      });
    }

    return groups;
  }

  function getCustomFieldValue(name: string) {
    return post?.frontmatter.customFields?.[name];
  }

  function getCustomFieldString(name: string): string {
    const value = getCustomFieldValue(name);
    return typeof value === 'string' ? value : value == null ? '' : String(value);
  }

  function getCustomFieldInputValue(name: string, fieldType: string): string {
    const value = getCustomFieldValue(name);
    if (value == null) return '';
    if (fieldType === 'datetime') {
      return typeof value === 'string' ? formatDatetimeLocal(value) : String(value);
    }
    if (fieldType === 'object' || fieldType === 'array') {
      if (typeof value === 'string') return value;
      try {
        return JSON.stringify(value, null, 2);
      } catch {
        return String(value);
      }
    }
    return typeof value === 'string' ? value : String(value);
  }

  function parseCustomFieldValue(fieldType: string, rawValue: string) {
    const trimmed = rawValue.trim();
    if (!trimmed) return null;

    if (fieldType === 'number') {
      const parsed = Number(trimmed);
      return Number.isNaN(parsed) ? null : parsed;
    }

    if (fieldType === 'array') {
      if (trimmed.startsWith('[')) {
        try {
          const parsed = JSON.parse(trimmed);
          return Array.isArray(parsed) ? parsed : null;
        } catch {
          return trimmed.split(',').map((entry) => entry.trim()).filter(Boolean);
        }
      }
      return trimmed.split(',').map((entry) => entry.trim()).filter(Boolean);
    }

    if (fieldType === 'object') {
      try {
        const parsed = JSON.parse(trimmed);
        return typeof parsed === 'object' && parsed !== null ? parsed : null;
      } catch {
        return trimmed;
      }
    }

    return rawValue;
  }

  function setCustomFieldValue(name: string, value: unknown) {
    if (!post) return;
    post.frontmatter.customFields = post.frontmatter.customFields || {};
    if (value === '' || value === null || value === undefined) {
      delete post.frontmatter.customFields[name];
    } else {
      post.frontmatter.customFields[name] = value;
    }
    handleFrontmatterChange();
  }

  function toggleCustomGroup(name: string) {
    customGroupCollapsed = {
      ...customGroupCollapsed,
      [name]: !customGroupCollapsed[name]
    };
  }

  // Computed
  let wordCount = $derived(markdownContent.trim() ? markdownContent.trim().split(/\s+/).length : 0);
  let characterCount = $derived(markdownContent.length);
  let previewHtml = $derived(renderMarkdown(markdownContent));
  let postTitle = $derived(post?.title || 'Untitled Post');
  let customFieldGroups = $derived(buildCustomFieldGroups(frontmatterConfig));

  // Navigation guard
  let allowNavigation = false;
  let confirmNavigationInFlight = false;
  beforeNavigate((navigation) => {
    if (!hasUnsavedChanges || allowNavigation) {
      allowNavigation = false;
      return;
    }

    navigation.cancel();
    if (confirmNavigationInFlight) return;

    const targetUrl = navigation.to?.url;
    confirmNavigationInFlight = true;
    void (async () => {
      const confirmed = await confirm('You have unsaved changes. Are you sure you want to leave?');
      confirmNavigationInFlight = false;
      if (!confirmed || !targetUrl) return;
      allowNavigation = true;
      await goto(`${targetUrl.pathname}${targetUrl.search}${targetUrl.hash}`);
    })();
  });

  onMount(async () => {
    // Get post ID from URL
    const urlParams = new URLSearchParams(window.location.search);
    const postId = urlParams.get('id');
    const typeParam = urlParams.get('type');
    if (typeParam === 'page' || typeParam === 'draft' || typeParam === 'post') {
      entryType = typeParam;
    }

    if (postId) {
      await loadEntry(postId);
    } else {
      loadError = 'No post ID provided';
      isLoading = false;
    }

    // Setup keyboard shortcuts
    shortcutsCleanup = setupKeyboardShortcuts();

    try {
      frontmatterConfig = await backend.getFrontmatterConfig();
      const collapsedState: Record<string, boolean> = {};
      for (const group of frontmatterConfig.fieldGroups || []) {
        collapsedState[group.name] = group.collapsed ?? false;
      }
      customGroupCollapsed = collapsedState;
    } catch (err) {
      console.error('Failed to load frontmatter config:', err);
    }

    // Setup auto-save (every 30 seconds)
    autoSaveTimer = setInterval(() => {
      if (hasUnsavedChanges && saveStatus === 'unsaved') {
        savePost();
      }
    }, 30000);

  });

  onDestroy(() => {
    if (shortcutsCleanup) {
      shortcutsCleanup();
    }
    if (autoSaveTimer) {
      clearInterval(autoSaveTimer);
    }
  });

  async function loadEntry(postId: string) {
    isLoading = true;
    loadError = null;

    try {
      if (entryType === 'page') {
        post = await backend.getPage(postId);
      } else if (entryType === 'draft') {
        post = await backend.getDraft(postId);
      } else {
        post = await backend.getPost(postId);
      }
      // Backend returns post.content as just the markdown body (no frontmatter)
      // Frontmatter is stored separately in post.frontmatter
      markdownContent = post.content;
      originalContent = markdownContent;
      post.frontmatter.customFields = post.frontmatter.customFields || {};
      hasUnsavedChanges = false;
      saveStatus = 'saved';
      document.title = `${post.title} - Hugo Bros`;
    } catch (error) {
      console.error('Failed to load entry:', error);
      loadError = error instanceof Error ? error.message : 'Failed to load entry';
    } finally {
      isLoading = false;
    }
  }

  function handleContentChange() {
    hasUnsavedChanges = markdownContent !== originalContent;
    saveStatus = hasUnsavedChanges ? 'unsaved' : 'saved';
  }

  async function savePost() {
    if (!post) return;

    saveStatus = 'saving';
    saveMessage = 'Saving...';

    try {
      // Update post content (just markdown, no frontmatter)
      // Backend handles frontmatter serialization via to_markdown()
      post.content = markdownContent;

      // Sync title between post.title and post.frontmatter.title
      post.frontmatter.title = post.title;

      if (entryType === 'page') {
        await backend.savePage(post as Page);
      } else if (entryType === 'draft') {
        await backend.saveDraft(post as Draft);
      } else {
        await backend.savePost(post as Post);
      }

      originalContent = markdownContent;
      hasUnsavedChanges = false;
      saveStatus = 'saved';
      saveMessage = 'Saved';
      lastSavedAt = new Date();
    } catch (error) {
      console.error('Failed to save entry:', error);
      saveStatus = 'error';
      saveMessage = error instanceof Error ? error.message : 'Failed to save';
    }
  }

  function setupKeyboardShortcuts() {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const modKey = isMac ? e.metaKey : e.ctrlKey;

      if (modKey && e.key === 's') {
        e.preventDefault();
        savePost();
      }

      if (modKey && e.key === 'b') {
        e.preventDefault();
        insertFormatting('bold');
      }

      if (modKey && e.key === 'i') {
        e.preventDefault();
        insertFormatting('italic');
      }

      if (modKey && e.key === 'k') {
        e.preventDefault();
        insertLink();
      }

      if (e.key === 'Escape') {
        if (showImageGallery) {
          showImageGallery = false;
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }

  // Toolbar functions
  function insertFormatting(type: string) {
    if (!textareaRef) return;

    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const selectedText = markdownContent.substring(start, end);
    let insertion = '';
    let cursorOffset = 0;

    switch (type) {
      case 'bold':
        insertion = `**${selectedText || 'bold text'}**`;
        cursorOffset = selectedText ? insertion.length : 2;
        break;
      case 'italic':
        insertion = `*${selectedText || 'italic text'}*`;
        cursorOffset = selectedText ? insertion.length : 1;
        break;
      case 'strikethrough':
        insertion = `~~${selectedText || 'strikethrough text'}~~`;
        cursorOffset = selectedText ? insertion.length : 2;
        break;
      case 'code':
        insertion = `\`${selectedText || 'code'}\``;
        cursorOffset = selectedText ? insertion.length : 1;
        break;
      case 'h1':
        insertion = `# ${selectedText || 'Heading 1'}`;
        cursorOffset = insertion.length;
        break;
      case 'h2':
        insertion = `## ${selectedText || 'Heading 2'}`;
        cursorOffset = insertion.length;
        break;
      case 'h3':
        insertion = `### ${selectedText || 'Heading 3'}`;
        cursorOffset = insertion.length;
        break;
      case 'ul':
        insertion = `- ${selectedText || 'list item'}`;
        cursorOffset = insertion.length;
        break;
      case 'ol':
        insertion = `1. ${selectedText || 'list item'}`;
        cursorOffset = insertion.length;
        break;
      case 'quote':
        insertion = `> ${selectedText || 'quote'}`;
        cursorOffset = insertion.length;
        break;
      case 'hr':
        insertion = '\n---\n';
        cursorOffset = insertion.length;
        break;
    }

    const before = markdownContent.substring(0, start);
    const after = markdownContent.substring(end);
    markdownContent = before + insertion + after;

    // Restore focus and set cursor position
    setTimeout(() => {
      if (textareaRef) {
        textareaRef.focus();
        const newPos = start + cursorOffset;
        textareaRef.setSelectionRange(newPos, newPos);
      }
    }, 0);

    handleContentChange();
  }

  function insertLink() {
    const url = prompt('Enter URL:');
    if (!url) return;

    if (!textareaRef) return;

    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const selectedText = markdownContent.substring(start, end) || 'link text';

    const insertion = `[${selectedText}](${url})`;
    const before = markdownContent.substring(0, start);
    const after = markdownContent.substring(end);
    markdownContent = before + insertion + after;

    setTimeout(() => {
      if (textareaRef) {
        textareaRef.focus();
        const newPos = start + insertion.length;
        textareaRef.setSelectionRange(newPos, newPos);
      }
    }, 0);

    handleContentChange();
  }

  function openImageGalleryForContent() {
    pendingImageField = { kind: 'content' };
    showImageGallery = true;
  }

  function openImageGalleryForFrontmatter(fieldName: string) {
    pendingImageField = { kind: 'frontmatter', fieldName };
    showImageGallery = true;
  }

  function handleImageSelect(entry: StaticEntry) {
    if (!post) return;
    if (!entry.url) return;

    if (pendingImageField?.kind === 'frontmatter') {
      setCustomFieldValue(pendingImageField.fieldName, entry.url);
    } else if (pendingImageField?.kind === 'content' && textareaRef) {
      const start = textareaRef.selectionStart;
      const insertion = `![${entry.name}](${entry.url})`;
      const before = markdownContent.substring(0, start);
      const after = markdownContent.substring(start);
      markdownContent = before + insertion + after;
      handleContentChange();
    }

    showImageGallery = false;
    pendingImageField = null;
  }

  // Frontmatter functions
  function handleFrontmatterChange() {
    hasUnsavedChanges = true;
    saveStatus = 'unsaved';
  }

  function addTag() {
    if (!post || !newTagName.trim()) return;
    post.frontmatter.tags = [...(post.frontmatter.tags || []), newTagName.trim()];
    newTagName = '';
    handleFrontmatterChange();
  }

  function removeTag(index: number) {
    if (!post) return;
    post.frontmatter.tags = post.frontmatter.tags?.filter((_, i) => i !== index) || [];
    handleFrontmatterChange();
  }

  function addCategory() {
    if (!post || !newCategoryName.trim()) return;
    post.frontmatter.categories = [...(post.frontmatter.categories || []), newCategoryName.trim()];
    newCategoryName = '';
    handleFrontmatterChange();
  }

  function removeCategory(index: number) {
    if (!post) return;
    post.frontmatter.categories = post.frontmatter.categories?.filter((_, i) => i !== index) || [];
    handleFrontmatterChange();
  }

  function renderMarkdown(md: string): string {
    try {
      marked.setOptions({
        breaks: true,
        gfm: true,
      });
      let html = marked.parse(md) as string;

      // Convert project-relative image URLs to Tauri asset URLs
      const projectPath = backend.getProjectPath();
      if (projectPath) {
        // Replace src="/..."" with Tauri asset URLs
        html = html.replace(/src="(\/[^"]+)"/g, (match, relativePath) => {
          const decodedPath = decodeURIComponent(relativePath);
          const fullPath = `${projectPath}/static${decodedPath}`;
          return `src="${convertFileSrc(fullPath)}"`;
        });
      }

      return html;
    } catch (error) {
      console.error('Failed to parse markdown:', error);
      return '<p class="error">Failed to render preview</p>';
    }
  }

  function goBack() {
    goto('/posts');
  }

  // Panel resizing
  function startResize(e: MouseEvent) {
    isResizing = true;
    e.preventDefault();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isResizing) return;
    const container = (e.target as HTMLElement).closest('.editor-panels');
    if (!container) return;

    const rect = container.getBoundingClientRect();
    const newWidth = ((e.clientX - rect.left) / rect.width) * 100;
    editorWidth = Math.min(Math.max(newWidth, 25), 75);
  }

  function stopResize() {
    isResizing = false;
  }

  function formatLastSaved(): string {
    if (!lastSavedAt) return '';
    const now = new Date();
    const diff = Math.floor((now.getTime() - lastSavedAt.getTime()) / 1000);
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    return lastSavedAt.toLocaleTimeString();
  }
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={stopResize} />

<div class="editor-page">
  <!-- Header -->
  <header class="editor-header">
    <div class="header-left">
      <button onclick={goBack} class="back-btn" title="Back to posts" type="button">
        <ArrowLeft size={20} />
      </button>
      <h1 class="post-title">{postTitle}</h1>
    </div>

    <div class="header-center">
      <div class="toolbar">
        <div class="toolbar-group">
          <button onclick={() => insertFormatting('h1')} class="toolbar-btn" title="Heading 1" type="button">
            <Heading1 size={18} />
          </button>
          <button onclick={() => insertFormatting('h2')} class="toolbar-btn" title="Heading 2" type="button">
            <Heading2 size={18} />
          </button>
          <button onclick={() => insertFormatting('h3')} class="toolbar-btn" title="Heading 3" type="button">
            <Heading3 size={18} />
          </button>
        </div>

        <div class="toolbar-divider"></div>

        <div class="toolbar-group">
          <button onclick={() => insertFormatting('bold')} class="toolbar-btn" title="Bold (Ctrl+B)" type="button">
            <Bold size={18} />
          </button>
          <button onclick={() => insertFormatting('italic')} class="toolbar-btn" title="Italic (Ctrl+I)" type="button">
            <Italic size={18} />
          </button>
          <button onclick={() => insertFormatting('strikethrough')} class="toolbar-btn" title="Strikethrough" type="button">
            <Strikethrough size={18} />
          </button>
          <button onclick={() => insertFormatting('code')} class="toolbar-btn" title="Inline Code" type="button">
            <Code size={18} />
          </button>
        </div>

        <div class="toolbar-divider"></div>

        <div class="toolbar-group">
          <button onclick={() => insertFormatting('ul')} class="toolbar-btn" title="Bullet List" type="button">
            <List size={18} />
          </button>
          <button onclick={() => insertFormatting('ol')} class="toolbar-btn" title="Numbered List" type="button">
            <ListOrdered size={18} />
          </button>
          <button onclick={() => insertFormatting('quote')} class="toolbar-btn" title="Blockquote" type="button">
            <Quote size={18} />
          </button>
          <button onclick={() => insertFormatting('hr')} class="toolbar-btn" title="Horizontal Rule" type="button">
            <Minus size={18} />
          </button>
        </div>

        <div class="toolbar-divider"></div>

        <div class="toolbar-group">
          <button onclick={insertLink} class="toolbar-btn" title="Insert Link (Ctrl+K)" type="button">
            <LinkIcon size={18} />
          </button>
          <button onclick={openImageGalleryForContent} class="toolbar-btn" title="Insert Image" type="button">
            <ImageIcon size={18} />
          </button>
        </div>
      </div>
    </div>

    <div class="header-right">
      <button
        onclick={() => (showFrontmatter = !showFrontmatter)}
        class="toggle-btn"
        class:active={showFrontmatter}
        title="Toggle Frontmatter Panel"
        type="button"
      >
        <Settings size={18} />
        <span>Meta</span>
      </button>

      <button
        onclick={() => (showPreview = !showPreview)}
        class="toggle-btn"
        class:active={showPreview}
        title="Toggle Preview"
        type="button"
      >
        {#if showPreview}
          <EyeOff size={18} />
        {:else}
          <Eye size={18} />
        {/if}
        <span>Preview</span>
      </button>

      <div class="save-status">
        {#if saveStatus === 'saving'}
          <Loader2 size={16} class="spin" />
          <span>Saving...</span>
        {:else if saveStatus === 'saved'}
          <CheckCircle size={16} class="text-green" />
          <span>{lastSavedAt ? formatLastSaved() : 'Saved'}</span>
        {:else if saveStatus === 'error'}
          <AlertCircle size={16} class="text-red" />
          <span>{saveMessage}</span>
        {:else if saveStatus === 'unsaved'}
          <span class="text-orange">Unsaved changes</span>
        {/if}
      </div>

      <button onclick={savePost} class="save-btn" title="Save (Ctrl+S)" type="button">
        <Save size={18} />
        <span>Save</span>
      </button>
    </div>
  </header>

  <!-- Main Content -->
  {#if isLoading}
    <div class="loading-state">
      <Loader2 size={32} class="spin" />
      <p>Loading post...</p>
    </div>
  {:else if loadError}
    <div class="error-state">
      <AlertCircle size={32} />
      <p>{loadError}</p>
      <button onclick={goBack} class="back-link">Back to posts</button>
    </div>
  {:else if post}
    <div class="editor-main">
      <!-- Frontmatter Sidebar -->
      {#if showFrontmatter}
        <aside class="frontmatter-sidebar">
          <div class="sidebar-header">
            <h2>Post Settings</h2>
            <button onclick={() => (showFrontmatter = false)} class="close-btn" type="button">
              <X size={18} />
            </button>
          </div>

          <div class="sidebar-content">
            <!-- Title -->
            <div class="field-group">
              <label for="title">Title</label>
              <input
                id="title"
                type="text"
                bind:value={post.title}
                oninput={handleFrontmatterChange}
                placeholder="Post title"
              />
            </div>

            <!-- Date -->
            <div class="field-group">
              <label for="date">Date</label>
              <input
                id="date"
                type="datetime-local"
                value={formatDatetimeLocal(post.frontmatter.date)}
                oninput={(e) => {
                  if (post) {
                    post.frontmatter.date = toRfc3339Local((e.target as HTMLInputElement).value);
                    handleFrontmatterChange();
                  }
                }}
              />
            </div>

            <!-- Updated -->
            <div class="field-group">
              <label for="updated">Updated</label>
              <input
                id="updated"
                type="datetime-local"
                value={formatDatetimeLocal(post.frontmatter.updated)}
                oninput={(e) => {
                  if (post) {
                    post.frontmatter.updated = normalizeOptionalDatetimeLocal(
                      (e.target as HTMLInputElement).value
                    );
                    handleFrontmatterChange();
                  }
                }}
              />
            </div>

            <!-- Description -->
            <div class="field-group">
              <label for="description">Description</label>
              <textarea
                id="description"
                bind:value={post.frontmatter.description}
                oninput={handleFrontmatterChange}
                placeholder="Brief description for SEO"
                rows="3"
              ></textarea>
            </div>

            <!-- Tags -->
            <div class="field-group">
              <span class="field-label">Tags</span>
              <div class="chips-list">
                {#each post.frontmatter.tags || [] as tag, index}
                  <span class="chip tag-chip">
                    #{tag}
                    <button onclick={() => removeTag(index)} type="button" class="chip-remove">
                      <X size={12} />
                    </button>
                  </span>
                {/each}
              </div>
              <div class="add-row">
                <input
                  type="text"
                  bind:value={newTagName}
                  placeholder="Add tag..."
                  onkeypress={(e) => e.key === 'Enter' && (e.preventDefault(), addTag())}
                />
                <button onclick={addTag} type="button" class="add-btn">
                  <Plus size={16} />
                </button>
              </div>
            </div>

            <!-- Categories -->
            <div class="field-group">
              <span class="field-label">Categories</span>
              <div class="chips-list">
                {#each post.frontmatter.categories || [] as category, index}
                  <span class="chip category-chip">
                    {category}
                    <button onclick={() => removeCategory(index)} type="button" class="chip-remove">
                      <X size={12} />
                    </button>
                  </span>
                {/each}
              </div>
              <div class="add-row">
                <input
                  type="text"
                  bind:value={newCategoryName}
                  placeholder="Add category..."
                  onkeypress={(e) => e.key === 'Enter' && (e.preventDefault(), addCategory())}
                />
                <button onclick={addCategory} type="button" class="add-btn">
                  <Plus size={16} />
                </button>
              </div>
            </div>

            <!-- Permalink -->
            <div class="field-group">
              <label for="permalink">Permalink</label>
              <input
                id="permalink"
                type="text"
                bind:value={post.frontmatter.permalink}
                oninput={handleFrontmatterChange}
                placeholder="/custom/path"
              />
            </div>

            <!-- Layout -->
            <div class="field-group">
              <label for="layout">Layout</label>
              <input
                id="layout"
                type="text"
                bind:value={post.frontmatter.layout}
                oninput={handleFrontmatterChange}
                placeholder="post"
              />
            </div>

            <!-- Comments -->
            <div class="field-group checkbox-field">
              <label class="checkbox-label" for="comments">
                <input
                  id="comments"
                  type="checkbox"
                  checked={post.frontmatter.comments ?? false}
                  onchange={(e) => {
                    if (post) {
                      const target = e.target as HTMLInputElement;
                      post.frontmatter.comments = target.checked;
                      handleFrontmatterChange();
                    }
                  }}
                />
                <span>Allow comments</span>
              </label>
            </div>

            {#if customFieldGroups.length > 0}
              <div class="custom-fields">
                {#each customFieldGroups as group (group.name)}
                  <div class="custom-field-group">
                    <div
                      class="custom-field-group-header"
                      role="button"
                      tabindex="0"
                      onclick={() => toggleCustomGroup(group.name)}
                      onkeydown={(e) => {
                        if (e.key === 'Enter' || e.key === ' ') {
                          e.preventDefault();
                          toggleCustomGroup(group.name);
                        }
                      }}
                    >
                      <span>{group.label}</span>
                      <button class="group-toggle" type="button" tabindex="-1">
                        {#if customGroupCollapsed[group.name] ?? group.collapsed}
                          <ChevronDown size={16} />
                        {:else}
                          <ChevronUp size={16} />
                        {/if}
                      </button>
                    </div>
                    {#if !(customGroupCollapsed[group.name] ?? group.collapsed)}
                      <div class="custom-field-group-body">
                        {#each group.fields as field (field.name)}
                          {#if field.type === 'boolean'}
                            <div class="field-group checkbox-field">
                              <label class="checkbox-label" for={`custom-${field.name}`}>
                                <input
                                  id={`custom-${field.name}`}
                                  type="checkbox"
                                  checked={Boolean(getCustomFieldValue(field.name))}
                                  onchange={(e) =>
                                    setCustomFieldValue(
                                      field.name,
                                      (e.target as HTMLInputElement).checked
                                    )}
                                />
                                <span>{field.label || formatFieldLabel(field.name)}</span>
                              </label>
                              {#if field.description}
                                <span class="field-hint">{field.description}</span>
                              {/if}
                            </div>
                          {:else}
                            <div class="field-group">
                              <label for={`custom-${field.name}`}>
                                {field.label || formatFieldLabel(field.name)}
                              </label>
                              {#if field.type === 'text'}
                                <textarea
                                  id={`custom-${field.name}`}
                                  rows={field.ui?.rows || 3}
                                  placeholder={field.ui?.placeholder || ''}
                                  value={getCustomFieldInputValue(field.name, field.type)}
                                  oninput={(e) =>
                                    setCustomFieldValue(
                                      field.name,
                                      (e.target as HTMLTextAreaElement).value
                                    )}
                                ></textarea>
                              {:else if field.type === 'number'}
                                <input
                                  id={`custom-${field.name}`}
                                  type="number"
                                  placeholder={field.ui?.placeholder || ''}
                                  value={getCustomFieldInputValue(field.name, field.type)}
                                  oninput={(e) =>
                                    setCustomFieldValue(
                                      field.name,
                                      parseCustomFieldValue(
                                        field.type,
                                        (e.target as HTMLInputElement).value
                                      )
                                    )}
                                />
                              {:else if field.type === 'date' || field.type === 'datetime'}
                                <input
                                  id={`custom-${field.name}`}
                                  type={field.type === 'date' ? 'date' : 'datetime-local'}
                                  placeholder={field.ui?.placeholder || ''}
                                  value={getCustomFieldInputValue(field.name, field.type)}
                                  oninput={(e) =>
                                    setCustomFieldValue(
                                      field.name,
                                      field.type === 'date'
                                        ? (e.target as HTMLInputElement).value || null
                                        : normalizeOptionalDatetimeLocal(
                                            (e.target as HTMLInputElement).value
                                          )
                                    )}
                                />
                              {:else if field.type === 'image'}
                                <div class="image-field">
                                  <input
                                    id={`custom-${field.name}`}
                                    type="text"
                                    placeholder={field.ui?.placeholder || '/images/cover.jpg'}
                                    value={getCustomFieldString(field.name)}
                                    oninput={(e) =>
                                      setCustomFieldValue(
                                        field.name,
                                        (e.target as HTMLInputElement).value
                                      )}
                                  />
                                  <button
                                    onclick={() => openImageGalleryForFrontmatter(field.name)}
                                    type="button"
                                    class="image-btn"
                                  >
                                    <ImageIcon size={16} />
                                  </button>
                                </div>
                                {#if getCustomFieldString(field.name)}
                                  <div class="image-preview">
                                    <img
                                      src={getPreviewImageSrc(getCustomFieldString(field.name))}
                                      alt={`${field.label || formatFieldLabel(field.name)} preview`}
                                    />
                                  </div>
                                {/if}
                              {:else if field.type === 'array' || field.type === 'object'}
                                <textarea
                                  id={`custom-${field.name}`}
                                  rows={field.ui?.rows || 3}
                                  placeholder={field.ui?.placeholder || ''}
                                  value={getCustomFieldInputValue(field.name, field.type)}
                                  oninput={(e) =>
                                    setCustomFieldValue(
                                      field.name,
                                      parseCustomFieldValue(
                                        field.type,
                                        (e.target as HTMLTextAreaElement).value
                                      )
                                    )}
                                ></textarea>
                              {:else}
                                <input
                                  id={`custom-${field.name}`}
                                  type="text"
                                  placeholder={field.ui?.placeholder || ''}
                                  value={getCustomFieldInputValue(field.name, field.type)}
                                  oninput={(e) =>
                                    setCustomFieldValue(
                                      field.name,
                                      (e.target as HTMLInputElement).value
                                    )}
                                />
                              {/if}
                              {#if field.description}
                                <span class="field-hint">{field.description}</span>
                              {/if}
                            </div>
                          {/if}
                        {/each}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </aside>
      {/if}

      <!-- Editor Panels -->
      <div class="editor-panels" class:full-width={!showFrontmatter}>
        <!-- Markdown Editor -->
        <div class="editor-pane" style="width: {showPreview ? editorWidth : 100}%">
          <div class="pane-header">
            <FileText size={16} />
            <span>Markdown</span>
            <span class="word-count">{wordCount} words · {characterCount} chars</span>
          </div>
          <textarea
            bind:this={textareaRef}
            bind:value={markdownContent}
            oninput={handleContentChange}
            class="markdown-editor"
            placeholder="Write your content in Markdown..."
            spellcheck="true"
          ></textarea>
        </div>

        <!-- Resize Handle -->
        {#if showPreview}
          <button
            type="button"
            class="resize-handle"
            onmousedown={startResize}
            class:resizing={isResizing}
            aria-label="Resize editor panels"
          >
            <GripVertical size={16} />
          </button>
        {/if}

        <!-- Preview Pane -->
        {#if showPreview}
          <div class="preview-pane" style="width: {100 - editorWidth}%">
            <div class="pane-header">
              <Eye size={16} />
              <span>Preview</span>
            </div>
            <div class="markdown-preview">
              {@html previewHtml}
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Image Gallery Modal -->
  <ImageGallery
    bind:open={showImageGallery}
    onSelect={handleImageSelect}
  />
</div>

<style>
  .editor-page {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #fafafa;
  }

  :global(.dark .editor-page) {
    background-color: #1a1a1a;
  }

  /* Header */
  .editor-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 1rem;
    background-color: #ffffff;
    border-bottom: 1px solid #e5e5e5;
    gap: 1rem;
    flex-wrap: wrap;
  }

  :global(.dark .editor-header) {
    background-color: #2d2d2d;
    border-bottom-color: #404040;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 0;
  }

  .back-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background-color: transparent;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    color: #1a1a1a;
    cursor: pointer;
    transition: all 0.15s ease;
    flex-shrink: 0;
  }

  :global(.dark .back-btn) {
    border-color: #404040;
    color: #f5f5f5;
  }

  .back-btn:hover {
    background-color: #f7f7f7;
  }

  :global(.dark .back-btn:hover) {
    background-color: #404040;
  }

  .post-title {
    font-size: 1rem;
    font-weight: 600;
    color: #1a1a1a;
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  :global(.dark .post-title) {
    color: #f5f5f5;
  }
  /* Toolbar */
  .header-center {
    flex: 1;
    display: flex;
    justify-content: center;
    min-width: 0;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem;
    background-color: #f7f7f7;
    border-radius: 0.5rem;
    flex-wrap: wrap;
  }

  :global(.dark .toolbar) {
    background-color: #404040;
  }

  .toolbar-group {
    display: flex;
    align-items: center;
    gap: 0.125rem;
  }

  .toolbar-divider {
    width: 1px;
    height: 24px;
    background-color: #e5e5e5;
    margin: 0 0.375rem;
  }

  :global(.dark .toolbar-divider) {
    background-color: #525252;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    background-color: transparent;
    border: none;
    border-radius: 0.25rem;
    color: #525252;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  :global(.dark .toolbar-btn) {
    color: #a3a3a3;
  }

  .toolbar-btn:hover {
    background-color: #e5e5e5;
    color: #1a1a1a;
  }

  :global(.dark .toolbar-btn:hover) {
    background-color: #525252;
    color: #f5f5f5;
  }

  /* Header Right */
  .header-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
    background-color: transparent;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    color: #525252;
    cursor: pointer;
    font-size: 0.8125rem;
    transition: all 0.15s ease;
  }

  :global(.dark .toggle-btn) {
    border-color: #404040;
    color: #a3a3a3;
  }

  .toggle-btn:hover {
    background-color: #f7f7f7;
  }

  :global(.dark .toggle-btn:hover) {
    background-color: #404040;
  }

  .toggle-btn.active {
    background-color: #eff6ff;
    border-color: #3b82f6;
    color: #3b82f6;
  }

  :global(.dark .toggle-btn.active) {
    background-color: #1e3a8a;
    border-color: #3b82f6;
    color: #93c5fd;
  }

  .save-status {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.8125rem;
    color: #666666;
    padding: 0 0.5rem;
  }

  :global(.dark .save-status) {
    color: #a3a3a3;
  }

  .save-status :global(.spin) {
    animation: spin 1s linear infinite;
  }

  .save-status :global(.text-green) {
    color: #22c55e;
  }

  .save-status :global(.text-red) {
    color: #ef4444;
  }

  .save-status .text-orange {
    color: #f59e0b;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .save-btn {
    display: flex;
    align-items: center;
    gap: 0.375rem;
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

  .save-btn:hover {
    background-color: #2563eb;
  }

  /* Main Content */
  .editor-main {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  /* Loading/Error States */
  .loading-state,
  .error-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    color: #666666;
  }

  :global(.dark .loading-state),
  :global(.dark .error-state) {
    color: #a3a3a3;
  }

  .loading-state :global(.spin) {
    animation: spin 1s linear infinite;
  }

  .error-state {
    color: #ef4444;
  }

  .back-link {
    color: #3b82f6;
    background: none;
    border: none;
    cursor: pointer;
    text-decoration: underline;
  }

  /* Frontmatter Sidebar */
  .frontmatter-sidebar {
    width: 320px;
    background-color: #ffffff;
    border-right: 1px solid #e5e5e5;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex-shrink: 0;
  }

  :global(.dark .frontmatter-sidebar) {
    background-color: #2d2d2d;
    border-right-color: #404040;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #e5e5e5;
  }

  :global(.dark .sidebar-header) {
    border-bottom-color: #404040;
  }

  .sidebar-header h2 {
    font-size: 0.875rem;
    font-weight: 600;
    color: #1a1a1a;
    margin: 0;
  }

  :global(.dark .sidebar-header h2) {
    color: #f5f5f5;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.25rem;
    background: none;
    border: none;
    color: #666666;
    cursor: pointer;
    border-radius: 0.25rem;
  }

  .close-btn:hover {
    background-color: #f7f7f7;
  }

  :global(.dark .close-btn) {
    color: #a3a3a3;
  }

  :global(.dark .close-btn:hover) {
    background-color: #404040;
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .field-group label {
    font-size: 0.75rem;
    font-weight: 600;
    color: #525252;
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  :global(.dark .field-group label) {
    color: #a3a3a3;
  }

  .field-group input,
  .field-group textarea {
    padding: 0.5rem 0.75rem;
    background-color: #f7f7f7;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: #1a1a1a;
    transition: all 0.15s ease;
  }

  :global(.dark .field-group input),
  :global(.dark .field-group textarea) {
    background-color: #404040;
    border-color: #525252;
    color: #f5f5f5;
  }

  .field-group input:focus,
  .field-group textarea:focus {
    outline: none;
    border-color: #3b82f6;
    background-color: #ffffff;
  }

  :global(.dark .field-group input:focus),
  :global(.dark .field-group textarea:focus) {
    background-color: #2d2d2d;
  }

  .field-group textarea {
    resize: vertical;
    min-height: 60px;
    font-family: inherit;
  }

  .field-hint {
    font-size: 0.75rem;
    color: #737373;
  }

  :global(.dark .field-hint) {
    color: #a3a3a3;
  }

  .checkbox-field {
    gap: 0.5rem;
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

  .checkbox-label input {
    width: auto;
  }

  .custom-fields {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .custom-field-group {
    border: 1px solid #e5e5e5;
    border-radius: 0.5rem;
    overflow: hidden;
    background-color: #fafafa;
  }

  :global(.dark .custom-field-group) {
    background-color: #333333;
    border-color: #525252;
  }

  .custom-field-group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #525252;
  }

  :global(.dark .custom-field-group-header) {
    color: #a3a3a3;
  }

  .group-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
  }

  .custom-field-group-body {
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  /* Chips */
  .chips-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.375rem 0.25rem 0.5rem;
    font-size: 0.75rem;
    border-radius: 9999px;
  }

  .tag-chip {
    background-color: #dbeafe;
    color: #1d4ed8;
  }

  :global(.dark .tag-chip) {
    background-color: #1e3a8a;
    color: #93c5fd;
  }

  .category-chip {
    background-color: #dcfce7;
    color: #166534;
  }

  :global(.dark .category-chip) {
    background-color: #14532d;
    color: #86efac;
  }

  .chip-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.125rem;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    opacity: 0.7;
    border-radius: 9999px;
  }

  .chip-remove:hover {
    opacity: 1;
    background-color: rgba(0, 0, 0, 0.1);
  }

  .add-row {
    display: flex;
    gap: 0.375rem;
  }

  .add-row input {
    flex: 1;
  }

  .add-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
    flex-shrink: 0;
  }

  .add-btn:hover {
    background-color: #2563eb;
  }

  /* Image Fields */
  .image-field {
    display: flex;
    gap: 0.375rem;
  }

  .image-field input {
    flex: 1;
  }

  .image-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    padding: 0;
    background-color: #f7f7f7;
    border: 1px solid #e5e5e5;
    border-radius: 0.375rem;
    color: #666666;
    cursor: pointer;
    flex-shrink: 0;
  }

  :global(.dark .image-btn) {
    background-color: #404040;
    border-color: #525252;
    color: #a3a3a3;
  }

  .image-btn:hover {
    background-color: #e5e5e5;
  }

  :global(.dark .image-btn:hover) {
    background-color: #525252;
  }

  .image-preview {
    margin-top: 0.5rem;
    border-radius: 0.375rem;
    overflow: hidden;
    max-height: 120px;
  }

  .image-preview img {
    width: 100%;
    height: auto;
    object-fit: cover;
    max-height: 120px;
  }

  /* Editor Panels */
  .editor-panels {
    flex: 1;
    display: flex;
    overflow: hidden;
    background-color: #ffffff;
  }

  :global(.dark .editor-panels) {
    background-color: #2d2d2d;
  }

  .editor-pane,
  .preview-pane {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

  .pane-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background-color: #f7f7f7;
    border-bottom: 1px solid #e5e5e5;
    font-size: 0.75rem;
    font-weight: 600;
    color: #666666;
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  :global(.dark .pane-header) {
    background-color: #1a1a1a;
    border-bottom-color: #404040;
    color: #a3a3a3;
  }

  .word-count {
    margin-left: auto;
    font-weight: 400;
    text-transform: none;
  }

  /* Markdown Editor */
  .markdown-editor {
    flex: 1;
    width: 100%;
    padding: 1.5rem;
    background-color: #ffffff;
    border: none;
    font-family: 'JetBrains Mono', 'SF Mono', 'Fira Code', monospace;
    font-size: 0.9375rem;
    line-height: 1.75;
    color: #1a1a1a;
    resize: none;
  }

  :global(.dark .markdown-editor) {
    background-color: #2d2d2d;
    color: #f5f5f5;
  }

  .markdown-editor:focus {
    outline: none;
  }

  .markdown-editor::placeholder {
    color: #a3a3a3;
  }

  /* Resize Handle */
  .resize-handle {
    width: 8px;
    background-color: #e5e5e5;
    cursor: col-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #a3a3a3;
    transition: all 0.15s ease;
    flex-shrink: 0;
  }

  :global(.dark .resize-handle) {
    background-color: #404040;
    color: #666666;
  }

  .resize-handle:hover,
  .resize-handle.resizing {
    background-color: #3b82f6;
    color: white;
  }

  /* Preview Pane */
  .preview-pane {
    border-left: 1px solid #e5e5e5;
  }

  :global(.dark .preview-pane) {
    border-left-color: #404040;
  }

  .markdown-preview {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
    background-color: #ffffff;
    color: #1a1a1a;
    font-size: 1rem;
    line-height: 1.75;
  }

  :global(.dark .markdown-preview) {
    background-color: #2d2d2d;
    color: #f5f5f5;
  }

  /* Markdown Preview Styling */
  .markdown-preview :global(h1) {
    font-size: 2rem;
    font-weight: 700;
    margin-top: 1.5rem;
    margin-bottom: 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #e5e5e5;
  }

  :global(.dark .markdown-preview h1) {
    border-bottom-color: #404040;
  }

  .markdown-preview :global(h2) {
    font-size: 1.5rem;
    font-weight: 600;
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
  }

  .markdown-preview :global(h3) {
    font-size: 1.25rem;
    font-weight: 600;
    margin-top: 1.25rem;
    margin-bottom: 0.5rem;
  }

  .markdown-preview :global(p) {
    margin-bottom: 1rem;
  }

  .markdown-preview :global(a) {
    color: #3b82f6;
    text-decoration: underline;
  }

  .markdown-preview :global(strong) {
    font-weight: 700;
  }

  .markdown-preview :global(em) {
    font-style: italic;
  }

  .markdown-preview :global(code) {
    background-color: #f3f4f6;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.875em;
  }

  :global(.dark .markdown-preview code) {
    background-color: #404040;
  }

  .markdown-preview :global(pre) {
    background-color: #1f2937;
    color: #f9fafb;
    padding: 1rem;
    border-radius: 0.5rem;
    overflow-x: auto;
    margin: 1rem 0;
  }

  .markdown-preview :global(pre code) {
    background-color: transparent;
    padding: 0;
  }

  .markdown-preview :global(blockquote) {
    border-left: 4px solid #3b82f6;
    padding-left: 1rem;
    margin: 1rem 0;
    color: #6b7280;
    font-style: italic;
  }

  :global(.dark .markdown-preview blockquote) {
    color: #a3a3a3;
  }

  .markdown-preview :global(ul),
  .markdown-preview :global(ol) {
    padding-left: 1.5rem;
    margin: 0.5rem 0 1rem;
  }

  .markdown-preview :global(li) {
    margin: 0.25rem 0;
  }

  .markdown-preview :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: 0.5rem;
    margin: 1rem 0;
  }

  .markdown-preview :global(hr) {
    border: none;
    border-top: 1px solid #e5e5e5;
    margin: 2rem 0;
  }

  :global(.dark .markdown-preview hr) {
    border-top-color: #404040;
  }
</style>

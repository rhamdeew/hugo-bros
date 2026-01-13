<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { ArrowLeft, Save, Settings } from 'lucide-svelte';
  import TipTapEditor from '$lib/components/TipTapEditor.svelte';
  import EditorToolbar from '$lib/components/EditorToolbar.svelte';
  import WordCounter from '$lib/components/WordCounter.svelte';
  import SaveIndicator from '$lib/components/SaveIndicator.svelte';
  import FrontmatterEditor from '$lib/components/FrontmatterEditor.svelte';
  import ImageGallery from '$lib/components/ImageGallery.svelte';
  import { backend } from '$lib/services/backend';
  import type { Post, ImageInfo } from '$lib/types';

  let editor: any;
  let post = $state<Post | null>(null);
  let showFrontmatter = $state(false);
  let showImageGallery = $state(false);
  let saveStatus = $state<'saved' | 'saving' | 'unsaved' | 'error'>('saved');
  let saveMessage = $state('');
  let wordCount = $state(0);
  let characterCount = $state(0);
  let autoSaveTimer: ReturnType<typeof setInterval> | null = null;
  let images = $state<ImageInfo[]>([]);
  let pendingImageField = $state<'listImage' | 'mainImage' | null>(null);
  let editingContent = $state('');

  onMount(async () => {
    // Get post ID from URL
    const urlParams = new URLSearchParams(window.location.search);
    const postId = urlParams.get('id');
    if (postId) {
      await loadPost(postId);
    }

    // Setup keyboard shortcuts
    const cleanup = setupKeyboardShortcuts();

    // Load images
    try {
      images = await backend.listImages();
    } catch (err) {
      console.error('Failed to load images:', err);
    }

    // Setup auto-save (every 30 seconds)
    autoSaveTimer = setInterval(() => {
      if (saveStatus === 'unsaved') {
        savePost();
      }
    }, 30000);

    onDestroy(() => {
      cleanup();
      if (autoSaveTimer) {
        clearInterval(autoSaveTimer);
      }
    });
  });

  onDestroy(() => {
    if (autoSaveTimer) {
      clearInterval(autoSaveTimer);
    }
  });

  async function loadPost(postId: string) {
    try {
      post = await backend.getPost(postId);
      editingContent = post.content;
      document.title = `${post.title} - Hex Tool`;
    } catch (error) {
      console.error('Failed to load post:', error);
      showSaveError('Failed to load post');
    }
  }

  function setupKeyboardShortcuts() {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const modKey = isMac ? e.metaKey : e.ctrlKey;

      // Ctrl/Cmd + S - Save
      if (modKey && e.key === 's') {
        e.preventDefault();
        savePost();
      }

      // Ctrl/Cmd + K - Insert Link
      if (modKey && e.key === 'k') {
        e.preventDefault();
        insertLink();
      }

      // Escape - Close modals
      if (e.key === 'Escape') {
        if (showImageGallery) {
          showImageGallery = false;
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }

  function handleContentUpdate() {
    saveStatus = 'unsaved';
    updateCounts();
  }

  function updateCounts() {
    if (!editor) return;
    wordCount = editor.getWordCount();
    characterCount = editor.getCharacterCount();
  }

  async function savePost() {
    if (!editor || !post) return;

    saveStatus = 'saving';
    saveMessage = 'Saving...';

    try {
      const content = editor.getContent();
      const markdown = editor.getMarkdown();

      // Update post content
      post.content = markdown;
      editingContent = markdown;

      await backend.savePost(post);

      saveStatus = 'saved';
      saveMessage = 'Saved';
    } catch (error) {
      console.error('Failed to save post:', error);
      saveStatus = 'error';
      saveMessage = 'Failed to save';
    }
  }

  function insertLink() {
    const url = prompt('Enter URL:');
    if (url) {
      editor?.chain().focus().setLink({ href: url }).run();
    }
  }

  function insertImage() {
    pendingImageField = null;
    showImageGallery = true;
  }

  async function handleImageSelect(image: ImageInfo) {
    if (pendingImageField && post) {
      // Update frontmatter
      if (pendingImageField === 'listImage') {
        post.frontmatter.listImage = image.url;
      } else if (pendingImageField === 'mainImage') {
        post.frontmatter.mainImage = image.url;
      }
      savePost();
    } else {
      // Insert into editor
      editor?.chain().focus().setImage({ src: image.url }).run();
    }
  }

  async function handleImageUpload() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/*';

    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      try {
        // In Tauri, file.path should be available
        // @ts-ignore - Tauri-specific property
        const sourcePath = file.path || file.name;
        const imageUrl = await backend.copyImageToProject(sourcePath);
        images = await backend.listImages();

        // Insert into editor or update frontmatter
        if (pendingImageField && post) {
          if (pendingImageField === 'listImage') {
            post.frontmatter.listImage = imageUrl;
          } else if (pendingImageField === 'mainImage') {
            post.frontmatter.mainImage = imageUrl;
          }
        } else {
          editor?.chain().focus().setImage({ src: imageUrl }).run();
        }
      } catch (err) {
        console.error('Failed to upload image:', err);
        alert('Failed to upload image: ' + (err instanceof Error ? err.message : 'Unknown error'));
      }
    };

    input.click();
  }

  function handleImageDelete(image: ImageInfo) {
    backend.deleteImage(image.path).then(() => {
      images = images.filter((img) => img.fullPath !== image.fullPath);
    });
  }

  function toggleFrontmatter() {
    showFrontmatter = !showFrontmatter;
  }

  function goBack() {
    if (saveStatus === 'unsaved') {
      if (confirm('You have unsaved changes. Are you sure you want to leave?')) {
        goto('/posts');
      }
    } else {
      goto('/posts');
    }
  }

  function showSaveError(message: string) {
    saveStatus = 'error';
    saveMessage = message;
  }

  let postTitle = $derived(post?.title || 'New Post');
</script>

<div class="editor-page">
  <!-- Header -->
  <header class="editor-header">
    <div class="header-left">
      <button onclick={goBack} class="back-btn" title="Back to posts" type="button">
        <ArrowLeft size={20} />
      </button>
      <h1 class="post-title">{postTitle}</h1>
    </div>

    <div class="header-right">
      <button
        onclick={toggleFrontmatter}
        class="icon-btn"
        class:active={showFrontmatter}
        title="Toggle frontmatter editor"
        type="button"
      >
        <Settings size={20} />
      </button>
      <SaveIndicator status={saveStatus} message={saveMessage} />
      <button onclick={savePost} class="save-btn" title="Save (Ctrl/Cmd + S)" type="button">
        <Save size={18} />
        <span>Save</span>
      </button>
    </div>
  </header>

  <!-- Main Content -->
  <div class="editor-main">
    <!-- Frontmatter Editor (collapsible) -->
    {#if post && showFrontmatter}
      <div class="frontmatter-section">
        <FrontmatterEditor bind:frontmatter={post.frontmatter} />
      </div>
    {/if}

    <!-- Editor Container -->
    <div class="editor-container">
      <div class="editor-wrapper">
        <!-- Toolbar -->
        <EditorToolbar bind:editor onLink={insertLink} onImage={insertImage} />

        <!-- Editor -->
        <TipTapEditor
          bind:this={editor}
          content={editingContent}
          placeholder="Start writing your post..."
          onUpdate={handleContentUpdate}
        />

        <!-- Footer with Word Counter -->
        <div class="editor-footer">
          <WordCounter words={wordCount} characters={characterCount} />
        </div>
      </div>
    </div>
  </div>

  <!-- Image Gallery Modal -->
  <ImageGallery
    bind:open={showImageGallery}
    {images}
    onSelect={handleImageSelect}
    onUpload={handleImageUpload}
    onDelete={handleImageDelete}
  />
</div>

<style>
  .editor-page {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #ffffff;
  }

  :global(.dark .editor-page) {
    background-color: #1a1a1a;
  }

  /* Header */
  .editor-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #e5e5e5;
    background-color: #ffffff;
  }

  :global(.dark .editor-header) {
    background-color: #2d2d2d;
    border-bottom-color: #404040;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
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
  }

  :global(.dark .back-btn) {
    background-color: transparent;
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
    font-size: 1.125rem;
    font-weight: 600;
    color: #1a1a1a;
  }

  :global(.dark .post-title) {
    color: #f5f5f5;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .icon-btn {
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
  }

  :global(.dark .icon-btn) {
    background-color: transparent;
    border-color: #404040;
    color: #f5f5f5;
  }

  .icon-btn:hover {
    background-color: #f7f7f7;
  }

  :global(.dark .icon-btn:hover) {
    background-color: #404040;
  }

  .icon-btn.active {
    background-color: #eff6ff;
    border-color: #3b82f6;
    color: #3b82f6;
  }

  :global(.dark .icon-btn.active) {
    background-color: #1e3a8a;
    border-color: #3b82f6;
    color: #93c5fd;
  }

  .save-btn {
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

  .save-btn:hover {
    background-color: #2563eb;
  }

  .save-btn:active {
    transform: scale(0.98);
  }

  /* Main Content */
  .editor-main {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .frontmatter-section {
    padding: 0 1rem 1rem 1rem;
  }

  /* Editor Container */
  .editor-container {
    flex: 1;
    min-height: 0;
    padding: 0 1rem 1rem 1rem;
  }

  .editor-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 0.5rem;
    overflow: hidden;
  }

  :global(.dark .editor-wrapper) {
    background-color: #2d2d2d;
    border-color: #404040;
  }

  /* Editor Footer */
  .editor-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 0.5rem 1rem;
    border-top: 1px solid #e5e5e5;
    background-color: #f7f7f7;
  }

  :global(.dark .editor-footer) {
    background-color: #2d2d2d;
    border-top-color: #404040;
  }
</style>

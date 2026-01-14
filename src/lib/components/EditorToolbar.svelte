<script lang="ts">
  import {
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
    Image,
    Undo,
    Redo,
    Eye
  } from 'lucide-svelte';

  export let editor: any;
  export let onLink: () => void = () => {};
  export let onImage: () => void = () => {};
  export let showPreview: boolean = false;
  export let onTogglePreview: () => void = () => {};

  const headingLevels = [
    { label: 'H1', level: 1, icon: Heading1 },
    { label: 'H2', level: 2, icon: Heading2 },
    { label: 'H3', level: 3, icon: Heading3 },
  ];

  function toggleHeading(level: number) {
    editor?.chain().focus().toggleHeading({ level }).run();
  }

  function toggleBold() {
    editor?.chain().focus().toggleBold().run();
  }

  function toggleItalic() {
    editor?.chain().focus().toggleItalic().run();
  }

  function toggleStrike() {
    editor?.chain().focus().toggleStrike().run();
  }

  function toggleCode() {
    editor?.chain().focus().toggleCode().run();
  }

  function toggleBulletList() {
    editor?.chain().focus().toggleBulletList().run();
  }

  function toggleOrderedList() {
    editor?.chain().focus().toggleOrderedList().run();
  }

  function toggleBlockquote() {
    editor?.chain().focus().toggleBlockquote().run();
  }

  function insertHorizontalRule() {
    editor?.chain().focus().setHorizontalRule().run();
  }

  function undo() {
    editor?.chain().focus().undo().run();
  }

  function redo() {
    editor?.chain().focus().redo().run();
  }

  $: isActive = (name: string, attrs = {}) => editor?.isActive(name, attrs) || false;
  $: canUndo = editor?.can?.()?.undo() ?? false;
  $: canRedo = editor?.can?.()?.redo() ?? false;
</script>

<div class="editor-toolbar">
  <div class="toolbar-group">
    <!-- Undo/Redo -->
    <button
      class="toolbar-btn"
      class:disabled={!canUndo}
      onclick={undo}
      title="Undo (Ctrl/Cmd + Z)"
      type="button"
    >
      <Undo size={18} />
    </button>
    <button
      class="toolbar-btn"
      class:disabled={!canRedo}
      onclick={redo}
      title="Redo (Ctrl/Cmd + Shift + Z)"
      type="button"
    >
      <Redo size={18} />
    </button>
  </div>

  <div class="toolbar-divider"></div>

  <div class="toolbar-group">
    <!-- Preview Toggle -->
    <button
      class="toolbar-btn"
      class:active={showPreview}
      onclick={onTogglePreview}
      title="Toggle Markdown Preview"
      type="button"
    >
      <Eye size={18} />
    </button>
  </div>

  <div class="toolbar-divider"></div>

  <div class="toolbar-group">
    <!-- Headings -->
    {#each headingLevels as { label, level, icon: Icon }}
      <button
        class="toolbar-btn"
        class:active={isActive('heading', { level })}
        onclick={() => toggleHeading(level)}
        title="{label}"
        type="button"
      >
        <Icon size={18} />
      </button>
    {/each}
  </div>

  <div class="toolbar-divider"></div>

  <div class="toolbar-group">
    <!-- Text Formatting -->
    <button
      class="toolbar-btn"
      class:active={isActive('bold')}
      onclick={toggleBold}
      title="Bold (Ctrl/Cmd + B)"
      type="button"
    >
      <Bold size={18} />
    </button>
    <button
      class="toolbar-btn"
      class:active={isActive('italic')}
      onclick={toggleItalic}
      title="Italic (Ctrl/Cmd + I)"
      type="button"
    >
      <Italic size={18} />
    </button>
    <button
      class="toolbar-btn"
      class:active={isActive('strike')}
      onclick={toggleStrike}
      title="Strikethrough (Ctrl/Cmd + Shift + X)"
      type="button"
    >
      <Strikethrough size={18} />
    </button>
    <button
      class="toolbar-btn"
      class:active={isActive('code')}
      onclick={toggleCode}
      title="Inline Code (Ctrl/Cmd + Shift + C)"
      type="button"
    >
      <Code size={18} />
    </button>
  </div>

  <div class="toolbar-divider"></div>

  <div class="toolbar-group">
    <!-- Lists -->
    <button
      class="toolbar-btn"
      class:active={isActive('bulletList')}
      onclick={toggleBulletList}
      title="Bullet List"
      type="button"
    >
      <List size={18} />
    </button>
    <button
      class="toolbar-btn"
      class:active={isActive('orderedList')}
      onclick={toggleOrderedList}
      title="Numbered List"
      type="button"
    >
      <ListOrdered size={18} />
    </button>
    <button
      class="toolbar-btn"
      class:active={isActive('blockquote')}
      onclick={toggleBlockquote}
      title="Blockquote"
      type="button"
    >
      <Quote size={18} />
    </button>
    <button
      class="toolbar-btn"
      onclick={insertHorizontalRule}
      title="Horizontal Rule"
      type="button"
    >
      <Minus size={18} />
    </button>
  </div>

  <div class="toolbar-divider"></div>

  <div class="toolbar-group">
    <!-- Insert -->
    <button
      class="toolbar-btn"
      onclick={onLink}
      title="Insert Link (Ctrl/Cmd + K)"
      type="button"
    >
      <LinkIcon size={18} />
    </button>
    <button
      class="toolbar-btn"
      onclick={onImage}
      title="Insert Image"
      type="button"
    >
      <Image size={18} />
    </button>
  </div>
</div>

<style>
  .editor-toolbar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background-color: #f7f7f7;
    border-bottom: 1px solid #e5e5e5;
    border-radius: 0.5rem 0.5rem 0 0;
    flex-wrap: wrap;
  }

  :global(.dark .editor-toolbar) {
    background-color: #2d2d2d;
    border-bottom-color: #404040;
  }

  .toolbar-group {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .toolbar-divider {
    width: 1px;
    height: 24px;
    background-color: #e5e5e5;
    margin: 0 0.5rem;
  }

  :global(.dark .toolbar-divider) {
    background-color: #404040;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background-color: transparent;
    border: 1px solid transparent;
    border-radius: 0.375rem;
    color: #1a1a1a;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  :global(.dark .toolbar-btn) {
    color: #f5f5f5;
  }

  .toolbar-btn:hover:not(.disabled) {
    background-color: #e5e5e5;
  }

  :global(.dark .toolbar-btn:hover:not(.disabled)) {
    background-color: #404040;
  }

  .toolbar-btn.active {
    background-color: #dbeafe;
    color: #1d4ed8;
  }

  :global(.dark .toolbar-btn.active) {
    background-color: #1e40af;
    color: #93c5fd;
  }

  .toolbar-btn.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .toolbar-btn:focus-visible {
    outline: 2px solid #3b82f6;
    outline-offset: 2px;
  }
</style>

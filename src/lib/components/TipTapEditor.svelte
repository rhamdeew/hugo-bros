<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import Placeholder from '@tiptap/extension-placeholder';
  import CharacterCount from '@tiptap/extension-character-count';
  import type { Editor as TipTapEditor } from '@tiptap/core';

  interface Props {
    content?: string;
    editable?: boolean;
    placeholder?: string;
    editorClass?: string;
    onUpdate?: () => void;
  }

  let {
    content = '',
    editable = true,
    placeholder = 'Start writing your post...',
    editorClass = '',
    onUpdate
  }: Props = $props();

  let editorContainer: HTMLElement;
  let editor: TipTapEditor | null = null;
  let isUpdatingFromExternal = false;

  onMount(() => {
    const initialHtml = markdownToHtml(content || '');
    editor = new Editor({
      element: editorContainer,
      extensions: [
        StarterKit.configure({
          codeBlock: false, // We'll use custom code block with syntax highlighting
        }),
        Placeholder.configure({
          placeholder,
          emptyEditorClass: 'is-editor-empty',
        }),
        CharacterCount,
      ],
      content: initialHtml,
      editable,
      editorProps: {
        attributes: {
          class: 'prose prose-sm sm:prose lg:prose-lg dark:prose-invert max-w-none focus:outline-none min-h-[400px] p-4',
        },
      },
      onUpdate: ({ editor: e }) => {
        if (!isUpdatingFromExternal) {
          onUpdate?.();
        }
      },
    });
  });

  // Use $effect to track content changes properly in Svelte 5
  $effect(() => {
    const contentValue = content;
    if (editor && contentValue !== undefined) {
      const htmlContent = markdownToHtml(contentValue || '');
      const currentHTML = editor.getHTML();

      // Only update if content is actually different
      if (currentHTML !== htmlContent) {
        isUpdatingFromExternal = true;
        editor.commands.setContent(htmlContent, { emitUpdate: false });
        // Reset the flag after a brief delay
        Promise.resolve().then(() => {
          isUpdatingFromExternal = false;
        });
      }
    }
  });

  onDestroy(() => {
    if (editor) {
      editor.destroy();
    }
  });

  export function setContent(html: string) {
    editor?.commands.setContent(html);
  }

  export function getContent(): string {
    return editor?.getHTML() || '';
  }

  export function getMarkdown(): string {
    // Simple HTML to Markdown conversion
    if (!editor) return '';
    let html = editor.getHTML();
    return htmlToMarkdown(html);
  }

  export function getWordCount(): number {
    return editor?.storage.characterCount.words() || 0;
  }

  export function getCharacterCount(): number {
    return editor?.storage.characterCount.characters() || 0;
  }

  function htmlToMarkdown(html: string): string {
    let md = html
      .replace(/<h1[^>]*>(.*?)<\/h1>/gi, '# $1\n\n')
      .replace(/<h2[^>]*>(.*?)<\/h2>/gi, '## $1\n\n')
      .replace(/<h3[^>]*>(.*?)<\/h3>/gi, '### $1\n\n')
      .replace(/<h4[^>]*>(.*?)<\/h4>/gi, '#### $1\n\n')
      .replace(/<h5[^>]*>(.*?)<\/h5>/gi, '##### $1\n\n')
      .replace(/<h6[^>]*>(.*?)<\/h6>/gi, '###### $1\n\n')
      .replace(/<strong[^>]*>(.*?)<\/strong>/gi, '**$1**')
      .replace(/<b[^>]*>(.*?)<\/b>/gi, '**$1**')
      .replace(/<em[^>]*>(.*?)<\/em>/gi, '*$1*')
      .replace(/<i[^>]*>(.*?)<\/i>/gi, '*$1*')
      .replace(/<s[^>]*>(.*?)<\/s>/gi, '~~$1~~')
      .replace(/<code[^>]*>(.*?)<\/code>/gi, '`$1`')
      .replace(/<pre[^>]*><code[^>]*>(.*?)<\/code><\/pre>/gis, '```\n$1\n```\n\n')
      .replace(/<a[^>]*href="([^"]*)"[^>]*>(.*?)<\/a>/gi, '[$2]($1)')
      .replace(/<img[^>]*src="([^"]*)"[^>]*alt="([^"]*)"[^>]*>/gi, '![$2]($1)')
      .replace(/<ul[^>]*>([\s\S]*?)<\/ul>/gim, (match, content) => {
        return content.replace(/<li[^>]*>(.*?)<\/li>/gim, '- $1\n') + '\n';
      })
      .replace(/<ol[^>]*>([\s\S]*?)<\/ol>/gim, (match, content) => {
        let index = 1;
        return (
          content.replace(/<li[^>]*>(.*?)<\/li>/gim, () => `${index++}. $1\n`) + '\n'
        );
      })
      .replace(/<p[^>]*>(.*?)<\/p>/gis, '$1\n\n')
      .replace(/<br[^>]*>/gi, '\n')
      .replace(/<blockquote[^>]*>(.*?)<\/blockquote>/gis, '> $1\n\n')
      .replace(/<[^>]+>/g, '')
      .replace(/&nbsp;/gi, ' ')
      .replace(/&amp;/gi, '&')
      .replace(/&lt;/gi, '<')
      .replace(/&gt;/gi, '>')
      .replace(/&quot;/gi, '"')
      .trim();
    return md;
  }

  function markdownToHtml(markdown: string): string {
    // Simple Markdown to HTML conversion
    let html = markdown
      .replace(/^# (.*$)/gim, '<h1>$1</h1>')
      .replace(/^## (.*$)/gim, '<h2>$1</h2>')
      .replace(/^### (.*$)/gim, '<h3>$1</h3>')
      .replace(/^#### (.*$)/gim, '<h4>$1</h4>')
      .replace(/^##### (.*$)/gim, '<h5>$1</h5>')
      .replace(/^###### (.*$)/gim, '<h6>$1</h6>')
      .replace(/\*\*\*(.+?)\*\*\*/g, '<strong><em>$1</em></strong>')
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/\*(.+?)\*/g, '<em>$1</em>')
      .replace(/~~(.+?)~~/g, '<s>$1</s>')
      .replace(/`([^`]+)`/g, '<code>$1</code>')
      .replace(/```([^`]+)```/gs, '<pre><code>$1</code></pre>')
      .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')
      .replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" />')
      .replace(/^\* (.+)$/gim, '<li>$1</li>')
      .replace(/^- (.+)$/gim, '<li>$1</li>')
      .replace(/^\d+\. (.+)$/gim, '<li>$1</li>')
      .replace(/^> (.+)$/gim, '<blockquote>$1</blockquote>')
      .split('\n\n')
      .map((para) => {
        if (!para.match(/^<[h|u|o|l|b|p]/)) {
          return `<p>${para}</p>`;
        }
        return para;
      })
      .join('\n');

    // Wrap consecutive list items in ul/ol tags
    html = html.replace(/(<li>.*<\/li>\n)+/g, (match) => `<ul>${match}</ul>`);

    return html;
  }
</script>

<div class="tiptap-editor {editorClass}">
  <div bind:this={editorContainer} class="tiptap-content"></div>
</div>

<style>
  .tiptap-editor {
    width: 100%;
    height: 100%;
  }

  :global(.tiptap-content) {
    background-color: white;
    border-radius: 0.5rem;
    min-height: 400px;
  }

  :global(.tiptap-content.is-editor-empty:before) {
    color: #adb5bd;
    content: attr(data-placeholder);
    float: left;
    height: 0;
    pointer-events: none;
  }

  /* Dark mode */
  :global(.dark .tiptap-content) {
    background-color: #2d2d2d;
    color: #f5f5f5;
  }

  /* Basic styling for editor content */
  :global(.tiptap-content :is(h1, h2, h3, h4, h5, h6)) {
    font-weight: 600;
    line-height: 1.25;
    margin-top: 1em;
    margin-bottom: 0.5em;
  }

  :global(.tiptap-content h1) {
    font-size: 2em;
  }

  :global(.tiptap-content h2) {
    font-size: 1.5em;
  }

  :global(.tiptap-content h3) {
    font-size: 1.25em;
  }

  :global(.tiptap-content p) {
    margin-top: 0.5em;
    margin-bottom: 0.5em;
  }

  :global(.tiptap-content ul),
  :global(.tiptap-content ol) {
    padding-left: 1.5em;
    margin-top: 0.5em;
    margin-bottom: 0.5em;
  }

  :global(.tiptap-content li) {
    margin-top: 0.25em;
    margin-bottom: 0.25em;
  }

  :global(.tiptap-content code) {
    background-color: #f3f4f6;
    border-radius: 0.25rem;
    padding: 0.125rem 0.25rem;
    font-family: 'JetBrains Mono', 'SF Mono', monospace;
    font-size: 0.875em;
  }

  :global(.dark .tiptap-content code) {
    background-color: #404040;
  }

  :global(.tiptap-content pre) {
    background-color: #1f2937;
    border-radius: 0.5rem;
    color: #f9fafb;
    font-family: 'JetBrains Mono', 'SF Mono', monospace;
    padding: 0.75rem;
    margin-top: 0.5em;
    margin-bottom: 0.5em;
    overflow-x: auto;
  }

  :global(.tiptap-content pre code) {
    background-color: transparent;
    color: inherit;
    padding: 0;
  }

  :global(.tiptap-content blockquote) {
    border-left: 4px solid #3b82f6;
    padding-left: 1rem;
    color: #6b7280;
    font-style: italic;
  }

  :global(.dark .tiptap-content blockquote) {
    color: #a3a3a3;
    border-left-color: #60a5fa;
  }

  :global(.tiptap-content a) {
    color: #3b82f6;
    text-decoration: underline;
  }

  :global(.dark .tiptap-content a) {
    color: #60a5fa;
  }

  /* Selection */
  :global(.tiptap-content ::selection) {
    background-color: #bfdbfe;
  }

  :global(.dark .tiptap-content ::selection) {
    background-color: #1e40af;
  }
</style>

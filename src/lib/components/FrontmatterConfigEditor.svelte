<script lang="ts">
  import { X } from 'lucide-svelte';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { backend } from '$lib/services/backend';
  import type { FrontmatterConfig } from '$lib/types';

  interface Props {
    open: boolean;
    /** Called with the parsed config after a successful save. */
    onSaved?: (config: FrontmatterConfig) => void;
  }

  let { open = $bindable(false), onSaved }: Props = $props();

  const EMPTY_CONFIG = JSON.stringify(
    { version: '1.0', previewImageField: null, customFields: [], fieldGroups: [] },
    null,
    2
  );

  let content = $state('');
  let original = $state('');
  let fileExists = $state(false);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let dirty = $derived(content !== original);

  $effect(() => {
    if (open) {
      load();
    }
  });

  async function load() {
    loading = true;
    error = null;
    try {
      const raw = await backend.getFrontmatterConfigRaw();
      fileExists = raw !== null;
      content = raw ?? EMPTY_CONFIG;
      original = content;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  async function save() {
    if (saving) return;
    error = null;

    try {
      JSON.parse(content);
    } catch (err) {
      error = 'Invalid JSON: ' + (err instanceof Error ? err.message : String(err));
      return;
    }

    saving = true;
    try {
      const config = await backend.saveFrontmatterConfigRaw(content);
      original = content;
      fileExists = true;
      onSaved?.(config);
      open = false;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      saving = false;
    }
  }

  function format() {
    error = null;
    try {
      content = JSON.stringify(JSON.parse(content), null, 2) + '\n';
    } catch (err) {
      error = 'Invalid JSON: ' + (err instanceof Error ? err.message : String(err));
    }
  }

  async function close() {
    if (dirty) {
      const discard = await confirm('Discard unsaved changes to the frontmatter config?', {
        title: 'Hugo Bros',
        kind: 'warning'
      });
      if (!discard) return;
    }
    open = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 's') {
      e.preventDefault();
      save();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      close();
    } else if (e.key === 'Tab' && !e.shiftKey) {
      e.preventDefault();
      const target = e.target as HTMLTextAreaElement;
      target.setRangeText('  ', target.selectionStart, target.selectionEnd, 'end');
      content = target.value;
    }
  }
</script>

{#if open}
  <div
    class="config-overlay"
    onclick={(e) => {
      if (e.currentTarget === e.target) close();
    }}
    onkeydown={(e) => {
      if (e.key === 'Escape') close();
    }}
    role="button"
    tabindex="-1"
    aria-label="Close config editor"
  >
    <div class="config-modal" role="dialog" aria-modal="true" aria-labelledby="config-title">
      <div class="config-header">
        <div>
          <h3 id="config-title">Frontmatter Config</h3>
          <p class="config-path">
            .hugo-bros/frontmatter-config.json{fileExists ? '' : ' (will be created)'}
          </p>
        </div>
        <button class="close-btn" onclick={close} type="button" aria-label="Close">
          <X size={20} />
        </button>
      </div>

      <div class="config-body">
        {#if loading}
          <p class="config-hint">Loading...</p>
        {:else}
          <textarea
            class="config-textarea"
            bind:value={content}
            onkeydown={handleKeydown}
            spellcheck="false"
            autocapitalize="off"
            autocomplete="off"
          ></textarea>
        {/if}
        {#if error}
          <p class="config-error">{error}</p>
        {/if}
      </div>

      <div class="config-footer">
        <button class="config-btn" onclick={format} type="button" disabled={loading}>
          Format
        </button>
        <span class="config-spacer"></span>
        <button class="config-btn" onclick={close} type="button">Cancel</button>
        <button
          class="config-btn primary"
          onclick={save}
          type="button"
          disabled={loading || saving || (fileExists && !dirty)}
        >
          {saving ? 'Saving...' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .config-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1100;
    padding: 1rem;
  }

  .config-modal {
    width: 100%;
    max-width: 760px;
    height: min(80vh, 760px);
    background-color: #ffffff;
    border-radius: 0.75rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  :global(.dark) .config-modal {
    background-color: #2d2d2d;
  }

  .config-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid #e5e7eb;
  }

  :global(.dark) .config-header {
    border-bottom-color: #404040;
  }

  .config-header h3 {
    margin: 0;
    color: #111827;
  }

  :global(.dark) .config-header h3 {
    color: #f5f5f5;
  }

  .config-path {
    margin: 0.25rem 0 0;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.75rem;
    color: #6b7280;
  }

  .close-btn {
    background: none;
    border: none;
    color: #6b7280;
    cursor: pointer;
  }

  :global(.dark) .close-btn {
    color: #d1d5db;
  }

  .config-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 1rem 1.25rem 0.5rem;
  }

  .config-textarea {
    flex: 1;
    width: 100%;
    resize: none;
    padding: 0.75rem;
    border-radius: 0.5rem;
    border: 1px solid #e5e7eb;
    background-color: #fafafa;
    color: #111827;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.8125rem;
    line-height: 1.5;
    tab-size: 2;
    white-space: pre;
    overflow: auto;
  }

  .config-textarea:focus {
    outline: none;
    border-color: #3b82f6;
  }

  :global(.dark) .config-textarea {
    background-color: #1f1f1f;
    border-color: #404040;
    color: #e5e7eb;
  }

  .config-hint {
    color: #6b7280;
    font-size: 0.875rem;
  }

  .config-error {
    margin: 0.5rem 0 0;
    color: #dc2626;
    font-size: 0.85rem;
    white-space: pre-wrap;
  }

  .config-footer {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.25rem 1.25rem;
  }

  .config-spacer {
    flex: 1;
  }

  .config-btn {
    padding: 0.5rem 0.9rem;
    border-radius: 0.5rem;
    border: 1px solid #d1d5db;
    background-color: #ffffff;
    color: #111827;
    cursor: pointer;
  }

  .config-btn.primary {
    background-color: #2563eb;
    border-color: #2563eb;
    color: #ffffff;
  }

  .config-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  :global(.dark) .config-btn:not(.primary) {
    background-color: #1f1f1f;
    border-color: #404040;
    color: #e5e7eb;
  }
</style>

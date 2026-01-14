<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Server, Play, Square, Globe, Package, Trash2, Upload } from 'lucide-svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { confirm, message } from '@tauri-apps/plugin-dialog';
  import { backend } from '$lib/services/backend';
  import type { CommandOutput } from '$lib/types';

  let serverRunning = $state(false);
  let serverId = $state<string | null>(null);
  let loading = $state(false);
  let showCommandOutput = $state(false);
  let commandOutput = $state<CommandOutput | null>(null);
  let checkInterval: number | null = null;

  onMount(() => {
    checkServerStatus();
    // Check server status every 5 seconds
    checkInterval = window.setInterval(checkServerStatus, 5000);
  });

  onDestroy(() => {
    if (checkInterval) {
      clearInterval(checkInterval);
    }
  });

  async function checkServerStatus() {
    try {
      serverRunning = await backend.isHexoServerRunning();
    } catch (err) {
      console.error('Failed to check server status:', err);
    }
  }

  async function startServer() {
    if (loading || serverRunning) return;

    loading = true;
    try {
      serverId = await backend.startHexoServer();
      serverRunning = true;
      await message(
        'Hexo server started successfully!\nAccess your blog at http://localhost:4000',
        { title: 'Hex Tool' }
      );
    } catch (err) {
      console.error('Failed to start server:', err);
      await message(
        'Failed to start server: ' + (err instanceof Error ? err.message : 'Unknown error'),
        { title: 'Hex Tool', kind: 'error' }
      );
    } finally {
      loading = false;
    }
  }

  async function stopServer() {
    if (loading || !serverRunning || !serverId) return;

    loading = true;
    try {
      await backend.stopHexoServer(serverId);
      serverRunning = false;
      serverId = null;
      await message('Hexo server stopped', { title: 'Hex Tool' });
    } catch (err) {
      console.error('Failed to stop server:', err);
      await message(
        'Failed to stop server: ' + (err instanceof Error ? err.message : 'Unknown error'),
        { title: 'Hex Tool', kind: 'error' }
      );
    } finally {
      loading = false;
    }
  }

  async function openInBrowser() {
    if (!serverRunning) {
      await message('Server is not running. Please start the server first.', {
        title: 'Hex Tool',
        kind: 'warning'
      });
      return;
    }
    try {
      await openUrl('http://localhost:4000');
    } catch (err) {
      console.error('Failed to open browser:', err);
      await message(
        'Failed to open browser: ' + (err instanceof Error ? err.message : 'Unknown error'),
        { title: 'Hex Tool', kind: 'error' }
      );
    }
  }

  async function runCommand(command: string, commandName: string) {
    if (loading) return;

    const confirmed = await confirm(`Run "hexo ${command}"?`, {
      title: 'Hex Tool',
      kind: 'warning'
    });
    if (!confirmed) return;

    loading = true;
    showCommandOutput = false;
    commandOutput = null;

    try {
      const output = await backend.runHexoCommand(command);
      commandOutput = {
        ...output,
        stdout: stripAnsi(output.stdout),
        stderr: stripAnsi(output.stderr)
      };
      showCommandOutput = true;

    } catch (err) {
      console.error(`Failed to run ${command}:`, err);
      await message(
        `Failed to run ${command}: ` + (err instanceof Error ? err.message : 'Unknown error'),
        { title: 'Hex Tool', kind: 'error' }
      );
    } finally {
      loading = false;
    }
  }

  function stripAnsi(text: string) {
    return text.replace(/\x1b\[[0-9;]*m/g, '');
  }

  function closeOutput() {
    showCommandOutput = false;
    commandOutput = null;
  }
</script>

<div class="hexo-controls">
  <!-- Server Status Indicator -->
  <div class="status-indicator">
    <div class="status-dot {serverRunning ? 'running' : 'stopped'}"></div>
    <span class="status-text">Server {serverRunning ? 'Running' : 'Stopped'}</span>
  </div>

  <!-- Server Controls -->
  <div class="button-group">
    {#if !serverRunning}
      <button
        class="control-btn primary"
        onclick={startServer}
        disabled={loading}
        type="button"
      >
        <Play size={16} />
        <span>Start Server</span>
      </button>
    {:else}
      <button
        class="control-btn danger"
        onclick={stopServer}
        disabled={loading}
        type="button"
      >
        <Square size={16} />
        <span>Stop Server</span>
      </button>
    {/if}

    <button
      class="control-btn"
      onclick={openInBrowser}
      disabled={!serverRunning}
      type="button"
    >
      <Globe size={16} />
      <span>Open in Browser</span>
    </button>
  </div>

  <!-- Hexo Commands -->
  <div class="button-group">
    <button
      class="control-btn"
      onclick={() => runCommand('generate', 'Generate')}
      disabled={loading}
      type="button"
      title="Generate static files"
    >
      <Package size={16} />
      <span>Generate</span>
    </button>

    <button
      class="control-btn"
      onclick={() => runCommand('clean', 'Clean')}
      disabled={loading}
      type="button"
      title="Clean generated files and cache"
    >
      <Trash2 size={16} />
      <span>Clean</span>
    </button>

    <button
      class="control-btn"
      onclick={() => runCommand('deploy', 'Deploy')}
      disabled={loading}
      type="button"
      title="Deploy to remote"
    >
      <Upload size={16} />
      <span>Deploy</span>
    </button>
  </div>
</div>

<!-- Command Output Modal -->
{#if showCommandOutput && commandOutput}
  <div class="modal-overlay" onclick={closeOutput}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h3>Command Output</h3>
        <button class="close-btn" onclick={closeOutput} type="button">&times;</button>
      </div>
      <div class="modal-body">
        <div class="output-section">
          <h4>Status: {commandOutput.success ? '✅ Success' : '❌ Failed'}</h4>
          <p class="exit-code">Exit Code: {commandOutput.exitCode}</p>
        </div>

        {#if commandOutput.stdout}
          <div class="output-section">
            <h4>Output:</h4>
            <pre class="output-text">{commandOutput.stdout}</pre>
          </div>
        {/if}

        {#if commandOutput.stderr}
          <div class="output-section">
            <h4>Errors:</h4>
            <pre class="output-text error">{commandOutput.stderr}</pre>
          </div>
        {/if}
      </div>
      <div class="modal-footer">
        <button class="control-btn" onclick={closeOutput} type="button">Close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .hexo-controls {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 1rem;
    background-color: #f9fafb;
    border-radius: 0.5rem;
    border: 1px solid #e5e7eb;
  }

  :global(.dark .hexo-controls) {
    background-color: #1f1f1f;
    border-color: #404040;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding-right: 1rem;
    border-right: 1px solid #e5e7eb;
  }

  :global(.dark .status-indicator) {
    border-right-color: #404040;
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    animation: pulse 2s ease-in-out infinite;
  }

  .status-dot.running {
    background-color: #10b981;
    box-shadow: 0 0 8px rgba(16, 185, 129, 0.5);
  }

  .status-dot.stopped {
    background-color: #6b7280;
    animation: none;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .status-text {
    font-size: 0.875rem;
    font-weight: 500;
    color: #374151;
  }

  :global(.dark .status-text) {
    color: #d1d5db;
  }

  .button-group {
    display: flex;
    gap: 0.5rem;
  }

  .control-btn {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.875rem;
    background-color: #ffffff;
    color: #374151;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.813rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  :global(.dark .control-btn) {
    background-color: #2d2d2d;
    color: #d1d5db;
    border-color: #404040;
  }

  .control-btn:hover:not(:disabled) {
    background-color: #f3f4f6;
    border-color: #9ca3af;
  }

  :global(.dark .control-btn:hover:not(:disabled)) {
    background-color: #3d3d3d;
    border-color: #525252;
  }

  .control-btn.primary {
    background-color: #10b981;
    color: white;
    border-color: #10b981;
  }

  .control-btn.primary:hover:not(:disabled) {
    background-color: #059669;
    border-color: #059669;
  }

  .control-btn.danger {
    background-color: #ef4444;
    color: white;
    border-color: #ef4444;
  }

  .control-btn.danger:hover:not(:disabled) {
    background-color: #dc2626;
    border-color: #dc2626;
  }

  .control-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Modal Styles */
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
  }

  .modal-content {
    background-color: white;
    border-radius: 0.5rem;
    width: 90%;
    max-width: 800px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
  }

  :global(.dark .modal-content) {
    background-color: #2d2d2d;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid #e5e7eb;
  }

  :global(.dark .modal-header) {
    border-bottom-color: #404040;
  }

  .modal-header h3 {
    font-size: 1.25rem;
    font-weight: 600;
    color: #1a1a1a;
    margin: 0;
  }

  :global(.dark .modal-header h3) {
    color: #f5f5f5;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: #6b7280;
    cursor: pointer;
    padding: 0;
    width: 2rem;
    height: 2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.25rem;
  }

  .close-btn:hover {
    background-color: #f3f4f6;
  }

  :global(.dark .close-btn) {
    color: #9ca3af;
  }

  :global(.dark .close-btn:hover) {
    background-color: #3d3d3d;
  }

  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }

  .output-section {
    margin-bottom: 1.5rem;
  }

  .output-section:last-child {
    margin-bottom: 0;
  }

  .output-section h4 {
    font-size: 0.875rem;
    font-weight: 600;
    color: #374151;
    margin: 0 0 0.5rem 0;
  }

  :global(.dark .output-section h4) {
    color: #d1d5db;
  }

  .exit-code {
    font-size: 0.813rem;
    color: #6b7280;
    margin: 0;
  }

  :global(.dark .exit-code) {
    color: #9ca3af;
  }

  .output-text {
    background-color: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 0.375rem;
    padding: 1rem;
    font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
    font-size: 0.75rem;
    line-height: 1.5;
    color: #1a1a1a;
    white-space: pre-wrap;
    word-wrap: break-word;
    overflow-x: auto;
    margin: 0;
  }

  :global(.dark .output-text) {
    background-color: #1a1a1a;
    border-color: #404040;
    color: #e5e7eb;
  }

  .output-text.error {
    background-color: #fef2f2;
    border-color: #fecaca;
    color: #991b1b;
  }

  :global(.dark .output-text.error) {
    background-color: #3f1a1a;
    border-color: #7f1d1d;
    color: #fca5a5;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: 1rem 1.5rem;
    border-top: 1px solid #e5e7eb;
  }

  :global(.dark .modal-footer) {
    border-top-color: #404040;
  }
</style>

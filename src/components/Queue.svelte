<script>
  import QueueItem from './QueueItem.svelte';
  
  let { tasks, onRemove, onClearCompleted } = $props();
  
  $effect(() => {
    console.log('Tasks updated:', tasks);
  });
  
  const completedCount = $derived(
    tasks.filter(t => t.status === 'completed' || t.status === 'failed').length
  );
</script>

<div class="queue">
  <div class="queue-header">
    <span class="queue-title">
      Download Queue ({tasks.length})
    </span>
    {#if completedCount > 0}
      <button class="clear-btn" onclick={onClearCompleted}>
        Clear Completed
      </button>
    {/if}
  </div>
  
  <div class="queue-list">
    {#if tasks.length === 0}
      <div class="empty">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
          <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
        </svg>
        <p>No downloads yet</p>
        <p class="hint">Paste a YouTube URL above to get started</p>
      </div>
    {:else}
      {#each tasks as task (task.id)}
        <QueueItem {task} onRemove={() => onRemove(task.id)} />
      {/each}
    {/if}
  </div>
</div>

<style>
  .queue {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .queue-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
  }

  .queue-title {
    font-weight: 500;
    font-size: 13px;
  }

  .clear-btn {
    font-size: 12px;
    color: var(--text-secondary);
    padding: 4px 8px;
    border-radius: 4px;
  }

  .clear-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .queue-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--text-secondary);
    text-align: center;
  }

  .empty p {
    margin-top: 8px;
  }

  .empty .hint {
    font-size: 12px;
    opacity: 0.7;
  }
</style>

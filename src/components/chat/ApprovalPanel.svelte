<script lang="ts">
  import { pendingToolCalls } from "../../lib/stores/agent";
</script>

<section class="panel">
  <div class="header">
    <h3>Approval Queue</h3>
    <span>{$pendingToolCalls.length}</span>
  </div>

  {#if $pendingToolCalls.length === 0}
    <p>Нет ожидающих действий.</p>
  {:else}
    {#each $pendingToolCalls as toolCall}
      <article>
        <strong>{toolCall.name}</strong>
        <code>{toolCall.argumentsPreview}</code>
        <div class="actions">
          <md-filled-button>Approve</md-filled-button>
          <md-outlined-button>Reject</md-outlined-button>
        </div>
      </article>
    {/each}
  {/if}
</section>

<style>
  .panel,
  article {
    display: grid;
    gap: 0.8rem;
  }

  .panel {
    padding: 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 24px;
  }

  .header,
  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  article {
    padding: 0.9rem;
    border-radius: 18px;
    background: rgba(255, 255, 255, 0.05);
  }

  code,
  h3,
  p,
  strong,
  span {
    margin: 0;
  }
</style>

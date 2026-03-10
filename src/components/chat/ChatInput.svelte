<script lang="ts">
  let draft = $state("");
</script>

<form class="composer">
  <textarea bind:value={draft} rows="4" placeholder="Опиши задачу для агента..."></textarea>
  <div class="actions">
    <small>Enter позже привяжем к `send_message`.</small>
    <md-filled-button type="submit">Send</md-filled-button>
  </div>
</form>

<style>
  .composer {
    display: grid;
    gap: 0.75rem;
    padding: 1rem;
    background: var(--surface-strong);
    border: 1px solid var(--border);
    border-radius: 24px;
  }

  textarea {
    resize: vertical;
    min-height: 7rem;
    border-radius: 18px;
    border: 1px solid var(--border);
    background: rgba(7, 11, 26, 0.8);
    color: inherit;
    padding: 0.9rem 1rem;
  }

  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  small {
    color: var(--text-muted);
  }
</style>

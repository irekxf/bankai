<script lang="ts">
  import { currentSessionId, sessions } from "../../lib/stores/sessions";
</script>

<aside class="panel">
  <div class="header">
    <h2>Sessions</h2>
    <span>Local</span>
  </div>

  {#each $sessions as session}
    <button
      class:selected={session.id === $currentSessionId}
      on:click={() => currentSessionId.set(session.id)}
      type="button"
    >
      <strong>{session.title}</strong>
      <small>{new Date(session.updatedAt).toLocaleTimeString()}</small>
    </button>
  {/each}
</aside>

<style>
  .panel {
    display: grid;
    gap: 0.75rem;
    padding: 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 24px;
    backdrop-filter: blur(16px);
  }

  .header,
  button {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  button {
    padding: 0.9rem 1rem;
    border-radius: 18px;
    border: 1px solid transparent;
    background: rgba(255, 255, 255, 0.04);
    color: inherit;
    cursor: pointer;
  }

  button.selected {
    border-color: rgba(127, 212, 255, 0.4);
    background: rgba(127, 212, 255, 0.12);
  }

  h2,
  small,
  strong,
  span {
    margin: 0;
  }
</style>

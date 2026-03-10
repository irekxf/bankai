<script lang="ts">
  import { providerSettings } from "../../lib/stores/settings";
</script>

<section class="panel">
  <div class="header">
    <div>
      <p class="eyebrow">Provider</p>
      <h3>{$providerSettings.displayName}</h3>
    </div>
    <span class:ready={$providerSettings.apiKeyStatus === "configured"}>
      {$providerSettings.apiKeyStatus}
    </span>
  </div>

  <div class="grid">
    <label>
      <span>Base URL</span>
      <input bind:value={$providerSettings.baseUrl} />
    </label>

    <label>
      <span>Model</span>
      <input bind:value={$providerSettings.model} />
    </label>
  </div>

  <p class="hint">MVP идёт через OpenAI API key в keyring. OAuth-логин не закладываем.</p>
</section>

<style>
  .panel {
    display: grid;
    gap: 0.9rem;
    padding: 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 24px;
  }

  .header,
  .grid {
    display: grid;
    gap: 0.9rem;
  }

  .header {
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: start;
  }

  .grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  label {
    display: grid;
    gap: 0.35rem;
  }

  input {
    border-radius: 14px;
    border: 1px solid var(--border);
    background: rgba(7, 11, 26, 0.8);
    color: inherit;
    padding: 0.7rem 0.85rem;
  }

  .eyebrow,
  .hint,
  h3,
  p,
  span {
    margin: 0;
  }

  .eyebrow,
  .hint {
    color: var(--text-muted);
  }

  span.ready {
    color: var(--accent-2);
  }

  @media (max-width: 960px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>

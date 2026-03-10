<script lang="ts">
  import { onMount } from "svelte";
  import { getOAuthStatus, getProviderConfig, saveProviderConfig } from "../../lib/tauri/commands";
  import { oauthStatus } from "../../lib/stores/auth";
  import { providerSettings } from "../../lib/stores/settings";

  onMount(async () => {
    try {
      const [config, oauth] = await Promise.all([getProviderConfig(), getOAuthStatus()]);
      providerSettings.update((current) => ({
        ...current,
        ...config,
        apiKeyDraft: "",
        saveState: "idle"
      }));
      oauthStatus.set(oauth);
    } catch {
      providerSettings.update((current) => ({ ...current, saveState: "error" }));
    }
  });

  async function saveSettings() {
    providerSettings.update((current) => ({ ...current, saveState: "saving" }));

    try {
      const snapshot = $providerSettings;
      const config = await saveProviderConfig({
        provider: snapshot.provider,
        displayName: snapshot.displayName,
        baseUrl: snapshot.baseUrl,
        model: snapshot.model,
        apiKey: snapshot.apiKeyDraft
      });

      providerSettings.update((current) => ({
        ...current,
        ...config,
        apiKeyDraft: "",
        saveState: "saved"
      }));
    } catch {
      providerSettings.update((current) => ({ ...current, saveState: "error" }));
    }
  }

  function handleSaveKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void saveSettings();
    }
  }
</script>

<section class="panel">
  <div class="header">
    <div>
      <p class="eyebrow">Provider</p>
      <h3>{$providerSettings.displayName}</h3>
    </div>
    <div class="status-stack">
      <span class:ready={$providerSettings.apiKeyStatus === "configured"}>
        api key: {$providerSettings.apiKeyStatus}
      </span>
      <span class:ready={$oauthStatus.loggedIn}>oauth: {$oauthStatus.loggedIn ? "connected" : "missing"}</span>
    </div>
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

  <label>
    <span>API Key</span>
    <input
      bind:value={$providerSettings.apiKeyDraft}
      autocomplete="off"
      placeholder="sk-..."
      type="password"
    />
  </label>

  <div class="actions">
    <small>
      {#if $providerSettings.saveState === "saving"}
        Сохраняю...
      {:else if $providerSettings.saveState === "saved"}
        Настройки сохранены.
      {:else if $providerSettings.saveState === "error"}
        Не удалось сохранить настройки.
      {:else}
        Храним API key в системном keyring.
      {/if}
    </small>
    <md-filled-button
      onclick={() => void saveSettings()}
      onkeydown={handleSaveKeydown}
      role="button"
      tabindex="0"
    >
      Save
    </md-filled-button>
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

  .status-stack {
    display: grid;
    gap: 0.25rem;
    justify-items: end;
  }

  .grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  label {
    display: grid;
    gap: 0.35rem;
  }

  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
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

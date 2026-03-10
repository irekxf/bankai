<script lang="ts">
  import { onMount } from "svelte";
  import {
    clearOAuthSession,
    getOAuthStatus,
    getProviderConfig,
    listProviderModels,
    saveProviderConfig,
    startOAuthLogin
  } from "../../lib/tauri/commands";
  import { oauthLoginState, oauthStatus } from "../../lib/stores/auth";
  import { providerSettings } from "../../lib/stores/settings";

  type PresetKey = "default" | "fast" | "reasoning" | "coding";
  const presetDescriptions: Record<PresetKey, string> = {
    default: "Balanced general model for everyday chat and agent tasks.",
    fast: "Lower latency pick for quick iterations and lightweight tasks.",
    reasoning: "Stronger planning and analysis, usually with higher latency.",
    coding: "Best fit for code editing, shell usage, and repo work."
  };

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
      void refreshModels();
    } catch {
      providerSettings.update((current) => ({ ...current, saveState: "error" }));
    }
  });

  async function refreshModels() {
    providerSettings.update((current) => ({ ...current, modelsState: "loading" }));

    try {
      const models = await listProviderModels();
      const recommendations = buildRecommendations(models);
      providerSettings.update((current) => ({
        ...current,
        availableModels: models,
        recommendedDefaultModel: recommendations.default,
        recommendedFastModel: recommendations.fast,
        recommendedReasoningModel: recommendations.reasoning,
        recommendedCodingModel: recommendations.coding,
        modelsState: "loaded"
      }));
    } catch {
      providerSettings.update((current) => ({
        ...current,
        availableModels: [],
        recommendedDefaultModel: undefined,
        recommendedFastModel: undefined,
        recommendedReasoningModel: undefined,
        recommendedCodingModel: undefined,
        modelsState: "error"
      }));
    }
  }

  async function saveSettings() {
    providerSettings.update((current) => ({ ...current, saveState: "saving" }));

    try {
      const snapshot = $providerSettings;
      const config = await saveProviderConfig({
        provider: snapshot.provider,
        displayName: snapshot.displayName,
        baseUrl: snapshot.baseUrl,
        model: snapshot.model,
        preferredAuth: snapshot.preferredAuth,
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

  function currentAuthLabel(): string {
    if ($providerSettings.preferredAuth === "api_key") {
      return $providerSettings.apiKeyStatus === "configured" ? "API key" : "API key (not set)";
    }
    if ($providerSettings.preferredAuth === "oauth") {
      return $oauthStatus.loggedIn ? "OAuth" : "OAuth (not connected)";
    }
    if ($providerSettings.apiKeyStatus === "configured") {
      return "Auto → API key";
    }
    if ($oauthStatus.loggedIn) {
      return "Auto → OAuth";
    }
    return "Not configured";
  }

  function formattedExpiry(): string {
    if (!$oauthStatus.expiresAt) {
      return "Unknown";
    }

    return new Date($oauthStatus.expiresAt * 1000).toLocaleString();
  }

  async function connectOAuth() {
    oauthLoginState.set("launching");

    try {
      const status = await startOAuthLogin();
      const config = await getProviderConfig();
      oauthStatus.set(status);
      oauthLoginState.set(status.loggedIn ? "connected" : "error");
      providerSettings.update((current) => ({
        ...current,
        ...config
      }));
      void refreshModels();
    } catch {
      oauthLoginState.set("error");
      providerSettings.update((current) => ({ ...current, saveState: "error" }));
    }
  }

  async function disconnectOAuth() {
    await clearOAuthSession();
    oauthStatus.set({
      loggedIn: false
    });
    providerSettings.update((current) => ({
      ...current,
      availableModels: [],
      recommendedDefaultModel: undefined,
      recommendedFastModel: undefined,
      recommendedReasoningModel: undefined,
      recommendedCodingModel: undefined,
      modelsState: "idle",
      preferredAuth: current.apiKeyStatus === "configured" ? "api_key" : "auto"
    }));
  }

  function handleDisconnectKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void disconnectOAuth();
    }
  }

  function handleConnectKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void connectOAuth();
    }
  }

  function handleRefreshModelsKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void refreshModels();
    }
  }

  function buildRecommendations(models: string[]) {
    return {
      default: pickFirst(models, ["gpt-5", "gpt-4.1", "gpt-4o", "chatgpt-4o", "gpt-"]),
      fast: pickFirst(models, ["gpt-4o-mini", "gpt-4.1-mini", "gpt-4o", "gpt-4.1", "gpt-"]),
      reasoning: pickFirst(models, ["o4", "o3", "o1", "gpt-5", "gpt-4.1"]),
      coding: pickFirst(models, ["codex-", "gpt-5", "gpt-4.1", "gpt-4o", "o3"])
    };
  }

  function pickFirst(models: string[], prefixes: string[]): string | undefined {
    for (const prefix of prefixes) {
      const match = models.find((modelName) => modelName.startsWith(prefix));
      if (match) {
        return match;
      }
    }

    return models[0];
  }

  function presetModel(preset: PresetKey): string | undefined {
    if (preset === "default") {
      return $providerSettings.recommendedDefaultModel;
    }
    if (preset === "fast") {
      return $providerSettings.recommendedFastModel;
    }
    if (preset === "reasoning") {
      return $providerSettings.recommendedReasoningModel;
    }
    return $providerSettings.recommendedCodingModel;
  }

  function applyPreset(preset: PresetKey) {
    const modelName = presetModel(preset);
    if (!modelName) {
      return;
    }

    providerSettings.update((current) => ({
      ...current,
      model: modelName
    }));
  }

  function handlePresetKeydown(event: KeyboardEvent, preset: PresetKey) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      applyPreset(preset);
    }
  }
</script>

<section class="panel">
  <div class="header">
    <div>
      <p class="eyebrow">Provider</p>
      <h3>{$providerSettings.displayName}</h3>
      <p class="subtitle">Current model: <strong>{$providerSettings.model}</strong></p>
    </div>
    <div class="summary">
      <span class="badge primary">{currentAuthLabel()}</span>
      <span class:ready={$providerSettings.apiKeyStatus === "configured"} class="badge">
        API key: {$providerSettings.apiKeyStatus}
      </span>
      <span class:ready={$oauthStatus.loggedIn} class="badge">
        OAuth: {$oauthStatus.loggedIn ? "connected" : "missing"}
      </span>
    </div>
  </div>

  <div class="overview">
    <article>
      <span class="label">Model</span>
      <strong>{$providerSettings.model}</strong>
    </article>
    <article>
      <span class="label">Base URL</span>
      <strong>{$providerSettings.baseUrl}</strong>
    </article>
    <article>
      <span class="label">Preferred auth</span>
      <strong>
        {#if $providerSettings.preferredAuth === "api_key"}
          API key
        {:else if $providerSettings.preferredAuth === "oauth"}
          OAuth
        {:else}
          Auto
        {/if}
      </strong>
    </article>
  </div>

  <div class="grid">
    <label>
      <span>Base URL</span>
      <input bind:value={$providerSettings.baseUrl} />
    </label>

    <label>
      <span>Model</span>
      {#if $providerSettings.availableModels.length > 0}
        <select bind:value={$providerSettings.model}>
          {#each $providerSettings.availableModels as modelName}
            <option value={modelName}>{modelName}</option>
          {/each}
        </select>
      {:else}
        <input bind:value={$providerSettings.model} />
      {/if}
    </label>
  </div>

  <div class="models-row">
    <small>
      {#if $providerSettings.modelsState === "loading"}
        Загружаю список моделей...
      {:else if $providerSettings.modelsState === "loaded"}
        Найдено чат-моделей: {$providerSettings.availableModels.length}
      {:else if $providerSettings.modelsState === "error"}
        Не удалось загрузить список моделей. Можно ввести модель вручную.
      {:else}
        Можно загрузить список моделей с текущими настройками авторизации.
      {/if}
    </small>
    <md-outlined-button
      onclick={() => void refreshModels()}
      onkeydown={handleRefreshModelsKeydown}
      role="button"
      tabindex="0"
    >
      Refresh models
    </md-outlined-button>
  </div>

  <div class="presets">
    <button
      class:selected={presetModel("default") === $providerSettings.model}
      disabled={!presetModel("default")}
      onclick={() => applyPreset("default")}
      onkeydown={(event) => handlePresetKeydown(event, "default")}
      type="button"
    >
      <span>Default</span>
      <small>{presetModel("default") ?? "Unavailable"}</small>
      <small class="preset-copy">{presetDescriptions.default}</small>
    </button>

    <button
      class:selected={presetModel("fast") === $providerSettings.model}
      disabled={!presetModel("fast")}
      onclick={() => applyPreset("fast")}
      onkeydown={(event) => handlePresetKeydown(event, "fast")}
      type="button"
    >
      <span>Fast</span>
      <small>{presetModel("fast") ?? "Unavailable"}</small>
      <small class="preset-copy">{presetDescriptions.fast}</small>
    </button>

    <button
      class:selected={presetModel("reasoning") === $providerSettings.model}
      disabled={!presetModel("reasoning")}
      onclick={() => applyPreset("reasoning")}
      onkeydown={(event) => handlePresetKeydown(event, "reasoning")}
      type="button"
    >
      <span>Reasoning</span>
      <small>{presetModel("reasoning") ?? "Unavailable"}</small>
      <small class="preset-copy">{presetDescriptions.reasoning}</small>
    </button>

    <button
      class:selected={presetModel("coding") === $providerSettings.model}
      disabled={!presetModel("coding")}
      onclick={() => applyPreset("coding")}
      onkeydown={(event) => handlePresetKeydown(event, "coding")}
      type="button"
    >
      <span>Coding</span>
      <small>{presetModel("coding") ?? "Unavailable"}</small>
      <small class="preset-copy">{presetDescriptions.coding}</small>
    </button>
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

  <label>
    <span>Preferred auth</span>
    <select bind:value={$providerSettings.preferredAuth}>
      <option value="auto">Auto</option>
      <option value="api_key">API key</option>
      <option value="oauth">OAuth</option>
    </select>
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

  <div class="oauth-panel">
    <div class="oauth-meta">
      <small>
        {#if $oauthStatus.loggedIn}
          OAuth session is stored in the system keyring and refreshed automatically.
        {:else if $providerSettings.preferredAuth === "oauth"}
          OAuth is selected as preferred auth, but no active session found. Connect below.
        {:else if $providerSettings.preferredAuth === "api_key" && $providerSettings.apiKeyStatus !== "configured"}
          API key is selected but not saved. Enter your key above and save.
        {:else if $providerSettings.apiKeyStatus !== "configured"}
          No auth configured. Enter an API key or connect via OAuth to use the provider.
        {:else}
          OAuth is an additional auth option. API key is already configured.
        {/if}
      </small>
      {#if $oauthStatus.loggedIn}
        <small>Expires: {formattedExpiry()}</small>
      {/if}
    </div>

    <div class="oauth-buttons">
      {#if !$oauthStatus.loggedIn}
        <md-outlined-button
          onclick={() => void connectOAuth()}
          onkeydown={handleConnectKeydown}
          role="button"
          tabindex="0"
        >
          {$oauthLoginState === "launching" ? "Connecting..." : "Connect OAuth"}
        </md-outlined-button>
      {/if}

      {#if $oauthStatus.loggedIn}
        <md-outlined-button
          onclick={() => void disconnectOAuth()}
          onkeydown={handleDisconnectKeydown}
          role="button"
          tabindex="0"
        >
          Disconnect OAuth
        </md-outlined-button>
      {/if}
    </div>
  </div>
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
    grid-template-columns: minmax(0, 1fr);
    align-items: start;
  }

  .summary {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }

  .overview {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.75rem;
  }

  .overview article {
    display: grid;
    gap: 0.3rem;
    padding: 0.8rem 0.9rem;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.04);
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

  .models-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .presets {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.75rem;
  }

  .presets button {
    display: grid;
    gap: 0.3rem;
    text-align: left;
    padding: 0.85rem 0.9rem;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.04);
    color: inherit;
    cursor: pointer;
  }

  .presets button.selected {
    border-color: var(--accent);
    background: rgba(120, 186, 255, 0.12);
  }

  .preset-copy {
    color: var(--text-muted);
    line-height: 1.35;
  }

  .presets button:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .oauth-panel {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
  }

  .oauth-panel {
    align-items: center;
  }

  .oauth-meta {
    display: grid;
    gap: 0.25rem;
  }

  .oauth-buttons {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  input {
    border-radius: 14px;
    border: 1px solid var(--border);
    background: rgba(7, 11, 26, 0.8);
    color: inherit;
    padding: 0.7rem 0.85rem;
  }

  select {
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

  .subtitle {
    margin-top: 0.35rem;
    color: var(--text-muted);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 0.35rem 0.6rem;
    border-radius: 999px;
    border: 1px solid var(--border);
    color: var(--text-muted);
    background: rgba(255, 255, 255, 0.04);
  }

  .badge.primary {
    color: #08101f;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    border-color: transparent;
  }

  .label {
    color: var(--text-muted);
    font-size: 0.8rem;
  }

  span.ready {
    color: var(--accent-2);
  }

  @media (max-width: 960px) {
    .overview,
    .grid,
    .presets {
      grid-template-columns: 1fr;
    }

    .oauth-panel {
      align-items: start;
      flex-direction: column;
    }

    .models-row {
      align-items: start;
      flex-direction: column;
    }
  }
</style>

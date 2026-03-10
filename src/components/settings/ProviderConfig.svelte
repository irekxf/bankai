<script lang="ts">
  import { onMount } from "svelte";
  import {
    clearOAuthSession,
    getProviderStatus,
    listProviderModels,
    saveProviderConfig,
    startOAuthLogin
  } from "../../lib/tauri/commands";
  import { oauthLoginState, oauthStatus } from "../../lib/stores/auth";
  import { applyProviderStatus } from "../../lib/stores/providerStatus";
  import { providerSettings } from "../../lib/stores/settings";

  type PresetKey = "default" | "fast" | "reasoning" | "coding";
  type EffectiveAuthMode = "api_key" | "oauth" | "none";

  const presetDescriptions: Record<PresetKey, string> = {
    default: "Balanced general model for everyday chat and agent tasks.",
    fast: "Lower latency pick for quick iterations and lightweight tasks.",
    reasoning: "Stronger planning and analysis, usually with higher latency.",
    coding: "Best fit for code editing, shell usage, and repo work."
  };

  onMount(async () => {
    try {
      const status = await getProviderStatus();
      applyProviderStatus(status, {
        clearApiKeyDraft: true,
        saveState: "idle",
        saveError: null
      });

      if (status.canLoadModels) {
        void refreshModels();
      } else {
        resetModelsState();
      }
    } catch (error) {
      providerSettings.update((current) => ({
        ...current,
        saveState: "error",
        saveError: getErrorMessage(error, "Failed to load provider settings.")
      }));
    }
  });

  function getErrorMessage(error: unknown, fallback: string): string {
    if (typeof error === "string" && error.trim().length > 0) {
      return error;
    }

    if (error instanceof Error && error.message.trim().length > 0) {
      return error.message;
    }

    return fallback;
  }

  function hasApiKeyConfigured(): boolean {
    return $providerSettings.apiKeyStatus === "configured";
  }

  function hasOAuthSession(): boolean {
    return $oauthStatus.loggedIn;
  }

  function effectiveAuthMode(): EffectiveAuthMode {
    return $providerSettings.activeAuth;
  }

  function activeAuthLabel(): string {
    const mode = effectiveAuthMode();

    if (mode === "api_key") {
      return $providerSettings.preferredAuth === "auto" ? "Auto -> API key" : "API key";
    }

    if (mode === "oauth") {
      return $providerSettings.preferredAuth === "auto" ? "Auto -> OAuth" : "OAuth";
    }

    if ($providerSettings.preferredAuth === "api_key") {
      return "API key required";
    }

    if ($providerSettings.preferredAuth === "oauth") {
      return "OAuth required";
    }

    return "Not configured";
  }

  function authReadinessLabel(): string {
    return $providerSettings.authReady ? "Ready" : "Needs setup";
  }

  function authSupportCopy(): string {
    return $providerSettings.authMessage;
  }

  function modelsHelperText(): string {
    if ($providerSettings.modelsState === "loading") {
      return "Loading available chat models...";
    }

    if ($providerSettings.modelsState === "loaded") {
      return `Loaded ${$providerSettings.availableModels.length} models for the current auth state.`;
    }

    if ($providerSettings.modelsState === "error") {
      return $providerSettings.modelsError ?? "Failed to load models. You can still type a model name manually.";
    }

    if (effectiveAuthMode() === "none") {
      return "Configure OAuth or save an API key before loading models.";
    }

    return "Load available models using the currently active auth method.";
  }

  function saveHelperText(): string {
    if ($providerSettings.saveState === "saving") {
      return "Saving provider settings...";
    }

    if ($providerSettings.saveState === "saved") {
      return "Provider settings saved.";
    }

    if ($providerSettings.saveState === "error") {
      return $providerSettings.saveError ?? "Failed to save provider settings.";
    }

    return "API keys are stored in the system keyring. OAuth sessions stay separate.";
  }

  function resetModelsState() {
    providerSettings.update((current) => ({
      ...current,
      availableModels: [],
      recommendedDefaultModel: undefined,
      recommendedFastModel: undefined,
      recommendedReasoningModel: undefined,
      recommendedCodingModel: undefined,
      modelsState: "idle",
      modelsError: null
    }));
  }

  async function refreshModels() {
    if (!$providerSettings.canLoadModels) {
      resetModelsState();
      return;
    }

    providerSettings.update((current) => ({
      ...current,
      modelsState: "loading",
      modelsError: null
    }));

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
        modelsState: "loaded",
        modelsError: null
      }));
    } catch (error) {
      providerSettings.update((current) => ({
        ...current,
        availableModels: [],
        recommendedDefaultModel: undefined,
        recommendedFastModel: undefined,
        recommendedReasoningModel: undefined,
        recommendedCodingModel: undefined,
        modelsState: "error",
        modelsError: getErrorMessage(error, "Failed to load models. Check the selected auth method.")
      }));
    }
  }

  async function saveSettings() {
    providerSettings.update((current) => ({
      ...current,
      saveState: "saving",
      saveError: null
    }));

    try {
      const snapshot = $providerSettings;
      const status = await saveProviderConfig({
        provider: snapshot.provider,
        displayName: snapshot.displayName,
        baseUrl: snapshot.baseUrl,
        model: snapshot.model,
        preferredAuth: snapshot.preferredAuth,
        apiKey: snapshot.apiKeyDraft
      });

      applyProviderStatus(status, {
        clearApiKeyDraft: true,
        saveState: "saved",
        saveError: null
      });

      if (status.canLoadModels) {
        void refreshModels();
      } else {
        resetModelsState();
      }
    } catch (error) {
      providerSettings.update((current) => ({
        ...current,
        saveState: "error",
        saveError: getErrorMessage(error, "Failed to save provider settings.")
      }));
    }
  }

  function handleSaveKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void saveSettings();
    }
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
      applyProviderStatus(status, {
        saveState: "idle",
        saveError: null
      });

      if (status.canLoadModels) {
        void refreshModels();
      } else {
        resetModelsState();
      }
    } catch (error) {
      oauthLoginState.set("error");
      providerSettings.update((current) => ({
        ...current,
        saveState: "error",
        saveError: getErrorMessage(error, "OAuth login failed.")
      }));
    }
  }

  async function disconnectOAuth() {
    try {
      const status = await clearOAuthSession();
      applyProviderStatus(status, {
        saveState: "idle",
        saveError: null
      });

      if (status.canLoadModels) {
        void refreshModels();
      } else {
        resetModelsState();
      }
    } catch (error) {
      providerSettings.update((current) => ({
        ...current,
        saveState: "error",
        saveError: getErrorMessage(error, "Failed to disconnect OAuth.")
      }));
    }
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
      <span class="badge primary">{activeAuthLabel()}</span>
      <span class:ready={hasApiKeyConfigured()} class="badge">
        API key: {$providerSettings.apiKeyStatus}
      </span>
      <span class:ready={hasOAuthSession()} class="badge">
        OAuth: {hasOAuthSession() ? "connected" : "missing"}
      </span>
      <span class:ready={effectiveAuthMode() !== "none"} class="badge">
        Provider: {authReadinessLabel()}
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

  <div class="auth-callout" class:warning={effectiveAuthMode() === "none"}>
    <strong>{authReadinessLabel()}</strong>
    <small>{authSupportCopy()}</small>
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
    <small>{modelsHelperText()}</small>
    <md-outlined-button
      disabled={$providerSettings.modelsState === "loading" || effectiveAuthMode() === "none"}
      onclick={() => void refreshModels()}
      onkeydown={handleRefreshModelsKeydown}
      role="button"
      tabindex="0"
    >
      {$providerSettings.modelsState === "loading" ? "Loading..." : "Refresh models"}
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
    <small>{saveHelperText()}</small>
    <md-filled-button
      disabled={$providerSettings.saveState === "saving"}
      onclick={() => void saveSettings()}
      onkeydown={handleSaveKeydown}
      role="button"
      tabindex="0"
    >
      {$providerSettings.saveState === "saving" ? "Saving..." : "Save"}
    </md-filled-button>
  </div>

  <div class="oauth-panel">
    <div class="oauth-meta">
      <small>
        {#if hasOAuthSession()}
          OAuth session is stored in the system keyring and refreshed automatically.
        {:else if $providerSettings.preferredAuth === "oauth"}
          OAuth is selected as the preferred auth mode, but no active session is available yet.
        {:else if $providerSettings.preferredAuth === "api_key" && !hasApiKeyConfigured()}
          API key is selected as the preferred auth mode, but no saved key is available yet.
        {:else if effectiveAuthMode() === "none"}
          Neither OAuth nor API key is ready. Configure one method to start using the provider.
        {:else}
          OAuth remains available as an alternate auth method.
        {/if}
      </small>
      {#if hasOAuthSession()}
        <small>Expires: {formattedExpiry()}</small>
      {/if}
    </div>

    <div class="oauth-buttons">
      {#if !hasOAuthSession()}
        <md-outlined-button
          onclick={() => void connectOAuth()}
          onkeydown={handleConnectKeydown}
          role="button"
          tabindex="0"
        >
          {$oauthLoginState === "launching" ? "Connecting..." : "Connect OAuth"}
        </md-outlined-button>
      {/if}

      {#if hasOAuthSession()}
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

  .overview article,
  .auth-callout {
    display: grid;
    gap: 0.3rem;
    padding: 0.8rem 0.9rem;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.04);
  }

  .auth-callout.warning {
    border-color: rgba(255, 195, 113, 0.55);
    background: rgba(255, 195, 113, 0.08);
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

  input,
  select {
    border-radius: 14px;
    border: 1px solid var(--border);
    background: rgba(7, 11, 26, 0.8);
    color: inherit;
    padding: 0.7rem 0.85rem;
  }

  .eyebrow,
  h3,
  p,
  span,
  strong,
  small {
    margin: 0;
  }

  .eyebrow {
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

    .oauth-panel,
    .models-row,
    .actions {
      align-items: start;
      flex-direction: column;
    }
  }
</style>


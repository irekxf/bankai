<script lang="ts">
  import { onMount } from "svelte";
  import {
    getProviderStatus,
    startOAuthLogin,
    type ProviderStatusDto
  } from "../../lib/tauri/commands";
  import { oauthLoginState, oauthStatus } from "../../lib/stores/auth";
  import { providerSettings } from "../../lib/stores/settings";

  let shouldShow = $state(false);

  onMount(async () => {
    await refreshState();
  });

  function syncProviderStatus(status: ProviderStatusDto) {
    oauthStatus.set({
      loggedIn: status.oauthLoggedIn,
      authMode: status.oauthAuthMode,
      accountId: status.oauthAccountId,
      expiresAt: status.oauthExpiresAt
    });

    oauthLoginState.update((current) => {
      if (status.oauthLoggedIn) {
        return "connected";
      }

      return current === "connected" ? "idle" : current;
    });

    providerSettings.update((current) => ({
      ...current,
      provider: status.provider,
      displayName: status.displayName,
      baseUrl: status.baseUrl,
      model: status.model,
      preferredAuth: status.preferredAuth,
      apiKeyStatus: status.apiKeyStatus,
      activeAuth: status.activeAuth,
      authReady: status.authReady,
      authMessage: status.authMessage,
      canLoadModels: status.canLoadModels,
      canSendMessages: status.canSendMessages
    }));
  }

  function shouldShowOnboarding(provider: ProviderStatusDto) {
    return (
      provider.preferredAuth === "auto" &&
      !provider.oauthLoggedIn &&
      provider.apiKeyStatus !== "configured" &&
      $oauthLoginState !== "dismissed"
    );
  }

  async function refreshState() {
    try {
      const status = await getProviderStatus();
      syncProviderStatus(status);
      shouldShow = shouldShowOnboarding(status);
    } catch {
      oauthLoginState.set("error");
      shouldShow = false;
    }
  }

  async function beginLogin() {
    oauthLoginState.set("launching");
    try {
      const status = await startOAuthLogin();
      syncProviderStatus(status);
      oauthLoginState.set(status.oauthLoggedIn ? "connected" : "error");
      shouldShow = shouldShowOnboarding(status);
    } catch {
      oauthLoginState.set("error");
      shouldShow = false;
    }
  }

  function useApiKeyInstead() {
    oauthLoginState.set("dismissed");
    shouldShow = false;
  }

  function handleLoginKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void beginLogin();
    }
  }

  function handleApiKeyKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      useApiKeyInstead();
    }
  }
</script>

{#if shouldShow}
  <section class="overlay">
    <div class="card">
      <p class="eyebrow">OpenAI Setup</p>
      <h2>Connect OAuth or continue with an API key</h2>
      <p class="copy">
        Bankai supports both auth modes. OAuth uses a localhost callback on
        <code>http://localhost:1455/auth/callback</code> to complete sign-in.
      </p>

      <div class="actions">
        <md-filled-button
          onclick={() => void beginLogin()}
          onkeydown={handleLoginKeydown}
          role="button"
          tabindex="0"
        >
          Continue with OpenAI
        </md-filled-button>
        <md-outlined-button
          onclick={useApiKeyInstead}
          onkeydown={handleApiKeyKeydown}
          role="button"
          tabindex="0"
        >
          Use API key instead
        </md-outlined-button>
      </div>

      <small>
        {#if $oauthLoginState === "launching"}
          Browser login started. Finish the OAuth flow and return to Bankai.
        {:else if $oauthLoginState === "error"}
          OAuth login failed. You can retry here or switch to the API key flow in Provider settings.
        {:else}
          Pick either auth method to unlock model listing and message sending.
        {/if}
      </small>
    </div>
  </section>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    padding: 1.5rem;
    background: rgba(5, 8, 18, 0.72);
    backdrop-filter: blur(10px);
    z-index: 30;
  }

  .card {
    width: min(38rem, 100%);
    display: grid;
    gap: 1rem;
    padding: 1.5rem;
    border-radius: 28px;
    background: rgba(14, 21, 39, 0.96);
    border: 1px solid var(--border);
    box-shadow: 0 20px 80px rgba(0, 0, 0, 0.35);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .eyebrow,
  .copy,
  h2,
  p,
  small {
    margin: 0;
  }

  .eyebrow,
  .copy,
  small {
    color: var(--text-muted);
  }
</style>

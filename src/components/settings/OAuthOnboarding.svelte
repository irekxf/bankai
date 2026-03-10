<script lang="ts">
  import { onMount } from "svelte";
  import { getOAuthStatus, getProviderConfig, startOAuthLogin } from "../../lib/tauri/commands";
  import { oauthLoginState, oauthStatus } from "../../lib/stores/auth";

  let shouldShow = $state(false);

  onMount(async () => {
    await refreshState();
  });

  async function refreshState() {
    try {
      const [oauth, provider] = await Promise.all([getOAuthStatus(), getProviderConfig()]);
      oauthStatus.set(oauth);

      shouldShow =
        !$oauthStatus.loggedIn &&
        provider.apiKeyStatus !== "configured" &&
        $oauthLoginState !== "dismissed";

      if ($oauthStatus.loggedIn) {
        oauthLoginState.set("connected");
      }
    } catch {
      oauthLoginState.set("error");
      shouldShow = true;
    }
  }

  async function beginLogin() {
    oauthLoginState.set("launching");
    try {
      const status = await startOAuthLogin();
      oauthStatus.set(status);
      oauthLoginState.set(status.loggedIn ? "connected" : "error");
      shouldShow = !status.loggedIn;
    } catch {
      oauthLoginState.set("error");
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
      <p class="eyebrow">OpenAI Login</p>
      <h2>Authorize through your browser or continue with API key</h2>
      <p class="copy">
        Experimental ChatGPT OAuth uses a localhost callback on `http://localhost:1455/auth/callback`.
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
          Use API key
        </md-outlined-button>
      </div>

      <small>
        {#if $oauthLoginState === "launching"}
          Browser flow started. Complete the login and return to Bankai.
        {:else if $oauthLoginState === "error"}
          OAuth login failed. You can retry or use an API key instead.
        {:else}
          API key support remains available and is the stable path.
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

<script lang="ts">
  import "@material/web/all.js";
  import ChatView from "../components/chat/ChatView.svelte";
  import OAuthOnboarding from "../components/settings/OAuthOnboarding.svelte";
  import ProviderConfig from "../components/settings/ProviderConfig.svelte";
  import SessionList from "../components/sidebar/SessionList.svelte";
  import { agentStatus } from "../lib/stores/agent";
</script>

<svelte:head>
  <title>Bankai</title>
</svelte:head>

<main class="shell">
  <OAuthOnboarding />

  <section class="hero">
    <div>
      <p class="eyebrow">Bankai</p>
      <h1>Native AI agent desktop with explicit control over every action.</h1>
    </div>
    <div class="status-card">
      <span>Status</span>
      <strong>{$agentStatus}</strong>
    </div>
  </section>

  <section class="layout">
    <div class="sidebar-stack">
      <SessionList />
      <ProviderConfig />
    </div>
    <ChatView />
  </section>
</main>

<style>
  .shell {
    display: grid;
    gap: 1.5rem;
    padding: 1.5rem;
  }

  .hero,
  .layout {
    display: grid;
    gap: 1.5rem;
  }

  .sidebar-stack {
    display: grid;
    gap: 1rem;
    align-content: start;
  }

  .hero {
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: end;
  }

  .layout {
    grid-template-columns: 20rem minmax(0, 1fr);
  }

  .eyebrow {
    color: var(--accent-2);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    font-size: 0.8rem;
  }

  .status-card {
    display: grid;
    gap: 0.3rem;
    padding: 1rem 1.2rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 22px;
  }

  h1,
  p,
  span,
  strong {
    margin: 0;
  }

  @media (max-width: 960px) {
    .hero,
    .layout {
      grid-template-columns: 1fr;
    }
  }
</style>

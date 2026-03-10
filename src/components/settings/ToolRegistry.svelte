<script lang="ts">
  import { onMount } from "svelte";
  import {
    listTools,
    setToolEnabled,
    type ToolRegistryEntryDto
  } from "../../lib/tauri/commands";

  type LoadState = "idle" | "loading" | "loaded" | "error";

  let tools = $state<ToolRegistryEntryDto[]>([]);
  let loadState = $state<LoadState>("idle");
  let errorMessage = $state<string | null>(null);
  let busyToolName = $state<string | null>(null);

  onMount(() => {
    void loadToolRegistry();
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

  function enabledCount(): number {
    return tools.filter((tool) => tool.enabled).length;
  }

  async function loadToolRegistry() {
    loadState = "loading";
    errorMessage = null;

    try {
      tools = await listTools();
      loadState = "loaded";
    } catch (error) {
      loadState = "error";
      errorMessage = getErrorMessage(error, "Failed to load tools.");
    }
  }

  async function updateToolEnabled(toolName: string, enabled: boolean) {
    busyToolName = toolName;
    errorMessage = null;

    try {
      const updated = await setToolEnabled(toolName, enabled);
      tools = tools.map((tool) => (tool.name === toolName ? updated : tool));
    } catch (error) {
      errorMessage = getErrorMessage(error, `Failed to update ${toolName}.`);
    } finally {
      busyToolName = null;
    }
  }

  function handleToggle(event: Event, toolName: string) {
    const target = event.currentTarget;
    if (!(target instanceof HTMLInputElement)) {
      return;
    }

    void updateToolEnabled(toolName, target.checked);
  }

  function handleRefreshKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void loadToolRegistry();
    }
  }
</script>

<section class="panel">
  <div class="header">
    <div>
      <p class="eyebrow">Tools</p>
      <h3>Tool Registry</h3>
      <p class="subtitle">
        {enabledCount()} of {tools.length} tools enabled
      </p>
    </div>
    <md-outlined-button
      disabled={loadState === "loading" || busyToolName !== null}
      onclick={() => void loadToolRegistry()}
      onkeydown={handleRefreshKeydown}
      role="button"
      tabindex="0"
    >
      {loadState === "loading" ? "Loading..." : "Refresh"}
    </md-outlined-button>
  </div>

  <div class="callout">
    <strong>Only enabled tools are sent to the model.</strong>
    <small>Shell and filesystem actions still keep their existing approval flow.</small>
  </div>

  {#if loadState === "loading" && tools.length === 0}
    <p class="empty">Loading tool registry...</p>
  {:else if loadState === "error" && tools.length === 0}
    <p class="empty">{errorMessage ?? "Failed to load tools."}</p>
  {:else}
    <div class="list">
      {#each tools as tool}
        <label class:pending={busyToolName === tool.name} class="tool-card">
          <div class="tool-copy">
            <div class="tool-header">
              <strong>{tool.displayName}</strong>
              {#if tool.requiresApproval}
                <span class="badge">Approval required</span>
              {/if}
            </div>
            <small>{tool.description}</small>
          </div>

          <div class="toggle-wrap">
            <span>{tool.enabled ? "On" : "Off"}</span>
            <input
              checked={tool.enabled}
              disabled={busyToolName !== null}
              onchange={(event) => handleToggle(event, tool.name)}
              type="checkbox"
            />
          </div>
        </label>
      {/each}
    </div>
  {/if}

  {#if errorMessage && tools.length > 0}
    <small class="error">{errorMessage}</small>
  {/if}
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
  .tool-header,
  .toggle-wrap {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .callout,
  .tool-card,
  .tool-copy,
  .list {
    display: grid;
    gap: 0.5rem;
  }

  .callout,
  .tool-card {
    padding: 0.85rem 0.9rem;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.04);
  }

  .tool-card.pending {
    opacity: 0.7;
  }

  .toggle-wrap {
    justify-content: end;
  }

  .toggle-wrap input {
    width: 2.8rem;
    height: 1.1rem;
    accent-color: var(--accent);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 0.25rem 0.55rem;
    border-radius: 999px;
    border: 1px solid var(--border);
    color: var(--text-muted);
    background: rgba(255, 255, 255, 0.04);
    font-size: 0.75rem;
  }

  .eyebrow,
  .subtitle,
  .empty,
  .error,
  h3,
  p,
  span,
  strong,
  small {
    margin: 0;
  }

  .eyebrow,
  .subtitle,
  .empty {
    color: var(--text-muted);
  }

  .error {
    color: #ffb4b4;
  }

  @media (max-width: 960px) {
    .tool-header,
    .toggle-wrap {
      align-items: start;
      flex-direction: column;
    }
  }
</style>

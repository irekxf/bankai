<script lang="ts">
  import { onMount } from "svelte";
  import { listTools, setToolEnabled } from "../../lib/tauri/commands";
  import type { ToolInfoDto } from "../../lib/tauri/commands";

  let tools: ToolInfoDto[] = [];
  let loadError = false;

  onMount(async () => {
    try {
      tools = await listTools();
    } catch {
      loadError = true;
    }
  });

  async function toggle(tool: ToolInfoDto) {
    try {
      tools = await setToolEnabled(tool.name, !tool.enabled);
    } catch {
      // keep existing state on failure
    }
  }

  function handleToggleKeydown(event: KeyboardEvent, tool: ToolInfoDto) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void toggle(tool);
    }
  }
</script>

<section class="panel">
  <div class="header">
    <p class="eyebrow">Tools</p>
    <h3>Available tools</h3>
  </div>

  {#if loadError}
    <p class="notice">Could not load tool list.</p>
  {:else if tools.length === 0}
    <p class="notice">Loading...</p>
  {:else}
    <ul class="tool-list">
      {#each tools as tool}
        <li class="tool-row">
          <div class="tool-meta">
            <strong>{tool.name}</strong>
            <small class="desc">{tool.description}</small>
            {#if tool.requiresApproval}
              <span class="badge">Requires approval</span>
            {/if}
          </div>
          <button
            aria-checked={tool.enabled}
            aria-label="{tool.enabled ? 'Disable' : 'Enable'} {tool.name}"
            class="toggle"
            class:on={tool.enabled}
            onclick={() => void toggle(tool)}
            onkeydown={(event) => handleToggleKeydown(event, tool)}
            role="switch"
            tabindex="0"
            type="button"
          >
            {tool.enabled ? "On" : "Off"}
          </button>
        </li>
      {/each}
    </ul>
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

  .header {
    display: grid;
    gap: 0.25rem;
  }

  .eyebrow {
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    font-size: 0.75rem;
    margin: 0;
  }

  h3 {
    margin: 0;
  }

  .tool-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.5rem;
  }

  .tool-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 0.9rem;
    border-radius: 16px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.04);
  }

  .tool-meta {
    display: grid;
    gap: 0.2rem;
  }

  .desc {
    color: var(--text-muted);
    line-height: 1.35;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 0.2rem 0.5rem;
    border-radius: 999px;
    border: 1px solid var(--border);
    color: var(--text-muted);
    background: rgba(255, 255, 255, 0.04);
    font-size: 0.75rem;
    width: fit-content;
  }

  .toggle {
    flex-shrink: 0;
    padding: 0.35rem 0.85rem;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-muted);
    cursor: pointer;
    font-size: 0.85rem;
  }

  .toggle.on {
    color: var(--accent-2);
    border-color: var(--accent-2);
    background: rgba(120, 186, 255, 0.1);
  }

  .notice {
    color: var(--text-muted);
    margin: 0;
    font-size: 0.9rem;
  }
</style>

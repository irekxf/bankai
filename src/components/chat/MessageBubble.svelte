<script lang="ts">
  import { formatToolPreview } from "../../lib/chat/toolPreview";
  import type { ChatMessage, ToolCallStatus } from "../../lib/stores/messages";

  let { message }: { message: ChatMessage } = $props();

  const title = $derived(getMessageTitle(message));
  const toolPreview = $derived(
    message.toolCall ? formatToolPreview(message.toolCall.name, message.toolCall.argumentsJson) : ""
  );
  const toolStatus = $derived(getToolStatusLabel(message.toolCall?.status));

  function getMessageTitle(current: ChatMessage): string {
    if (current.toolCall?.kind === "request") {
      return "tool request";
    }

    if (current.toolCall?.kind === "result") {
      return "tool result";
    }

    if (current.toolCall?.kind === "rejection") {
      return "tool rejected";
    }

    if (current.role === "user") {
      return "you";
    }

    return current.role;
  }

  function getToolStatusLabel(status?: ToolCallStatus): string {
    if (!status) {
      return "";
    }

    return status.replace("_", " ");
  }
</script>

<article
  class:assistant={message.role === "assistant"}
  class:user={message.role === "user"}
  class:tool={message.role === "tool"}
  class:toolActivity={Boolean(message.toolCall)}
>
  <header>{title}</header>

  {#if message.toolCall}
    <div class="tool-summary">
      <strong>{message.toolCall.name}</strong>
      {#if toolStatus}
        <span class="status">{toolStatus}</span>
      {/if}
    </div>

    {#if toolPreview}
      <code>{toolPreview}</code>
    {/if}
  {/if}

  {#if message.content}
    <p class="content">{message.content}</p>
  {/if}
</article>

<style>
  article {
    max-width: 48rem;
    padding: 1rem 1.1rem;
    border-radius: 22px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border);
  }

  article.user {
    justify-self: end;
    background: rgba(120, 240, 203, 0.12);
  }

  article.assistant {
    background: rgba(127, 212, 255, 0.1);
  }

  article.tool,
  article.toolActivity {
    background: rgba(255, 215, 140, 0.1);
  }

  header,
  p,
  code,
  strong {
    margin: 0;
  }

  header {
    margin-bottom: 0.45rem;
    color: var(--text-muted);
    text-transform: uppercase;
    font-size: 0.75rem;
    letter-spacing: 0.08em;
  }

  .tool-summary {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    margin-bottom: 0.6rem;
  }

  .status {
    padding: 0.12rem 0.5rem;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-muted);
    font-size: 0.78rem;
    text-transform: capitalize;
  }

  code {
    display: inline-block;
    margin-bottom: 0.6rem;
    padding: 0.35rem 0.55rem;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.06);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .content {
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>

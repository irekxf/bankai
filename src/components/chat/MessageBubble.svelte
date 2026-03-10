<script lang="ts">
  import type { ChatMessage } from "../../lib/stores/messages";

  let { message }: { message: ChatMessage } = $props();
</script>

<article class:assistant={message.role === "assistant"} class:user={message.role === "user"}>
  <header>{message.role}</header>
  <p>{message.content}</p>
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

  header,
  p {
    margin: 0;
  }

  header {
    margin-bottom: 0.4rem;
    color: var(--text-muted);
    text-transform: uppercase;
    font-size: 0.75rem;
    letter-spacing: 0.08em;
  }
</style>

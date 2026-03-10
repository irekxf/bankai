<script lang="ts">
  import { messages } from "../../lib/stores/messages";
  import ApprovalPanel from "./ApprovalPanel.svelte";
  import ChatInput from "./ChatInput.svelte";
  import MessageBubble from "./MessageBubble.svelte";
</script>

<section class="chat-shell">
  <div class="messages">
    {#each $messages as message}
      <MessageBubble {message} />
    {/each}
  </div>

  <ApprovalPanel />
  <ChatInput />
</section>

<style>
  .chat-shell,
  .messages {
    display: grid;
    gap: 1rem;
  }

  .messages {
    align-content: start;
    min-height: 24rem;
  }
</style>

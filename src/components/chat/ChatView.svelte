<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { formatToolPreview } from "../../lib/chat/toolPreview";
  import { agentStatus, pendingToolCalls, type PendingToolCall } from "../../lib/stores/agent";
  import {
    appendAssistantDelta,
    appendMessage,
    loadSessionMessages,
    messages
  } from "../../lib/stores/messages";
  import { currentSessionId } from "../../lib/stores/sessions";
  import {
    onAgentError,
    onAgentMessageDelta,
    onAgentStatus,
    onAgentToolCallRequest,
    onAgentToolCallResult
  } from "../../lib/tauri/events";
  import { listPendingToolCalls, type PendingToolCallDto } from "../../lib/tauri/commands";
  import ApprovalPanel from "./ApprovalPanel.svelte";
  import ChatInput from "./ChatInput.svelte";
  import MessageBubble from "./MessageBubble.svelte";

  $effect(() => {
    void loadSessionMessages($currentSessionId ?? "");
  });

  onMount(() => {
    const unlisteners: Array<() => void> = [];

    void loadPendingApprovals();

    void onAgentStatus((payload) => {
      agentStatus.set(payload.status);
    }).then((unlisten) => unlisteners.push(unlisten));

    void onAgentMessageDelta((payload) => {
      if (payload.sessionId !== get(currentSessionId)) {
        return;
      }

      appendAssistantDelta(payload.messageId, payload.delta);
    }).then((unlisten) => unlisteners.push(unlisten));

    void onAgentError((payload) => {
      appendMessage({
        id: crypto.randomUUID(),
        role: "assistant",
        content: `Agent error: ${payload.message}`,
        createdAt: new Date().toISOString()
      });
    }).then((unlisten) => unlisteners.push(unlisten));

    void onAgentToolCallRequest((payload) => {
      upsertPendingApproval({
        id: payload.id,
        sessionId: payload.sessionId,
        name: payload.toolName,
        argumentsPreview: formatToolPreview(payload.toolName, payload.argumentsJson)
      });

      if (payload.sessionId === get(currentSessionId)) {
        void loadSessionMessages(payload.sessionId);
      }
    }).then((unlisten) => unlisteners.push(unlisten));

    void onAgentToolCallResult((payload) => {
      pendingToolCalls.update((items) => items.filter((item) => item.id !== payload.callId));

      if (payload.sessionId === get(currentSessionId)) {
        void loadSessionMessages(payload.sessionId);
      }
    }).then((unlisten) => unlisteners.push(unlisten));

    return () => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  });

  async function loadPendingApprovals(): Promise<void> {
    const items = await listPendingToolCalls();
    pendingToolCalls.set(items.map(mapPendingApproval));
  }

  function mapPendingApproval(item: PendingToolCallDto): PendingToolCall {
    return {
      id: item.id,
      sessionId: item.sessionId,
      name: item.toolName,
      argumentsPreview: formatToolPreview(item.toolName, item.argumentsJson)
    };
  }

  function upsertPendingApproval(next: PendingToolCall): void {
    pendingToolCalls.update((items) => {
      if (items.some((item) => item.id === next.id)) {
        return items;
      }

      return [...items, next];
    });
  }
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

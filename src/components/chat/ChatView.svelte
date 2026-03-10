<script lang="ts">
  import { onMount } from "svelte";
  import { agentStatus, pendingToolCalls } from "../../lib/stores/agent";
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

  function formatToolPreview(toolName: string, argumentsJson: string): string {
    try {
      const payload = JSON.parse(argumentsJson) as Record<string, string>;
      if (toolName === "shell") {
        return payload.command ?? argumentsJson;
      }
      if (toolName === "filesystem") {
        const action = payload.action ?? "unknown";
        const path = payload.path ?? "";
        return `${action} ${path}`.trim();
      }
    } catch {
      return argumentsJson;
    }

    return argumentsJson;
  }

  $effect(() => {
    if ($currentSessionId) {
      void loadSessionMessages($currentSessionId);
    }
  });

  onMount(() => {
    const unlisteners: Array<() => void> = [];

    void listPendingToolCalls().then((items: PendingToolCallDto[]) => {
      pendingToolCalls.set(
        items.map((item: PendingToolCallDto) => ({
          id: item.id,
          sessionId: item.sessionId,
          name: item.toolName,
          argumentsPreview: formatToolPreview(item.toolName, item.argumentsJson)
        }))
      );
    });

    void onAgentStatus((payload) => {
      agentStatus.set(payload.status);
    }).then((unlisten) => unlisteners.push(unlisten));

    void onAgentMessageDelta((payload) => {
      appendAssistantDelta(payload.messageId, payload.delta);
    }).then((unlisten) => unlisteners.push(unlisten));

    void onAgentError((payload) => {
      appendMessage({
        id: crypto.randomUUID(),
        role: "assistant",
        content: `Ошибка агента: ${payload.message}`,
        createdAt: new Date().toISOString()
      });
    }).then((unlisten) => unlisteners.push(unlisten));

    void onAgentToolCallRequest((payload) => {
      pendingToolCalls.update((items) => [
        ...items,
        {
          id: payload.id,
          sessionId: payload.sessionId,
          name: payload.toolName,
          argumentsPreview: formatToolPreview(payload.toolName, payload.argumentsJson)
        }
      ]);
    }).then((unlisten) => unlisteners.push(unlisten));

    void onAgentToolCallResult((payload) => {
      if (payload.sessionId === $currentSessionId) {
        void loadSessionMessages(payload.sessionId);
      }
    }).then((unlisten) => unlisteners.push(unlisten));

    return () => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  });
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

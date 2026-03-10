<script lang="ts">
  import { agentStatus } from "../../lib/stores/agent";
  import { appendMessage } from "../../lib/stores/messages";
  import { currentSessionId } from "../../lib/stores/sessions";
  import { sendMessage } from "../../lib/tauri/commands";

  let draft = $state("");

  async function submitPrompt(event: SubmitEvent) {
    event.preventDefault();

    const prompt = draft.trim();
    if (!prompt || $agentStatus !== "idle") {
      return;
    }

    appendMessage({
      id: crypto.randomUUID(),
      role: "user",
      content: prompt,
      createdAt: new Date().toISOString()
    });

    draft = "";

    try {
      await sendMessage({
        sessionId: $currentSessionId,
        text: prompt
      });
    } catch (error) {
      appendMessage({
        id: crypto.randomUUID(),
        role: "assistant",
        content: `Не удалось отправить сообщение: ${error instanceof Error ? error.message : "unknown error"}`,
        createdAt: new Date().toISOString()
      });
    }
  }
</script>

<form class="composer" onsubmit={submitPrompt}>
  <textarea bind:value={draft} rows="4" placeholder="Опиши задачу для агента..."></textarea>
  <div class="actions">
    <small>
      {$agentStatus === "idle" ? "Запрос уйдёт в OpenAI API." : "Агент сейчас обрабатывает запрос."}
    </small>
    <md-filled-button disabled={$agentStatus !== "idle"} type="submit">Send</md-filled-button>
  </div>
</form>

<style>
  .composer {
    display: grid;
    gap: 0.75rem;
    padding: 1rem;
    background: var(--surface-strong);
    border: 1px solid var(--border);
    border-radius: 24px;
  }

  textarea {
    resize: vertical;
    min-height: 7rem;
    border-radius: 18px;
    border: 1px solid var(--border);
    background: rgba(7, 11, 26, 0.8);
    color: inherit;
    padding: 0.9rem 1rem;
  }

  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  small {
    color: var(--text-muted);
  }
</style>

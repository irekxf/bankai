import { writable } from "svelte/store";
import { getSessionMessages } from "../tauri/commands";

export type MessageRole = "user" | "assistant" | "tool";

export interface ChatMessage {
  id: string;
  role: MessageRole;
  content: string;
  createdAt: string;
}

export const messages = writable<ChatMessage[]>([
  {
    id: "welcome",
    role: "assistant",
    content:
      "Каркас Bankai поднят. Сейчас MVP ориентирован на ChatGPT/OpenAI API, следующий шаг: живой provider backend и keyring.",
    createdAt: new Date().toISOString()
  }
]);

export async function loadSessionMessages(sessionId: string): Promise<void> {
  if (!sessionId) {
    messages.set([]);
    return;
  }

  const records = await getSessionMessages(sessionId);
  messages.set(
    records.map((record) => ({
      id: record.id,
      role: record.role === "system" ? "assistant" : record.role === "tool" ? "tool" : record.role,
      content: record.content,
      createdAt: record.createdAt
    }))
  );
}

export function appendMessage(message: ChatMessage): void {
  messages.update((items) => [...items, message]);
}

export function appendAssistantDelta(messageId: string, delta: string): void {
  messages.update((items) => {
    const existing = items.find((item) => item.id === messageId);
    if (existing) {
      return items.map((item) =>
        item.id === messageId ? { ...item, content: `${item.content}${delta}` } : item
      );
    }

    return [
      ...items,
      {
        id: messageId,
        role: "assistant",
        content: delta,
        createdAt: new Date().toISOString()
      }
    ];
  });
}

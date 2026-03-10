import { writable } from "svelte/store";

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

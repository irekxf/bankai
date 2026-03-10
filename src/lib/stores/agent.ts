import { writable } from "svelte/store";

export type AgentStatus = "idle" | "thinking" | "awaiting_approval" | "executing_tool";

export interface PendingToolCall {
  id: string;
  name: string;
  argumentsPreview: string;
}

export const agentStatus = writable<AgentStatus>("idle");
export const pendingToolCalls = writable<PendingToolCall[]>([
  {
    id: "draft-shell-call",
    name: "shell",
    argumentsPreview: "bun install"
  }
]);

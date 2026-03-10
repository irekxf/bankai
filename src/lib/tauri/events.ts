import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";

export interface AgentStatusEvent {
  status: "idle" | "thinking" | "awaiting_approval" | "executing_tool";
}

export function onAgentStatus(handler: (payload: AgentStatusEvent) => void): Promise<UnlistenFn> {
  return listen<AgentStatusEvent>("agent:status", (event) => handler(event.payload));
}

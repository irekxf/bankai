import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";

export interface AgentStatusEvent {
  status: "idle" | "thinking" | "awaiting_approval" | "executing_tool";
}

export interface AgentMessageDeltaEvent {
  sessionId: string;
  messageId: string;
  delta: string;
}

export interface AgentErrorEvent {
  message: string;
}

export interface AgentToolCallRequestEvent {
  id: string;
  sessionId: string;
  toolName: string;
  argumentsJson: string;
}

export interface AgentToolCallResultEvent {
  callId: string;
  sessionId: string;
  result: string;
}

export function onAgentStatus(handler: (payload: AgentStatusEvent) => void): Promise<UnlistenFn> {
  return listen<AgentStatusEvent>("agent:status", (event) => handler(event.payload));
}

export function onAgentMessageDelta(
  handler: (payload: AgentMessageDeltaEvent) => void
): Promise<UnlistenFn> {
  return listen<AgentMessageDeltaEvent>("agent:message-delta", (event) => handler(event.payload));
}

export function onAgentError(handler: (payload: AgentErrorEvent) => void): Promise<UnlistenFn> {
  return listen<AgentErrorEvent>("agent:error", (event) => handler(event.payload));
}

export function onAgentToolCallRequest(
  handler: (payload: AgentToolCallRequestEvent) => void
): Promise<UnlistenFn> {
  return listen<AgentToolCallRequestEvent>("agent:tool-call-request", (event) =>
    handler(event.payload)
  );
}

export function onAgentToolCallResult(
  handler: (payload: AgentToolCallResultEvent) => void
): Promise<UnlistenFn> {
  return listen<AgentToolCallResultEvent>("agent:tool-call-result", (event) =>
    handler(event.payload)
  );
}

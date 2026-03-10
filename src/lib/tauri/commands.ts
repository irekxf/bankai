import { invoke } from "@tauri-apps/api/core";

export interface SendMessagePayload {
  sessionId: string;
  text: string;
}

export interface SessionDto {
  id: string;
  title: string;
}

export async function sendMessage(payload: SendMessagePayload): Promise<void> {
  await invoke("send_message", { ...payload });
}

export async function listSessions(): Promise<SessionDto[]> {
  return invoke<SessionDto[]>("list_sessions");
}

export async function approveToolCall(callId: string): Promise<void> {
  await invoke("approve_tool_call", { callId });
}

export async function rejectToolCall(callId: string, reason?: string): Promise<void> {
  await invoke("reject_tool_call", { callId, reason });
}

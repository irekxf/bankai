import { invoke } from "@tauri-apps/api/core";

export interface SendMessagePayload {
  sessionId: string;
  text: string;
}

export interface SessionDto {
  id: string;
  title: string;
  updatedAt: string;
}

export interface MessageDto {
  id: string;
  sessionId: string;
  role: "user" | "assistant" | "system" | "tool";
  content: string;
  createdAt: string;
}

export interface ProviderConfigDto {
  provider: "openai";
  displayName: string;
  baseUrl: string;
  model: string;
  apiKeyStatus: "missing" | "configured";
}

export interface SaveProviderConfigPayload {
  provider: "openai";
  displayName: string;
  baseUrl: string;
  model: string;
  apiKey?: string;
}

export async function sendMessage(payload: SendMessagePayload): Promise<void> {
  await invoke("send_message", { ...payload });
}

export async function listSessions(): Promise<SessionDto[]> {
  return invoke<SessionDto[]>("list_sessions");
}

export async function createSession(title?: string): Promise<SessionDto> {
  return invoke<SessionDto>("create_session", { title });
}

export async function getSessionMessages(sessionId: string): Promise<MessageDto[]> {
  return invoke<MessageDto[]>("get_session_messages", { sessionId });
}

export async function approveToolCall(callId: string): Promise<void> {
  await invoke("approve_tool_call", { callId });
}

export async function rejectToolCall(callId: string, reason?: string): Promise<void> {
  await invoke("reject_tool_call", { callId, reason });
}

export async function getProviderConfig(): Promise<ProviderConfigDto> {
  return invoke<ProviderConfigDto>("get_provider_config");
}

export async function saveProviderConfig(
  payload: SaveProviderConfigPayload
): Promise<ProviderConfigDto> {
  return invoke<ProviderConfigDto>("save_provider_config_command", { config: payload });
}

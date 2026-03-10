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

export type ToolMessageKindDto = "request" | "result" | "rejection";
export type ToolCallStatusDto = "pending" | "approved" | "rejected" | "completed";

export interface MessageDto {
  id: string;
  sessionId: string;
  role: "user" | "assistant" | "system" | "tool";
  content: string;
  toolCallId?: string;
  toolMessageKind?: ToolMessageKindDto;
  toolName?: string;
  toolStatus?: ToolCallStatusDto;
  toolArgumentsJson?: string;
  toolResultText?: string;
  toolRejectionReason?: string;
  createdAt: string;
}

export interface ProviderStatusDto {
  provider: "openai";
  displayName: string;
  baseUrl: string;
  model: string;
  preferredAuth: "auto" | "api_key" | "oauth";
  apiKeyStatus: "missing" | "configured";
  oauthLoggedIn: boolean;
  oauthAuthMode?: string;
  oauthAccountId?: string;
  oauthExpiresAt?: number;
  activeAuth: "api_key" | "oauth" | "none";
  authReady: boolean;
  canLoadModels: boolean;
  authMessage: string;
}

export interface PendingToolCallDto {
  id: string;
  sessionId: string;
  responseId?: string;
  toolCallId?: string;
  toolName: string;
  argumentsJson: string;
}

export interface ToolRegistryEntryDto {
  name: string;
  displayName: string;
  description: string;
  enabled: boolean;
  requiresApproval: boolean;
}

export interface SaveProviderConfigPayload {
  provider: "openai";
  displayName: string;
  baseUrl: string;
  model: string;
  preferredAuth?: "auto" | "api_key" | "oauth";
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

export async function listPendingToolCalls(): Promise<PendingToolCallDto[]> {
  return invoke<PendingToolCallDto[]>("list_pending_tool_calls");
}

export async function listTools(): Promise<ToolRegistryEntryDto[]> {
  return invoke<ToolRegistryEntryDto[]>("list_tools");
}

export async function setToolEnabled(
  toolName: string,
  enabled: boolean
): Promise<ToolRegistryEntryDto> {
  return invoke<ToolRegistryEntryDto>("set_tool_enabled_command", { toolName, enabled });
}

export async function startOAuthLogin(): Promise<ProviderStatusDto> {
  return invoke<ProviderStatusDto>("start_oauth_login_command");
}

export async function clearOAuthSession(): Promise<ProviderStatusDto> {
  return invoke<ProviderStatusDto>("clear_oauth_session_command");
}

export async function approveToolCall(callId: string): Promise<void> {
  await invoke("approve_tool_call", { callId });
}

export async function rejectToolCall(callId: string, reason?: string): Promise<void> {
  await invoke("reject_tool_call", { callId, reason });
}

export async function getProviderStatus(): Promise<ProviderStatusDto> {
  return invoke<ProviderStatusDto>("get_provider_status_command");
}

export async function listProviderModels(): Promise<string[]> {
  return invoke<string[]>("list_provider_models");
}

export async function saveProviderConfig(
  payload: SaveProviderConfigPayload
): Promise<ProviderStatusDto> {
  return invoke<ProviderStatusDto>("save_provider_config_command", { config: payload });
}

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
  preferredAuth: "auto" | "api_key" | "oauth";
  apiKeyStatus: "missing" | "configured";
}

export interface PendingToolCallDto {
  id: string;
  sessionId: string;
  responseId?: string;
  toolCallId?: string;
  toolName: string;
  argumentsJson: string;
}

export interface OAuthStatusDto {
  loggedIn: boolean;
  authMode?: string;
  accountId?: string;
  expiresAt?: number;
}

export interface ProviderModelsDto {
  models: string[];
}

export interface ToolInfoDto {
  name: string;
  description: string;
  requiresApproval: boolean;
  enabled: boolean;
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

export async function getOAuthStatus(): Promise<OAuthStatusDto> {
  return invoke<OAuthStatusDto>("get_oauth_status_command");
}

export async function startOAuthLogin(): Promise<OAuthStatusDto> {
  return invoke<OAuthStatusDto>("start_oauth_login_command");
}

export async function clearOAuthSession(): Promise<void> {
  return invoke("clear_oauth_session_command");
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

export async function listProviderModels(): Promise<string[]> {
  return invoke<string[]>("list_provider_models");
}

export async function saveProviderConfig(
  payload: SaveProviderConfigPayload
): Promise<ProviderConfigDto> {
  return invoke<ProviderConfigDto>("save_provider_config_command", { config: payload });
}

export async function listTools(): Promise<ToolInfoDto[]> {
  return invoke<ToolInfoDto[]>("list_tools");
}

export async function setToolEnabled(name: string, enabled: boolean): Promise<ToolInfoDto[]> {
  return invoke<ToolInfoDto[]>("set_tool_enabled_command", { name, enabled });
}

import { writable } from "svelte/store";
import { getSessionMessages, type MessageDto } from "../tauri/commands";

export type MessageRole = "user" | "assistant" | "tool";
export type ToolMessageKind = "request" | "result" | "rejection";
export type ToolCallStatus = "pending" | "approved" | "rejected" | "completed";

export interface ChatToolCall {
  id: string;
  kind?: ToolMessageKind;
  name: string;
  status?: ToolCallStatus;
  argumentsJson?: string;
  resultText?: string;
  rejectionReason?: string;
}

export interface ChatMessage {
  id: string;
  role: MessageRole;
  content: string;
  toolCall?: ChatToolCall;
  createdAt: string;
}

export const messages = writable<ChatMessage[]>([
  {
    id: "welcome",
    role: "assistant",
    content:
      "Bankai is ready. This build already supports chat sessions, provider setup, and approval-based tools.",
    createdAt: new Date().toISOString()
  }
]);

export async function loadSessionMessages(sessionId: string): Promise<void> {
  if (!sessionId) {
    messages.set([]);
    return;
  }

  const records = await getSessionMessages(sessionId);
  messages.set(records.map(mapMessageRecord));
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

const TOOL_MESSAGE_KINDS: ToolMessageKind[] = ["request", "result", "rejection"];
const TOOL_CALL_STATUSES: ToolCallStatus[] = ["pending", "approved", "rejected", "completed"];

function mapMessageRecord(record: MessageDto): ChatMessage {
  return {
    id: record.id,
    role: normalizeRole(record.role),
    content: record.content,
    toolCall: mapToolCall(record),
    createdAt: record.createdAt
  };
}

function normalizeRole(role: MessageDto["role"]): MessageRole {
  if (role === "tool") {
    return "tool";
  }

  return role === "system" ? "assistant" : role;
}

function mapToolCall(record: MessageDto): ChatToolCall | undefined {
  if (!record.toolCallId || !record.toolName) {
    return undefined;
  }

  const kind = normalizeToolMessageKind(record.toolMessageKind);
  return {
    id: record.toolCallId,
    kind,
    name: record.toolName,
    status: normalizeToolCallStatus(record.toolStatus),
    argumentsJson: record.toolArgumentsJson,
    resultText: record.toolResultText ?? (kind === "result" ? record.content : undefined),
    rejectionReason:
      record.toolRejectionReason ?? (kind === "rejection" ? record.content : undefined)
  };
}

function normalizeToolMessageKind(value: string | undefined): ToolMessageKind | undefined {
  return TOOL_MESSAGE_KINDS.find((candidate) => candidate === value);
}

function normalizeToolCallStatus(value: string | undefined): ToolCallStatus | undefined {
  return TOOL_CALL_STATUSES.find((candidate) => candidate === value);
}

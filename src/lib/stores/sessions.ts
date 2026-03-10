import { writable } from "svelte/store";
import { createSession, listSessions } from "../tauri/commands";

export interface SessionSummary {
  id: string;
  title: string;
  updatedAt: string;
}

const initialSessions: SessionSummary[] = [];

export const sessions = writable<SessionSummary[]>(initialSessions);
export const currentSessionId = writable<string>("");

export async function refreshSessions(): Promise<void> {
  const records = await listSessions();
  sessions.set(records);
  currentSessionId.update((current) => current || records[0]?.id || "");
}

export async function createLocalSession(title?: string): Promise<void> {
  const created = await createSession(title);
  await refreshSessions();
  currentSessionId.set(created.id);
}

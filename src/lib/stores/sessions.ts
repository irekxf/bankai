import { writable } from "svelte/store";

export interface SessionSummary {
  id: string;
  title: string;
  updatedAt: string;
}

const initialSessions: SessionSummary[] = [
  {
    id: "local-draft",
    title: "ChatGPT agent bootstrap",
    updatedAt: new Date().toISOString()
  }
];

export const sessions = writable<SessionSummary[]>(initialSessions);
export const currentSessionId = writable<string>(initialSessions[0].id);

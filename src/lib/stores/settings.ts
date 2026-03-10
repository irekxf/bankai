import { writable } from "svelte/store";

export interface ProviderSettings {
  provider: "openai";
  displayName: string;
  baseUrl: string;
  model: string;
  apiKeyStatus: "missing" | "configured";
  apiKeyDraft: string;
  saveState: "idle" | "saving" | "saved" | "error";
}

export const providerSettings = writable<ProviderSettings>({
  provider: "openai",
  displayName: "ChatGPT / OpenAI",
  baseUrl: "https://api.openai.com/v1",
  model: "gpt-4.1",
  apiKeyStatus: "missing",
  apiKeyDraft: "",
  saveState: "idle"
});

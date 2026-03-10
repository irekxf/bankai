import { writable } from "svelte/store";

export interface ProviderSettings {
  provider: "openai";
  displayName: string;
  baseUrl: string;
  model: string;
  availableModels: string[];
  recommendedDefaultModel?: string;
  recommendedFastModel?: string;
  recommendedReasoningModel?: string;
  recommendedCodingModel?: string;
  modelsState: "idle" | "loading" | "loaded" | "error";
  apiKeyStatus: "missing" | "configured";
  preferredAuth: "oauth" | "api_key" | "auto";
  apiKeyDraft: string;
  saveState: "idle" | "saving" | "saved" | "error";
}

export const providerSettings = writable<ProviderSettings>({
  provider: "openai",
  displayName: "ChatGPT / OpenAI",
  baseUrl: "https://api.openai.com/v1",
  model: "gpt-4.1",
  availableModels: [],
  recommendedDefaultModel: undefined,
  recommendedFastModel: undefined,
  recommendedReasoningModel: undefined,
  recommendedCodingModel: undefined,
  modelsState: "idle",
  apiKeyStatus: "missing",
  preferredAuth: "auto",
  apiKeyDraft: "",
  saveState: "idle"
});

import { writable } from "svelte/store";

export type ProviderModelsState = "idle" | "loading" | "loaded" | "error";
export type ProviderSaveState = "idle" | "saving" | "saved" | "error";

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
  modelsState: ProviderModelsState;
  modelsError: string | null;
  apiKeyStatus: "missing" | "configured";
  preferredAuth: "oauth" | "api_key" | "auto";
  activeAuth: "api_key" | "oauth" | "none";
  authReady: boolean;
  authMessage: string;
  canLoadModels: boolean;
  apiKeyDraft: string;
  saveState: ProviderSaveState;
  saveError: string | null;
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
  modelsError: null,
  apiKeyStatus: "missing",
  preferredAuth: "auto",
  activeAuth: "none",
  authReady: false,
  authMessage:
    "Choose either OAuth or API key. Until one is configured, the provider cannot load models or send requests.",
  canLoadModels: false,
  apiKeyDraft: "",
  saveState: "idle",
  saveError: null
});

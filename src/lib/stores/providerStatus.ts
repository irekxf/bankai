import type { ProviderStatusDto } from "../tauri/commands";
import { oauthLoginState, oauthStatus } from "./auth";
import { providerSettings, type ProviderSaveState } from "./settings";

export interface ApplyProviderStatusOptions {
  clearApiKeyDraft?: boolean;
  saveState?: ProviderSaveState;
  saveError?: string | null;
}

export function applyProviderStatus(
  status: ProviderStatusDto,
  options: ApplyProviderStatusOptions = {}
): void {
  oauthStatus.set({
    loggedIn: status.oauthLoggedIn,
    authMode: status.oauthAuthMode,
    accountId: status.oauthAccountId,
    expiresAt: status.oauthExpiresAt
  });

  oauthLoginState.update((current) => {
    if (status.oauthLoggedIn) {
      return "connected";
    }

    return current === "connected" ? "idle" : current;
  });

  providerSettings.update((current) => ({
    ...current,
    provider: status.provider,
    displayName: status.displayName,
    baseUrl: status.baseUrl,
    model: status.model,
    preferredAuth: status.preferredAuth,
    apiKeyStatus: status.apiKeyStatus,
    activeAuth: status.activeAuth,
    authReady: status.authReady,
    authMessage: status.authMessage,
    canLoadModels: status.canLoadModels,
    apiKeyDraft: options.clearApiKeyDraft ? "" : current.apiKeyDraft,
    saveState: options.saveState !== undefined ? options.saveState : current.saveState,
    saveError: options.saveError !== undefined ? options.saveError : current.saveError
  }));
}

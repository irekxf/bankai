import { writable } from "svelte/store";

export interface OAuthStatus {
  loggedIn: boolean;
  authMode?: string;
  accountId?: string;
  expiresAt?: number;
}

export type OAuthLoginState = "idle" | "launching" | "connected" | "error" | "dismissed";

export const oauthStatus = writable<OAuthStatus>({
  loggedIn: false
});

export const oauthLoginState = writable<OAuthLoginState>("idle");

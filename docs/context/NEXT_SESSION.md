# Next Session

Last updated: 2026-03-10

## Best Next Task

Stabilize provider and auth UX.

## Why This First

- It is the first user-facing entry path.
- The current repository already supports both OAuth and API key flows, but the UX still needs alignment.
- This work is upstream of model selection and send-message reliability.

## Start Here

Read these files first:

- [docs/tasks/BACKLOG.md](../tasks/BACKLOG.md)
- [src/components/settings/ProviderConfig.svelte](../../src/components/settings/ProviderConfig.svelte)
- [src/components/settings/OAuthOnboarding.svelte](../../src/components/settings/OAuthOnboarding.svelte)
- [src/lib/stores/auth.ts](../../src/lib/stores/auth.ts)
- [src/lib/stores/settings.ts](../../src/lib/stores/settings.ts)
- [src-tauri/src/settings.rs](../../src-tauri/src/settings.rs)
- [src-tauri/src/ipc/commands.rs](../../src-tauri/src/ipc/commands.rs)

## Desired Outcome

- One clear source of auth state in the UI.
- No contradictory OAuth vs API key copy.
- Explicit empty/error states when neither auth path is usable.
- Model list and send flow behave predictably with the chosen auth mode.

## Watch Outs

- Do not remove OAuth support.
- Do not remove API key support.
- Keep documentation aligned with `project.md`.
- Be careful with simultaneous edits to shared files like `src-tauri/src/ipc/commands.rs`.

## Before Ending The Next Session

- Update this file with the next best task.
- Add a handoff note under `docs/handoffs/`.
- Update `docs/tasks/BACKLOG.md` statuses.

# Next Session

Last updated: 2026-03-10

## Best Next Task

Start Task B: add a tool registry and tool toggles.

## Why This Next

- The auth/provider path is now in a much better place and passes frontend/Rust checks.
- Tools are still hardcoded as always enabled, which blocks a core product capability.
- This is the next roadmap item after auth UX stabilization.

## Start Here

Read these files first:

- [docs/tasks/BACKLOG.md](../tasks/BACKLOG.md)
- [src-tauri/src/tools/mod.rs](../../src-tauri/src/tools/mod.rs)
- [src-tauri/src/providers/openai.rs](../../src-tauri/src/providers/openai.rs)
- [src-tauri/src/db/mod.rs](../../src-tauri/src/db/mod.rs)
- [src-tauri/src/ipc/commands.rs](../../src-tauri/src/ipc/commands.rs)
- [src/lib/tauri/commands.ts](../../src/lib/tauri/commands.ts)
- [src/lib/stores/settings.ts](../../src/lib/stores/settings.ts)

## Current Focus

- define a backend tool registry
- persist enabled/disabled state
- expose toggles in the frontend
- make OpenAI tool definitions respect enabled state

## Desired Outcome

- Tools are listed from backend instead of being implicitly hardcoded.
- The user can enable or disable tools in the UI.
- Disabled tools are excluded from model requests.
- Approval-required tools still keep their existing safety behavior.

## Watch Outs

- Do not weaken approval requirements for shell or filesystem writes.
- Keep persistence simple enough for one PR.
- Be careful with shared files like `src-tauri/src/ipc/commands.rs` and provider wiring.

## Before Ending The Next Session

- Update this file with the next best task.
- Add a handoff note under `docs/handoffs/`.
- Update `docs/tasks/BACKLOG.md` statuses.

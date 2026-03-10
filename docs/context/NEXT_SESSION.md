# Next Session

Last updated: 2026-03-10

## Best Next Task

Start Task D: improve the agent loop for real multi-step runs.

## Why This Next

- Auth UX, tool registry, and richer persistence are now all in place.
- The main product gap is still that the agent stops after a single tool continuation.
- Multi-step looping is safer now because tool history survives restart and can be inspected from the UI.

## Start Here

Read these files first:

- [docs/tasks/BACKLOG.md](../tasks/BACKLOG.md)
- [src-tauri/src/agent/loop.rs](../../src-tauri/src/agent/loop.rs)
- [src-tauri/src/ipc/commands.rs](../../src-tauri/src/ipc/commands.rs)
- [src-tauri/src/providers/openai.rs](../../src-tauri/src/providers/openai.rs)
- [src-tauri/src/db/messages.rs](../../src-tauri/src/db/messages.rs)
- [src-tauri/src/db/tool_calls.rs](../../src-tauri/src/db/tool_calls.rs)

## Current Focus

- support repeated tool-call -> approval -> continuation cycles until a final assistant answer is reached
- keep the approval queue and persisted timeline in sync during multi-step runs
- avoid regressing the current single-step happy path while expanding the loop

## Desired Outcome

- The agent can continue after one approved tool result and either finish or request another tool.
- Persisted messages and tool-call rows stay coherent during each loop step.
- Frontend still receives the same event shapes while the backend grows more capable.

## Watch Outs

- Keep loop termination conditions obvious so the app cannot spin forever.
- Be careful with provider response IDs and function-call IDs across continuation steps.
- Keep Task D scoped to loop behavior, not a full provider abstraction rewrite.

## Before Ending The Next Session

- Update this file with the next best task.
- Add a handoff note under `docs/handoffs/`.
- Update `docs/tasks/BACKLOG.md` statuses.

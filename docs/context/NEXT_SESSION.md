# Next Session

Last updated: 2026-03-10

## Best Next Task

Start Task C: persist richer message and tool-call data.

## Why This Next

- Auth UX and the tool registry are now both in place.
- Session replay and tool history are still thinner than the product direction expects.
- Tightening persistence will make the next agent-loop work safer.

## Start Here

Read these files first:

- [docs/tasks/BACKLOG.md](../tasks/BACKLOG.md)
- [src-tauri/src/db/messages.rs](../../src-tauri/src/db/messages.rs)
- [src-tauri/src/db/tool_calls.rs](../../src-tauri/src/db/tool_calls.rs)
- [src-tauri/src/db/mod.rs](../../src-tauri/src/db/mod.rs)
- [src-tauri/src/ipc/commands.rs](../../src-tauri/src/ipc/commands.rs)
- [src/lib/tauri/commands.ts](../../src/lib/tauri/commands.ts)

## Current Focus

- preserve structured tool-call metadata across restart
- keep assistant/tool timeline reconstructable from the database
- make returned DTOs explicit enough for future richer chat rendering

## Desired Outcome

- Completed and pending tool calls survive restart with enough metadata to replay the session meaningfully.
- Message and tool-call DTOs stay explicit and typed.
- Frontend still renders current history correctly while gaining room for richer activity views.

## Watch Outs

- Avoid breaking existing session history reads.
- Keep migrations simple and safe for an existing local SQLite database.
- Be careful with shared files like `src-tauri/src/ipc/commands.rs` and DTO definitions.

## Before Ending The Next Session

- Update this file with the next best task.
- Add a handoff note under `docs/handoffs/`.
- Update `docs/tasks/BACKLOG.md` statuses.

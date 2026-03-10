# Tool Persistence Handoff

Date: 2026-03-10
Branch: `codex/task-c-tool-persistence`
Task: Persist richer message and tool-call data

## Done

- Expanded SQLite persistence so `messages` can link to `tool_calls` with explicit `tool_message_kind`.
- Added safe local-column migrations for existing SQLite databases.
- Wrote tool request, result, and rejection activity into the chat timeline instead of keeping it only in memory.
- Returned richer message DTOs to the frontend and rendered structured tool activity in chat history.
- Reloaded session history on tool-call request events so pending approvals now appear in the transcript immediately.

## Current State

- Session replay can now reconstruct pending and completed tool activity after restart.
- Chat history shows tool request/result/rejection cards with tool name, status, and argument preview.
- Pending approvals still use the in-memory queue plus SQLite, but the agent loop itself is still single-step.

## Files Touched

- `src-tauri/src/db/mod.rs`
- `src-tauri/src/db/messages.rs`
- `src-tauri/src/db/tool_calls.rs`
- `src-tauri/src/agent/loop.rs`
- `src-tauri/src/ipc/commands.rs`
- `src/lib/tauri/commands.ts`
- `src/lib/stores/messages.ts`
- `src/lib/chat/toolPreview.ts`
- `src/components/chat/ChatView.svelte`
- `src/components/chat/MessageBubble.svelte`
- `docs/tasks/BACKLOG.md`
- `docs/context/CURRENT_STATE.md`
- `docs/context/NEXT_SESSION.md`

## Verification

- `cargo check` passed in `src-tauri` with only the existing Windows incremental-compilation cleanup warning.
- `C:\Users\i\.bun\bin\bun.exe run check` passed.
- `C:\Users\i\.bun\bin\bun.exe run build` passed with the existing Vite chunk-size warning.

## Remaining Risks

- If a tool execution fails after approval, the current loop still does not recover gracefully back into the pending queue.
- Some older UI copy still has encoding issues outside the files touched in this task.
- Multi-step continuation is not implemented yet, so a run still stops after one tool continuation.

## Next Recommended Step

- Start Task D and turn the current single continuation path into a bounded multi-step loop with clear stop conditions.

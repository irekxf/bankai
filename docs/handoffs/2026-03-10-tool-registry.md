# Handoff

Date: 2026-03-10
Branch: current workspace
Task: Tool registry and toggles

## Done

- Added a backend tool registry with static metadata for shell, filesystem, and browser tools.
- Added SQLite persistence for tool enabled state in `tool_settings`.
- Exposed `list_tools` and `set_tool_enabled_command` over Tauri IPC.
- Updated OpenAI request construction so only enabled tools are sent to the model.
- Added a frontend Tool Registry panel with enable/disable toggles.
- Verified Rust and frontend checks after the changes.

## Current State

- Task B is effectively complete.
- Tool toggles now work through the backend instead of being frontend-only switches.
- Disabled tools are excluded from provider requests, while existing approval flow remains intact.

## Files Touched

- `src-tauri/src/tools/mod.rs`
- `src-tauri/src/db/tools.rs`
- `src-tauri/src/db/mod.rs`
- `src-tauri/src/agent/loop.rs`
- `src-tauri/src/providers/openai.rs`
- `src-tauri/src/ipc/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/tauri/commands.ts`
- `src/components/settings/ToolRegistry.svelte`
- `src/routes/+page.svelte`
- `docs/tasks/BACKLOG.md`
- `docs/context/NEXT_SESSION.md`
- `docs/context/CURRENT_STATE.md`
- `docs/handoffs/2026-03-10-tool-registry.md`

## Verification

- `cargo check`
- `C:\Users\i\.bun\bin\bun.exe run check`
- `C:\Users\i\.bun\bin\bun.exe run build`

## Remaining Risks

- Tool activity persistence is still minimal and is the next major data-layer gap.
- Frontend bundle size warning from Vite still exists and is unrelated to this task.

## Next Recommended Step

- Start Task C and enrich message/tool-call persistence for restart-safe session replay.

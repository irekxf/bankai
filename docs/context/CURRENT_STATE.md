# Current State

Last updated: 2026-03-10

## Stage

Bankai is at the early MVP stage.
The repository already contains a working vertical slice, but core systems are still incomplete.

## What Works

- Tauri desktop shell is set up.
- Svelte frontend renders the main app layout.
- Chat UI, session list, provider settings, and OAuth onboarding exist.
- OpenAI provider integration is wired through the Rust backend.
- Provider/auth status now has a unified backend summary used by the frontend.
- Tool registry state is persisted in SQLite and can be toggled from the UI.
- User messages can trigger a model response or a tool-call approval flow.
- Pending tool calls are stored in SQLite.
- Tool request, result, and rejection activity is now written into the message timeline and linked to persisted `tool_calls`.
- Chat history now restores structured tool metadata from the database and renders it in the UI.
- Shell, filesystem, and browser tool foundations exist.

## Known Gaps

- Agent loop is not yet a full multi-step tool loop.
- Some UI strings show encoding issues and need cleanup.
- Local verification is environment-dependent because Bun/Tauri/Rust setup may differ by machine.

## Current Priorities

1. Improve agent loop for true multi-step runs.
2. Add baseline CI and PR workflow support.
3. Continue smoothing chat/history UX details.
4. Add richer desktop surfaces after core flows are stable.
5. Expand tooling and editor surfaces once the agent core is steadier.

## Important Constraints

- Approval flow is a core product feature.
- OAuth and API key are both valid auth modes.
- Keep PRs small and task-scoped.
- Prefer updating docs alongside implementation when code changes behavior.

## Verification Notes

On this machine at review time:

- `cargo check` passed in `src-tauri` with only an existing incremental-compilation cleanup warning from the local Windows filesystem.
- `C:\Users\i\.bun\bin\bun.exe run check` passed.
- `C:\Users\i\.bun\bin\bun.exe run build` passed, with the existing Vite chunk-size warning.

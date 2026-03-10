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
- Shell, filesystem, and browser tool foundations exist.

## Known Gaps

- Agent loop is not yet a full multi-step tool loop.
- Tool-call persistence is still minimal.
- Some UI strings show encoding issues and need cleanup.
- Local verification is environment-dependent because Bun/Tauri/Rust setup may differ by machine.

## Current Priorities

1. Persist richer message and tool-call data.
2. Improve agent loop for true multi-step runs.
3. Add baseline CI and PR workflow support.
4. Add richer desktop surfaces after core flows are stable.
5. Continue smoothing chat/history UX details.

## Important Constraints

- Approval flow is a core product feature.
- OAuth and API key are both valid auth modes.
- Keep PRs small and task-scoped.
- Prefer updating docs alongside implementation when code changes behavior.

## Verification Notes

On this machine at review time:

- `bun run check` could not be run because `bun` was unavailable in the environment.
- `cargo check` could not be run because the sandbox could not execute `rustc`.

That means current stage assessment is based on repository contents, not a full local build.

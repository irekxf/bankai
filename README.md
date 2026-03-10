# Bankai

Bankai is a native desktop app for AI agents with explicit user control over tool execution, model configuration, and approval flow.

The project is built with `Tauri v2`, `Svelte 5`, and `Rust`. The goal is to provide a desktop-first agent experience where the user can see what the model is doing, approve risky actions, and work with multiple tools and providers from one UI.

## Current stack

- Desktop: `Tauri v2`
- Frontend: `Svelte 5`
- Backend: `Rust + tokio`
- Provider integration: `async-openai`
- Storage: `SQLite + sqlx`
- Secrets: `keyring`
- UI components: `@material/web`

## What already exists

- chat UI with session list
- provider configuration UI
- OpenAI / ChatGPT integration
- OAuth and API key auth handling
- basic agent loop with tool-call approval
- shell, filesystem, and browser tool foundations
- SQLite persistence for sessions, messages, and pending tool calls

## Product direction

Bankai is not just a chat client. It is an agent desktop:
- the model can request tools
- the app exposes those requests in UI
- the user can approve or reject actions
- tool results feed back into the model
- the whole flow stays visible in one place

## Near-term roadmap

1. Stabilize provider and auth UX so OAuth and API key flows are consistent.
2. Add a real tool registry with enable/disable controls.
3. Improve persistence for tool-call metadata and session replay.
4. Extend the agent loop to support multiple sequential tool calls.
5. Add richer desktop surfaces like terminal and editor panes.

## Multi-agent development

The repository is intended to be developed by multiple coding agents in parallel.

Rules:
- `main` changes only through Pull Requests
- one task per branch
- branch naming:
  - `codex/<task-slug>`
  - `claude/<task-slug>`

See [docs/AGENT_COORDINATION.md](/D:/bankai/docs/AGENT_COORDINATION.md) for coordination rules and [docs/tasks/claude-next.md](/D:/bankai/docs/tasks/claude-next.md) for the current task queue.

## Local commands

```bash
bun run dev
bun run build
bun run check
```

Rust/Tauri build commands depend on local Tauri setup and platform requirements.

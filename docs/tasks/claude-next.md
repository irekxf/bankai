# Claude Task Queue

This file is the current handoff for Claude Code.

Work rule:
- Pick one task.
- Create a branch from latest `main` using `claude/<task-slug>`.
- Open a separate PR for each task.
- Do not combine multiple tasks in one PR unless the task explicitly says so.

## Priority Order

1. Task A: Stabilize provider and auth UX
2. Task B: Add tool registry and tool toggles
3. Task C: Persist richer message and tool call data
4. Task D: Improve agent loop for real multi-step runs
5. Task E: Add PR template and baseline CI checks

## Task A: Stabilize provider and auth UX

### Why

The project now supports both OAuth and API key flows. The current goal is not to remove one of them, but to make the combined UX consistent and understandable.

### Scope

Clean up provider configuration and auth-related UI and backend flow so the user can clearly understand:
- which auth mode is active
- whether credentials are usable
- how model listing and send-message depend on current auth state

Expected areas:
- `src/components/settings/ProviderConfig.svelte`
- `src/components/settings/OAuthOnboarding.svelte`
- `src/lib/stores/auth.ts`
- `src/lib/stores/settings.ts`
- `src/lib/tauri/commands.ts`
- `src-tauri/src/settings.rs`
- `src-tauri/src/ipc/commands.rs`

### Deliverable

- consistent auth state in UI
- no contradictory copy about OAuth vs API key
- clear failure states when neither flow is configured
- model selection remains usable after auth cleanup

### Acceptance

- frontend type check passes
- Rust build passes
- both OAuth and API key remain supported
- README / project docs are not contradicted by UI copy introduced in this PR

## Task B: Add tool registry and tool toggles

### Why

`project.md` expects Bankai to expose tools as configurable capabilities. Current app still hardcodes tool definitions on the provider side and does not offer real toggles in UI.

### Scope

Implement a minimal tool registry:
- backend command to list tools
- backend command to toggle tools on/off
- persistence for enabled state
- frontend UI to view and toggle tools

Expected areas:
- `src-tauri/src/tools`
- `src-tauri/src/ipc/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/db`
- `src/lib/tauri/commands.ts`
- `src/lib/stores/settings.ts`
- new sidebar/settings component if needed

### Deliverable

- tools are not hardcoded as always-enabled
- frontend can fetch available tools and update enabled state
- agent request sends only enabled tools to OpenAI

### Acceptance

- shell remains approval-required
- filesystem write remains approval-required
- toggled-off tools are excluded from model tool definitions
- pending tool approvals still work for enabled tools

## Task C: Persist richer message and tool call data

### Why

Current persistence is enough for a rough demo, but it does not fully match the product needs for replaying sessions and understanding tool activity after restart.

### Scope

Improve database schema and CRUD so messages and tool calls preserve structured metadata.

Target outcomes:
- assistant messages can preserve tool call metadata
- tool call rows store arguments, result, status, and linkages consistently
- session reload reconstructs tool activity without relying only on transient in-memory state

Suggested areas:
- `src-tauri/src/db/messages.rs`
- `src-tauri/src/db/tool_calls.rs`
- `src-tauri/src/db/mod.rs`
- related DTOs used by frontend

### Deliverable

- schema changes and migration path
- updated Rust records and queries
- frontend still renders existing chat history correctly

### Acceptance

- restarting the app does not lose pending/completed tool-call history for existing sessions
- returned DTOs are explicit and typed
- no `any` introduced on frontend

## Task D: Improve agent loop for real multi-step runs

### Why

Current loop handles one user message and at most one tool continuation. The product vision expects a proper loop where the model can request multiple tools sequentially before finalizing a response.

### Scope

Refactor the backend loop so tool execution can continue through more than one round when the model asks for multiple sequential tools.

Expected areas:
- `src-tauri/src/agent/loop.rs`
- `src-tauri/src/ipc/commands.rs`
- `src-tauri/src/providers/openai.rs`
- related DB writes

### Deliverable

- agent can continue after first tool result and handle later tool calls if returned
- status events remain correct: `thinking`, `awaiting_approval`, `executing_tool`, `idle`
- error path is explicit and recoverable

### Acceptance

- no duplicate assistant message rows for one final turn
- each tool call is persisted before approval
- session timeline remains readable on frontend

## Task E: Add PR template and baseline CI checks

### Why

Multiple agents will keep opening PRs. The repository needs a minimum review and verification baseline so PR quality does not depend on memory.

### Scope

Add lightweight repository workflow infrastructure:
- GitHub PR template
- baseline GitHub Actions workflow for frontend checks
- if practical, add Rust build or check step that is realistic for the repo state

Expected areas:
- `.github/PULL_REQUEST_TEMPLATE.md`
- `.github/workflows/*`
- optional small docs updates if commands or expectations need to be documented

### Deliverable

- PRs open with a consistent structure
- at least one CI workflow runs on pull requests
- checks are small and realistic for the current codebase

### Acceptance

- workflow is not overengineered
- checks match commands that can reasonably pass in this repo
- PR template aligns with `docs/AGENT_COORDINATION.md`

## Suggested PR Order

Recommended execution order:
1. Task A
2. Task B
3. Task C
4. Task D
5. Task E

Reason:
- Task A stabilizes the entry path users hit first
- Task B adds missing configurability for tools
- Task C stabilizes persistence before deeper loop work
- Task D is safer after the model and tool data layer is cleaner
- Task E is useful at any point, but it is best informed by the checks the repo can actually support

## PR Template For Claude

Use this structure in each PR description:

```md
## Summary
- what changed

## Why
- product or architecture reason

## Verification
- commands run
- results

## Risks / Follow-ups
- remaining gaps
```

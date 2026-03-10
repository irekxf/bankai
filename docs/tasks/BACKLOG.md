# Backlog

Last updated: 2026-03-10

## Status Legend

- `todo`: not started
- `in_progress`: currently being worked on
- `blocked`: cannot continue without a decision or dependency
- `done`: completed and merged or ready to merge

## Active Priority Queue

| ID | Task | Status | Why it matters | Main areas |
| --- | --- | --- | --- | --- |
| A | Stabilize provider and auth UX | todo | Make auth state understandable and reliable | `src/components/settings`, `src/lib/stores`, `src-tauri/src/settings.rs`, `src-tauri/src/ipc/commands.rs` |
| B | Add tool registry and tool toggles | todo | Stop treating all tools as always enabled | `src-tauri/src/tools`, `src-tauri/src/db`, frontend settings UI |
| C | Persist richer message and tool-call data | todo | Make session replay and tool history reliable | `src-tauri/src/db`, DTOs, chat rendering |
| D | Improve agent loop for real multi-step runs | todo | Support sequential tool usage before final answer | `src-tauri/src/agent/loop.rs`, provider integration |
| E | Add PR template and baseline CI checks | todo | Standardize team workflow and verification | `.github/`, docs |

## Ready-To-Pick Task Notes

### A. Stabilize provider and auth UX

- Goal: align OAuth and API key flows into one consistent experience.
- Deliverable: clear active auth mode, failure states, and usable model selection.
- Dependencies: none.

### B. Add tool registry and tool toggles

- Goal: list tools from backend and let user enable or disable them.
- Deliverable: model receives only enabled tools.
- Dependencies: none, but easier after task A.

### C. Persist richer message and tool-call data

- Goal: preserve structured tool metadata across restart.
- Deliverable: session history can reconstruct tool activity.
- Dependencies: none, but easier before task D.

### D. Improve agent loop for real multi-step runs

- Goal: support more than one tool continuation.
- Deliverable: repeated tool-call/approval/continue cycles until final answer.
- Dependencies: safer after task C.

### E. Add PR template and baseline CI checks

- Goal: make repo collaboration repeatable.
- Deliverable: PR template plus at least one realistic CI workflow.
- Dependencies: none.

## Parking Lot

Ideas that matter but are not active:

- terminal pane in the desktop UI
- code editor pane
- agent profiles
- memory between sessions
- MCP integration
- extension system
- multi-agent workflows inside the app

## How To Update This File

- Move `todo` to `in_progress` when a branch is created.
- Move `in_progress` to `done` only after code and docs are updated.
- If a task is split, add new rows with concrete names.
- Keep each row small enough for one PR.

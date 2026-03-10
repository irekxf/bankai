# Repository Workflow

This repository keeps project context in Markdown so a new human or agent can resume work quickly.

## Read Order

When starting fresh, read files in this order:

1. [project.md](/D:/bankai/project.md)
2. [README.md](/D:/bankai/README.md)
3. [docs/context/CURRENT_STATE.md](/D:/bankai/docs/context/CURRENT_STATE.md)
4. [docs/tasks/BACKLOG.md](/D:/bankai/docs/tasks/BACKLOG.md)
5. [docs/context/NEXT_SESSION.md](/D:/bankai/docs/context/NEXT_SESSION.md)
6. [docs/AGENT_COORDINATION.md](/D:/bankai/docs/AGENT_COORDINATION.md)

## Source Of Truth

- `project.md`: product vision and architecture direction
- `docs/context/CURRENT_STATE.md`: what is true in code right now
- `docs/tasks/BACKLOG.md`: prioritized tasks and status
- `docs/context/NEXT_SESSION.md`: the best next starting point
- `docs/handoffs/*.md`: session-by-session implementation notes

If docs conflict:
- trust code over stale notes
- update the stale note in the same branch when practical

## Update Rules

Update these files as part of normal development:

- After changing scope or architecture:
  update `docs/context/CURRENT_STATE.md`
- After finishing or splitting work:
  update `docs/tasks/BACKLOG.md`
- Before ending a session:
  update `docs/context/NEXT_SESSION.md`
- After meaningful work:
  create or update one file in `docs/handoffs/`

## Session Close Checklist

Before stopping work:

1. Write what changed.
2. Write what is still broken or unverified.
3. Write the exact next task to pick up.
4. List touched files.
5. Note commands run and what blocked verification.

## Task Granularity

Prefer tasks that:

- fit in one branch and one PR
- touch one vertical slice
- have clear acceptance criteria
- can be resumed from a short handoff note

Avoid backlog items like "improve app" or "refactor everything".

## Handoff Naming

Use filenames like:

- `docs/handoffs/2026-03-10-auth-ux.md`
- `docs/handoffs/2026-03-10-tool-registry-spike.md`

Each handoff should be short and factual.

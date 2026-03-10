# Multi-Agent Workflow

## Goal

This repository is developed by multiple coding agents in parallel.

Rules:
- `main` is protected and updated only through Pull Requests.
- One task equals one branch equals one PR.
- Agents do not share a working branch.
- The coordinator assigns tasks so agents avoid editing the same files at the same time.

## Branch Naming

- Codex branches: `codex/<task-slug>`
- Claude branches: `claude/<task-slug>`

Examples:
- `codex/mvp-planning-docs`
- `claude/provider-auth-ux`
- `claude/tool-registry-ui`

## Task Boundaries

Use vertical slices with clear ownership.

Preferred split:
- Rust backend flow: `src-tauri/src/agent`, `src-tauri/src/providers`, `src-tauri/src/db`
- Frontend chat and settings: `src/components`, `src/lib/stores`, `src/lib/tauri`
- Cross-cutting refactors only in isolated PRs

Avoid:
- Mixing unrelated fixes in one PR
- Simultaneous edits to the same Svelte component by multiple agents
- Simultaneous edits to `src-tauri/src/ipc/commands.rs` without explicit coordination

## PR Requirements

Each PR should include:
- short problem statement
- implemented scope
- files touched
- risks / follow-ups
- verification steps

Each PR should stay small enough for review. Target:
- preferred: up to 400 changed lines
- acceptable for refactor PRs: up to 800 changed lines

## Definition Of Done

A task is done only if:
- code builds or the agent explains exactly why it could not be verified
- types remain strict
- no placeholder TODOs are left without explanation
- PR description states what remains out of scope

## Current Coordination Decision

Current product direction comes from [project.md](../project.md):
- OAuth is an accepted auth path for the project
- API key is also supported
- shell tool requires approval
- approval flow is a core product feature, not optional UX
- desktop app should expose explicit tool execution state to the user

If current code conflicts with `project.md`, task PRs should move the code toward the latest documented product direction unless the coordinator says otherwise.


# RFC 0001: Per-Version Environment Isolation

## Status

Draft

## Problem

OpenClaw versions may not share compatible plugins, skills, MCP configuration, cache files, or runtime state. A version manager that only switches binaries can still leave users with state pollution after upgrades or experiments.

## Goals

- Define a storage layout for per-version config, cache, and runtime state.
- Preserve the current lightweight MVP behavior by default.
- Require explicit user action before migrating or modifying existing OpenClaw config.
- Leave room for snapshot and rollback integration.
- Keep CI and project-pinned workflows reproducible.

## Non-Goals

- Do not sandbox processes with OS-level isolation in this RFC.
- Do not manage Node.js, pnpm, or system dependencies.
- Do not silently rewrite existing OpenClaw configuration.
- Do not define plugin compatibility rules here.

## Proposed Design

Add an opt-in environment mode:

```bash
ocvm env enable
ocvm env disable
ocvm env current
```

When enabled, `ocvm exec` and the generated `openclaw` shim set environment variables that point OpenClaw to version-scoped state:

```text
~/.ocvm/
  envs/
    2026.3.28/
      config/
      cache/
      runtime/
    2026.4.01/
      config/
      cache/
      runtime/
```

The concrete OpenClaw environment variables must be confirmed against upstream OpenClaw before implementation. Until then, the env layer should use an internal adapter so OpenClaw-specific names do not leak across the codebase.

Configuration migration should be explicit:

```bash
ocvm migrate-config --to 2026.4.01
```

`migrate-config` copies existing config into the selected env directory and prints a diff-like summary before applying changes. Destructive edits require a confirmation flag.

## Snapshot Integration

Snapshots should eventually capture:

- active default version
- active session version
- env mode
- selected env directory
- optional config/cache manifest metadata

Rollback should restore metadata first and only restore config/cache content when the snapshot explicitly includes those artifacts.

## Rollout Plan

1. Keep current MVP behavior unchanged.
2. Add `ocvm env current` as a read-only diagnostic.
3. Add `ocvm env enable/disable` without migration.
4. Add explicit migration command after upstream OpenClaw env vars are confirmed.
5. Extend snapshot/rollback to include env metadata.

## Risks

- OpenClaw may change config discovery behavior.
- Copying config can duplicate credentials or local machine paths.
- Users may expect stronger isolation than environment variables can provide.
- Plugins may store state outside documented OpenClaw directories.

## Follow-Up Issues

- Implement `ocvm env current`.
- Confirm upstream OpenClaw config/cache environment variables.
- Add explicit `ocvm migrate-config`.
- Extend snapshots with env metadata.


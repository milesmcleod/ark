---
id: ADR-007
title: Lifecycle hooks via shell execution
status: accepted
date: 2026-04-02
tags: [extensibility, hooks, automation]
---

## Context

Artifact state transitions - creation, status changes, archival - may need to trigger external actions such as notifications, changelog updates, or CI triggers. Building these integrations directly into ark would create unbounded scope and couple the tool to specific external services.

## Decision

YAML-configured hooks in `.ark/hooks.yml`. Supported events: `on_create`, `on_status_change`, `on_archive`. Hooks execute via `sh -c` with environment variables providing context: `ARK_ARTIFACT_ID`, `ARK_ARTIFACT_TYPE`, `ARK_FROM_STATUS`, `ARK_TO_STATUS`. The trust model mirrors git hooks - the project owner controls what runs.

## Consequences

- Maximum flexibility - any shell command or script works as a hook.
- Security relies on the project owner controlling `hooks.yml`, same as git hooks. No sandboxing.
- Hook failures warn on stderr but do not block the operation, preventing a broken hook from stopping all artifact management.
- No embedded scripting engine needed - the shell is the extension runtime.

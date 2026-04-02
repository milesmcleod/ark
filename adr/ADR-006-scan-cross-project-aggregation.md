---
id: ADR-006
title: Scan as read-only cross-project aggregation
status: accepted
date: 2026-04-02
tags: [multi-project, aggregation, agent-ergonomics]
---

## Context

In multi-project ecosystems, agents need to survey artifact state across many repos. Each repo may have different schemas and naming conventions. A single agent session might need to answer "what are all the open tasks across every project?" without knowing each project's schema ahead of time.

## Decision

`ark scan` recursively discovers `.ark/` directories from a given root, loads schemas independently per project, and aggregates results. Scan is strictly read-only - it never writes to any project. Write operations (`new`, `edit`, `archive`) always target a specific project. Type aliasing via comma-separated names in the type filter handles heterogeneous schema naming across projects.

## Consequences

- One command gives a full ecosystem view across any number of projects.
- No cross-project write coordination needed - each project's data integrity is maintained independently.
- `.arkignore` controls scan scope, allowing exclusion of archived projects or vendored repos.
- Type aliasing means a query like `--type "task,story,ticket"` works across teams with different naming conventions for the same concept.

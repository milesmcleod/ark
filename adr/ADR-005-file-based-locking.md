---
id: ADR-005
title: File-based locking for concurrent operations
status: accepted
date: 2026-04-02
tags: [concurrency, safety, file-system]
---

## Context

Multiple agents or terminal sessions may run `ark new`, `ark edit`, `ark archive`, or `ark rebalance` concurrently. Without coordination, duplicate IDs or file conflicts can occur when two processes read the same state and write conflicting results.

## Decision

Create-exclusive file lock at `.ark/.lock` with 10-second stale detection and automatic cleanup via `Drop`. The lock is acquired for `new`, `edit`, `archive`, and `rebalance` operations. Lock creation uses exclusive file creation semantics so only one process can acquire it.

## Consequences

- Prevents concurrent ID collisions and conflicting writes.
- Simple and portable - no external dependencies, no OS-specific APIs.
- Stale lock recovery handles crashed processes via timestamp-based detection at 10 seconds.
- Not as robust as `flock()` (race window between stale detection and lock acquisition), but adequate for the use case where concurrent ark operations are infrequent.

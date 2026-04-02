---
id: ADR-001
title: Design philosophy and core architecture
status: accepted
date: 2026-04-01
tags: [architecture, design]
---

## Context

Project management tools (Jira, Trello, Linear) are human-first - designed around boards, swimlanes, and browser UIs. AI coding agents interact with these tools through clunky API connectors, opaque field configurations, and HTTP round-trips. The result: wasted tokens discovering field schemas, brittle integrations, and a fundamental mismatch between how agents work (reading/writing local files, running CLI commands) and how these tools expect to be used.

Meanwhile, developers already use markdown files with YAML frontmatter for specs, ADRs, and documentation. These files are git-native, human-readable, and trivially parseable. The missing piece is tooling that makes them queryable, enforceable, and discoverable at machine speed.

## Decision

Build ark as a local-first, schema-driven CLI for managing structured markdown artifacts. Core design choices:

1. **Markdown files with YAML frontmatter are the storage layer.** No database, no server, no external service. Files in directories, tracked by git.

2. **User-defined YAML schemas drive everything.** The CLI is agnostic to artifact semantics. It doesn't know what a "task" or "spec" or "ADR" is. Schemas define artifact types, their fields, valid values, and directory locations. The CLI reads schemas and operates generically.

3. **The CLI is the API.** Agents run commands. Output is designed for token efficiency - columnar, minimal, predictable. Every command communicates state and prompts the next action.

4. **Priority ordering replaces dependency graphs.** A single integer priority field provides global sequencing. Dependency graphs go stale; priority ordering is maintained as part of normal triage. Agents infer dependency reasoning from specs and ADRs.

5. **Schema discovery is a first-class command.** `ark types` and `ark fields` let an agent self-orient in any ark-managed project without reading documentation. The CLI teaches the agent how to use it.

6. **Linting is built in.** `ark lint` validates all artifacts against their schemas. Formatting and field constraints are enforced by the tool, not by convention.

## Consequences

- Adopting ark in an existing project requires only `ark init` and writing schema files. Existing markdown artifacts can be managed immediately if they follow the declared frontmatter schema.
- Teams using ark get git-native version control, merge conflict resolution, and blame/bisect on their project management artifacts for free.
- The schema system means ark is infinitely extensible without code changes - new artifact types are just new YAML files.
- The tradeoff is that ark provides no web UI, no real-time collaboration, and no notification system. It optimizes for the local development loop, not the management dashboard.

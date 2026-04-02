---
id: ADR-003
title: Schema-driven artifact types with agnostic CLI
status: accepted
date: 2026-04-02
tags: [architecture, schema, extensibility]
---

## Context

The tool needs to manage tasks, specs, ADRs, and arbitrary future types without code changes. Hardcoding artifact semantics into the CLI creates a maintenance burden for every new type and prevents adoption by teams with different workflows or terminology.

## Decision

User-defined YAML schemas in `.ark/schemas/` define everything about an artifact type - name, prefix, directory, fields, field types, constraints, enums, and body template. The CLI operates generically on whatever schemas exist. There is no hardcoded knowledge of what a "task" or "spec" is. Commands like `ark new`, `ark list`, and `ark lint` resolve behavior entirely from schema definitions at runtime.

## Consequences

- Infinitely extensible without code changes. Adding a new artifact type means adding a new YAML file.
- Any organization can define their own types, field conventions, and directory layouts.
- The schema IS the configuration - there is no separate config layer to keep in sync.
- Tradeoff: the CLI cannot provide type-specific smart defaults or specialized commands. Everything must be expressible through the generic schema language.

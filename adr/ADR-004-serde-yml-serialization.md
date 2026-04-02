---
id: ADR-004
title: serde_yml for YAML serialization
status: accepted
date: 2026-04-02
tags: [serialization, yaml, correctness]
---

## Context

Initial implementation used a hand-rolled YAML serializer for writing frontmatter. Two rounds of guinea pig testing found edge cases - colons in values, numeric strings interpreted as numbers, boolean-like strings ("yes", "true") not being quoted. Expert review flagged the hand-rolled serializer as the highest-risk code in the project.

## Decision

Replace the hand-rolled serializer with serde_yml. Field ordering is controlled via an ordered mapping so frontmatter fields appear in schema-declared order. serde_yml handles all quoting, escaping, and type-aware formatting.

## Consequences

- Eliminates a class of data corruption bugs. Values containing colons, quotes, booleans, and numeric strings are handled correctly by a battle-tested library.
- Depends on serde_yml for correctness - a reasonable trade given the Rust serde ecosystem's maturity.
- Minor format differences from the hand-rolled output (single quotes vs double quotes in some cases) but all output is YAML-spec compliant.

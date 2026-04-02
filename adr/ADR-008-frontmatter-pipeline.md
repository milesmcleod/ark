---
id: ADR-008
title: Two-stage frontmatter pipeline (gray_matter + JSON Schema)
status: accepted
date: 2026-04-02
tags: [parsing, validation, frontmatter, architecture]
---

## Context

Artifact files have YAML frontmatter that must be parsed (reading) and validated (linting/writing). Schemas define field types and constraints at runtime, so compile-time struct deserialization is not possible - the set of fields and their types varies per artifact type and per project.

## Decision

Parse frontmatter with gray_matter, which returns dynamic Pod values. Convert Pod values to `serde_json::Value` for the internal representation. Generate JSON Schema from ark schema definitions. Validate with the `jsonschema` crate. Write-time validation uses the same schema definitions directly.

## Consequences

- Clean separation between parsing, representation, and validation - each stage uses the best tool for its job.
- Any YAML frontmatter is parseable, enabling graceful degradation for malformed files (parse succeeds, validation reports specific errors).
- JSON Schema gives detailed error paths for lint violations, making it clear which field failed and why.
- The Pod-to-JSON conversion is a one-time cost per artifact load, not a performance concern at the scale ark operates.

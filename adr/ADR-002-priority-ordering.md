---
id: ADR-002
title: Priority ordering over dependency graphs
status: accepted
date: 2026-04-02
tags: [data-model, ordering, agent-ergonomics]
---

## Context

Project management tools use dependency graphs (blocks/blocked-by) to express sequencing. These graphs go stale as work evolves - edges that were meaningful at creation time become noise as scope shifts, tasks split, or priorities change. Agents maintaining dependency graphs waste tokens on graph traversal and edge maintenance, and the graph itself becomes a source of bugs when stale edges contradict actual project state.

## Decision

A single integer priority field provides global ordering within each artifact type. There is no dependency field. Agents infer dependency reasoning from specs and ADRs, which already contain the rationale for why things are sequenced the way they are.

## Consequences

- Simpler data model - one integer field replaces an unbounded set of edges.
- No stale edges to maintain or clean up. Reordering is a single field edit, not a graph restructure.
- Priority values are rebalanceable via `ark rebalance` when gaps compress.
- Tradeoff: implicit dependencies require context awareness from the consumer. An agent must read specs and ADRs to understand why item 3 should come before item 7. The ordering communicates the what, not the why.

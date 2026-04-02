---
id: SPEC-003
title: Cross-references and context
status: active
date: 2026-04-02
tags: [cross-references, relate, context]
---

Feature: Cross-references and context
  As a developer or AI agent
  I want to link related artifacts and gather their context
  So that I can navigate the relationships between work items

  # --- relate ---

  Scenario: Relate two artifacts bidirectionally
    Given an initialized ark project with a task BL-001 and a spec SPEC-001
    When I run `ark relate BL-001 SPEC-001`
    Then BL-001's frontmatter contains SPEC-001 in its related list
    And SPEC-001's frontmatter contains BL-001 in its related list
    And the updated date is set on both artifacts
    And the output reads "Related BL-001 <-> SPEC-001"

  Scenario: Relate multiple targets at once
    Given an initialized ark project with BL-001, BL-002, and SPEC-001
    When I run `ark relate BL-001 BL-002 SPEC-001`
    Then BL-001's related list contains both BL-002 and SPEC-001
    And BL-002's related list contains BL-001
    And SPEC-001's related list contains BL-001
    And the output reads "Related BL-001 <-> [BL-002, SPEC-001]"

  Scenario: Relate deduplicates repeated relations
    Given BL-001 is already related to SPEC-001
    When I run `ark relate BL-001 SPEC-001` again
    Then BL-001's related list still contains only one entry for SPEC-001

  Scenario: Cross-type relations work
    Given an initialized ark project with a task schema (prefix BL) and a spec schema (prefix SPEC)
    When I relate a task to a spec using `ark relate BL-001 SPEC-001`
    Then the relation is stored in both artifacts regardless of type differences

  Scenario: Self-reference is rejected
    Given an initialized ark project with a task BL-001
    When I run `ark relate BL-001 BL-001`
    Then the command fails with "cannot relate an artifact to itself"

  Scenario: Relating to a nonexistent artifact fails
    Given an initialized ark project with a task BL-001 but no SPEC-999
    When I run `ark relate BL-001 SPEC-999`
    Then the command fails with "artifact not found"

  # --- context ---

  Scenario: Context shows primary artifact and resolved relations (pretty)
    Given BL-001 is related to SPEC-001
    When I run `ark context BL-001`
    Then I see the full content of BL-001
    And I see a "Related:" section with SPEC-001's frontmatter summary

  Scenario: Context in JSON format
    Given BL-001 is related to SPEC-001
    When I run `ark -F json context BL-001`
    Then the JSON output has a "primary" object with full content including body
    And the JSON output has a "related" array of frontmatter-only objects (no body)

  Scenario: Context with no relations
    Given an initialized ark project with a task BL-001 that has no related artifacts
    When I run `ark context BL-001`
    Then I see the full content of BL-001 with no "Related:" section

  Scenario: Context with no relations in JSON
    Given an initialized ark project with a task BL-001 that has no related artifacts
    When I run `ark -F json context BL-001`
    Then the "related" array is empty

  Scenario: Context for nonexistent artifact fails
    Given an initialized ark project with no artifact BL-999
    When I run `ark context BL-999`
    Then the command fails with "artifact not found"

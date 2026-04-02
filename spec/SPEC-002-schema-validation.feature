---
id: SPEC-002
title: Schema validation and linting
status: draft
date: 2026-04-01
tags: [schema, validation, lint]
---

Feature: Schema validation and linting
  As a developer or AI agent
  I want to validate artifacts against their schemas
  So that formatting and field constraints are enforced consistently

  Scenario: Lint all artifacts
    Given an initialized ark project with artifacts and schemas
    When I run `ark lint`
    Then each artifact is validated against its type's schema
    And violations are reported with file path, field name, and reason
    And the exit code is non-zero if any violations exist

  Scenario: Lint a specific artifact
    Given an initialized ark project with a task artifact BL-001
    When I run `ark lint BL-001`
    Then only BL-001 is validated against the task schema
    And the result reports pass or specific violations

  Scenario: Lint a specific artifact type
    Given an initialized ark project with task and spec artifacts
    When I run `ark lint task`
    Then only task artifacts are validated
    And spec artifacts are not checked

  Scenario: Detect missing required fields
    Given a task artifact missing the required title field
    When I run `ark lint`
    Then the output reports a missing required field violation for title

  Scenario: Detect invalid enum values
    Given a task artifact with status set to "wip" which is not a valid enum value
    When I run `ark lint`
    Then the output reports an invalid enum value violation for status

  Scenario: Detect duplicate IDs
    Given two task artifacts both claiming id BL-001
    When I run `ark lint`
    Then the output reports a duplicate ID violation

  Scenario: Detect duplicate priorities
    Given two task artifacts both claiming priority 10
    When I run `ark lint`
    Then the output reports a duplicate priority warning

  Scenario: Schema help reference
    Given an initialized ark project
    When I run `ark schema-help`
    Then I see the complete schema file format reference
    And it includes all supported field types and constraints
    And it includes an example schema

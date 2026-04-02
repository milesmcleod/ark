---
id: SPEC-005
title: Schema extensibility
status: active
date: 2026-04-02
tags: [schema, inheritance, registry, scaffold]
---

Feature: Schema extensibility
  As a developer
  I want to reuse, inherit, and fetch schemas
  So that I can reduce duplication and share artifact type definitions

  # --- Schema inheritance ---

  Scenario: Child schema inherits fields from base
    Given a base-task schema with fields id, title, and status
    And a task schema with `extends: base-task` and additional fields priority and project
    When I load schemas with `ark types`
    Then the task type has all base fields plus its own fields
    And the task type uses its own directory and prefix

  Scenario: Child schema overrides base fields by name
    Given a base-item schema with status values [open, closed]
    And a ticket schema extending base-item with status values [new, triaged, in-progress, resolved]
    When I create a ticket artifact
    Then the default status uses the child's values (new)
    And the base status values (open, closed) are rejected on edit

  Scenario: Circular inheritance is detected
    Given schema A extends schema B
    And schema B extends schema A
    When schemas are loaded
    Then the command fails with a circular inheritance error

  Scenario: Missing base schema is detected
    Given a child schema with `extends: nonexistent`
    When schemas are loaded
    Then the command fails because the base schema was not found

  # --- Schema registry ---

  Scenario: Registry pull with no registry schemas
    Given an initialized ark project with schemas that have no registry URLs
    When I run `ark registry-pull`
    Then the output says "no schemas with registry URLs found"

  Scenario: Registry pull fetches and replaces schema files
    Given a schema with a `registry: <url>` field pointing to a valid schema URL
    When I run `ark registry-pull`
    Then the schema file is replaced with the fetched content
    And the fetched content is validated as a valid schema before writing

  # --- Scaffold ---

  Scenario: Scaffold from a template directory
    Given a directory containing .yml schema files
    When I run `ark scaffold <template-dir>`
    Then .ark/schemas/ is created if needed
    And schema files are copied into .ark/schemas/
    And the output reports how many schemas were scaffolded

  Scenario: Scaffold skips existing schemas
    Given an initialized ark project with an existing task.yml schema
    When I run `ark scaffold <template-dir>` where the template also contains task.yml
    Then the existing task.yml is not overwritten
    And a warning is emitted on stderr

  Scenario: Scaffold from nonexistent directory fails
    When I run `ark scaffold /nonexistent/path`
    Then the command fails with an error about the directory not being found

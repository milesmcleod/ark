---
id: SPEC-008
title: Write-time validation
status: active
date: 2026-04-02
tags: [validation, new, edit, safety]
---

Feature: Write-time validation
  As a developer or AI agent
  I want artifact creation and editing to validate inputs at write time
  So that invalid data never enters the artifact store

  # --- Required field validation ---

  Scenario: Missing required fields are rejected on new
    Given a task schema where "project" is a required field
    When I run `ark new task --title "Incomplete" --priority 10` without --project
    Then the command fails with "missing required fields"

  Scenario: Required derived fields are not required from the user
    Given a task schema where "id", "created", and "updated" are required and derived
    When I create a new artifact without supplying those fields
    Then the command succeeds because derived fields are auto-populated

  # --- Enum validation ---

  Scenario: Invalid enum value rejected on new
    Given a task schema where "project" has values [alpha, beta]
    When I run `ark new task --title "Bad" --project nonexistent --priority 10`
    Then the command fails with "invalid value"

  Scenario: Invalid enum value rejected on edit
    Given a task artifact BL-001
    When I run `ark edit BL-001 --status invalid_status`
    Then the command fails with "invalid value"

  # --- Type coercion for --set ---

  Scenario: Integer fields are coerced from string input
    Given a task schema with an integer priority field
    When I run `ark new task --title "Test" --project alpha --priority 10`
    Then the priority is stored as an integer value in frontmatter, not a string

  Scenario: Boolean fields via --set are coerced
    Given a schema with a boolean field
    When I run `ark new <type> --set flag=true`
    Then the value is stored as a boolean true, not the string "true"

  # --- Derived field protection ---

  Scenario: Derived fields cannot be set via --set
    Given a task schema where "id" is a derived field
    When I run `ark new task --title "Test" --project alpha --priority 10 --set id=FAKE-999`
    Then the command fails with an error indicating the field is derived

  # --- Unique priority enforcement ---

  Scenario: Duplicate priority rejected on new
    Given a task artifact with priority 10 already exists
    When I run `ark new task --title "Dupe" --project alpha --priority 10`
    Then the command fails with "priority 10 is already used"

  Scenario: Duplicate priority rejected on edit
    Given task artifacts BL-001 at priority 10 and BL-002 at priority 20
    When I run `ark edit BL-002 --priority 10`
    Then the command fails because priority 10 is already taken by BL-001

  # --- Title validation ---

  Scenario: Empty title rejected
    When I run `ark new task --title "" --project alpha --priority 10`
    Then the command fails with an error about empty title

  Scenario: Newline in title rejected
    When I run `ark new task --title "line1\nline2" --project alpha --priority 10`
    Then the command fails with an error about newlines in title

  # --- --set conflict detection ---

  Scenario: Named flag and --set for same field is a conflict
    Given a task artifact BL-001
    When I run `ark edit BL-001 --status active --set status=done`
    Then the command fails with a "conflict" error
    Because the same field is being set by both a named flag and --set

  # --- Path traversal prevention ---

  Scenario: Schema directory cannot escape project root
    Given a schema with directory "../outside"
    When the schema is loaded
    Then ark rejects it with an error about escaping the project root

  Scenario: Archive directory cannot escape project root
    Given a schema with archive directory "../outside/done"
    When the schema is loaded
    Then ark rejects it with an error about escaping the project root

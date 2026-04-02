---
id: SPEC-001
title: Core artifact management
status: active
date: 2026-04-02
tags: [core, crud]
---

Feature: Core artifact management
  As a developer or AI agent
  I want to create, list, show, edit, and archive structured markdown artifacts
  So that I can manage project work items without external tooling

  Scenario: Initialize an ark project
    Given a directory without an .ark/ subdirectory
    When I run `ark init`
    Then an .ark/schemas/ directory is created
    And the output explains that schema files are needed
    And the output suggests next steps including `ark types` and `ark schema-help`

  Scenario: Reject double initialization
    Given a directory that has already been initialized with `ark init`
    When I run `ark init` again
    Then the command fails with an "already initialized" error

  Scenario: List artifact types
    Given an initialized ark project with schema files in .ark/schemas/
    When I run `ark types`
    Then I see a list of registered artifact types with their prefix and directory

  Scenario: Discover field values for an artifact type
    Given an initialized ark project with a task schema defining enum fields
    When I run `ark fields task status`
    Then I see the valid values for the status field

  Scenario: Create a new artifact
    Given an initialized ark project with a task schema
    When I run `ark new task --title "Build prototype" --project frontend --kind feature --priority 10`
    Then a new markdown file is created in the schema's declared directory
    And the file has YAML frontmatter with all provided fields
    And the id is auto-generated using the schema's prefix and next available number
    And the created and updated dates are set to today
    And the filename matches the pattern {prefix}-{NNN}-{slugified-title}.md
    And the body contains the schema's template

  Scenario: List artifacts with default output
    Given an initialized ark project with task artifacts
    When I run `ark list task`
    Then I see a columnar listing of all non-archived tasks sorted by priority
    And each row shows id, priority, status, project, and title

  Scenario: List artifacts with filters
    Given an initialized ark project with task artifacts across multiple projects
    When I run `ark list task --status active --project frontend`
    Then I see only active tasks belonging to the frontend project

  Scenario: List artifacts with --all flag
    Given an initialized ark project with both active and archived task artifacts
    When I run `ark list task --all`
    Then I see all artifacts including those with archive status

  Scenario: Filter warns on invalid enum values
    Given an initialized ark project with task artifacts
    When I run `ark list task --status nonexistent`
    Then a warning is emitted on stderr that "nonexistent" is not a known value for status
    And the command still succeeds with zero matching results

  Scenario: Show a single artifact
    Given an initialized ark project with a task artifact BL-001
    When I run `ark show BL-001`
    Then I see the full content of the artifact including frontmatter and body

  Scenario: Show a single artifact in JSON format
    Given an initialized ark project with a task artifact BL-001
    When I run `ark -F json show BL-001`
    Then I receive valid JSON with all frontmatter fields and the body

  Scenario: Show returns error for nonexistent artifact
    Given an initialized ark project with no artifact BL-999
    When I run `ark show BL-999`
    Then the command fails with an "artifact not found" error

  Scenario: Edit artifact frontmatter
    Given an initialized ark project with a task artifact BL-001
    When I run `ark edit BL-001 --status active`
    Then the status field in BL-001's frontmatter is updated to active
    And the updated date is set to today

  Scenario: Edit with no changes is a no-op
    Given an initialized ark project with a task artifact BL-001
    When I run `ark edit BL-001` with no field flags
    Then the output says "No changes"

  Scenario: Archive completed artifacts
    Given an initialized ark project with task artifacts where some have status done
    When I run `ark archive task`
    Then done artifacts are moved to the schema's archive subdirectory
    And the output shows how many artifacts were archived

  Scenario: Show the next work items
    Given an initialized ark project with active and backlog task artifacts
    When I run `ark next task`
    Then I see "Active:" section with currently active items
    And I see "Up next:" section with the highest priority backlog items
    And blocked and archived items are excluded

  Scenario: Rebalance priorities
    Given an initialized ark project with task artifacts at priorities 5 and 7
    When I run `ark rebalance task`
    Then priorities are renumbered in increments of 10 (10, 20)
    And the updated date is set to today on changed artifacts

  Scenario: Show artifact statistics
    Given an initialized ark project with task artifacts
    When I run `ark stats task --by project`
    Then I see a table of artifact counts grouped by project

  Scenario: Show statistics for all types
    Given an initialized ark project with multiple artifact types
    When I run `ark stats`
    Then I see a summary of each type with its count and directory

  Scenario: Generate shell completions
    Given an initialized ark project
    When I run `ark completions bash`
    Then I receive shell completion script output for bash

  Scenario: Search artifact bodies
    Given an initialized ark project with task artifacts containing various text
    When I run `ark search XYZ`
    Then I see matching artifacts with their type, id, title, and where the match occurred

  Scenario: Case-insensitive search
    Given an initialized ark project with a task titled "UPPERCASE title"
    When I run `ark search -i uppercase`
    Then the artifact is found

  Scenario: Search with no matches
    Given an initialized ark project with task artifacts
    When I run `ark search nonexistent`
    Then I see "No artifacts matching" message

  Scenario: Parent directory walk
    Given an initialized ark project
    When I run an ark command from a subdirectory within the project
    Then ark finds the .ark/ root by walking up the directory tree

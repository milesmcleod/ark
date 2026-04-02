---
id: SPEC-001
title: Core artifact management
status: draft
date: 2026-04-01
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
    When I run `ark new task --title "Build prototype" --project bellflower --type feature --priority 10`
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
    And each row shows id, priority, status, project, type, and title

  Scenario: List artifacts with filters
    Given an initialized ark project with task artifacts across multiple projects
    When I run `ark list task --status active --project bellflower`
    Then I see only active tasks belonging to the bellflower project

  Scenario: Show a single artifact
    Given an initialized ark project with a task artifact BL-001
    When I run `ark show BL-001`
    Then I see the full content of the artifact including frontmatter and body

  Scenario: Edit artifact frontmatter
    Given an initialized ark project with a task artifact BL-001
    When I run `ark edit BL-001 --status active`
    Then the status field in BL-001's frontmatter is updated to active
    And the updated date is set to today

  Scenario: Archive completed artifacts
    Given an initialized ark project with task artifacts where some have status done
    When I run `ark archive task`
    Then done artifacts are moved to the schema's archive subdirectory
    And the output shows how many artifacts were archived

---
id: SPEC-004
title: Cross-project scanning
status: active
date: 2026-04-02
tags: [scan, cross-project, discovery]
---

Feature: Cross-project scanning
  As a developer managing multiple ark projects
  I want to scan across nested projects from a parent directory
  So that I can get an aggregated view of all work items

  Background:
    Given a parent directory containing multiple ark projects (project-a, project-b, project-c)
    And each project has its own .ark/schemas/ and artifacts

  # --- scan types ---

  Scenario: Discover all artifact types across projects
    When I run `ark scan types` from the parent directory
    Then I see a table with columns: project, type, prefix, directory, fields
    And all projects and their artifact types are listed

  # --- scan list ---

  Scenario: List artifacts of a type across all projects
    Given project-a has tasks with prefix BL and project-c has tasks with prefix TK
    When I run `ark scan list task`
    Then I see tasks from both project-a and project-c
    And each row includes the project name for disambiguation

  Scenario: Type aliasing matches by schema name
    Given project-a defines a "task" schema with prefix BL
    And project-c defines a "task" schema with prefix TK
    When I run `ark scan list task`
    Then artifacts from both projects appear because they share the type name "task"

  Scenario: Filter scan list by project
    When I run `ark scan list task --project project-a`
    Then I see only tasks from project-a

  # --- scan next ---

  Scenario: Show cross-project work queue
    When I run `ark scan next task`
    Then I see "Up next:" with the highest priority items across all projects
    And each row includes the project name

  # --- scan stats ---

  Scenario: Aggregate statistics across projects
    When I run `ark scan stats`
    Then I see statistics for all projects
    And each entry includes the project name, type, and count

  # --- scan search ---

  Scenario: Search across all projects
    When I run `ark scan search "Task"`
    Then I see matching artifacts from all projects
    And results include the project name

  # --- scan lint ---

  Scenario: Lint all projects
    When I run `ark scan lint`
    Then each project's artifacts are validated against their schemas
    And the output summarizes results per project
    And the summary reports total projects scanned

  # --- .arkignore ---

  Scenario: Exclude directories via .arkignore
    Given a .arkignore file in the parent directory containing "project-c"
    When I run `ark scan types`
    Then project-a and project-b appear in the results
    And project-c is excluded

  Scenario: Glob patterns in .arkignore
    Given a .arkignore file containing "project-*"
    When I run `ark scan types`
    Then no projects matching the glob pattern are discovered

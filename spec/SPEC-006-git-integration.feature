---
id: SPEC-006
title: Git integration
status: active
date: 2026-04-02
tags: [git, diff, stale]
---

Feature: Git integration
  As a developer using git for version control
  I want to see how artifacts have changed between commits and find stale items
  So that I can track progress and identify neglected work

  # --- diff ---

  Scenario: Diff shows added artifacts
    Given an initialized ark project in a git repository
    And a new artifact was created after the base ref
    When I run `ark diff <base-ref>`
    Then the output shows the artifact as "added" with its id and title

  Scenario: Diff shows modified artifacts with field-level changes
    Given an initialized ark project in a git repository
    And an artifact's status was changed from backlog to active after the base ref
    When I run `ark diff <base-ref>`
    Then the output shows the artifact as "modified"
    And the changed fields are listed (e.g. "status: backlog -> active")

  Scenario: Diff shows removed artifacts
    Given an initialized ark project in a git repository
    And an artifact file existed at the base ref but has been deleted
    When I run `ark diff <base-ref>`
    Then the output shows the artifact as "removed" with its former id

  Scenario: Diff can be filtered by artifact type
    Given an initialized ark project with task and spec artifacts
    When I run `ark diff <base-ref> --artifact-type task`
    Then only changes to task artifacts are shown

  Scenario: Diff requires a git repository
    Given an initialized ark project that is not in a git repository
    When I run `ark diff HEAD~1`
    Then the command fails with "not a git repository"

  # --- stale ---

  Scenario: Find stale artifacts by days threshold
    Given an initialized ark project with artifacts that have not been updated in 30 days
    When I run `ark stale task --days 14`
    Then I see a table of stale artifacts with id, status, updated date, last commit date, and title

  Scenario: Stale cross-references git log
    Given an initialized ark project in a git repository
    And an artifact's frontmatter updated date is old but git shows a recent commit
    When I run `ark stale task --days 14`
    Then the artifact is not considered stale because git activity is recent

  Scenario: No stale artifacts found
    Given an initialized ark project with all artifacts recently updated
    When I run `ark stale task --days 14`
    Then the output says "No stale task artifacts found"

  Scenario: Stale excludes archived artifacts
    Given an initialized ark project with archived task artifacts
    When I run `ark stale task`
    Then archived artifacts are not included in the stale check

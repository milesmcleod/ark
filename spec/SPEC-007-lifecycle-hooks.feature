---
id: SPEC-007
title: Lifecycle hooks
status: active
date: 2026-04-02
tags: [hooks, automation, lifecycle]
---

Feature: Lifecycle hooks
  As a developer
  I want to run scripts when artifact lifecycle events occur
  So that I can automate workflows triggered by artifact changes

  Background:
    Given an initialized ark project with a hooks configuration at .ark/hooks.yml

  Scenario: Hook configuration structure
    Given a .ark/hooks.yml file with the following structure:
      """
      on_create:
        - type: task
          run: echo "created $ARK_ARTIFACT_ID"
      on_status_change:
        - type: task
          from_status: backlog
          to_status: active
          run: echo "activated $ARK_ARTIFACT_ID"
      on_archive:
        - run: echo "archived $ARK_ARTIFACT_ID"
      """
    When hooks are loaded
    Then three event types are recognized: on_create, on_status_change, on_archive

  Scenario: on_create hook fires when a new artifact is created
    Given an on_create hook configured for type "task"
    When I run `ark new task --title "Test" --project alpha --priority 10`
    Then the on_create hook's command is executed

  Scenario: on_status_change hook fires on status edit
    Given an on_status_change hook configured for type "task" with to_status "active"
    And a task artifact BL-001 with status "backlog"
    When I run `ark edit BL-001 --status active`
    Then the on_status_change hook's command is executed

  Scenario: on_status_change hook respects from_status filter
    Given an on_status_change hook with from_status "backlog" and to_status "active"
    And a task artifact BL-001 with status "blocked"
    When I run `ark edit BL-001 --status active`
    Then the hook does not fire because from_status does not match "blocked"

  Scenario: on_archive hook fires when artifacts are archived
    Given an on_archive hook configured for all types
    And a task artifact BL-001 with status "done"
    When I run `ark archive task`
    Then the on_archive hook's command is executed for BL-001

  Scenario: Hook receives environment variables
    Given any lifecycle hook
    When the hook fires
    Then the environment includes ARK_ARTIFACT_ID with the artifact's ID
    And the environment includes ARK_ARTIFACT_TYPE with the artifact's type name
    And for status change hooks, ARK_FROM_STATUS and ARK_TO_STATUS are set

  Scenario: Hook type filter restricts which types trigger
    Given an on_create hook with type filter "spec"
    When a task artifact is created
    Then the hook does not fire because the type does not match

  Scenario: Hook without type filter matches all types
    Given an on_archive hook with no type filter
    When any artifact type is archived
    Then the hook fires regardless of type

  Scenario: No hooks.yml means no hooks run
    Given an initialized ark project without a .ark/hooks.yml file
    When I create, edit, or archive artifacts
    Then no hooks are executed and the commands succeed normally

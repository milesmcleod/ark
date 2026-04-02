use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ark() -> Command {
    Command::cargo_bin("ark").unwrap()
}

fn setup_project() -> TempDir {
    let dir = TempDir::new().unwrap();
    ark().arg("init").current_dir(dir.path()).assert().success();

    let schema = r#"name: task
directory: backlog
prefix: BL
fields:
  - name: id
    type: string
    required: true
    derived: true
  - name: title
    type: string
    required: true
  - name: status
    type: enum
    required: true
    values: [backlog, active, blocked, done]
    default: backlog
  - name: priority
    type: integer
    required: true
    unique: true
  - name: project
    type: enum
    required: true
    values: [alpha, beta]
  - name: tags
    type: list
  - name: created
    type: date
    derived: true
  - name: updated
    type: date
    derived: true
archive:
  field: status
  value: done
  directory: backlog/done
template: |
  ## Context

  ## Acceptance criteria

  - [ ]
"#;
    let schemas_dir = dir.path().join(".ark").join("schemas");
    fs::write(schemas_dir.join("task.yml"), schema).unwrap();
    dir
}

#[test]
fn test_init_creates_ark_dir() {
    let dir = TempDir::new().unwrap();
    ark()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized ark"));

    assert!(dir.path().join(".ark").join("schemas").is_dir());
}

#[test]
fn test_init_fails_if_already_initialized() {
    let dir = TempDir::new().unwrap();
    ark().arg("init").current_dir(dir.path()).assert().success();
    ark()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn test_types_lists_schemas() {
    let dir = setup_project();
    ark()
        .arg("types")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("task"));
}

#[test]
fn test_fields_lists_all_fields() {
    let dir = setup_project();
    ark()
        .args(["fields", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("priority"));
}

#[test]
fn test_fields_shows_enum_values() {
    let dir = setup_project();
    ark()
        .args(["fields", "task", "status"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("backlog"))
        .stdout(predicate::str::contains("active"))
        .stdout(predicate::str::contains("done"));
}

#[test]
fn test_new_creates_artifact() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Test task",
            "--project",
            "alpha",
            "--kind",
            "chore",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created BL-001"));

    let backlog = dir.path().join("backlog");
    assert!(backlog.is_dir());
    let files: Vec<_> = fs::read_dir(&backlog)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);
}

#[test]
fn test_new_rejects_invalid_enum() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Bad task",
            "--project",
            "nonexistent",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_list_shows_artifacts() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "First",
            "--project",
            "alpha",
            "--priority",
            "20",
        ])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Second",
            "--project",
            "beta",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    // Default list shows both, sorted by priority
    ark()
        .args(["list", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("First"))
        .stdout(predicate::str::contains("Second"));

    // Filter by project
    ark()
        .args(["list", "task", "--project", "alpha"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("First"))
        .stdout(predicate::str::contains("Second").not());
}

#[test]
fn test_tsv_output() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "TSV test",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["-F", "tsv", "list", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("id\t"))
        .stdout(predicate::str::contains("BL-001"));
}

#[test]
fn test_edit_updates_field() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Edit me",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["edit", "BL-001", "--status", "active"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["show", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("status: active"));
}

#[test]
fn test_lint_passes_valid_artifacts() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Valid task",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["lint", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Lint passed"));
}

#[test]
fn test_lint_catches_invalid_enum() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Will be broken",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    // Manually corrupt the file
    let backlog = dir.path().join("backlog");
    let file = fs::read_dir(&backlog)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap()
        .path();
    let content = fs::read_to_string(&file).unwrap();
    let corrupted = content.replace("status: backlog", "status: invalid");
    fs::write(&file, corrupted).unwrap();

    ark()
        .args(["lint", "task"])
        .current_dir(dir.path())
        .assert()
        .failure();
}

#[test]
fn test_archive_moves_done_items() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "To archive",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["edit", "BL-001", "--status", "done"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["archive", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Archived 1"));

    let done_dir = dir.path().join("backlog").join("done");
    assert!(done_dir.is_dir());
    let files: Vec<_> = fs::read_dir(&done_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);
}

#[test]
fn test_search_finds_matches() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Build prototype XYZ",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["search", "XYZ"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("BL-001"));

    ark()
        .args(["search", "nonexistent"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No artifacts matching"));
}

#[test]
fn test_schema_help() {
    let dir = TempDir::new().unwrap();
    ark().arg("init").current_dir(dir.path()).assert().success();
    ark()
        .arg("schema-help")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Schema File Reference"))
        .stdout(predicate::str::contains("Field Types"));
}

#[test]
fn test_rebalance() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "First",
            "--project",
            "alpha",
            "--priority",
            "5",
        ])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Second",
            "--project",
            "alpha",
            "--priority",
            "7",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["rebalance", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Rebalanced"));

    // After rebalance, priorities should be 10, 20
    ark()
        .args(["-F", "tsv", "list", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("20"));
}

#[test]
fn test_new_rejects_missing_required_fields() {
    let dir = setup_project();
    // Missing --project which is required
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Incomplete task",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing required fields"));
}

#[test]
fn test_new_rejects_duplicate_priority() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "First task",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args([
            "new",
            "task",
            "--title",
            "Dupe priority",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("priority 10 is already used"));
}

#[test]
fn test_set_validates_against_schema() {
    let dir = setup_project();
    // --set on a derived field should fail
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Test set validation",
            "--project",
            "alpha",
            "--priority",
            "10",
            "--set",
            "id=FAKE-999",
        ])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("derived"));
}

#[test]
fn test_edit_noop_no_changes() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "No-op test",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["edit", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No changes"));
}

#[test]
fn test_parent_directory_walk() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Walk test",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    // Create a subdirectory and run from there
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();

    ark()
        .args(["list", "task"])
        .current_dir(&subdir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Walk test"));
}

#[test]
fn test_colon_in_title_roundtrip() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Fix: colon handling",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    // Verify it survives a show
    ark()
        .args(["show", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Fix: colon handling"));

    // Edit and verify it still survives
    ark()
        .args(["edit", "BL-001", "--status", "active"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["show", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Fix: colon handling"));
}

#[test]
fn test_next_command() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Active item",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Queued item",
            "--project",
            "alpha",
            "--priority",
            "20",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["edit", "BL-001", "--status", "active"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["next", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Active:"))
        .stdout(predicate::str::contains("Up next:"));
}

#[test]
fn test_show_json_format() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "JSON test",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    let output = ark()
        .args(["-F", "json", "show", "BL-001"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["title"], "JSON test");
    assert_eq!(parsed["id"], "BL-001");
}

// --- Scan tests ---

fn setup_multi_project() -> TempDir {
    let root = TempDir::new().unwrap();

    // Project A: has tasks
    let proj_a = root.path().join("project-a");
    fs::create_dir(&proj_a).unwrap();
    ark().arg("init").current_dir(&proj_a).assert().success();
    fs::write(
        proj_a.join(".ark/schemas/task.yml"),
        r#"name: task
directory: backlog
prefix: BL
fields:
  - name: id
    type: string
    required: true
    derived: true
  - name: title
    type: string
    required: true
  - name: status
    type: enum
    required: true
    values: [backlog, active, done]
    default: backlog
  - name: priority
    type: integer
    required: true
    unique: true
  - name: created
    type: date
    derived: true
  - name: updated
    type: date
    derived: true
archive:
  field: status
  value: done
  directory: backlog/done
"#,
    )
    .unwrap();
    ark()
        .args(["new", "task", "--title", "Task in A", "--priority", "10"])
        .current_dir(&proj_a)
        .assert()
        .success();

    // Project B: has specs (different type, same ecosystem)
    let proj_b = root.path().join("project-b");
    fs::create_dir(&proj_b).unwrap();
    ark().arg("init").current_dir(&proj_b).assert().success();
    fs::write(
        proj_b.join(".ark/schemas/spec.yml"),
        r#"name: spec
directory: spec
prefix: SPEC
fields:
  - name: id
    type: string
    required: true
    derived: true
  - name: title
    type: string
    required: true
  - name: status
    type: enum
    required: true
    values: [draft, active]
    default: draft
  - name: created
    type: date
    derived: true
  - name: updated
    type: date
    derived: true
"#,
    )
    .unwrap();
    ark()
        .args(["new", "spec", "--title", "Spec in B"])
        .current_dir(&proj_b)
        .assert()
        .success();

    // Project C: also has tasks (different schema, aliasing test)
    let proj_c = root.path().join("project-c");
    fs::create_dir(&proj_c).unwrap();
    ark().arg("init").current_dir(&proj_c).assert().success();
    fs::write(
        proj_c.join(".ark/schemas/task.yml"),
        r#"name: task
directory: backlog
prefix: TK
fields:
  - name: id
    type: string
    required: true
    derived: true
  - name: title
    type: string
    required: true
  - name: status
    type: enum
    required: true
    values: [todo, doing, done]
    default: todo
  - name: created
    type: date
    derived: true
  - name: updated
    type: date
    derived: true
"#,
    )
    .unwrap();
    ark()
        .args(["new", "task", "--title", "Task in C"])
        .current_dir(&proj_c)
        .assert()
        .success();

    root
}

#[test]
fn test_scan_types_discovers_all_projects() {
    let root = setup_multi_project();
    ark()
        .args(["scan", "types"])
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project-a"))
        .stdout(predicate::str::contains("project-b"))
        .stdout(predicate::str::contains("project-c"));
}

#[test]
fn test_scan_list_aggregates_across_projects() {
    let root = setup_multi_project();
    ark()
        .args(["scan", "list", "task"])
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Task in A"))
        .stdout(predicate::str::contains("Task in C"));
}

#[test]
fn test_scan_list_filters_by_project() {
    let root = setup_multi_project();
    ark()
        .args(["scan", "list", "task", "--project", "project-a"])
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Task in A"))
        .stdout(predicate::str::contains("Task in C").not());
}

#[test]
fn test_scan_stats_shows_all() {
    let root = setup_multi_project();
    ark()
        .args(["scan", "stats"])
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project-a"))
        .stdout(predicate::str::contains("project-b"))
        .stdout(predicate::str::contains("project-c"));
}

#[test]
fn test_scan_search_finds_across_projects() {
    let root = setup_multi_project();
    ark()
        .args(["scan", "search", "Task"])
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project-a"))
        .stdout(predicate::str::contains("project-c"));
}

#[test]
fn test_scan_lint_validates_all() {
    let root = setup_multi_project();
    ark()
        .args(["scan", "lint"])
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("3 projects"));
}

#[test]
fn test_scan_next_shows_cross_project_queue() {
    let root = setup_multi_project();
    ark()
        .args(["scan", "next", "task"])
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Up next:"))
        .stdout(predicate::str::contains("project-a"))
        .stdout(predicate::str::contains("project-c"));
}

#[test]
fn test_arkignore_excludes_directories() {
    let root = setup_multi_project();

    // Create .arkignore that excludes project-c
    fs::write(root.path().join(".arkignore"), "project-c\n").unwrap();

    // scan types should not include project-c
    ark()
        .args(["scan", "types"])
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project-a"))
        .stdout(predicate::str::contains("project-b"))
        .stdout(predicate::str::contains("project-c").not());
}

#[test]
fn test_arkignore_glob_patterns() {
    let root = setup_multi_project();

    // Exclude all projects starting with "project-"
    fs::write(root.path().join(".arkignore"), "project-*\n").unwrap();

    ark()
        .args(["scan", "types"])
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project-a").not());
}

// --- Coverage gap tests ---

#[test]
fn test_numeric_title_roundtrip() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "42",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    // Title "42" should survive roundtrip (quoted in YAML, not parsed as integer)
    ark()
        .args(["show", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("title: '42'"));

    ark()
        .args(["edit", "BL-001", "--status", "active"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["show", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("title: '42'"));
}

#[test]
fn test_stats_single_project() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "A",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "B",
            "--project",
            "beta",
            "--priority",
            "20",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["stats", "task", "--by", "project"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn test_newline_in_title_rejected() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "line1\nline2",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("newline"));
}

#[test]
fn test_empty_title_rejected() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));
}

#[test]
fn test_list_empty_shows_helpful_message() {
    let dir = setup_project();
    ark()
        .args(["list", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No task artifacts found"));
}

#[test]
fn test_edit_rejects_invalid_enum() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Test",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["edit", "BL-001", "--status", "invalid_status"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_show_not_found() {
    let dir = setup_project();
    ark()
        .args(["show", "BL-999"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("artifact not found"));
}

// --- Relate and Context tests ---

/// Set up a project with two artifact types (task + spec) for cross-type testing
fn setup_multi_type_project() -> TempDir {
    let dir = TempDir::new().unwrap();
    ark().arg("init").current_dir(dir.path()).assert().success();

    let task_schema = r#"name: task
directory: backlog
prefix: BL
fields:
  - name: id
    type: string
    required: true
    derived: true
  - name: title
    type: string
    required: true
  - name: status
    type: enum
    required: true
    values: [backlog, active, done]
    default: backlog
  - name: priority
    type: integer
    required: true
    unique: true
  - name: tags
    type: list
  - name: related
    type: list
  - name: created
    type: date
    derived: true
  - name: updated
    type: date
    derived: true
archive:
  field: status
  value: done
  directory: backlog/done
"#;

    let spec_schema = r#"name: spec
directory: spec
prefix: SPEC
fields:
  - name: id
    type: string
    required: true
    derived: true
  - name: title
    type: string
    required: true
  - name: status
    type: enum
    required: true
    values: [draft, active]
    default: draft
  - name: related
    type: list
  - name: created
    type: date
    derived: true
  - name: updated
    type: date
    derived: true
"#;

    let schemas_dir = dir.path().join(".ark").join("schemas");
    fs::write(schemas_dir.join("task.yml"), task_schema).unwrap();
    fs::write(schemas_dir.join("spec.yml"), spec_schema).unwrap();
    dir
}

#[test]
fn test_relate_basic_bidirectional() {
    let dir = setup_multi_type_project();
    ark()
        .args(["new", "task", "--title", "Build thing", "--priority", "10"])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args(["new", "spec", "--title", "Thing spec"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["relate", "BL-001", "SPEC-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Related BL-001 <-> SPEC-001"));

    // BL-001 should have SPEC-001 in its related field
    ark()
        .args(["show", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SPEC-001"));

    // SPEC-001 should have BL-001 in its related field (bidirectional)
    ark()
        .args(["show", "SPEC-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("BL-001"));
}

#[test]
fn test_relate_deduplication() {
    let dir = setup_multi_type_project();
    ark()
        .args(["new", "task", "--title", "Build thing", "--priority", "10"])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args(["new", "spec", "--title", "Thing spec"])
        .current_dir(dir.path())
        .assert()
        .success();

    // Relate twice
    ark()
        .args(["relate", "BL-001", "SPEC-001"])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args(["relate", "BL-001", "SPEC-001"])
        .current_dir(dir.path())
        .assert()
        .success();

    // Should only have SPEC-001 once in the related list
    let output = ark()
        .args(["-F", "json", "show", "BL-001"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let related = parsed["related"].as_array().unwrap();
    assert_eq!(related.len(), 1);
    assert_eq!(related[0], "SPEC-001");
}

#[test]
fn test_relate_multiple_targets() {
    let dir = setup_multi_type_project();
    ark()
        .args(["new", "task", "--title", "Task A", "--priority", "10"])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args(["new", "task", "--title", "Task B", "--priority", "20"])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args(["new", "spec", "--title", "Spec A"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["relate", "BL-001", "BL-002", "SPEC-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Related BL-001 <-> [BL-002, SPEC-001]",
        ));

    // BL-001 should list both
    let output = ark()
        .args(["-F", "json", "show", "BL-001"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let related = parsed["related"].as_array().unwrap();
    assert_eq!(related.len(), 2);
}

#[test]
fn test_relate_self_reference_rejected() {
    let dir = setup_multi_type_project();
    ark()
        .args(["new", "task", "--title", "Task A", "--priority", "10"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["relate", "BL-001", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot relate an artifact to itself",
        ));
}

#[test]
fn test_relate_nonexistent_artifact() {
    let dir = setup_multi_type_project();
    ark()
        .args(["new", "task", "--title", "Task A", "--priority", "10"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["relate", "BL-001", "SPEC-999"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("artifact not found"));
}

#[test]
fn test_relate_updates_date() {
    let dir = setup_multi_type_project();
    ark()
        .args(["new", "task", "--title", "Task A", "--priority", "10"])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args(["new", "spec", "--title", "Spec A"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["relate", "BL-001", "SPEC-001"])
        .current_dir(dir.path())
        .assert()
        .success();

    // Both should have an updated field
    ark()
        .args(["show", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("updated:"));

    ark()
        .args(["show", "SPEC-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("updated:"));
}

#[test]
fn test_context_pretty() {
    let dir = setup_multi_type_project();
    ark()
        .args(["new", "task", "--title", "Build thing", "--priority", "10"])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args(["new", "spec", "--title", "Thing spec"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["relate", "BL-001", "SPEC-001"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["context", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Build thing"))
        .stdout(predicate::str::contains("Related:"))
        .stdout(predicate::str::contains("SPEC-001"))
        .stdout(predicate::str::contains("Thing spec"));
}

#[test]
fn test_context_json() {
    let dir = setup_multi_type_project();
    ark()
        .args(["new", "task", "--title", "Build thing", "--priority", "10"])
        .current_dir(dir.path())
        .assert()
        .success();
    ark()
        .args(["new", "spec", "--title", "Thing spec"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["relate", "BL-001", "SPEC-001"])
        .current_dir(dir.path())
        .assert()
        .success();

    let output = ark()
        .args(["-F", "json", "context", "BL-001"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Primary has full content including body
    assert_eq!(parsed["primary"]["id"], "BL-001");
    assert_eq!(parsed["primary"]["title"], "Build thing");
    assert!(parsed["primary"]["body"].is_string());

    // Related has frontmatter only (no body)
    let related = parsed["related"].as_array().unwrap();
    assert_eq!(related.len(), 1);
    assert_eq!(related[0]["id"], "SPEC-001");
    assert_eq!(related[0]["title"], "Thing spec");
    assert!(related[0].get("body").is_none());
}

#[test]
fn test_context_no_relations() {
    let dir = setup_multi_type_project();
    ark()
        .args(["new", "task", "--title", "Lonely task", "--priority", "10"])
        .current_dir(dir.path())
        .assert()
        .success();

    // Context should still show the primary artifact
    ark()
        .args(["context", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Lonely task"));

    // JSON should have empty related array
    let output = ark()
        .args(["-F", "json", "context", "BL-001"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let related = parsed["related"].as_array().unwrap();
    assert!(related.is_empty());
}

#[test]
fn test_context_nonexistent_artifact() {
    let dir = setup_multi_type_project();
    ark()
        .args(["context", "BL-999"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("artifact not found"));
}

// --- Schema inheritance tests ---

#[test]
fn test_schema_inheritance_types() {
    let dir = TempDir::new().unwrap();
    ark().arg("init").current_dir(dir.path()).assert().success();

    let base_schema = r#"name: base-task
directory: base-backlog
prefix: BB
fields:
  - name: id
    type: string
    required: true
    derived: true
  - name: title
    type: string
    required: true
  - name: status
    type: enum
    required: true
    values: [backlog, active, blocked, done]
    default: backlog
"#;

    let child_schema = r#"name: task
extends: base-task
directory: backlog
prefix: BL
fields:
  - name: priority
    type: integer
    required: true
    unique: true
  - name: project
    type: enum
    required: true
    values: [alpha, beta]
"#;

    let schemas_dir = dir.path().join(".ark").join("schemas");
    fs::write(schemas_dir.join("base-task.yml"), base_schema).unwrap();
    fs::write(schemas_dir.join("task.yml"), child_schema).unwrap();

    // Both types should show up in ark types
    ark()
        .args(["types"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("base-task"))
        .stdout(predicate::str::contains("task"));
}

#[test]
fn test_schema_inheritance_create_artifact() {
    let dir = TempDir::new().unwrap();
    ark().arg("init").current_dir(dir.path()).assert().success();

    let base_schema = r#"name: base-task
directory: base-backlog
prefix: BB
fields:
  - name: id
    type: string
    required: true
    derived: true
  - name: title
    type: string
    required: true
  - name: status
    type: enum
    required: true
    values: [backlog, active, blocked, done]
    default: backlog
  - name: created
    type: date
    derived: true
  - name: updated
    type: date
    derived: true
"#;

    let child_schema = r#"name: task
extends: base-task
directory: backlog
prefix: BL
fields:
  - name: priority
    type: integer
    required: true
    unique: true
  - name: project
    type: enum
    required: true
    values: [alpha, beta]
"#;

    let schemas_dir = dir.path().join(".ark").join("schemas");
    fs::write(schemas_dir.join("base-task.yml"), base_schema).unwrap();
    fs::write(schemas_dir.join("task.yml"), child_schema).unwrap();

    // Create an artifact using the child type - it should have inherited fields
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Inherited task",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    // The artifact should have inherited status from base
    ark()
        .args(["show", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Inherited task"))
        .stdout(predicate::str::contains("backlog"));

    // Should be able to list artifacts
    ark()
        .args(["list", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Inherited task"));
}

#[test]
fn test_schema_inheritance_field_override() {
    let dir = TempDir::new().unwrap();
    ark().arg("init").current_dir(dir.path()).assert().success();

    let base_schema = r#"name: base-item
directory: items
prefix: IT
fields:
  - name: id
    type: string
    required: true
    derived: true
  - name: title
    type: string
    required: true
  - name: status
    type: enum
    required: true
    values: [open, closed]
    default: open
  - name: created
    type: date
    derived: true
  - name: updated
    type: date
    derived: true
"#;

    // Child overrides status values
    let child_schema = r#"name: ticket
extends: base-item
directory: tickets
prefix: TK
fields:
  - name: status
    type: enum
    required: true
    values: [new, triaged, in-progress, resolved]
    default: new
"#;

    let schemas_dir = dir.path().join(".ark").join("schemas");
    fs::write(schemas_dir.join("base-item.yml"), base_schema).unwrap();
    fs::write(schemas_dir.join("ticket.yml"), child_schema).unwrap();

    // Create a ticket - status should use the overridden values
    ark()
        .args(["new", "ticket", "--title", "My ticket"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["show", "TK-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("new"));

    // The base status values (open/closed) should not be valid
    ark()
        .args(["edit", "TK-001", "--status", "open"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// --- BL fixes tests ---

#[test]
fn test_filter_warns_invalid_enum() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Test",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    // Filtering by invalid status should warn on stderr
    ark()
        .args(["list", "task", "--status", "nonexistent"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("not a known value"));
}

#[test]
fn test_case_insensitive_search() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "UPPERCASE title",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    // Case sensitive (default) should not match lowercase
    ark()
        .args(["search", "uppercase"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No artifacts matching"));

    // Case insensitive should match
    ark()
        .args(["search", "-i", "uppercase"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("BL-001"));
}

#[test]
fn test_edit_archived_item() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "To archive",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["edit", "BL-001", "--status", "done"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark()
        .args(["archive", "task"])
        .current_dir(dir.path())
        .assert()
        .success();

    // Should be able to edit the archived item (e.g. reopen it)
    ark()
        .args(["edit", "BL-001", "--status", "active"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated BL-001"));
}

#[test]
fn test_set_conflict_detection() {
    let dir = setup_project();
    ark()
        .args([
            "new",
            "task",
            "--title",
            "Test",
            "--project",
            "alpha",
            "--priority",
            "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    // --status and --set status should conflict
    ark()
        .args([
            "edit",
            "BL-001",
            "--status",
            "active",
            "--set",
            "status=done",
        ])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("conflict"));
}

// --- Registry pull tests ---

#[test]
fn test_registry_pull_no_registry_schemas() {
    let dir = setup_project();
    ark()
        .args(["registry-pull"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("no schemas with registry URLs"));
}

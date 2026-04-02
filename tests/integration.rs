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

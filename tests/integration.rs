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
    ark().arg("init")
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
    ark().arg("init")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn test_types_lists_schemas() {
    let dir = setup_project();
    ark().arg("types")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("task"));
}

#[test]
fn test_fields_lists_all_fields() {
    let dir = setup_project();
    ark().args(["fields", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("priority"));
}

#[test]
fn test_fields_shows_enum_values() {
    let dir = setup_project();
    ark().args(["fields", "task", "status"])
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
    ark().args([
            "new", "task",
            "--title", "Test task",
            "--project", "alpha",
            "--kind", "chore",
            "--priority", "10",
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
    ark().args([
            "new", "task",
            "--title", "Bad task",
            "--project", "nonexistent",
            "--priority", "10",
        ])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_list_shows_artifacts() {
    let dir = setup_project();
    ark().args([
            "new", "task",
            "--title", "First",
            "--project", "alpha",
            "--priority", "20",
        ])
        .current_dir(dir.path())
        .assert()
        .success();
    ark().args([
            "new", "task",
            "--title", "Second",
            "--project", "beta",
            "--priority", "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    // Default list shows both, sorted by priority
    ark().args(["list", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("First"))
        .stdout(predicate::str::contains("Second"));

    // Filter by project
    ark().args(["list", "task", "--project", "alpha"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("First"))
        .stdout(predicate::str::contains("Second").not());
}

#[test]
fn test_tsv_output() {
    let dir = setup_project();
    ark().args([
            "new", "task",
            "--title", "TSV test",
            "--project", "alpha",
            "--priority", "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark().args(["-F", "tsv", "list", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("id\t"))
        .stdout(predicate::str::contains("BL-001"));
}

#[test]
fn test_edit_updates_field() {
    let dir = setup_project();
    ark().args([
            "new", "task",
            "--title", "Edit me",
            "--project", "alpha",
            "--priority", "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark().args(["edit", "BL-001", "--status", "active"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark().args(["show", "BL-001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("status: active"));
}

#[test]
fn test_lint_passes_valid_artifacts() {
    let dir = setup_project();
    ark().args([
            "new", "task",
            "--title", "Valid task",
            "--project", "alpha",
            "--priority", "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark().args(["lint", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Lint passed"));
}

#[test]
fn test_lint_catches_invalid_enum() {
    let dir = setup_project();
    ark().args([
            "new", "task",
            "--title", "Will be broken",
            "--project", "alpha",
            "--priority", "10",
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

    ark().args(["lint", "task"])
        .current_dir(dir.path())
        .assert()
        .failure();
}

#[test]
fn test_archive_moves_done_items() {
    let dir = setup_project();
    ark().args([
            "new", "task",
            "--title", "To archive",
            "--project", "alpha",
            "--priority", "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark().args(["edit", "BL-001", "--status", "done"])
        .current_dir(dir.path())
        .assert()
        .success();

    ark().args(["archive", "task"])
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
    ark().args([
            "new", "task",
            "--title", "Build prototype XYZ",
            "--project", "alpha",
            "--priority", "10",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark().args(["search", "XYZ"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("BL-001"));

    ark().args(["search", "nonexistent"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No artifacts matching"));
}

#[test]
fn test_schema_help() {
    let dir = TempDir::new().unwrap();
    ark().arg("init").current_dir(dir.path()).assert().success();
    ark().arg("schema-help")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Schema File Reference"))
        .stdout(predicate::str::contains("Field Types"));
}

#[test]
fn test_rebalance() {
    let dir = setup_project();
    ark().args([
            "new", "task",
            "--title", "First",
            "--project", "alpha",
            "--priority", "5",
        ])
        .current_dir(dir.path())
        .assert()
        .success();
    ark().args([
            "new", "task",
            "--title", "Second",
            "--project", "alpha",
            "--priority", "7",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    ark().args(["rebalance", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Rebalanced"));

    // After rebalance, priorities should be 10, 20
    ark().args(["-F", "tsv", "list", "task"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("20"));
}

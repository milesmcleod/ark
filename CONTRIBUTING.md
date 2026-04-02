# Contributing to ark

Thanks for your interest in contributing. ark is in early development and we welcome bug reports, feature ideas, and pull requests.

## Getting started

```bash
git clone https://github.com/milesmcleod/ark.git
cd ark
cargo build
cargo test
```

Activate the pre-commit hook:

```bash
git config core.hooksPath .githooks
```

This runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` before every commit.

## Development workflow

1. Fork and create a feature branch
2. Write your changes
3. Add or update tests - every behavior change needs a test
4. Run `cargo test` (the pre-commit hook does this, but run it early and often)
5. Run `cargo clippy -- -D warnings` and `cargo fmt`
6. Open a PR against `main`

## Code organization

```
src/
  main.rs           - entry point, arg parsing, command dispatch
  cli.rs            - clap derive structs (CLI shape)
  commands/         - one file per subcommand
  schema.rs         - schema loading and JSON Schema generation
  artifact.rs       - artifact parsing, serialization, YAML handling
  validate.rs       - write-time validation (enums, types, required fields)
  output.rs         - output format dispatch (pretty/tsv/json)
  lock.rs           - file-based locking for concurrent ID safety
  error.rs          - error types
tests/
  integration.rs    - end-to-end CLI tests
```

## Conventions

- **Rust 2024 edition.** We use let chains, `is_some_and`, and other modern Rust features.
- **Tests alongside source.** Unit tests go in `#[cfg(test)]` modules in the source file. Integration tests go in `tests/`.
- **No unsafe.** There is no reason to use unsafe in this codebase.
- **Error handling.** `anyhow` for commands, typed errors in `error.rs` for domain-specific cases.
- **Output.** All normal output goes to stdout. Errors and warnings go to stderr. This matters - agents pipe stdout.

## Testing

Unit tests cover core logic (parsing, serialization, schema validation). Integration tests cover the full CLI end-to-end using `assert_cmd` and `tempfile`.

When adding a new command or fixing a bug, add an integration test that exercises the full command pipeline. The pattern is:

```rust
#[test]
fn test_your_feature() {
    let dir = setup_project();  // creates tempdir with ark init + task schema
    ark().args(["your", "command"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

## Schema changes

If you're adding a new field type, constraint, or schema feature:

1. Update `Schema` and related structs in `schema.rs`
2. Update `to_json_schema()` for lint validation
3. Update `validate_field_value()` in `validate.rs` for write-time validation
4. Update `schema-help` output in `commands/schema_help.rs`
5. Add tests for both the happy path and error cases

## Reporting bugs

Open an issue with:

- What you did (command you ran)
- What you expected
- What happened instead
- Your schema file (if relevant)
- The artifact file content (if relevant)

If ark corrupted a file or lost data, that's a critical bug - please include as much detail as possible.

## Feature ideas

ark is opinionated by design. Before building a large feature, open an issue to discuss the approach. Things we care about:

- **Token efficiency.** Every byte of CLI output costs tokens when an agent reads it. Less is more.
- **Schema-driven.** New features should work with any artifact type, not just tasks or specs.
- **Git-friendly.** Artifacts are files. Changes are diffs. Don't fight this.
- **Zero infrastructure.** No servers, no databases, no accounts.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

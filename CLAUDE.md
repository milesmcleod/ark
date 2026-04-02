# ark

Human-legible, agent-optimized project artifact management.

A local-first, git-native, schema-enforced CLI for managing structured markdown artifacts - tasks, specs, ADRs, or whatever you define. The CLI is the API. Markdown is the storage layer. Schemas make it queryable and enforceable.

## Dogfooding

This project uses ark to manage itself. The backlog, specs, and ADRs in this repo are all ark-managed artifacts. Use `ark` commands to interact with them:

```bash
ark types                    # see what artifact types are defined
ark list task                # view the backlog
ark next task                # see what's active and queued
ark list spec                # view specs
ark list adr                 # view architectural decisions
ark lint                     # validate everything
ark search "pattern"         # find artifacts by content
ark diff HEAD~1              # see what changed recently
```

When creating new backlog items, specs, or ADRs, use `ark new` rather than creating files manually. This ensures correct IDs, frontmatter, and validation.

## Design Principles

- **Agent-first, human-legible.** Every command's output is designed for LLM token efficiency while remaining readable by humans. The CLI output IS the agent's context - it should communicate state AND prompt the next action.
- **Schema-driven.** The tool doesn't know what a "task" or "spec" is. It knows what an artifact type is, and user-defined YAML schemas tell it everything else. Artifact types are pluggable. See ADR-003.
- **Git-native.** Every artifact is a markdown file. Every change is a diff. Branching, merging, blame, bisect all work on your project management artifacts.
- **Zero infrastructure.** No server, no database, no account, no API keys. Files in a directory.
- **Priority over dependencies.** Global integer priority ordering replaces dependency graphs. The ordering IS the dependency information. Agents infer the why from specs and ADRs. See ADR-002.

## Architecture

```
.ark/                       # marks an ark-managed project (like .git/)
  schemas/                  # artifact type definitions (YAML)
  hooks.yml                 # optional lifecycle hooks
  .lock                     # transient lock file for concurrent safety
.arkignore                  # optional glob patterns to exclude from scan
backlog/                    # task artifacts (directory declared in schema)
  done/                     # archived artifacts
spec/                       # Gherkin feature specs (8 files, 94 scenarios)
adr/                        # architectural decision records (8 files)
```

Key architectural decisions are documented in adr/:
- ADR-001: Design philosophy
- ADR-002: Priority ordering over dependency graphs
- ADR-003: Schema-driven artifact types
- ADR-004: serde_yml for YAML serialization
- ADR-005: File-based locking
- ADR-006: Scan as read-only cross-project aggregation
- ADR-007: Lifecycle hooks via shell execution
- ADR-008: Two-stage frontmatter pipeline

## Code Organization

```
src/
  main.rs                   # entry point, arg parsing, command dispatch
  cli.rs                    # clap derive structs - Cli, Command enum, all Args
  artifact.rs               # Artifact struct, frontmatter parsing, YAML serialization, find_artifact_by_id
  schema.rs                 # schema loading, inheritance resolution, JSON Schema generation
  validate.rs               # write-time validation (enums, types, required fields, coercion)
  discover.rs               # recursive .ark/ project discovery for scan, .arkignore support
  output.rs                 # format dispatch (pretty/tsv/json)
  lock.rs                   # file-based locking with stale detection
  error.rs                  # error types (thiserror)
  commands/                 # one file per subcommand
    init.rs                 # ark init
    types.rs                # ark types
    fields.rs               # ark fields <type> [field]
    list.rs                 # ark list <type> [filters]
    next.rs                 # ark next <type> [n]
    show.rs                 # ark show <id>
    new.rs                  # ark new <type> [fields]
    edit.rs                 # ark edit <id> [fields]
    lint.rs                 # ark lint [target]
    archive.rs              # ark archive <type>
    rebalance.rs            # ark rebalance <type>
    stats.rs                # ark stats [type]
    relate.rs               # ark relate <id> <ids...>
    context.rs              # ark context <id>
    diff.rs                 # ark diff <ref>
    stale.rs                # ark stale <type>
    search.rs               # ark search <pattern>
    scaffold.rs             # ark scaffold <path>
    hooks.rs                # ark hooks + hook execution
    registry_pull.rs        # ark registry-pull
    scan.rs                 # ark scan (types/list/next/stats/search/lint)
    completions.rs          # ark completions <shell>
    schema_help.rs          # ark schema-help
tests/
  integration.rs            # 58 end-to-end CLI tests
```

## Development

```bash
cargo build                 # build debug
cargo test                  # run all tests (77 total)
cargo clippy -- -D warnings # lint
cargo fmt                   # format
cargo install --path .      # install to PATH
```

Pre-commit hook runs fmt, clippy, and tests automatically. Activate after clone:

```bash
git config core.hooksPath .githooks
```

## Conventions

- Rust 2024 edition with let chains
- Tests alongside source in #[cfg(test)] modules, integration tests in tests/
- Error handling: thiserror for ArkError variants, anyhow for command-level errors
- All normal output to stdout, errors and warnings to stderr
- CLI flag `--kind` maps to the `type` field in frontmatter (--type conflicts with clap)
- Write-time validation enforces schema constraints on new/edit, not just lint
- File locking on all write operations (new, edit, archive, rebalance)
- Scan is read-only - never writes across project boundaries

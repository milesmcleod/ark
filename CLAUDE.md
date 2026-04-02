# ark

Human-legible, agent-optimized project artifact management.

A local-first, git-native, schema-enforced CLI for managing structured markdown artifacts - tasks, specs, ADRs, or whatever you define. The CLI is the API. Markdown is the storage layer. Schemas make it queryable and enforceable.

## Design Principles

- **Agent-first, human-legible.** Every command's output is designed for LLM token efficiency while remaining readable by humans. The CLI output IS the agent's context - it should communicate state AND prompt the next action.
- **Schema-driven.** The tool doesn't know what a "task" or "spec" is. It knows what an artifact type is, and user-defined YAML schemas tell it everything else. Artifact types are pluggable.
- **Git-native.** Every artifact is a markdown file. Every change is a diff. Branching, merging, blame, bisect all work on your project management artifacts.
- **Zero infrastructure.** No server, no database, no account, no API keys. Files in a directory.
- **Priority over dependencies.** Global integer priority ordering replaces dependency graphs. The ordering IS the dependency information. Agents infer the why from specs and ADRs.

## Architecture

```
.ark/                       # marks an ark-managed project (like .git/)
  schemas/                  # artifact type definitions (YAML)
    task.yml
    spec.yml
    adr.yml
backlog/                    # task artifacts (directory declared in schema)
  done/                     # archived artifacts
spec/                       # spec artifacts
adr/                        # ADR artifacts
```

Each schema declares its artifact type's name, prefix, directory, fields (with types, constraints, enums), and body template. The CLI reads schemas to know where artifacts live and how to validate them.

## Code Organization

```
src/
  main.rs                   # entry point, arg parsing, command dispatch
  cli.rs                    # clap derive structs - Cli, Command enum, all Args
  commands/                 # one file per subcommand
    mod.rs
    init.rs                 # ark init
    list.rs                 # ark list <type>
    show.rs                 # ark show <id>
    new.rs                  # ark new <type>
    edit.rs                 # ark edit <id>
    lint.rs                 # ark lint
    archive.rs              # ark archive <type>
    rebalance.rs            # ark rebalance <type>
    fields.rs               # ark fields <type> [field]
    types.rs                # ark types
    stats.rs                # ark stats
    search.rs               # ark search <query>
    completions.rs          # ark completions <shell>
    schema_help.rs          # ark schema-help
  schema.rs                 # schema loading, parsing, JSON Schema generation
  artifact.rs               # core Artifact struct, frontmatter parsing, serialization
  output.rs                 # format dispatch (pretty/tsv/json)
  error.rs                  # error types
```

## Development

```bash
cargo build                 # build debug
cargo test                  # run all tests
cargo run -- init           # test commands locally
cargo run -- list task      # etc.
```

## Conventions

- Rust 2024 edition
- No semicolons in the last expression of a block (Rust convention - return values are implicit)
- Tests live alongside source in #[cfg(test)] modules
- Integration tests in tests/ directory
- Error handling via anyhow in commands, thiserror for typed errors in core modules when needed
- All CLI output goes to stdout. Errors and warnings to stderr.

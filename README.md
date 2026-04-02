# ark

Human-legible, agent-optimized project artifact management.

ark is a local-first, git-native CLI for managing structured markdown artifacts - tasks, specs, ADRs, or whatever you define. You bring your own artifact types via YAML schemas. ark makes them queryable, enforceable, and discoverable.

The CLI is the API. Markdown is the storage layer. Git is the version control. There is no server, no database, no account.

## Why

Project management tools like Jira, Linear, and Trello are human-first - designed for browsers and dashboards. AI coding agents interact with them through API connectors, opaque field configurations, and HTTP round-trips. The result: wasted tokens, brittle integrations, and a fundamental mismatch between how agents work and how these tools expect to be used.

ark takes a different approach: **human-legible, agent-optimized**. Every artifact is a markdown file you can read in any editor. Every operation is a CLI command that returns minimal, predictable output. Schema discovery is built in - an agent can self-orient in any ark project by running `ark types` and `ark fields`.

## Install

```
cargo install --path .
```

Or build from source:

```
git clone https://github.com/milesmcleod/ark.git
cd ark
cargo build --release
# binary is at target/release/ark
```

## Quick start

```bash
# Initialize ark in your project
ark init

# Create a schema for your artifact type
cat > .ark/schemas/task.yml << 'EOF'
name: task
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
EOF

# Verify your schema
ark types
ark fields task
ark fields task status

# Create artifacts
ark new task --title "Build the thing" --priority 10
ark new task --title "Test the thing" --priority 20

# Query
ark list task
ark list task --status active
ark next task
ark search "thing"

# Edit
ark edit BL-001 --status active
ark edit BL-001 --set notes="looking good"

# Validate
ark lint

# Manage
ark archive task
ark rebalance task
ark stats task --by status
```

## Commands

| Command | Description |
|---|---|
| `ark init` | Initialize ark in the current directory |
| `ark types` | List registered artifact types |
| `ark fields <type> [field]` | Show fields and valid values for an artifact type |
| `ark list <type>` | List artifacts with filters and priority sorting |
| `ark next <type> [n]` | Show active items and top n queued items |
| `ark show <id>` | Show a single artifact by ID |
| `ark new <type>` | Create a new artifact with auto-generated ID |
| `ark edit <id>` | Update artifact frontmatter fields |
| `ark lint [target]` | Validate artifacts against their schemas |
| `ark archive <type>` | Move completed artifacts to archive directory |
| `ark rebalance <type>` | Re-number priorities in even increments |
| `ark stats [type]` | Show artifact counts and groupings |
| `ark search <pattern>` | Search artifact titles and bodies (regex) |
| `ark schema-help` | Show the full schema format reference |
| `ark completions <shell>` | Generate shell completions |

## Output formats

All listing commands support three output formats via the global `-F` flag:

- **pretty** (default) - formatted tables for human reading
- **tsv** - tab-separated values for agent consumption and piping
- **json** - structured JSON for programmatic use

```bash
ark list task              # pretty table
ark -F tsv list task       # tab-separated, minimal tokens
ark -F json list task      # structured JSON
```

## Schemas

Schemas are YAML files in `.ark/schemas/` that define artifact types. Each schema declares:

- **name** and **prefix** - the type name and ID prefix (e.g. `task` / `BL`)
- **directory** - where artifacts of this type live
- **fields** - typed, constrained field definitions
- **archive** - rules for archiving completed artifacts
- **template** - body template for new artifacts

Field types: `string`, `integer`, `date`, `enum`, `list`, `boolean`

Run `ark schema-help` for the complete format reference.

## Design principles

**Schema-driven.** The CLI doesn't know what a "task" or "spec" or "ADR" is. It knows what an artifact type is. Schemas tell it everything else. Define whatever artifact types your project needs.

**Priority over dependencies.** A single integer priority field replaces dependency graphs. Dependency graphs go stale. Priority ordering is maintained as part of normal triage. Agents infer dependency reasoning from context.

**Write-time validation.** Enum constraints, required fields, type checking, unique priorities, and pattern matching are all enforced when you create or edit artifacts - not just at lint time.

**Agent-friendly output.** Every command's output is designed for LLM token efficiency. Commands communicate state and prompt the next action. Schema discovery (`ark types`, `ark fields`) lets an agent self-orient in any ark project without documentation.

**Git-native.** Every artifact is a file. Every change is a diff. Branching, merging, blame, and bisect work on your project management artifacts the same way they work on code.

## License

MIT

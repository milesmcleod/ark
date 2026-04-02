use anyhow::Result;

pub fn run() -> Result<()> {
    println!(
        r#"ark Schema File Reference
========================

Schema files live in .ark/schemas/ and define artifact types. Each .yml
file in that directory registers one artifact type.

Format
------

  name: <string>           # artifact type name (used in all commands)
  directory: <string>      # where artifacts of this type live (relative to project root)
  prefix: <string>         # ID prefix (e.g. BL, SPEC, ADR)
  fields:                  # list of field definitions
    - name: <string>       # field name (used in frontmatter)
      type: <type>         # field type (see below)
      required: <bool>     # is this field required? (default: false)
      unique: <bool>       # must values be unique across artifacts? (default: false)
      derived: <bool>      # auto-managed by ark? (default: false)
      default: <value>     # default value when creating new artifacts
      values: [<strings>]  # valid values (for enum type)
      pattern: <regex>     # validation pattern (for string type)
  archive:                 # optional archive configuration
    field: <string>        # field to check for archive eligibility
    value: <string>        # value that triggers archiving
    directory: <string>    # where to move archived artifacts
  extends: <string>        # optional: inherit fields from another schema by name
  registry: <url>          # optional: URL to fetch this schema from (ark registry-pull)
  template: |              # optional body template for new artifacts
    ## Section
    Content here.

Schema Inheritance
------------------

  A schema can extend another using the `extends` field:

    name: my-task
    extends: base-task
    directory: backlog
    prefix: BL
    fields:
      - name: project
        type: enum
        values: [alpha, beta]

  The child inherits all fields from the base. Child fields with the
  same name override the base. Directory, prefix, archive, and template
  are inherited if not set by the child.

Schema Registry
---------------

  A schema can declare a registry URL:

    name: task
    registry: https://raw.githubusercontent.com/org/schemas/main/task.yml

  Run `ark registry-pull` to fetch the latest version from all registry
  URLs. The fetched content replaces the local schema file. Edit locally
  after fetch to customize.

CLI Flag Mapping
----------------

  Some schema field names map to different CLI flags:
    title    -> --title
    status   -> --status
    priority -> --priority
    project  -> --project
    type     -> --kind    (--type conflicts with clap, use --kind)

  For any other field, use --set key=value.

Field Types
-----------

  string    - free text, optionally validated by pattern
  integer   - whole number
  date      - ISO date (YYYY-MM-DD)
  enum      - one of a fixed set of values (defined in 'values')
  list      - array of strings (rendered as [a, b, c] in frontmatter)
  boolean   - true or false

Example
-------

  name: task
  directory: backlog
  prefix: BL
  fields:
    - name: id
      type: string
      required: true
      derived: true
      pattern: "^BL-\\d{{3,}}$"
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
      values: [ecosystem, myproject]
    - name: type
      type: enum
      required: true
      values: [feature, bug, chore, research]
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
"#
    );

    Ok(())
}

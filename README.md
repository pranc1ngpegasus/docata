# docata

`docata` is a Rust workspace that builds and queries a document catalog for Markdown documents using YAML frontmatter.

`docata` can be read as doc(ument) + catalog, and in Japanese it also sounds like 土方 👷.

The workspace contains two crates:

- `docata`  
  Core library for scanning Markdown files and building/querying dependency data
- `docata-cli`  
  CLI implementation (`docata`) backed by the `docata` library

## Features

- Scans Markdown files (`.md`) recursively in a target directory
- Extracts frontmatter fields:
  - `id`: unique document identifier (required)
  - `deps`: dependency IDs (optional)
  - `type` / `domain` / `status` / `source_of_truth` (optional, for node metadata output)
- Generates a JSON catalog representing nodes and edges
- Deterministic output for `nodes`/`edges` ordering and normalized `path` strings
- Validation checks:
  - duplicate IDs
  - unresolved dependencies
  - dependency cycles
- Queries the catalog:
  - `deps`: direct dependencies for a given ID
  - `refs`: documents that reference a given ID

## Frontmatter format

Each document should start with frontmatter in this form:

```md
---
id: foo
deps:
  - bar
  - baz
type: spec
domain: billing
status: draft
source_of_truth: handbook
---

Body...
```

- `id` is required
- `deps` is optional
- `type` / `domain` / `status` / `source_of_truth` are optional
- Files without valid frontmatter including `id` are skipped

## Installation

- Requires Rust 2024 toolchain (`cargo`)
- Install the CLI from repository root:

```bash
cargo install --path docata-cli
```

## Usage

### Build a catalog

```bash
# Default: scan `./docs` and output `./docs/catalog.json`
docata build

# Specify paths explicitly
docata build ./docs ./docs/catalog.json

# Include node metadata (`type`, `domain`, `status`, `source_of_truth`) in output
docata build ./docs ./docs/catalog.json --with-node-metadata
```

### Check catalog in CI

```bash
# Structural checks (duplicate IDs, unresolved dependencies, cycles)
docata check ./docs

# Structural checks + ensure no regeneration diff against existing catalog
docata check ./docs --catalog ./docs/catalog.json

# Use the same metadata mode as build when catalog includes node metadata
docata check ./docs --catalog ./docs/catalog.json --with-node-metadata
```

### Query dependencies

```bash
# Text output: one ID per line
docata deps foo

# JSON output with metadata
docata deps foo --format json

# Fail with non-zero exit if `foo` does not exist in nodes
docata deps foo --strict
```

### Query reverse references

```bash
# Text output
docata refs foo

# JSON output
docata refs foo --format json

# Fail with non-zero exit if `foo` does not exist in nodes
docata refs foo --strict
```

### Example output

With these docs:

- `docs/foo.md` -> `id: foo`
- `docs/bar.md` -> `id: bar`, `deps: [foo]`
- `docs/foo/hoge.md` -> `id: hoge`, `deps: [bar]`

`cat docs/catalog.json` looks like:

```json
{
  "nodes": [
    { "id": "foo", "path": "docs/foo.md" },
    { "id": "bar", "path": "docs/bar.md" },
    { "id": "hoge", "path": "docs/foo/hoge.md" }
  ],
  "edges": [
    { "from": "bar", "to": "foo" },
    { "from": "hoge", "to": "bar" }
  ]
}
```

`docata refs foo` (text output):

```txt
bar
```

## Development

```bash
cargo build --workspace
cargo fmt
cargo clippy
```

## License

MIT

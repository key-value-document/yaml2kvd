# KVD — Key-Value Document format

KVD is a line-oriented config/data format that keeps YAML's readability
without its pitfalls: strict 2-space indentation, no flow collections, no
implicit coercion, no anchors or aliases, and exactly one way to spell most
things.

## Quickstart

Parse and serialize in Rust:

```toml
[dependencies]
kvd-rs = "1.0.0"
```

```rust
let doc = kvd_rs::deserialize::from_str("app:\n  port: 8080\n")?;
let port = doc.as_map().unwrap().get("app").unwrap()
    .as_map().unwrap().get("port").unwrap();
assert_eq!(port.as_scalar().unwrap().text, "8080");
let text = kvd_rs::serialize::to_string(&doc)?; // canonical form
```

Convert from YAML on the command line:

```sh
cargo install yaml2kvd
yaml2kvd values.yaml > values.kvd          # optional: --schema schema.yaml
yaml2kvd --reverse values.kvd             # KVD → YAML
```

## Example

Data (`app.kvd`):

```
# server config
app:
  name: "hello"
  port: 8080
  version: "1.5.2"
  endpoints:
    - path: "/health"
      method: "GET"
  hooks:
    - """
      #!/bin/sh
      echo hi
      """
  retries: null
```

Companion schema (`app.schema.kvd`) — a bare tree whose values are types:

```
app:
  name: str
  port: int
  version: str
  endpoints:
    - path: str
      method: str
  hooks:
    - str
  retries:
    type: int
    optional: true   # absent, int, or null
```

Notes:

- Values are one line (`"..."`) or a `"""..."""` block (also usable as a list
  item), or an indented subtree; `{}` and `[]` are the empty-collection
  literals. All string values are double-quoted — there are no bare words in
  value position.
- The only unquoted tokens with non-string meaning are `true`, `false`,
  `null`, integer and float literals, `{}`, `[]`, and type names matching the
  `type` grammar (`[a-z][a-z0-9_-]*`). Type names are
  meaningful only in schema position; in data documents they parse as strings.
  `null` is valid only where the schema says the type is optional
  (`optional: true`).
  Any other unquoted token is an `unexpected-character` error.
- Schemas are optional companions: data files parse standalone with default
  shapes. A schema file is a bare `key: type` tree (no metakeys); type names
  are the four builtins (`int`, `float`, `bool`, `str`), and may be embedded
  in a data document under `__schema__`.

## Crate

This repository provides the [`yaml2kvd`](https://crates.io/crates/yaml2kvd)
binary crate — the YAML↔KVD converter. It reads YAML and writes KVD to
stdout; `--schema <schema.yaml>` verifies the converted document before
emitting (the schema is a YAML file converted to a KVD schema internally).
`--reverse <input.kvd>` inverts the direction (KVD → YAML).

Core parsing/serialization and schema verification live in
[`kvd-rs`](https://crates.io/crates/kvd-rs) (`kvd_rs::deserialize::from_str`,
`kvd_rs::serialize::to_string`, `kvd_rs::schema::verify`).

### Schema verification

```rust
let doc = kvd_rs::deserialize::from_str(text)?;      // text -> Node
let schema = kvd_rs::deserialize::from_str(schema_text)?;   // bare key: type tree
if let Err(violations) = kvd_rs::schema::verify(&doc, &schema) {
    for v in &violations {
        eprintln!("{v}");                            // path + message per violation
    }
}
// optional types: verify(&doc, &schema) where `retries: { type: int, optional: true }` allows absence/null
```

### Serde support

With the optional `serde` feature, derived types read and write KVD
directly — no manual `Node` traversal:

```toml
[dependencies]
kvd-rs = { version = "1.0.0", features = ["serde"] }
serde = { version = "1.0.0", features = ["derive"] }
```

```rust
#[derive(Serialize, Deserialize)]
struct App {
    name: String,
    port: u16,
    retries: Option<u32>,
}

let app: App = kvd_rs::from_str(text)?;      // KVD text -> T
let text = kvd_rs::to_string(&app)?;         // canonical form
// also: from_reader / from_file / to_writer / to_file
```

Shapes are enforced strictly: an integer target rejects quoted text,
floats always emit with a fraction (`1.0`), and non-finite floats are an
error. Externally-tagged enums round-trip as single-entry maps; unit
variants read bare strings.

## Status

Spec 1.0 — see [docs/README.md](docs/README.md) (index) and
[docs/spec/](docs/spec/) for the sections.

## License

MIT

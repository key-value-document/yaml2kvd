# yaml2kvd

Convert YAML documents to KVD — and KVD to YAML with `--reverse`.

KVD is a line-oriented config/data format that keeps YAML readability without pitfalls: strict 2-space indentation, no flow collections, no implicit coercion, exactly one way to spell most things.

## Install

```sh
cargo install yaml2kvd
```

## Usage

```sh
yaml2kvd values.yaml > values.kvd          # optional: --schema schema.yaml
yaml2kvd --reverse values.kvd > values.yaml
```

- Without `--reverse`, reads YAML and writes KVD to stdout.
- With `--schema <schema.yaml>`, the YAML schema is converted to a KVD schema node and the converted document is verified against it before emitting.
- With `--reverse <input.kvd>`, reads KVD and writes YAML to stdout. `--schema` is not valid with `--reverse`.

Help:

```sh
yaml2kvd --help
```

## Example

Input `values.yaml`:

```yaml
app:
  name: hello
  port: 8080
```

Output `values.kvd`:

```
app:
  name: "hello"
  port: 8080
```

Reverse:

```sh
yaml2kvd --reverse values.kvd
```

## Library

```toml
[dependencies]
yaml2kvd = "1.0.0"
kvd-rs = "1.0.0"
serde_yaml_ng = "0.10.0"
```

```rust
let kvd = yaml2kvd::yaml_text_to_kvd(r#"port: 8080"#, None)?;
let yaml = yaml2kvd::kvd_text_to_yaml("port: 8080\n")?;
```

Core parsing/serialization and schema verification live in [`kvd-rs`](https://crates.io/crates/kvd-rs).

## License

MIT

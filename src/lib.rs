//! YAML ↔ KVD conversion.
//!
//! Semantics (spec §5):
//! - YAML nulls become the `null` literal.
//! - Empty containers become `{}` / `[]`.
//! - Strings colliding with KVD shapes are quoted on emit.
//! - Multi-line strings become `"""` blocks.
//! - Non-finite floats (`nan`, `inf`) are rejected.

#![warn(missing_docs)]

use kvd_rs::value::{Map, Node, Shape};
use serde_yaml_ng::Value;
use std::fmt;

/// Conversion error (e.g. non-finite floats, unsupported YAML key types).
#[derive(Debug, Clone)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// Converts a YAML value to a KVD node.
pub fn from_yaml(v: &Value) -> Result<Node, Error> {
    match v {
        Value::Null => Ok(Node::scalar(Shape::Null, "null")),
        Value::Bool(b) => Ok(Node::scalar(Shape::Bool, b.to_string())),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Node::scalar(Shape::Int, i.to_string()))
            } else if let Some(u) = n.as_u64() {
                Ok(Node::scalar(Shape::Int, u.to_string()))
            } else if let Some(f) = n.as_f64() {
                if f.is_finite() {
                    Ok(Node::scalar(Shape::Float, fmt_float(f)))
                } else {
                    Err(Error(format!(
                        "non-finite float {f} has no KVD representation"
                    )))
                }
            } else {
                Err(Error("unsupported YAML number".to_string()))
            }
        }
        Value::String(s) => Ok(Node::scalar(Shape::Str, s.clone())),
        Value::Sequence(seq) => {
            let items: Result<Vec<_>, _> = seq.iter().map(from_yaml).collect();
            Ok(Node::list(items?))
        }
        Value::Mapping(map) => {
            let mut m = Map::new();
            for (k, v) in map {
                m.insert(key_to_string(k)?, from_yaml(v)?);
            }
            Ok(Node::map(m))
        }
        Value::Tagged(t) => from_yaml(&t.value),
    }
}

/// Converts a YAML mapping key to a string.
pub fn key_to_string(k: &Value) -> Result<String, Error> {
    match k {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_string())
            } else if let Some(u) = n.as_u64() {
                Ok(u.to_string())
            } else if let Some(f) = n.as_f64() {
                Ok(fmt_float(f))
            } else {
                Err(Error("unsupported YAML number key".to_string()))
            }
        }
        Value::Bool(b) => Ok(b.to_string()),
        _ => Err(Error(format!("unsupported YAML key: {k:?}"))),
    }
}

/// Formats an f64 as a KVD float literal.
fn fmt_float(f: f64) -> String {
    let s = format!("{f}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{f:.1}")
    }
}

/// Converts a KVD node to a YAML value tree.
pub fn to_yaml_value(node: &Node) -> Result<Value, Error> {
    match node {
        Node::Scalar(s) => match s.shape {
            Shape::Bool => Ok(Value::Bool(s.text == "true")),
            Shape::Int => {
                let clean = s.text.replace('_', "");
                if let Ok(i) = clean.parse::<i64>() {
                    Ok(Value::Number(i.into()))
                } else if let Ok(u) = clean.parse::<u64>() {
                    Ok(Value::Number(u.into()))
                } else {
                    Err(Error(format!(
                        "int literal `{}` out of range for YAML",
                        s.text
                    )))
                }
            }
            Shape::Float => {
                let f: f64 = s
                    .text
                    .parse()
                    .map_err(|_| Error(format!("invalid float literal `{}`", s.text)))?;
                if !f.is_finite() {
                    return Err(Error(format!(
                        "float literal `{}` has no YAML representation",
                        s.text
                    )));
                }
                Ok(Value::Number(f.into()))
            }
            Shape::Str => Ok(Value::String(s.text.clone())),
            Shape::Null => Ok(Value::Null),
        },
        Node::Map(m) | Node::Dict(m) => {
            let mut out = serde_yaml_ng::Mapping::new();
            for (k, v) in m.iter() {
                out.insert(Value::String(k.to_string()), to_yaml_value(v)?);
            }
            Ok(Value::Mapping(out))
        }
        Node::List(items) => {
            let items: Result<Vec<_>, _> = items.iter().map(to_yaml_value).collect();
            Ok(Value::Sequence(items?))
        }
        _ => Err(Error("unsupported node kind".to_string())),
    }
}

/// Serializes a KVD node as YAML text.
pub fn to_yaml(node: &Node) -> Result<String, Error> {
    let v = to_yaml_value(node)?;
    serde_yaml_ng::to_string(&v).map_err(|e| Error(e.to_string()))
}

/// Converts KVD text to YAML text.
pub fn kvd_text_to_yaml(kvd_text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let node = kvd_rs::deserialize::from_str(kvd_text)?;
    Ok(to_yaml(&node)?)
}

/// Converts YAML text to KVD text, optionally verifying against a YAML schema.
///
/// The schema (if provided) is itself a YAML file that mirrors the data
/// structure — same format as a KVD schema document but written in YAML.
/// It is converted to a KVD node via [`from_yaml`] before verification.
pub fn yaml_text_to_kvd(
    yaml_text: &str,
    schema_text: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let value: Value = serde_yaml_ng::from_str(yaml_text)?;
    let node = from_yaml(&value)?;
    if let Some(schema_text) = schema_text {
        let schema_value: Value = serde_yaml_ng::from_str(schema_text)?;
        let schema_doc = from_yaml(&schema_value)?;
        if let Err(e) = kvd_rs::schema::verify(&node, &schema_doc) {
            let violations = match &e {
                kvd_rs::schema::VerifyError::Violations(v)
                | kvd_rs::schema::VerifyError::SchemaMalformed(v) => v,
                other => return Err(other.to_string().into()),
            };
            for v in violations {
                eprintln!("{v}");
            }
            let label = if violations.len() == 1 {
                "schema error"
            } else {
                "schema errors"
            };
            return Err(format!("{} {label}", violations.len()).into());
        }
    }
    Ok(kvd_rs::serialize::to_string(&node)?)
}

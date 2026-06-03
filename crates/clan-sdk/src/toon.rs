// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! TOON — Token-Oriented Object Notation (spec §14).
//!
//! TOON encodes the same data model as JSON/YAML using indentation and
//! explicit `[n]` length declarations instead of brackets, for roughly 40%
//! fewer tokens on structured data. It is an **input-side** serialisation:
//! the SDK emits TOON when assembling agent context. Agents read it; nothing
//! parses it back (agent output is always JSON), so only encoding is defined.
//!
//! Example (spec §14):
//! ```text
//! vendor: Acme Corporation
//! total: 15375.00
//! status: pending-approval
//! line_items [2]
//!   description: Software Development Services
//!   amount: 12500.00
//!   description: Consulting
//!   amount: 2875.00
//! ```

use serde_yaml::Value;

const INDENT: &str = "  ";

/// Encode any YAML value as TOON.
pub fn to_toon(value: &Value) -> String {
    let mut out = String::new();
    encode_value(value, 0, &mut out);
    // Trim a single trailing newline for tidy output.
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

/// Encode bytes of YAML into TOON.
pub fn yaml_to_toon(yaml: &[u8]) -> crate::Result<String> {
    let value: Value = serde_yaml::from_slice(yaml)?;
    Ok(to_toon(&value))
}

fn encode_value(value: &Value, depth: usize, out: &mut String) {
    match value {
        Value::Mapping(map) => {
            for (k, v) in map {
                encode_pair(&scalar_to_string(k), v, depth, out);
            }
        }
        Value::Sequence(seq) => {
            // A bare top-level sequence: emit one item per line.
            for item in seq {
                encode_inline_or_block("", item, depth, out);
            }
        }
        scalar => {
            push_indent(depth, out);
            out.push_str(&scalar_to_string(scalar));
            out.push('\n');
        }
    }
}

/// Encode a `key: value` / `key [n]` / nested-`key` pair.
fn encode_pair(key: &str, value: &Value, depth: usize, out: &mut String) {
    match value {
        Value::Sequence(seq) => {
            push_indent(depth, out);
            out.push_str(key);
            out.push_str(&format!(" [{}]\n", seq.len()));
            for item in seq {
                encode_seq_item(item, depth + 1, out);
            }
        }
        Value::Mapping(_) => {
            push_indent(depth, out);
            out.push_str(key);
            out.push('\n');
            encode_value(value, depth + 1, out);
        }
        scalar => {
            push_indent(depth, out);
            out.push_str(key);
            out.push_str(": ");
            out.push_str(&scalar_to_string(scalar));
            out.push('\n');
        }
    }
}

/// Encode one element of a sequence. Mappings flatten their fields at this
/// indent; scalars print on their own line; nested collections recurse.
fn encode_seq_item(item: &Value, depth: usize, out: &mut String) {
    match item {
        Value::Mapping(map) => {
            for (k, v) in map {
                encode_pair(&scalar_to_string(k), v, depth, out);
            }
        }
        Value::Sequence(seq) => {
            push_indent(depth, out);
            out.push_str(&format!("[{}]\n", seq.len()));
            for inner in seq {
                encode_seq_item(inner, depth + 1, out);
            }
        }
        scalar => {
            push_indent(depth, out);
            out.push_str(&scalar_to_string(scalar));
            out.push('\n');
        }
    }
}

fn encode_inline_or_block(key: &str, value: &Value, depth: usize, out: &mut String) {
    if key.is_empty() {
        encode_seq_item(value, depth, out);
    } else {
        encode_pair(key, value, depth, out);
    }
}

fn push_indent(depth: usize, out: &mut String) {
    for _ in 0..depth {
        out.push_str(INDENT);
    }
}

/// Render a scalar value without quotes (TOON is unquoted).
fn scalar_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        // Collections shouldn't reach here as "scalars"; fall back to compact.
        other => serde_yaml::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_example() {
        let yaml = r#"
vendor: Acme Corporation
total: 15375.00
status: pending-approval
line_items:
  - description: Software Development Services
    amount: 12500.00
  - description: Consulting
    amount: 2875.00
"#;
        let toon = yaml_to_toon(yaml.as_bytes()).unwrap();
        // Numbers are normalised by the YAML parser (15375.00 -> 15375.0).
        let expected = concat!(
            "vendor: Acme Corporation\n",
            "total: 15375.0\n",
            "status: pending-approval\n",
            "line_items [2]\n",
            "  description: Software Development Services\n",
            "  amount: 12500.0\n",
            "  description: Consulting\n",
            "  amount: 2875.0"
        );
        assert_eq!(toon, expected);
    }

    #[test]
    fn nested_mapping() {
        let yaml = r#"
meta:
  stage: 1
  ready: true
"#;
        let toon = yaml_to_toon(yaml.as_bytes()).unwrap();
        assert_eq!(toon, "meta\n  stage: 1\n  ready: true");
    }

    #[test]
    fn scalar_list() {
        let yaml = "tags:\n  - a\n  - b\n  - c\n";
        let toon = yaml_to_toon(yaml.as_bytes()).unwrap();
        assert_eq!(toon, "tags [3]\n  a\n  b\n  c");
    }
}

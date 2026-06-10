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

/// Encode a JSON value as TOON, **verified lossless**: the TOON text is
/// decoded again and compared against the input. Returns `None` whenever the
/// value cannot be represented unambiguously (e.g. strings that look like
/// numbers, multiline strings, sequences of heterogeneous objects) — callers
/// must then fall back to the raw JSON.
pub fn json_to_toon_verified(value: &serde_json::Value) -> Option<String> {
    let yaml: Value = serde_yaml::to_value(value).ok()?;
    let toon = to_toon(&yaml);
    if toon.trim().is_empty() {
        return None;
    }
    let parsed = parse_toon(&toon)?;
    let roundtrip: serde_json::Value = serde_json::to_value(&parsed).ok()?;
    (&roundtrip == value).then_some(toon)
}

// ---------------------------------------------------------------------------
// Verification decoder. TOON is an input-side format — agents read it, nothing
// in the pipeline parses it back. This decoder exists ONLY so that
// `json_to_toon_verified` can prove a particular encoding is lossless before
// it is injected in place of raw JSON. It inverts the encoder above; where
// the encoding is ambiguous (flattened sequence items are delimited by a
// repeated key), the roundtrip comparison rejects the result.
// ---------------------------------------------------------------------------

/// Parse TOON text produced by [`to_toon`] back into a YAML value.
/// Verification use only; not part of the TOON contract.
pub(crate) fn parse_toon(text: &str) -> Option<Value> {
    let lines: Vec<(usize, &str)> = text
        .lines()
        .map(|l| {
            let body = l.trim_start_matches(' ');
            let spaces = l.len() - body.len();
            (spaces, body)
        })
        .collect();
    if lines.iter().any(|(s, _)| s % INDENT.len() != 0) {
        return None;
    }

    let mut idx = 0;
    // A single scalar line is a bare scalar document.
    let value = if lines.len() == 1 && !looks_like_entry(lines[0].1) {
        idx = 1;
        parse_scalar(lines[0].1)
    } else {
        Value::Mapping(parse_mapping(&lines, &mut idx, 0)?)
    };
    (idx == lines.len()).then_some(value)
}

/// True when the line could open a mapping entry (kv or `key [n]` header).
fn looks_like_entry(line: &str) -> bool {
    line.contains(": ") || seq_header(line).is_some()
}

/// Match a key-less `[n]` header (nested sequence item).
fn bare_seq_header(line: &str) -> Option<usize> {
    let digits = line.strip_prefix('[')?.strip_suffix(']')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// Match `key [n]` headers. Returns (key, n).
fn seq_header(line: &str) -> Option<(&str, usize)> {
    if line.contains(": ") {
        return None; // scalar kv line, even if it ends in [n]
    }
    let open = line.rfind(" [")?;
    let rest = &line[open + 2..];
    let digits = rest.strip_suffix(']')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((&line[..open], digits.parse().ok()?))
}

fn parse_scalar(s: &str) -> Value {
    match s {
        "null" => Value::Null,
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => {
            if let Ok(i) = s.parse::<i64>() {
                if i.to_string() == s {
                    return Value::Number(i.into());
                }
            }
            if let Ok(f) = s.parse::<f64>() {
                if f.to_string() == s {
                    return Value::Number(serde_yaml::Number::from(f));
                }
            }
            Value::String(s.to_string())
        }
    }
}

fn parse_mapping(
    lines: &[(usize, &str)],
    idx: &mut usize,
    depth: usize,
) -> Option<serde_yaml::Mapping> {
    let mut map = serde_yaml::Mapping::new();
    let indent = depth * INDENT.len();
    while *idx < lines.len() {
        let (spaces, _line) = lines[*idx];
        if spaces < indent {
            break; // end of this block
        }
        if spaces > indent {
            return None; // unexpected deeper line
        }
        let (key, value) = parse_entry(lines, idx, depth)?;
        if map.insert(Value::String(key), value).is_some() {
            return None; // duplicate key — ambiguous
        }
    }
    Some(map)
}

/// Parse one mapping entry starting at `idx` (which sits at `depth`).
fn parse_entry(
    lines: &[(usize, &str)],
    idx: &mut usize,
    depth: usize,
) -> Option<(String, Value)> {
    let (_, line) = lines[*idx];
    if let Some((key, n)) = seq_header(line) {
        *idx += 1;
        let items = parse_seq_items(lines, idx, depth + 1)?;
        if items.len() != n {
            return None;
        }
        return Some((key.to_string(), Value::Sequence(items)));
    }
    if let Some(split) = line.find(": ") {
        let (key, value) = (&line[..split], &line[split + 2..]);
        *idx += 1;
        return Some((key.to_string(), parse_scalar(value)));
    }
    // Bare key: nested (possibly empty) mapping.
    let key = line.to_string();
    *idx += 1;
    let child = if *idx < lines.len() && lines[*idx].0 == (depth + 1) * INDENT.len() {
        parse_mapping(lines, idx, depth + 1)?
    } else {
        serde_yaml::Mapping::new()
    };
    Some((key, Value::Mapping(child)))
}

/// Parse the items of a sequence at `depth`. Mapping items are flattened by
/// the encoder; a new item starts when a key repeats within the current item.
fn parse_seq_items(lines: &[(usize, &str)], idx: &mut usize, depth: usize) -> Option<Vec<Value>> {
    let indent = depth * INDENT.len();
    let mut items: Vec<Value> = Vec::new();
    let mut current: Option<serde_yaml::Mapping> = None;

    while *idx < lines.len() {
        let (spaces, line) = lines[*idx];
        if spaces < indent {
            break;
        }
        if spaces > indent {
            return None;
        }
        let next_is_deeper = lines
            .get(*idx + 1)
            .map_or(false, |(s, _)| *s > indent);
        if let Some(n) = bare_seq_header(line) {
            // Nested sequence item: "[n]" header with its own items below.
            if let Some(m) = current.take() {
                items.push(Value::Mapping(m));
            }
            *idx += 1;
            let inner = parse_seq_items(lines, idx, depth + 1)?;
            if inner.len() != n {
                return None;
            }
            items.push(Value::Sequence(inner));
        } else if looks_like_entry(line) || next_is_deeper {
            // Field of a (flattened) mapping item — possibly a bare key whose
            // nested mapping value sits on the deeper lines that follow.
            let (key, value) = parse_entry(lines, idx, depth)?;
            let key_v = Value::String(key);
            match current.as_mut() {
                Some(m) if !m.contains_key(&key_v) => {
                    m.insert(key_v, value);
                }
                Some(_) | None => {
                    if let Some(m) = current.take() {
                        items.push(Value::Mapping(m));
                    }
                    let mut m = serde_yaml::Mapping::new();
                    m.insert(key_v, value);
                    current = Some(m);
                }
            }
        } else {
            // Scalar item.
            if let Some(m) = current.take() {
                items.push(Value::Mapping(m));
            }
            items.push(parse_scalar(line));
            *idx += 1;
        }
    }
    if let Some(m) = current.take() {
        items.push(Value::Mapping(m));
    }
    Some(items)
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

/// Heavy coverage for #23: TOON-encoding JSON Schemas must be provably
/// lossless or refuse (return None) — never silently wrong.
#[cfg(test)]
mod schema_roundtrip_tests {
    use super::*;
    use serde_json::json;

    fn assert_lossless(v: serde_json::Value) {
        let toon = json_to_toon_verified(&v)
            .unwrap_or_else(|| panic!("expected lossless TOON encoding for {v}"));
        let parsed = parse_toon(&toon).expect("verified TOON must parse");
        let back: serde_json::Value = serde_json::to_value(&parsed).unwrap();
        assert_eq!(back, v, "round-trip mismatch for TOON:\n{toon}");
    }

    fn assert_falls_back(v: serde_json::Value) {
        assert_eq!(
            json_to_toon_verified(&v),
            None,
            "ambiguous value must fall back to raw JSON: {v}"
        );
    }

    #[test]
    fn typical_json_schema_roundtrips() {
        assert_lossless(json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "required": ["title", "vendor"],
            "additionalProperties": false,
            "properties": {
                "title": {"type": "string", "minLength": 1},
                "vendor": {"type": "string"},
                "total": {"type": "number", "minimum": 0},
                "count": {"type": "integer", "maximum": 100},
                "status": {"enum": ["draft", "pending-approval", "approved"]},
                "line_items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "description": {"type": "string"},
                            "amount": {"type": "number"}
                        }
                    }
                }
            }
        }));
    }

    #[test]
    fn nested_objects_and_refs_roundtrip() {
        assert_lossless(json!({
            "type": "object",
            "properties": {
                "billing": {"$ref": "#/$defs/address"},
                "shipping": {"$ref": "#/$defs/address"}
            },
            "$defs": {
                "address": {
                    "type": "object",
                    "properties": {
                        "street": {"type": "string"},
                        "country": {"enum": ["IE", "GB", "US"]}
                    }
                }
            }
        }));
    }

    #[test]
    fn mixed_scalar_enum_roundtrips() {
        assert_lossless(json!({"enum": [1, "one", true, null, 2.5]}));
    }

    #[test]
    fn empty_containers_roundtrip() {
        assert_lossless(json!({
            "type": "object",
            "properties": {},
            "required": []
        }));
    }

    #[test]
    fn homogeneous_object_array_roundtrips() {
        // anyOf items share the key layout, so flattened items are
        // delimited by the repeating first key.
        assert_lossless(json!({
            "anyOf": [
                {"type": "string"},
                {"type": "number"},
                {"type": "boolean"}
            ]
        }));
    }

    // --- ambiguity: must fall back, never corrupt ---

    #[test]
    fn numeric_looking_string_falls_back() {
        assert_falls_back(json!({"enum": ["123"]}));
        assert_falls_back(json!({"default": "2.5"}));
    }

    #[test]
    fn bool_and_null_looking_strings_fall_back() {
        assert_falls_back(json!({"const": "true"}));
        assert_falls_back(json!({"default": "null"}));
    }

    #[test]
    fn multiline_string_falls_back() {
        assert_falls_back(json!({"description": "line one\nline two"}));
    }

    #[test]
    fn key_containing_separator_falls_back() {
        assert_falls_back(json!({"weird: key": 1}));
    }

    #[test]
    fn heterogeneous_object_array_falls_back() {
        // Items with disjoint keys cannot be delimited in flattened form.
        assert_falls_back(json!({
            "anyOf": [
                {"type": "object", "properties": {"a": {"type": "string"}}},
                {"required": ["a"]}
            ]
        }));
    }

    #[test]
    fn top_level_non_object_is_handled() {
        assert_lossless(json!({"x": 1}));
        // Boolean schemas (`true`/`false`) are valid JSON Schema documents.
        assert_lossless(json!(true));
    }

    #[test]
    fn encoding_is_deterministic() {
        let v = json!({"type": "object", "properties": {"a": {"type": "string"}}});
        assert_eq!(json_to_toon_verified(&v), json_to_toon_verified(&v));
    }

    // --- schema-validation equivalence before/after compression ---

    #[test]
    fn validation_equivalence_after_roundtrip() {
        let schema = json!({
            "type": "object",
            "required": ["title"],
            "properties": {
                "title": {"type": "string", "minLength": 2},
                "total": {"type": "number", "minimum": 0},
                "status": {"enum": ["draft", "approved"]}
            },
            "additionalProperties": false
        });
        let toon = json_to_toon_verified(&schema).expect("schema should be losslessly encodable");
        let recovered: serde_json::Value =
            serde_json::to_value(&parse_toon(&toon).unwrap()).unwrap();

        let before = jsonschema::validator_for(&schema).unwrap();
        let after = jsonschema::validator_for(&recovered).unwrap();

        let payloads = [
            json!({"title": "ok", "total": 5, "status": "draft"}),   // valid
            json!({"title": "ok"}),                                   // valid
            json!({}),                                                // missing required
            json!({"title": "x"}),                                    // too short
            json!({"title": "ok", "total": -1}),                      // below minimum
            json!({"title": "ok", "status": "nope"}),                 // bad enum
            json!({"title": "ok", "extra": 1}),                       // additionalProperties
        ];
        for p in payloads {
            assert_eq!(
                before.is_valid(&p),
                after.is_valid(&p),
                "validation diverged after TOON round-trip for payload {p}"
            );
        }
    }
}

/// Adversarial tests for issue #23. The safety claim under attack:
/// `json_to_toon_verified` must NEVER return `Some(toon)` whose
/// verification-parse differs from the original value. False negatives
/// (needless fallback to raw JSON) are acceptable; false positives are bugs.
#[cfg(test)]
mod adversarial_tests {
    use super::*;
    use serde_json::{json, Value as J};

    /// Minimal deterministic LCG so the loop is reproducible without a
    /// `rand` dependency (constants from PCG/Knuth).
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 11
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    const TRICKY_STRINGS: &[&str] = &[
        "123", "1e5", "true", "false", "null", "", " padded ", "a: b", "[3]",
        "key [2]", "naïve – ünïcode ✓", "line\nbreak", "tab\there", "-0",
        "007", "  ", "0.5", "-1.5e-3", "NaN", "inf", "[0]", ": ", "x [1]",
        "yes", "~", "-0.0", "1.0", "9223372036854775807", "carriage\rreturn",
        "crlf\r\nline", "9223372036854775808", "\t", "a\n  b: 1",
    ];
    const TRICKY_KEYS: &[&str] = &[
        "", "key [2]", "a: b", "decisions", "type", "properties", "x",
        " lead", "trail ", "[2]", "k\nk", "ünï", "a:b", "items [1]", "enum",
    ];

    /// The claim itself, checked independently of the implementation's own
    /// comparison: if `Some(toon)` comes back, parsing it must yield the
    /// original value — compared both as `serde_json::Value` AND by exact
    /// serialised text (the latter catches loose float equality such as
    /// `-0.0 == 0.0` slipping through `PartialEq`).
    fn assert_claim(v: &J) -> Option<String> {
        let toon = json_to_toon_verified(v)?;
        let parsed = parse_toon(&toon).unwrap_or_else(|| {
            panic!("Some(toon) returned but parse_toon rejected it:\n{toon}\nfor input {v}")
        });
        let back: J = serde_json::to_value(&parsed)
            .unwrap_or_else(|e| panic!("parsed TOON not JSON-convertible ({e}):\n{toon}"));
        assert_eq!(&back, v, "FALSE POSITIVE: lossy TOON was accepted:\n{toon}");
        assert_eq!(
            serde_json::to_string(&back).unwrap(),
            serde_json::to_string(v).unwrap(),
            "FALSE POSITIVE: values compare equal but serialise differently:\n{toon}"
        );
        // Determinism: a second encode must produce the identical text.
        assert_eq!(json_to_toon_verified(v).as_deref(), Some(toon.as_str()));
        Some(toon)
    }

    fn gen_value(rng: &mut Lcg, depth: u32) -> J {
        let pick = if depth >= 4 { rng.below(7) } else { rng.below(10) };
        match pick {
            0 => J::Null,
            1 => J::Bool(rng.below(2) == 0),
            2 => json!(rng.next() as i64),
            3 => {
                let f = match rng.below(8) {
                    0 => 0.0,
                    1 => -0.0,
                    2 => 1e300,
                    3 => -1e-300,
                    4 => 1.5,
                    5 => -2.25,
                    6 => 1e15 + 0.5,
                    _ => (rng.next() as i64 as f64) / 1000.0,
                };
                json!(f)
            }
            4 => match rng.below(7) {
                0 => json!(i64::MAX),
                1 => json!(i64::MIN),
                2 => json!(u64::MAX),
                3 => json!(i64::MAX as u64 + 1), // just above i64 range
                4 => json!(0i64),
                5 => json!(-1i64),
                _ => json!(7i64),
            },
            5 | 6 => {
                J::String(TRICKY_STRINGS[rng.below(TRICKY_STRINGS.len() as u64) as usize].into())
            }
            7 => {
                let n = rng.below(4) as usize;
                J::Array((0..n).map(|_| gen_value(rng, depth + 1)).collect())
            }
            8 => {
                // Arrays of (possibly heterogeneous, possibly key-colliding)
                // objects: the flattened sequence encoding's weakest spot.
                let n = rng.below(4) as usize;
                J::Array(
                    (0..n)
                        .map(|_| {
                            let fields = 1 + rng.below(3) as usize;
                            let mut m = serde_json::Map::new();
                            for _ in 0..fields {
                                let key =
                                    TRICKY_KEYS[rng.below(TRICKY_KEYS.len() as u64) as usize];
                                m.insert(key.into(), gen_value(rng, depth + 2));
                            }
                            J::Object(m)
                        })
                        .collect(),
                )
            }
            _ => {
                let n = rng.below(5) as usize;
                let mut m = serde_json::Map::new();
                for _ in 0..n {
                    let key = TRICKY_KEYS[rng.below(TRICKY_KEYS.len() as u64) as usize];
                    m.insert(key.into(), gen_value(rng, depth + 1));
                }
                J::Object(m)
            }
        }
    }

    /// Property loop: a few hundred deterministic pseudo-random values per
    /// seed. Every `Some(toon)` must parse back to the exact original.
    #[test]
    fn seeded_property_roundtrip_never_lies() {
        let mut encoded = 0u32;
        let mut total = 0u32;
        for seed in [1u64, 0xDEADBEEF, 0x23, 42, 0x5EED] {
            let mut rng = Lcg(seed);
            for _ in 0..400 {
                let v = gen_value(&mut rng, 0);
                total += 1;
                if assert_claim(&v).is_some() {
                    encoded += 1;
                }
            }
        }
        // Sanity: the test must not be vacuous — a healthy share of values
        // should actually be encodable.
        assert!(encoded > total / 20, "only {encoded}/{total} values encoded");
    }

    /// Hand-crafted nasty scalars and keys, each checked against the claim.
    #[test]
    fn handcrafted_nasty_values_never_corrupt() {
        let cases = vec![
            json!({"": 1}),                                   // empty key
            json!({"": {"a": 1}}),                            // empty key, mapping value
            json!({"key [2]": "x"}),                          // key ending in " [2]", scalar
            json!({"key [2]": {"a": 1, "b": 2}}),             // key ending in " [2]", object
            json!({"key [1]": [{"a": 1}]}),                   // header-shaped key + real seq
            json!({"key [1]": {"a": 1}}),                     // header-shaped key + object
            json!({"a": "-0"}),                               // string "-0"
            json!({"a": "007"}),                              // leading-zero numeric string
            json!({"a": "  "}),                               // string equal to INDENT
            json!({"a": ["  "]}),                             // INDENT string inside a sequence
            json!({"decisions": [{"id": 1}, {"id": 2}]}),     // key "decisions"
            json!({"a": [[1, 2], [3], []]}),                  // array of arrays
            json!({"a": [[[1]], [[2], [3]]]}),                // array of arrays of arrays
            json!({"x": {}, "y": 1}),                         // empty object next to scalar
            json!({"a": [{"x": {}}, {"y": 1}]}),              // empty-object item collision
            json!({"a": [{"x": {}, "y": 1}]}),                // flattened empty obj + scalar
            json!({"a": ["x", {"y": 1}]}),                    // scalar/object item collision
            json!({"a": "x\nb: 2", "b": 2}),                  // multiline forging a sibling
            json!({"k\nk": 1}),                               // key containing newline
            json!({"a": -0.0}),                               // negative zero
            json!({"a": 0.0}),                                // positive zero
            json!({"a": 1e300}),                              // huge float
            json!({"a": 5e-324}),                             // smallest subnormal
            json!({"a": i64::MAX}),
            json!({"a": i64::MIN}),
            json!({"a": u64::MAX}),
            json!({"a": 1e15 + 0.5}),
            json!({"a": ["[3]"]}),                            // string forging a seq header
            json!({"a": ["[0]"]}),
            json!({"a": [[]]}),                               // genuine empty nested seq
            json!({"a": [" lead", "trail ", "", "a: b"]}),
            json!({"a [1]": [1]}),                            // key vs header collision
            json!([1, 2, 3]),                                 // top-level array
            json!([]),
            json!({}),
            json!(null),
            json!("a: b"),                                    // top-level entry-shaped string
            json!("123"),
            json!(""),
        ];
        for v in &cases {
            assert_claim(v);
        }
        // Non-vacuity: the genuinely unambiguous ones must encode.
        assert!(assert_claim(&json!({"": 1})).is_some());
        assert!(assert_claim(&json!({"key [2]": "x"})).is_some());
        assert!(assert_claim(&json!({"x": {}, "y": 1})).is_some());
        assert!(assert_claim(&json!({"a": [[1, 2], [3], []]})).is_some());
        assert!(assert_claim(&json!({"decisions": [{"id": 1}, {"id": 2}]})).is_some());
        // Known-ambiguous ones must refuse.
        assert_eq!(json_to_toon_verified(&json!({"a": [{"x": {}}, {"y": 1}]})), None);
        assert_eq!(json_to_toon_verified(&json!({"a": ["[3]"]})), None);
        assert_eq!(json_to_toon_verified(&json!({"a": "x\nb: 2", "b": 2})), None);
    }

    /// 50-level nesting in both objects and arrays must neither panic nor lie.
    #[test]
    fn deep_nesting_is_safe() {
        let mut obj = json!({"leaf": 1});
        for i in 0..50 {
            obj = json!({ format!("level{i}"): obj });
        }
        assert!(assert_claim(&obj).is_some(), "deep object should be lossless");

        let mut arr = json!([1]);
        for _ in 0..50 {
            arr = json!([arr]);
        }
        let wrapped = json!({"deep": arr});
        assert!(assert_claim(&wrapped).is_some(), "deep array should be lossless");
    }

    /// Heterogeneous object arrays: every grouping the flattening could
    /// mis-split. The length check + roundtrip must catch all of them.
    #[test]
    fn heterogeneous_object_arrays_never_merge_or_split_silently() {
        let cases = vec![
            json!({"s": [{"a": 1}, {"b": 2}]}),               // disjoint keys merge
            json!({"s": [{"a": 1}, {"b": 2}, {"a": 3, "b": 4}]}),
            json!({"s": [{"a": 1, "b": 2}, {"a": 3}, {"b": 4}]}),
            json!({"s": [{"a": 1}, {"a": 2, "b": 3}]}),       // legitimately splittable
            json!({"s": [{"a": 1, "b": 2}, {"b": 3}]}),
            json!({"s": [{"a": {"x": 1}}, {"a": {"x": 2}}]}), // nested mapping items
            json!({"s": [{"a": 1}, 5, {"a": 2}]}),            // scalars interleaved
            json!({"s": [{"a": "b: c"}]}),                    // value forging a kv line
            json!({"s": [{"a": 1}, {"a": 1}]}),               // identical consecutive items
        ];
        for v in &cases {
            assert_claim(v);
        }
        // The splittable layouts are exactly recoverable and must encode.
        assert!(assert_claim(&json!({"s": [{"a": 1}, {"a": 2, "b": 3}]})).is_some());
        assert!(assert_claim(&json!({"s": [{"a": {"x": 1}}, {"a": {"x": 2}}]})).is_some());
    }

    /// jsonschema validation-equivalence: whenever a schema-shaped value is
    /// accepted, the recovered schema must validate identical payloads.
    #[test]
    fn validation_equivalence_for_generated_schemas() {
        let probes = [
            json!({"p": 1}),
            json!({"p": "1"}),
            json!({"p": "draft"}),
            json!({"p": true}),
            json!({"p": null}),
            json!({"p": []}),
            json!({"p": {}}),
            json!({"p": -0.0}),
            json!({}),
            json!({"p": 1, "extra": "x"}),
        ];
        let mut rng = Lcg(0xC0FFEE);
        let mut checked = 0u32;
        for _ in 0..300 {
            let body = gen_value(&mut rng, 1);
            let schema = json!({
                "type": "object",
                "properties": {"p": body},
                "required": ["p"]
            });
            let Some(toon) = assert_claim(&schema) else { continue };
            let recovered: J = serde_json::to_value(&parse_toon(&toon).unwrap()).unwrap();
            let Ok(before) = jsonschema::validator_for(&schema) else { continue };
            let after = jsonschema::validator_for(&recovered)
                .expect("recovered schema failed to compile although original did");
            for p in &probes {
                assert_eq!(
                    before.is_valid(p),
                    after.is_valid(p),
                    "validation diverged after TOON round-trip\nschema: {schema}\npayload: {p}"
                );
            }
            checked += 1;
        }
        assert!(checked > 10, "only {checked} schemas were actually compared");
    }
}

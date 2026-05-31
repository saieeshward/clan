//! Decision-chain compression pipeline (spec §7).
//!
//! Two tiers, applied during packaging — never on the agent read path:
//!   * **Verbatim window**: the most recent `window` entries are untouched.
//!   * **Compressed tail**: older entries have their `rationale` field
//!     compressed in place. All other fields are always preserved verbatim.
//!
//! The default compressor is a deterministic, model-free NLP pipeline:
//! YAKE-style keyword extraction → sentence scoring → position weights →
//! numeric lock → identifier preservation → pick-until-budget. Applications
//! may override it with a [`Compressor`] callback (e.g. a small local model).
//!
//! Short-circuits:
//!   * `pinned: true` entries are never compressed, regardless of age.
//!   * entries whose rationale is already within `char_budget` are left as-is.

use crate::decision::{Decision, DecisionChain};

/// Configuration for chain compression.
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Number of most-recent entries kept verbatim.
    pub window: usize,
    /// Target maximum characters for a compressed rationale.
    pub char_budget: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            window: 5,
            char_budget: 280,
        }
    }
}

/// A rationale compressor. Receives `(action, rationale, char_budget)` and
/// returns a compressed rationale. The default is [`nlp_compress`].
pub type Compressor<'a> = dyn Fn(&str, &str, usize) -> String + 'a;

/// Compress the tail of a decision chain in place.
///
/// `compressor` is optional; when `None`, the deterministic NLP pipeline is
/// used. Entries `[0, window)` are left untouched (newest-first ordering).
pub fn compress_chain(
    chain: &mut DecisionChain,
    config: &CompressionConfig,
    compressor: Option<&Compressor>,
) {
    for decision in chain.decisions.iter_mut().skip(config.window) {
        compress_decision(decision, config, compressor);
    }
}

fn compress_decision(
    decision: &mut Decision,
    config: &CompressionConfig,
    compressor: Option<&Compressor>,
) {
    if decision.pinned {
        return; // pinned entries are never compressed (spec §7)
    }
    if decision.rationale.chars().count() <= config.char_budget {
        return; // short-circuit: already within budget
    }
    let compressed = match compressor {
        Some(f) => f(&decision.action, &decision.rationale, config.char_budget),
        None => nlp_compress(&decision.action, &decision.rationale, config.char_budget),
    };
    decision.rationale = compressed;
}

/// The deterministic NLP compression pipeline.
pub fn nlp_compress(action: &str, rationale: &str, char_budget: usize) -> String {
    let sentences = split_sentences(rationale);
    if sentences.len() <= 1 {
        return truncate_chars(rationale.trim(), char_budget);
    }

    // 1. YAKE-style keyword extraction over action + rationale.
    let corpus = format!("{action} {rationale}");
    let keywords = extract_keywords(&corpus, 10);

    // 2–5. Score each sentence.
    let n = sentences.len();
    let scores: Vec<f32> = sentences
        .iter()
        .enumerate()
        .map(|(i, s)| score_sentence(s, &keywords, i, n))
        .collect();

    // 6. Numeric-locked sentences are mandatory; fill the rest by score until
    // the budget is reached. Output preserves original order for readability.
    let mut selected: Vec<usize> = Vec::new();
    let mut used = 0usize;

    // Mandatory: any sentence carrying numeric data (spec §7, numeric lock).
    for (i, s) in sentences.iter().enumerate() {
        if has_numeric(s) {
            selected.push(i);
            used += s.chars().count() + 1;
        }
    }

    // Remaining sentences, highest score first, until budget.
    let mut ranked: Vec<usize> = (0..n).filter(|i| !selected.contains(i)).collect();
    ranked.sort_by(|a, b| {
        scores[*b]
            .partial_cmp(&scores[*a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for i in ranked {
        let len = sentences[i].chars().count() + 1;
        if used + len > char_budget && !selected.is_empty() {
            break;
        }
        selected.push(i);
        used += len;
    }

    selected.sort_unstable();
    let result = selected
        .iter()
        .map(|i| sentences[*i].as_str())
        .collect::<Vec<_>>()
        .join(" ");

    truncate_chars(&result, char_budget.max(used.min(char_budget) + 0))
}

/// Score a sentence: keyword overlap normalised by length, plus position,
/// numeric, and identifier signals (spec §7).
fn score_sentence(sentence: &str, keywords: &[String], index: usize, total: usize) -> f32 {
    let words: Vec<String> = tokenize(sentence);
    if words.is_empty() {
        return 0.0;
    }
    let overlap = words
        .iter()
        .filter(|w| keywords.iter().any(|k| k == *w))
        .count() as f32;
    let base = overlap / (words.len() as f32).sqrt();

    // Position weights: last sentence carries the outcome; first restates task.
    let position = if index == total - 1 {
        0.20
    } else if index == 0 {
        0.10
    } else {
        0.0
    };

    let numeric = if has_numeric(sentence) { 0.50 } else { 0.0 };
    let identifier = if has_identifier(sentence) { 0.15 } else { 0.0 };

    base + position + numeric + identifier
}

/// YAKE-style statistical keyword extraction. Model-free and corpus-free:
/// ranks terms by frequency, earliest position, and casing (capitalised
/// terms and acronyms score higher), after stopword removal. Returns up to
/// `k` distinct lowercased terms. Higher rank = more important.
fn extract_keywords(text: &str, k: usize) -> Vec<String> {
    let raw_tokens: Vec<&str> = text.split_whitespace().collect();
    let total = raw_tokens.len().max(1) as f32;

    use std::collections::HashMap;
    let mut tf: HashMap<String, f32> = HashMap::new();
    let mut first_pos: HashMap<String, usize> = HashMap::new();
    let mut casing: HashMap<String, f32> = HashMap::new();

    for (pos, raw) in raw_tokens.iter().enumerate() {
        let norm = normalize_token(raw);
        if norm.is_empty() || is_stopword(&norm) {
            continue;
        }
        *tf.entry(norm.clone()).or_insert(0.0) += 1.0;
        first_pos.entry(norm.clone()).or_insert(pos);
        let boost = casing_boost(raw);
        let slot = casing.entry(norm.clone()).or_insert(1.0);
        if boost > *slot {
            *slot = boost;
        }
    }

    let mut scored: Vec<(String, f32)> = tf
        .iter()
        .map(|(term, freq)| {
            let pos = *first_pos.get(term).unwrap_or(&0) as f32;
            // Earlier position → higher; normalise to (0, 1].
            let position_factor = 1.0 - (pos / total) * 0.5;
            let case = *casing.get(term).unwrap_or(&1.0);
            (term.clone(), freq * position_factor * case)
        })
        .collect();

    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    scored.into_iter().take(k).map(|(t, _)| t).collect()
}

fn casing_boost(raw: &str) -> f32 {
    let letters: Vec<char> = raw.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return 1.0;
    }
    let all_upper = letters.iter().all(|c| c.is_uppercase());
    if all_upper && letters.len() > 1 {
        return 2.0; // acronym
    }
    if letters[0].is_uppercase() {
        return 1.5; // capitalised
    }
    1.0
}

/// Split text into sentences on `.`, `!`, `?` boundaries, keeping content
/// trimmed. Decimal points (digit.digit) do not split.
fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut sentences = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c == '.' || c == '!' || c == '?' {
            // Don't split on a decimal point between digits.
            let prev_digit = i > 0 && chars[i - 1].is_ascii_digit();
            let next_digit = i + 1 < chars.len() && chars[i + 1].is_ascii_digit();
            if c == '.' && prev_digit && next_digit {
                i += 1;
                continue;
            }
            // Consume the terminator and any following whitespace.
            let end = i + 1;
            let piece: String = chars[start..end].iter().collect();
            let trimmed = piece.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_string());
            }
            i = end;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            start = i;
        } else {
            i += 1;
        }
    }
    if start < chars.len() {
        let piece: String = chars[start..].iter().collect();
        let trimmed = piece.trim();
        if !trimmed.is_empty() {
            sentences.push(trimmed.to_string());
        }
    }
    sentences
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(normalize_token)
        .filter(|t| !t.is_empty())
        .collect()
}

/// Lowercase and strip surrounding punctuation, preserving inner hyphens and
/// digits (so identifiers like `extractor-v2` survive).
fn normalize_token(raw: &str) -> String {
    raw.trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

/// True if the sentence contains any digit.
fn has_numeric(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_digit())
}

/// True if the sentence contains a hyphenated or versioned identifier, e.g.
/// `extractor-v2`, `PO-2026-0018`, `v2.1.4`.
fn has_identifier(s: &str) -> bool {
    for token in s.split_whitespace() {
        let t = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        let has_inner_hyphen = t
            .char_indices()
            .any(|(i, c)| c == '-' && i > 0 && i + 1 < t.len());
        let letters = t.chars().any(|c| c.is_alphabetic());
        let digits = t.chars().any(|c| c.is_ascii_digit());
        if (has_inner_hyphen && letters) || (letters && digits) {
            return true;
        }
    }
    false
}

fn truncate_chars(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        return s.to_string();
    }
    let mut out: String = chars[..max.saturating_sub(1)].iter().collect();
    out.push('…');
    out
}

/// Minimal English stopword set — enough to keep keyword extraction focused
/// on content words without a large dependency.
fn is_stopword(word: &str) -> bool {
    const STOP: &[&str] = &[
        "the", "a", "an", "and", "or", "but", "of", "to", "in", "on", "at", "by", "for", "with",
        "from", "as", "is", "are", "was", "were", "be", "been", "being", "this", "that", "these",
        "those", "it", "its", "all", "any", "no", "not", "so", "if", "then", "than", "into",
        "over", "under", "up", "down", "out", "off", "via", "per", "each", "used", "use", "using",
        "across", "has", "have", "had", "will", "would", "can", "could", "should", "may", "might",
        "do", "does", "did", "their", "they", "them", "we", "our", "you", "your", "i",
    ];
    STOP.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_keeps_decimals() {
        let s = split_sentences("Confidence 0.94 across all fields. Vendor matched.");
        assert_eq!(s.len(), 2);
        assert!(s[0].contains("0.94"));
    }

    #[test]
    fn numeric_sentences_are_locked() {
        let action = "validated invoice";
        let rationale = "The vendor was confirmed against the supplier database after a careful \
            multi-step review of every available record and cross reference. The invoice total \
            was 3.2% above the purchase order threshold. The agent reviewed historical patterns \
            and prior submissions in considerable narrative detail to reach a conclusion here.";
        let out = nlp_compress(action, rationale, 120);
        // The numeric sentence must survive compression.
        assert!(out.contains("3.2%"), "numeric lock failed: {out}");
    }

    #[test]
    fn pinned_entries_skipped() {
        let mut chain = DecisionChain::default();
        let long = "x".repeat(500);
        for i in 0..8 {
            let mut d = Decision::new("agent", "act", long.clone(), "2026-05-31T10:00:00Z");
            if i == 7 {
                d.pinned = true;
            }
            chain.decisions.push(d);
        }
        compress_chain(&mut chain, &CompressionConfig::default(), None);
        // entry 7 is pinned → untouched even though it's in the tail
        assert_eq!(chain.decisions[7].rationale.chars().count(), 500);
        // entry 5 (tail, unpinned, over budget) → compressed
        assert!(chain.decisions[5].rationale.chars().count() < 500);
        // entry 0 (within window) → untouched
        assert_eq!(chain.decisions[0].rationale.chars().count(), 500);
    }

    #[test]
    fn short_rationale_untouched() {
        let out = nlp_compress("act", "All fields confirmed.", 280);
        assert_eq!(out, "All fields confirmed.");
    }

    #[test]
    fn identifier_detection() {
        assert!(has_identifier("processed by extractor-v2 today"));
        assert!(has_identifier("matched PO-2026-0018 exactly"));
        assert!(!has_identifier("the vendor was confirmed"));
    }

    #[test]
    fn custom_compressor_override() {
        let mut chain = DecisionChain::default();
        let long = "y".repeat(400);
        for _ in 0..7 {
            chain
                .decisions
                .push(Decision::new("a", "act", long.clone(), "2026-05-31T10:00:00Z"));
        }
        let compressor = |_a: &str, _r: &str, _b: usize| "OVERRIDDEN".to_string();
        compress_chain(
            &mut chain,
            &CompressionConfig::default(),
            Some(&compressor),
        );
        assert_eq!(chain.decisions[6].rationale, "OVERRIDDEN");
        assert_eq!(chain.decisions[0].rationale, long); // window preserved
    }
}

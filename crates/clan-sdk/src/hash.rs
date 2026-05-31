//! SHA-256 helpers. Hashes are computed over uncompressed bytes and
//! formatted as `sha256:<hex>` to match the spec (§5, §12).

use sha2::{Digest, Sha256};

/// Compute the `sha256:<hex>` digest of a byte slice.
pub fn sha256_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(bytes))
}

/// Compute the bare lowercase hex digest of a byte slice.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Verify that `bytes` hashes to the given `sha256:<hex>` value.
/// Comparison is case-insensitive on the hex portion.
pub fn verify_prefixed(bytes: &[u8], expected: &str) -> bool {
    let expected = expected.trim();
    let expected_hex = expected.strip_prefix("sha256:").unwrap_or(expected);
    sha256_hex(bytes).eq_ignore_ascii_case(expected_hex)
}

/// True if a string is a well-formed `sha256:<64 hex>` value.
pub fn is_valid_prefixed(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vector() {
        // SHA-256 of the empty string.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn roundtrip_verify() {
        let data = b"hello clan";
        let h = sha256_prefixed(data);
        assert!(verify_prefixed(data, &h));
        assert!(!verify_prefixed(b"tampered", &h));
        assert!(is_valid_prefixed(&h));
    }
}

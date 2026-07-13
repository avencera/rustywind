//! Contains the default [Sorter](SORTER) and default [Regex](RE)
use regex::Regex;
use std::sync::LazyLock;

/// Default class attribute candidate extractor for known source profiles
pub static RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\bclass(?:Name)?\s*=\s*(?:"([^"]+)"|'([^']+)')"#).unwrap());

/// Conservative class extractor used when the source language is unknown
///
/// This preserves the pre-0.25 behavior: only attribute values made entirely
/// from class-like characters are considered safe to sort
pub static CONSERVATIVE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\b(?:class(?:Name)?\s*=\s*["'])([_a-zA-Z0-9\.,\s\-:\[\]()/#&>+~=*@%]+)["']"#)
        .unwrap()
});

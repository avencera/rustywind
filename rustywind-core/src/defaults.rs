//! Contains the default class attribute extractor
use regex::Regex;
use std::sync::LazyLock;

/// Default class attribute candidate extractor
pub static RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\bclass(?:Name)?\s*=\s*(?:"([^"]+)"|'([^']+)')"#).unwrap());

//! Contains the default [Sorter](SORTER) and default [Regex](RE)
use regex::Regex;
use std::sync::LazyLock;

pub static RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\bclass(?:Name)?\s*=\s*(?:"([^"]+)"|'([^']+)')"#).unwrap());

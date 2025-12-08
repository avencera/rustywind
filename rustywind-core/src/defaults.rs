//! Contains the default [Sorter](SORTER) and default [Regex](RE)
use regex::Regex;
use std::sync::LazyLock;

pub static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\b(?:class(?:Name)?\s*=\s*["'])([_a-zA-Z0-9\.,\s\-:\[\]()/#]+)["']"#).unwrap()
});

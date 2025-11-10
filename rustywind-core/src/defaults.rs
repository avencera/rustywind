//! Contains the default [Sorter](SORTER) and default [Regex](RE)
use once_cell::sync::Lazy;
use regex::Regex;

pub static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"\b(?:class(?:Name)?\s*=\s*["'])([_a-zA-Z0-9\.,\s\-:\[\]()/#]+)["']"#).unwrap()
});

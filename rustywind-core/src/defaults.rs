//! Contains the default [Sorter](SORTER) and default [Regex](RE)
use regex::Regex;
use std::sync::LazyLock;

pub static RE: LazyLock<Regex> = LazyLock::new(|| {
    // Character class includes:
    // - Basic: _a-zA-Z0-9 (alphanumeric, underscore)
    // - Syntax: .,\s\-: (dot, comma, whitespace, hyphen, colon)
    // - Brackets: \[\]() (square brackets, parentheses)
    // - Values: /# (slash for opacity, hash for colors)
    // - Arbitrary variants: &>+~=*@% (selectors, combinators, at-rules, calc)
    Regex::new(r#"\b(?:class(?:Name)?\s*=\s*["'])([_a-zA-Z0-9\.,\s\-:\[\]()/#&>+~=*@%]+)["']"#)
        .unwrap()
});

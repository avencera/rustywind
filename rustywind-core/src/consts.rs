//! Contains different constants used in the library.
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind, StartKind};
use once_cell::sync::Lazy;

/// The default variants used in the variant searcher.
pub static VARIANTS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "sm",
        "md",
        "lg",
        "xl",
        "2xl",
        "3xl",
        "4xl",
        "5xl",
        "6xl",
        "dark",
        "first",
        "last",
        "odd",
        "even",
        "visited",
        "checked",
        "empty",
        "group-hover",
        "group-focus",
        "focus-within",
        "hover",
        "focus",
        "focus-visible",
        "active",
        "disabled",
    ]
});

/// The variant searcher used to find variants in a class name.
pub static VARIANT_SEARCHER: Lazy<AhoCorasick> = Lazy::new(|| {
    AhoCorasickBuilder::new()
        .start_kind(StartKind::Anchored)
        .match_kind(MatchKind::LeftmostLongest)
        .build(VARIANTS.iter())
        .expect("Failed to build variant searcher")
});

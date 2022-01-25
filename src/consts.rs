use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use once_cell::sync::Lazy;

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

pub static VARIANT_SEARCHER: Lazy<AhoCorasick> = Lazy::new(|| {
    AhoCorasickBuilder::new()
        .anchored(true)
        .match_kind(MatchKind::LeftmostLongest)
        .auto_configure(&VARIANTS)
        .build(VARIANTS.iter())
});

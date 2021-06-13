use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref VARIANTS: Vec<&'static str> = vec![
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
        "group-hover",
        "group-focus",
        "hover",
        "focus",
        "focus-visible",
        "focus-within",
        "active",
        "disabled"
    ];
}

lazy_static! {
    pub static ref VARIANT_SEARCHER: AhoCorasick = AhoCorasickBuilder::new()
        .anchored(true)
        .match_kind(MatchKind::LeftmostLongest)
        .auto_configure(&VARIANTS)
        .build(VARIANTS.iter());
}

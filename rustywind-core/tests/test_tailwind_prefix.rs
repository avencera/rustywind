use rustywind_core::RustyWind;
use rustywind_core::hybrid_sorter::HybridSorter;
use rustywind_core::sorter::{FinderRegex, Sorter};
use rustywind_core::tailwind_prefix::normalize_tailwind_prefix;

fn normalize_all(classes: Vec<&str>, prefix: &str) -> Vec<String> {
    classes
        .into_iter()
        .map(|class| normalize_tailwind_prefix(class, Some(prefix)).into_owned())
        .collect()
}

#[test]
fn tailwind_v3_prefixed_classes_sort_like_unprefixed_classes() {
    let prefixed_sorter = HybridSorter::new_with_tailwind_prefix(Some("tw"));
    let normal_sorter = HybridSorter::new();

    let prefixed = vec![
        "md:tw-text-xl",
        "tw-p-4",
        "hover:-tw-mr-4",
        "tw-bg-white",
        "tw-flex",
    ];
    let unprefixed = vec!["md:text-xl", "p-4", "hover:-mr-4", "bg-white", "flex"];

    let sorted_prefixed = prefixed_sorter.sort_classes(&prefixed);
    let sorted_unprefixed = normal_sorter.sort_classes(&unprefixed);

    assert_eq!(
        normalize_all(sorted_prefixed.clone(), "tw"),
        sorted_unprefixed
    );
    assert!(sorted_prefixed.iter().any(|class| class.starts_with("tw-")));
    assert!(sorted_prefixed.iter().any(|class| class.contains(":tw-")));
}

#[test]
fn tailwind_v4_prefixed_classes_sort_like_unprefixed_classes() {
    let prefixed_sorter = HybridSorter::new_with_tailwind_prefix(Some("tw"));
    let normal_sorter = HybridSorter::new();

    let prefixed = vec![
        "tw:md:text-xl",
        "tw:p-4",
        "tw:hover:-mr-4",
        "tw:bg-white",
        "tw:flex",
    ];
    let unprefixed = vec!["md:text-xl", "p-4", "hover:-mr-4", "bg-white", "flex"];

    let sorted_prefixed = prefixed_sorter.sort_classes(&prefixed);
    let sorted_unprefixed = normal_sorter.sort_classes(&unprefixed);

    assert_eq!(
        normalize_all(sorted_prefixed.clone(), "tw"),
        sorted_unprefixed
    );
    assert!(sorted_prefixed.iter().all(|class| class.starts_with("tw:")));
}

#[test]
fn prefixed_sort_key_uses_normalized_class() {
    let sorter = HybridSorter::new_with_tailwind_prefix(Some("tw"));

    assert!(HybridSorter::new().get_sort_key("tw-p-4").is_none());

    let key = sorter
        .get_sort_key("hover:-tw-mr-4")
        .expect("prefixed class should sort as a known Tailwind utility");

    assert_eq!(key.class.as_str(), "hover:-mr-4");
    assert!(key.is_negative);
}

#[test]
fn rustywind_flag_preserves_original_prefixed_classes_in_output() {
    let app = RustyWind {
        regex: FinderRegex::DefaultRegex,
        sorter: Sorter::PatternSorter,
        allow_duplicates: false,
        class_wrapping: Default::default(),
        tailwind_prefix: Some("tw".to_string()),
    };

    let input = r#"<div class="tw:p-4 tw:bg-white tw:md:text-xl tw:hover:-mr-4"></div>"#;
    let sorted = app.sort_file_contents(input);

    assert!(sorted.contains("tw:p-4"));
    assert!(sorted.contains("tw:bg-white"));
    assert!(sorted.contains("tw:md:text-xl"));
    assert!(sorted.contains("tw:hover:-mr-4"));
    assert!(!sorted.contains(r#"class="p-4"#));
}

#[test]
fn custom_sorter_uses_normalized_prefixed_fallback_after_exact_lookup() {
    let app = RustyWind {
        regex: FinderRegex::DefaultRegex,
        sorter: Sorter::new(
            [("bg-white".to_string(), 0), ("p-4".to_string(), 1)]
                .into_iter()
                .collect(),
        ),
        allow_duplicates: false,
        class_wrapping: Default::default(),
        tailwind_prefix: Some("tw".to_string()),
    };

    assert_eq!(app.sort_classes("tw-p-4 tw-bg-white"), "tw-bg-white tw-p-4");
    assert_eq!(app.sort_classes("tw:p-4 tw:bg-white"), "tw:bg-white tw:p-4");
}

#[test]
fn custom_sorter_variant_fallback_keeps_v3_prefixed_exact_order() {
    let app = RustyWind {
        regex: FinderRegex::DefaultRegex,
        sorter: Sorter::new(
            [("tw-bg-white".to_string(), 0), ("tw-p-4".to_string(), 1)]
                .into_iter()
                .collect(),
        ),
        allow_duplicates: false,
        class_wrapping: Default::default(),
        tailwind_prefix: Some("tw".to_string()),
    };

    assert_eq!(
        app.sort_classes("md:tw-p-4 md:tw-bg-white"),
        "md:tw-bg-white md:tw-p-4"
    );
}

#[test]
fn custom_sorter_variant_fallback_keeps_v4_prefixed_exact_order() {
    let app = RustyWind {
        regex: FinderRegex::DefaultRegex,
        sorter: Sorter::new(
            [("tw:bg-white".to_string(), 0), ("tw:p-4".to_string(), 1)]
                .into_iter()
                .collect(),
        ),
        allow_duplicates: false,
        class_wrapping: Default::default(),
        tailwind_prefix: Some("tw".to_string()),
    };

    assert_eq!(
        app.sort_classes("tw:md:p-4 tw:md:bg-white"),
        "tw:md:bg-white tw:md:p-4"
    );
}

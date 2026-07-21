use rustywind_core::{hybrid_sorter::HybridSorter, utility_map::UtilityMap};

#[test]
fn parenthesized_ring_colors_map_to_color_properties() {
    let map = UtilityMap::new();

    assert_eq!(
        map.get_properties("ring-(--color)"),
        Some(&["--tw-ring-color"][..])
    );
    assert_eq!(
        map.get_properties("ring-(color:--color)"),
        Some(&["--tw-ring-color"][..])
    );
    assert_eq!(
        map.get_properties("ring-offset-(--color)"),
        Some(&["--tw-ring-offset-color"][..])
    );
    assert_eq!(
        map.get_properties("ring-offset-(color:--color)"),
        Some(&["--tw-ring-offset-color"][..])
    );
    assert_eq!(
        map.get_properties("inset-ring-(--color)"),
        Some(&["--tw-inset-ring-color"][..])
    );
    assert_eq!(
        map.get_properties("inset-ring-(color:--color)"),
        Some(&["--tw-inset-ring-color"][..])
    );
    // named design-system colors resolve through the named-color fallback
    assert_eq!(
        map.get_properties("ring-offset-background"),
        Some(&["--tw-ring-offset-color"][..])
    );
}

#[test]
fn typed_parenthesized_ring_lengths_map_to_width_properties() {
    let map = UtilityMap::new();

    assert_eq!(
        map.get_properties("ring-(length:--ring-width)"),
        Some(
            &[
                "--tw-ring-offset-shadow",
                "--tw-ring-shadow",
                "--tw-shadow",
                "box-shadow",
            ][..]
        )
    );
    assert_eq!(
        map.get_properties("ring-offset-(length:--offset-width)"),
        Some(&["--tw-ring-offset-width"][..])
    );
    assert_eq!(
        map.get_properties("inset-ring-(length:--inset-width)"),
        Some(&["--tw-inset-ring-shadow"][..])
    );
    assert_eq!(
        map.get_properties("ring-(number:--ring-value)"),
        Some(&["--tw-ring-color"][..])
    );
}

#[test]
fn parenthesized_ring_colors_sort_with_ring_color_utilities() {
    let sorter = HybridSorter::new();
    let classes = vec![
        "group-data-[checked=true]/button:ring-(--color)",
        "ring-offset-(--color)",
        "ring-offset-2",
        "ring-transparent",
        "ring-2",
        "bg-(--color)",
        "rounded-full",
        "size-5",
        "group-data-[checked=true]/button:ring-offset-background",
    ];

    assert!(sorter.get_sort_key("ring-(--color)").is_some());
    assert!(sorter.get_sort_key("ring-offset-(--color)").is_some());
    // ring-offset-background is a named design-system color, no longer unknown
    assert!(sorter.get_sort_key("ring-offset-background").is_some());
    // matches prettier-plugin-tailwindcss with `background` defined as a theme color
    assert_eq!(
        sorter.sort_classes(&classes),
        vec![
            "size-5",
            "rounded-full",
            "bg-(--color)",
            "ring-2",
            "ring-transparent",
            "ring-offset-2",
            "ring-offset-(--color)",
            "group-data-[checked=true]/button:ring-(--color)",
            "group-data-[checked=true]/button:ring-offset-background",
        ]
    );
}

#[test]
fn typed_parenthesized_ring_lengths_sort_with_width_utilities() {
    let sorter = HybridSorter::new();
    let classes = vec![
        "ring-(--ring-color)",
        "ring-(color:--ring-color)",
        "ring-(length:--ring-width)",
        "ring-2",
        "ring-offset-(--offset-color)",
        "ring-offset-(color:--offset-color)",
        "ring-offset-(length:--offset-width)",
        "ring-offset-2",
        "inset-ring-(--inset-color)",
        "inset-ring-(color:--inset-color)",
        "inset-ring-(length:--inset-width)",
        "inset-ring-2",
    ];

    assert_eq!(
        sorter.sort_classes(&classes),
        vec![
            "ring-(length:--ring-width)",
            "ring-2",
            "inset-ring-(length:--inset-width)",
            "inset-ring-2",
            "ring-(--ring-color)",
            "ring-(color:--ring-color)",
            "inset-ring-(--inset-color)",
            "inset-ring-(color:--inset-color)",
            "ring-offset-(length:--offset-width)",
            "ring-offset-2",
            "ring-offset-(--offset-color)",
            "ring-offset-(color:--offset-color)",
        ]
    );
}

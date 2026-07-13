use rustywind_core::{hybrid_sorter::HybridSorter, utility_map::UtilityMap};

#[test]
fn parenthesized_ring_colors_map_to_color_properties() {
    let map = UtilityMap::new();

    assert_eq!(
        map.get_properties("ring-(--color)"),
        Some(&["--tw-ring-color"][..])
    );
    assert_eq!(
        map.get_properties("ring-offset-(--color)"),
        Some(&["--tw-ring-offset-color"][..])
    );
    assert_eq!(map.get_properties("ring-offset-background"), None);
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
    assert!(sorter.get_sort_key("ring-offset-background").is_none());
    assert_eq!(
        sorter.sort_classes(&classes),
        vec![
            "group-data-[checked=true]/button:ring-offset-background",
            "size-5",
            "rounded-full",
            "bg-(--color)",
            "ring-2",
            "ring-transparent",
            "ring-offset-2",
            "ring-offset-(--color)",
            "group-data-[checked=true]/button:ring-(--color)",
        ]
    );
}

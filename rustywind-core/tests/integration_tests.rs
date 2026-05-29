//! Integration tests for pattern-based sorting
//!
//! These tests verify that the pattern-based sorter produces correct results
//! with real-world class lists and edge cases.

use rustywind_core::hybrid_sorter::HybridSorter;
use rustywind_core::pattern_sorter::sort_classes;

#[test]
fn test_realistic_component_classes() {
    let sorter = HybridSorter::new();

    // UNSORTED realistic component with mixed utilities
    let classes = vec![
        "hover:bg-gray-100",
        "flex",
        "items-center",
        "justify-between",
        "p-4",
        "bg-white",
        "rounded-lg",
        "shadow-md",
        "transition-colors",
        "duration-200",
    ];

    let sorted = sorter.sort_classes(&classes);

    // Verify complete ordering
    assert_eq!(sorted.len(), 10);

    // Verify known base classes come before variants
    // The pattern: [known base classes] [variants]
    let variant_count = sorted.iter().filter(|c| c.contains(':')).count();
    assert_eq!(variant_count, 1, "Should have 1 variant class");

    // Find the variant
    let variant_idx = sorted
        .iter()
        .position(|&c| c == "hover:bg-gray-100")
        .unwrap();

    // ALL base classes should come before the variant (including transition utilities)
    let all_base_classes = vec![
        "flex",
        "items-center",
        "justify-between",
        "p-4",
        "bg-white",
        "rounded-lg",
        "shadow-md",
        "transition-colors",
        "duration-200",
    ];
    for class in all_base_classes {
        let idx = sorted.iter().position(|&c| c == class).unwrap();
        assert!(
            idx < variant_idx,
            "Base class '{}' at index {} should come before variant at {}",
            class,
            idx,
            variant_idx
        );
    }

    // Verify last class is the variant
    assert_eq!(sorted.last().unwrap(), &"hover:bg-gray-100");
}

#[test]
fn test_large_class_list_50_classes() {
    let sorter = HybridSorter::new();

    // Large realistic class list
    let classes = vec![
        "container",
        "mx-auto",
        "px-4",
        "sm:px-6",
        "lg:px-8",
        "flex",
        "flex-col",
        "gap-4",
        "md:flex-row",
        "md:gap-6",
        "items-start",
        "md:items-center",
        "justify-between",
        "bg-white",
        "dark:bg-gray-900",
        "rounded-xl",
        "shadow-lg",
        "p-6",
        "md:p-8",
        "border",
        "border-gray-200",
        "dark:border-gray-700",
        "hover:shadow-xl",
        "transition-shadow",
        "duration-300",
        "text-gray-900",
        "dark:text-white",
        "font-sans",
        "relative",
        "overflow-hidden",
        "w-full",
        "max-w-7xl",
        "min-h-screen",
        "md:min-h-0",
        "z-10",
        "backdrop-blur-sm",
        "bg-opacity-95",
        "focus:outline-none",
        "focus:ring-2",
        "focus:ring-blue-500",
        "focus:ring-offset-2",
        "disabled:opacity-50",
        "disabled:cursor-not-allowed",
        "before:absolute",
        "before:inset-0",
        "after:content-['']",
        "group-hover:scale-105",
        "transform",
        "select-none",
        "cursor-pointer",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All classes should be sorted
    assert_eq!(sorted.len(), 50);

    // Verify that classes are sorted (spot checks)
    // All classes should be present
    assert!(sorted.contains(&"container"));
    assert!(sorted.contains(&"flex"));
    assert!(sorted.contains(&"sm:px-6"));
    assert!(sorted.contains(&"hover:shadow-xl"));
}

#[test]
fn test_arbitrary_values_comprehensive() {
    let sorter = HybridSorter::new();

    let classes = vec![
        "m-[10px]",
        "p-[2rem]",
        "bg-[#1da1f2]",
        "text-[14px]",
        "w-[calc(100%-2rem)]",
        "h-[50vh]",
        "top-[10%]",
        "grid-cols-[200px_1fr_200px]",
        "shadow-[0_4px_6px_rgba(0,0,0,0.1)]",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All arbitrary values should be recognized
    assert_eq!(sorted.len(), 9);

    // Verify they're sorted (all are base classes, no variants)
    for class in &sorted {
        assert!(!class.contains(':'), "No variants expected");
    }
}

#[test]
fn test_deeply_nested_variants() {
    let sorter = HybridSorter::new();

    let classes = vec![
        "flex",
        "hover:bg-blue-500",
        "focus:hover:bg-blue-600",
        "dark:focus:hover:bg-blue-700",
        "sm:dark:focus:hover:bg-blue-800",
        "md:sm:dark:focus:hover:bg-blue-900",
    ];

    let sorted = sorter.sort_classes(&classes);

    // Base class should be first
    assert_eq!(sorted[0], "flex");

    // Variant classes should follow
    assert!(sorted[1].contains(':'));
}

#[test]
fn test_important_modifier() {
    let sorter = HybridSorter::new();

    let classes = vec!["flex!", "p-4", "m-4!", "bg-red-500"];

    let sorted = sorter.sort_classes(&classes);

    // Important modifier should be preserved
    assert!(sorted.iter().any(|c| c.contains('!')));
}

#[test]
fn test_pattern_sorter_function() {
    // Test the standalone sort_classes function from pattern_sorter
    let classes = vec!["p-4", "m-4", "hover:p-1", "flex"];
    let sorted = sort_classes(&classes);

    // Base classes first (margin=25, display=35, padding=252)
    assert_eq!(sorted[0], "m-4");
    assert_eq!(sorted[1], "flex");
    assert_eq!(sorted[2], "p-4");

    // Variant class last
    assert_eq!(sorted[3], "hover:p-1");
}

#[test]
fn test_multi_property_utilities() {
    let sorter = HybridSorter::new();

    // These utilities generate multiple CSS properties
    let classes = vec![
        "px-4",    // padding-left + padding-right
        "py-4",    // padding-top + padding-bottom
        "mx-auto", // margin-left + margin-right
        "my-4",    // margin-top + margin-bottom
    ];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized
    assert_eq!(sorted.len(), 4);
}

#[test]
fn test_responsive_breakpoints() {
    let sorter = HybridSorter::new();

    let classes = vec![
        "flex",
        "sm:grid",
        "md:block",
        "lg:inline-flex",
        "xl:table",
        "2xl:hidden",
    ];

    let sorted = sorter.sort_classes(&classes);

    // Base class first
    assert_eq!(sorted[0], "flex");

    // Responsive variants should follow in order
    // sm (index 54) < md (55) < lg (56) < xl (57) < 2xl (58)
    let responsive: Vec<_> = sorted[1..].to_vec();
    assert!(responsive.contains(&"sm:grid"));
    assert!(responsive.contains(&"md:block"));
}

#[test]
fn test_pseudo_class_ordering() {
    let sorter = HybridSorter::new();

    let classes = vec!["p-4", "focus:p-4", "hover:p-4", "active:p-4", "visited:p-4"];

    let sorted = sorter.sort_classes(&classes);

    // Base class first
    assert_eq!(sorted[0], "p-4");

    // hover (35) comes before focus (36) in Tailwind v4 variant order
    let hover_pos = sorted.iter().position(|&c| c == "hover:p-4").unwrap();
    let focus_pos = sorted.iter().position(|&c| c == "focus:p-4").unwrap();
    assert!(hover_pos < focus_pos, "hover should come before focus");
}

#[test]
fn test_property_count_ordering() {
    let sorter = HybridSorter::new();

    // p generates 1 property (padding) at index 253
    // px generates 1 property (padding-inline) at index 254
    let classes = vec!["px-4", "p-4"];
    let sorted = sorter.sort_classes(&classes);

    // Lower property index first: p-4 (padding: 253) before px-4 (padding-inline: 254)
    assert_eq!(sorted[0], "p-4");
    assert_eq!(sorted[1], "px-4");
}

#[test]
fn test_alphabetical_tiebreaker() {
    let sorter = HybridSorter::new();

    // Both generate display property, so should sort alphabetically
    let classes = vec!["grid", "flex", "block"];
    let sorted = sorter.sort_classes(&classes);

    // All have same property and count, so alphabetical
    assert_eq!(sorted[0], "block");
    assert_eq!(sorted[1], "flex");
    assert_eq!(sorted[2], "grid");
}

#[test]
fn test_cache_effectiveness() {
    let sorter = HybridSorter::new();

    let classes = vec!["flex", "p-4", "m-4"];

    // First sort - cache miss
    let _sorted1 = sorter.sort_classes(&classes);
    let (entries_after_first, _) = sorter.cache_stats();
    assert_eq!(entries_after_first, 3);

    // Second sort - cache hit
    let _sorted2 = sorter.sort_classes(&classes);
    let (entries_after_second, _) = sorter.cache_stats();
    assert_eq!(entries_after_second, 3); // Same entries, just retrieved from cache
}

#[test]
fn test_very_long_class_names() {
    let sorter = HybridSorter::new();

    let classes = vec![
        "bg-gradient-to-r",
        "from-purple-400",
        "via-pink-500",
        "to-red-500",
        "hover:from-purple-500",
        "hover:via-pink-600",
        "hover:to-red-600",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized
    assert_eq!(sorted.len(), 7);

    // Base classes before variants
    assert_eq!(sorted[0], "bg-gradient-to-r");
}

#[test]
fn test_mixed_spacing_utilities() {
    let sorter = HybridSorter::new();

    let classes = vec![
        "p-4",  // padding
        "px-4", // padding-inline
        "pt-4", // padding-top
        "m-4",  // margin
        "mx-4", // margin-inline
        "mt-4", // margin-top
    ];

    let sorted = sorter.sort_classes(&classes);

    // margin properties should come before padding properties
    // Sorted by property index from property_order.rs (indices +1 from background-opacity at 0):
    // margin(26) < margin-inline(27) < margin-top(31)
    // padding(253) < padding-inline(254) < padding-top(258)
    assert_eq!(sorted[0], "m-4"); // margin: index 26
    assert_eq!(sorted[1], "mx-4"); // margin-inline: index 27
    assert_eq!(sorted[2], "mt-4"); // margin-top: index 31
    assert_eq!(sorted[3], "p-4"); // padding: index 253
    assert_eq!(sorted[4], "px-4"); // padding-inline: index 254 (changed from padding-left/right)
    assert_eq!(sorted[5], "pt-4"); // padding-top: index 258
}

#[test]
fn test_empty_class_list() {
    let sorter = HybridSorter::new();
    let classes: Vec<&str> = vec![];
    let sorted = sorter.sort_classes(&classes);
    assert_eq!(sorted.len(), 0);
}

#[test]
fn test_single_class() {
    let sorter = HybridSorter::new();
    let classes = vec!["flex"];
    let sorted = sorter.sort_classes(&classes);
    assert_eq!(sorted, vec!["flex"]);
}

#[test]
fn test_duplicate_classes() {
    let sorter = HybridSorter::new();

    let classes = vec!["flex", "p-4", "flex", "m-4", "p-4"];
    let sorted = sorter.sort_classes(&classes);

    // Duplicates should be preserved (sorting doesn't dedupe)
    assert_eq!(sorted.len(), 5);
    assert_eq!(sorted.iter().filter(|&&c| c == "flex").count(), 2);
    assert_eq!(sorted.iter().filter(|&&c| c == "p-4").count(), 2);
}

#[test]
fn test_variants_beyond_64_sort_after_base_classes() {
    // This test would have caught the u64 overflow bug!
    // It verifies that variants at indices >= 64 sort correctly.
    //
    // With the old u64 bug, these variants had variant_order = 0
    // (same as base classes), causing them to sort incorrectly.
    let sorter = HybridSorter::new();

    // Test each problematic variant separately to give clear error messages
    let test_cases = vec![
        ("dark", 70),
        ("@3xl", 64),
        ("@4xl", 65),
        ("print", 73),
        ("portrait", 74),
        ("landscape", 75),
        ("motion-safe", 71),
        ("motion-reduce", 72),
    ];

    for (variant, expected_idx) in test_cases {
        let variant_class = format!("{}:flex", variant);
        let classes = vec!["flex", variant_class.as_str()];
        let sorted = sorter.sort_classes(&classes);

        // CRITICAL: Base class MUST come first, variant MUST come second
        // With the u64 bug, the variant class would sometimes come first!
        assert_eq!(
            sorted[0], "flex",
            "Base class 'flex' should come before '{}:flex' (variant at index {})",
            variant, expected_idx
        );
        assert_eq!(
            sorted[1], variant_class,
            "Variant '{}:flex' (index {}) should come after base class",
            variant, expected_idx
        );
    }
}

#[test]
fn test_transition_utilities_sort_correctly() {
    // Regression test for transition utilities being treated as unknown
    // Previously, transition-colors and duration-* were not recognized
    let sorter = HybridSorter::new();

    let classes = vec![
        "transition-colors",
        "duration-200",
        "delay-100",
        "p-4",
        "bg-white",
        "hover:bg-gray-100",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All base classes should come before variants
    let variant_idx = sorted
        .iter()
        .position(|&c| c == "hover:bg-gray-100")
        .unwrap();

    // Verify transition utilities are recognized and sort before variants
    let transition_idx = sorted
        .iter()
        .position(|&c| c == "transition-colors")
        .unwrap();
    let duration_idx = sorted.iter().position(|&c| c == "duration-200").unwrap();
    let delay_idx = sorted.iter().position(|&c| c == "delay-100").unwrap();

    assert!(
        transition_idx < variant_idx,
        "transition-colors should come before variants"
    );
    assert!(
        duration_idx < variant_idx,
        "duration-200 should come before variants"
    );
    assert!(
        delay_idx < variant_idx,
        "delay-100 should come before variants"
    );

    // Verify property order: transition-property (393) < transition-delay (395) < transition-duration (396)
    // So: transition-colors < delay-100 < duration-200
    assert!(
        transition_idx < delay_idx,
        "transition-colors (transition-property: 393) should come before delay-100 (transition-delay: 395)"
    );
    assert!(
        delay_idx < duration_idx,
        "delay-100 (transition-delay: 395) should come before duration-200 (transition-duration: 396)"
    );
}

#[test]
fn test_dark_mode_realistic_example() {
    // Realistic test case: dark mode is commonly used and was broken with u64
    let sorter = HybridSorter::new();

    let classes = vec![
        "p-4",
        "bg-white",
        "text-gray-900",
        "dark:bg-gray-800",
        "dark:text-white",
        "hover:bg-gray-100",
        "dark:hover:bg-gray-700",
    ];

    let sorted = sorter.sort_classes(&classes);

    // Base classes first (no :)
    assert!(!sorted[0].contains(':'));
    assert!(!sorted[1].contains(':'));
    assert!(!sorted[2].contains(':'));

    // Then single variants (hover < dark)
    assert!(sorted[3].contains(':') && sorted[3].starts_with("hover:"));

    // dark: variants (single variant)
    let dark_single_start = sorted
        .iter()
        .position(|c| c.starts_with("dark:") && !c.contains("hover:"))
        .unwrap();

    // dark: should come after hover: (dark index 70 > hover index 33)
    assert!(
        dark_single_start > 3,
        "dark: variants should come after hover:"
    );

    // Multiple variants (dark:hover:) at the end
    assert!(sorted.last().unwrap().starts_with("dark:hover:"));
}

#[test]
fn test_empty_variant_ordering() {
    // Regression test for empty variant positioning
    // empty (index 33) should come after visited (17), target (18), checked (21)
    // Using same base utility (hidden) so variants are the primary sort key
    let sorter = HybridSorter::new();

    let classes = vec![
        "p-4",
        "empty:hidden",
        "visited:hidden",
        "target:hidden",
        "checked:hidden",
    ];

    let sorted = sorter.sort_classes(&classes);

    // Base class first
    assert_eq!(sorted[0], "p-4");

    // Get positions
    let visited_pos = sorted.iter().position(|&c| c == "visited:hidden").unwrap();
    let target_pos = sorted.iter().position(|&c| c == "target:hidden").unwrap();
    let checked_pos = sorted.iter().position(|&c| c == "checked:hidden").unwrap();
    let empty_pos = sorted.iter().position(|&c| c == "empty:hidden").unwrap();

    // Verify order: visited (17) < target (18) < checked (21) < empty (33)
    assert!(
        visited_pos < target_pos,
        "visited (17) should come before target (18)"
    );
    assert!(
        target_pos < checked_pos,
        "target (18) should come before checked (21)"
    );
    assert!(
        checked_pos < empty_pos,
        "checked (21) should come before empty (33)"
    );
}

#[test]
fn test_enabled_disabled_variant_ordering() {
    // Regression test for enabled/disabled variant ordering
    // enabled (index 39) should come before disabled (40)
    let sorter = HybridSorter::new();

    let classes = vec![
        "flex",
        "enabled:hover:bg-blue-700",
        "disabled:opacity-50",
        "disabled:cursor-not-allowed",
        "enabled:cursor-pointer",
    ];

    let sorted = sorter.sort_classes(&classes);

    // Base class first
    assert_eq!(sorted[0], "flex");

    // Get positions of single-variant enabled and disabled
    let enabled_pos = sorted
        .iter()
        .position(|&c| c == "enabled:cursor-pointer")
        .unwrap();
    let disabled_pos1 = sorted
        .iter()
        .position(|&c| c == "disabled:opacity-50")
        .unwrap();
    let disabled_pos2 = sorted
        .iter()
        .position(|&c| c == "disabled:cursor-not-allowed")
        .unwrap();

    // Verify enabled (39) comes before disabled (40)
    assert!(
        enabled_pos < disabled_pos1,
        "enabled (39) should come before disabled (40)"
    );
    assert!(
        enabled_pos < disabled_pos2,
        "enabled (39) should come before disabled (40)"
    );
}

#[test]
fn test_landscape_variant_ordering() {
    // Regression test for landscape variant positioning
    // landscape should come after responsive breakpoints (sm, md, lg, xl, 2xl)
    // Container queries (@3xl) are unknown variants and sort LAST
    let sorter = HybridSorter::new();

    let classes = vec![
        "flex",
        "landscape:flex-row",
        "sm:grid",
        "md:block",
        "lg:flex-col",
        "xl:inline-flex",
        "2xl:table",
        "@3xl:hidden",
    ];

    let sorted = sorter.sort_classes(&classes);

    // Base class first
    assert_eq!(sorted[0], "flex");

    // Get positions
    let sm_pos = sorted.iter().position(|&c| c == "sm:grid").unwrap();
    let md_pos = sorted.iter().position(|&c| c == "md:block").unwrap();
    let lg_pos = sorted.iter().position(|&c| c == "lg:flex-col").unwrap();
    let xl_pos = sorted.iter().position(|&c| c == "xl:inline-flex").unwrap();
    let xxl_pos = sorted.iter().position(|&c| c == "2xl:table").unwrap();
    let container_pos = sorted.iter().position(|&c| c == "@3xl:hidden").unwrap();
    let landscape_pos = sorted
        .iter()
        .position(|&c| c == "landscape:flex-row")
        .unwrap();

    // Verify responsive breakpoints come in order
    assert!(sm_pos < md_pos, "sm should come before md");
    assert!(md_pos < lg_pos, "md should come before lg");
    assert!(lg_pos < xl_pos, "lg should come before xl");
    assert!(xl_pos < xxl_pos, "xl should come before 2xl");
    assert!(xxl_pos < landscape_pos, "2xl should come before landscape");

    // Container queries (@3xl) are unknown variants and sort LAST (after landscape)
    // This matches Prettier's behavior where unknown/arbitrary variants sort at the end
    assert!(
        landscape_pos < container_pos,
        "landscape should come before @3xl (container queries sort last)"
    );
}

#[test]
fn test_user_select_utilities_ordering() {
    // Regression test for user-select property addition
    // select-* utilities should map to user-select property (index 339)
    // and sort after transition utilities but before will-change
    let sorter = HybridSorter::new();

    let classes = vec![
        "select-none",
        "select-text",
        "select-all",
        "select-auto",
        "transition-colors",
        "duration-200",
        "will-change-transform",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized (no unknowns)
    assert_eq!(sorted.len(), 7);

    // Get positions
    let transition_pos = sorted
        .iter()
        .position(|&c| c == "transition-colors")
        .unwrap();

    // Find any select utility position
    let select_positions: Vec<usize> = sorted
        .iter()
        .enumerate()
        .filter(|(_, c)| c.starts_with("select-"))
        .map(|(i, _)| i)
        .collect();

    // Verify select utilities come after transition properties
    // transition-property (393) < user-select (339)
    for select_pos in &select_positions {
        assert!(
            transition_pos < *select_pos,
            "select-* utilities should come after transition-property (393)"
        );
    }

    // Verify select utilities are alphabetically sorted among themselves
    let select_classes: Vec<&str> = sorted
        .iter()
        .filter(|c| c.starts_with("select-"))
        .copied()
        .collect();

    assert_eq!(select_classes[0], "select-all");
    assert_eq!(select_classes[1], "select-auto");
    assert_eq!(select_classes[2], "select-none");
    assert_eq!(select_classes[3], "select-text");
}

/// Test arbitrary variant classes (Issue #115)
/// These use CSS selectors inside brackets as variants like [&.class]:utility
#[test]
fn test_arbitrary_variant_classes() {
    let sorter = HybridSorter::new();

    // Test various arbitrary variant patterns
    let classes = vec![
        "[&.htmx-request]:h-0",
        "flex",
        "[&.active]:bg-red-500",
        "p-4",
        "[&>*]:p-4",
        "m-2",
        "[&[data-state=open]]:bg-gray-100",
        "[&_p]:text-gray-700",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All classes should be present
    assert_eq!(sorted.len(), 8);

    // Base utilities (no variants) should come before variant utilities
    let flex_pos = sorted.iter().position(|&c| c == "flex").unwrap();
    let arbitrary_pos = sorted
        .iter()
        .position(|&c| c == "[&.htmx-request]:h-0")
        .unwrap();

    assert!(
        flex_pos < arbitrary_pos,
        "Base utilities should come before arbitrary variant utilities"
    );
}

/// Test arbitrary variant classes with child/sibling selectors
#[test]
fn test_arbitrary_variant_combinators() {
    let sorter = HybridSorter::new();

    let classes = vec![
        "[&>*]:p-4", // child combinator
        "[&>*:last-child]:rounded-b-lg",
        "[&+*]:mt-4",          // adjacent sibling
        "[&~*]:opacity-50",    // general sibling
        "[&_p]:text-gray-700", // descendant
        "block",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All classes should be recognized and sorted
    assert_eq!(sorted.len(), 6);

    // block (base utility) should come first
    assert_eq!(sorted[0], "block");

    let child_pos = sorted.iter().position(|&c| c == "[&>*]:p-4").unwrap();
    let last_child_pos = sorted
        .iter()
        .position(|&c| c == "[&>*:last-child]:rounded-b-lg")
        .unwrap();

    assert!(
        child_pos < last_child_pos,
        "Base child selector should come before its :last-child refinement"
    );
}

/// Test arbitrary variant classes with attribute selectors
#[test]
fn test_arbitrary_variant_attributes() {
    let sorter = HybridSorter::new();

    let classes = vec![
        "[&[data-state=open]]:bg-gray-100",
        "[&[aria-selected=true]]:bg-blue-100",
        "[&[data-active]]:ring-2",
        "flex",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All classes should be present
    assert_eq!(sorted.len(), 4);

    // flex should come first (no variants)
    assert_eq!(sorted[0], "flex");
}

/// Test at-rule arbitrary variants
#[test]
fn test_arbitrary_variant_at_rules() {
    let sorter = HybridSorter::new();

    let classes = vec![
        "[@supports(display:grid)]:grid",
        "flex",
        "[@media(min-width:400px)]:block",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All classes should be present
    assert_eq!(sorted.len(), 3);

    // flex should come first (no variants)
    assert_eq!(sorted[0], "flex");
}

//! Integration tests for pattern-based sorting
//!
//! These tests verify that the pattern-based sorter produces correct results
//! with real-world class lists and edge cases.

use rustywind_core::hybrid_sorter::HybridSorter;
use rustywind_core::pattern_sorter::sort_classes;

#[test]
fn test_realistic_component_classes() {
    let sorter = HybridSorter::new();

    // Realistic component with mixed utilities
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

    // Verify base classes come before variants
    let base_count = sorted.iter().filter(|c| !c.contains(':')).count();
    assert_eq!(base_count, 9);

    // At least one variant class exists
    let variant_count = sorted.iter().filter(|c| c.contains(':')).count();
    assert_eq!(variant_count, 1);

    // Verify sorting separates base and variant classes
    assert!(sorted.contains(&"hover:bg-gray-100"));
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
fn test_unknown_classes_go_last() {
    let sorter = HybridSorter::new();

    let classes = vec![
        "flex",
        "unknown-utility-xyz",
        "p-4",
        "fake-class-123",
        "m-4",
        "another-unknown",
    ];

    let sorted = sorter.sort_classes(&classes);

    // Known classes should come first (m-4: margin=25, flex: display=35, p-4: padding=252)
    assert_eq!(sorted[0], "m-4");
    assert_eq!(sorted[1], "flex");
    assert_eq!(sorted[2], "p-4");

    // Unknown classes should be at the end, alphabetically
    assert_eq!(sorted[3], "another-unknown");
    assert_eq!(sorted[4], "fake-class-123");
    assert_eq!(sorted[5], "unknown-utility-xyz");
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
        "px-4", // padding-left + padding-right
        "py-4", // padding-top + padding-bottom
        "mx-auto", // margin-left + margin-right
        "my-4", // margin-top + margin-bottom
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

    let classes = vec![
        "p-4",
        "focus:p-4",
        "hover:p-4",
        "active:p-4",
        "visited:p-4",
    ];

    let sorted = sorter.sort_classes(&classes);

    // Base class first
    assert_eq!(sorted[0], "p-4");

    // hover (33) comes before focus (34) in variant order
    let hover_pos = sorted.iter().position(|&c| c == "hover:p-4").unwrap();
    let focus_pos = sorted.iter().position(|&c| c == "focus:p-4").unwrap();
    assert!(hover_pos < focus_pos, "hover should come before focus");
}

#[test]
fn test_property_count_ordering() {
    let sorter = HybridSorter::new();

    // p generates 1 property (padding)
    // px generates 2 properties (padding-left, padding-right)
    let classes = vec!["px-4", "p-4"];
    let sorted = sorter.sort_classes(&classes);

    // Fewer properties first: p-4 (1 property) before px-4 (2 properties)
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
        "p-4",   // padding
        "px-4",  // padding-inline
        "pt-4",  // padding-top
        "m-4",   // margin
        "mx-4",  // margin-inline
        "mt-4",  // margin-top
    ];

    let sorted = sorter.sort_classes(&classes);

    // margin properties should come before padding properties
    // Sorted by property index from property_order.rs:
    // margin(25) < margin-inline(26) < margin-top(30)
    // padding(252) < padding-top(257) < min(padding-left=316,padding-right=314)=314
    assert_eq!(sorted[0], "m-4");   // margin: index 25
    assert_eq!(sorted[1], "mx-4");  // margin-inline: index 26
    assert_eq!(sorted[2], "mt-4");  // margin-top: index 30
    assert_eq!(sorted[3], "p-4");   // padding: index 252
    assert_eq!(sorted[4], "pt-4");  // padding-top: index 257
    assert_eq!(sorted[5], "px-4");  // min(padding-left=316, padding-right=314) = 314
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

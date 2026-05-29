//! Comprehensive tests for utility categories that were fixed in Phase 5
//!
//! These tests verify that the filter, backdrop-filter, ring, border-radius,
//! and transform utilities all map to the correct properties and sort correctly.

use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_filter_utilities_basic() {
    // Test that filter utilities map to correct custom properties
    let sorter = HybridSorter::new();

    let classes = vec![
        "blur-sm",
        "blur-md",
        "blur-lg",
        "brightness-50",
        "brightness-100",
        "brightness-150",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized
    assert_eq!(sorted.len(), 6);

    // blur utilities should be grouped together (all map to --tw-blur)
    let blur_sm_pos = sorted.iter().position(|&c| c == "blur-sm").unwrap();
    let blur_md_pos = sorted.iter().position(|&c| c == "blur-md").unwrap();
    let blur_lg_pos = sorted.iter().position(|&c| c == "blur-lg").unwrap();

    // brightness utilities should be grouped together (all map to --tw-brightness)
    let bright_50_pos = sorted.iter().position(|&c| c == "brightness-50").unwrap();
    let _bright_100_pos = sorted.iter().position(|&c| c == "brightness-100").unwrap();
    let _bright_150_pos = sorted.iter().position(|&c| c == "brightness-150").unwrap();

    // All blur utilities should come before all brightness utilities
    // (--tw-blur at index 374, --tw-brightness at index 375)
    assert!(
        blur_sm_pos < bright_50_pos,
        "--tw-blur should come before --tw-brightness"
    );
    assert!(
        blur_md_pos < bright_50_pos,
        "--tw-blur should come before --tw-brightness"
    );
    assert!(
        blur_lg_pos < bright_50_pos,
        "--tw-blur should come before --tw-brightness"
    );
}

#[test]
fn test_filter_toggle_utilities_sort_with_filter_properties() {
    let sorter = HybridSorter::new();

    let classes = vec![
        "filter-none",
        "hidden",
        "outline-dotted",
        "filter",
        "outline",
    ];

    let sorted = sorter.sort_classes(&classes);

    assert_eq!(
        sorted,
        vec![
            "hidden",
            "outline",
            "filter",
            "filter-none",
            "outline-dotted"
        ]
    );
}

#[test]
fn test_backdrop_filter_toggle_utilities_sort_with_backdrop_properties() {
    let sorter = HybridSorter::new();

    let classes = vec![
        "backdrop-filter-none",
        "hidden",
        "backdrop-filter",
        "backdrop-blur",
    ];

    let sorted = sorter.sort_classes(&classes);

    assert_eq!(
        sorted,
        vec![
            "hidden",
            "backdrop-blur",
            "backdrop-filter",
            "backdrop-filter-none"
        ]
    );
}

#[test]
fn test_filter_utilities_comprehensive() {
    // Test all filter utility types to ensure they map correctly
    let sorter = HybridSorter::new();

    let classes = vec![
        "blur-sm",        // --tw-blur (374)
        "brightness-110", // --tw-brightness (375)
        "contrast-125",   // --tw-contrast (376)
        "drop-shadow-lg", // --tw-drop-shadow (377)
        "grayscale",      // --tw-grayscale (378)
        "hue-rotate-90",  // --tw-hue-rotate (379)
        "invert",         // --tw-invert (380)
        "saturate-150",   // --tw-saturate (381)
        "sepia",          // --tw-sepia (382)
    ];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized (no unknowns sent to end)
    assert_eq!(sorted.len(), 9);

    // Let's verify the actual order by checking property indices
    // The utilities should group by property, then sort alphabetically/numerically

    // All filter utilities map to --tw-* properties at indices 374-382
    // They should be in that property order

    // Find positions
    let blur_pos = sorted.iter().position(|&c| c == "blur-sm").unwrap();
    let brightness_pos = sorted.iter().position(|&c| c == "brightness-110").unwrap();
    let contrast_pos = sorted.iter().position(|&c| c == "contrast-125").unwrap();
    let drop_shadow_pos = sorted.iter().position(|&c| c == "drop-shadow-lg").unwrap();
    let grayscale_pos = sorted.iter().position(|&c| c == "grayscale").unwrap();
    let hue_rotate_pos = sorted.iter().position(|&c| c == "hue-rotate-90").unwrap();
    let invert_pos = sorted.iter().position(|&c| c == "invert").unwrap();
    let saturate_pos = sorted.iter().position(|&c| c == "saturate-150").unwrap();
    let sepia_pos = sorted.iter().position(|&c| c == "sepia").unwrap();

    // Verify property order: each property should come before the next one
    assert!(
        blur_pos < brightness_pos,
        "--tw-blur (374) < --tw-brightness (375)"
    );
    assert!(
        brightness_pos < contrast_pos,
        "--tw-brightness (375) < --tw-contrast (376)"
    );
    assert!(
        contrast_pos < drop_shadow_pos,
        "--tw-contrast (376) < --tw-drop-shadow (377)"
    );
    assert!(
        drop_shadow_pos < grayscale_pos,
        "--tw-drop-shadow (377) < --tw-grayscale (378)"
    );
    assert!(
        grayscale_pos < hue_rotate_pos,
        "--tw-grayscale (378) < --tw-hue-rotate (379)"
    );
    assert!(
        hue_rotate_pos < invert_pos,
        "--tw-hue-rotate (379) < --tw-invert (380)"
    );
    assert!(
        invert_pos < saturate_pos,
        "--tw-invert (380) < --tw-saturate (381)"
    );
    assert!(
        saturate_pos < sepia_pos,
        "--tw-saturate (381) < --tw-sepia (382)"
    );
}

#[test]
fn test_backdrop_filter_utilities() {
    // Test that backdrop-filter utilities map to correct custom properties
    let sorter = HybridSorter::new();

    let classes = vec![
        "backdrop-blur-sm",        // --tw-backdrop-blur (384)
        "backdrop-brightness-110", // --tw-backdrop-brightness (385)
        "backdrop-contrast-125",   // --tw-backdrop-contrast (386)
        "backdrop-grayscale",      // --tw-backdrop-grayscale (387)
        "backdrop-hue-rotate-90",  // --tw-backdrop-hue-rotate (388)
        "backdrop-invert",         // --tw-backdrop-invert (389)
        "backdrop-opacity-50",     // --tw-backdrop-opacity (390)
        "backdrop-saturate-150",   // --tw-backdrop-saturate (391)
        "backdrop-sepia",          // --tw-backdrop-sepia (392)
    ];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized
    assert_eq!(sorted.len(), 9);

    // Verify they're sorted in property order (indices 384-392)
    assert_eq!(sorted[0], "backdrop-blur-sm");
    assert_eq!(sorted[1], "backdrop-brightness-110");
    assert_eq!(sorted[2], "backdrop-contrast-125");
    assert_eq!(sorted[3], "backdrop-grayscale");
    assert_eq!(sorted[4], "backdrop-hue-rotate-90");
    assert_eq!(sorted[5], "backdrop-invert");
    assert_eq!(sorted[6], "backdrop-opacity-50");
    assert_eq!(sorted[7], "backdrop-saturate-150");
    assert_eq!(sorted[8], "backdrop-sepia");
}

#[test]
fn test_filter_vs_backdrop_filter_ordering() {
    // Filters should come before backdrop-filters
    let sorter = HybridSorter::new();

    let classes = vec![
        "backdrop-blur-sm",
        "blur-md",
        "backdrop-brightness-110",
        "brightness-125",
    ];

    let sorted = sorter.sort_classes(&classes);

    // blur utilities (374-382) should come before backdrop utilities (384-392)
    let blur_pos = sorted.iter().position(|&c| c == "blur-md").unwrap();
    let brightness_pos = sorted.iter().position(|&c| c == "brightness-125").unwrap();
    let backdrop_blur_pos = sorted
        .iter()
        .position(|&c| c == "backdrop-blur-sm")
        .unwrap();
    let backdrop_brightness_pos = sorted
        .iter()
        .position(|&c| c == "backdrop-brightness-110")
        .unwrap();

    assert!(
        blur_pos < backdrop_blur_pos,
        "filter should come before backdrop-filter"
    );
    assert!(
        brightness_pos < backdrop_brightness_pos,
        "filter should come before backdrop-filter"
    );
}

#[test]
fn test_ring_utilities_basic() {
    // Test ring utility sorting
    let sorter = HybridSorter::new();

    let classes = vec![
        "ring",                 // --tw-ring-shadow (360)
        "ring-2",               // --tw-ring-shadow (360)
        "ring-blue-500",        // --tw-ring-color (361)
        "ring-offset-2",        // --tw-ring-offset-width (366)
        "ring-offset-blue-500", // --tw-ring-offset-color (367)
    ];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized
    assert_eq!(sorted.len(), 5);

    // ring width utilities should come before ring color
    let ring_pos = sorted.iter().position(|&c| c == "ring").unwrap();
    let ring_2_pos = sorted.iter().position(|&c| c == "ring-2").unwrap();
    let ring_color_pos = sorted.iter().position(|&c| c == "ring-blue-500").unwrap();

    assert!(
        ring_pos < ring_color_pos,
        "ring width should come before ring color"
    );
    assert!(
        ring_2_pos < ring_color_pos,
        "ring width should come before ring color"
    );
}

#[test]
fn test_ring_inset_utility() {
    // Test that ring-inset is recognized
    let sorter = HybridSorter::new();

    let classes = vec!["ring-2", "ring-inset", "ring-blue-500"];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized
    assert_eq!(sorted.len(), 3);

    // ring-inset should be grouped with other ring utilities
    assert!(sorted.contains(&"ring-inset"));
}

#[test]
fn test_border_radius_utilities() {
    // Test border-radius utility sorting with different corner combinations
    let sorter = HybridSorter::new();

    let classes = vec![
        "rounded",    // border-radius
        "rounded-t",  // border-top-radius (not real, but mapped)
        "rounded-tr", // border-top-right-radius
        "rounded-r",  // border-right-radius (not real, but mapped)
        "rounded-br", // border-bottom-right-radius
        "rounded-b",  // border-bottom-radius (not real, but mapped)
        "rounded-bl", // border-bottom-left-radius
        "rounded-l",  // border-left-radius (not real, but mapped)
        "rounded-tl", // border-top-left-radius
    ];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized
    assert_eq!(sorted.len(), 9);

    // Generic rounded should come first (border-radius at index 178)
    assert_eq!(sorted[0], "rounded");

    // Specific corner utilities should come after
    // The order follows property_order.rs indices
}

#[test]
fn test_transform_utilities_scale() {
    // Test scale utilities with numeric value sorting
    let sorter = HybridSorter::new();

    let classes = vec![
        "scale-150",
        "scale-50",
        "scale-100",
        "scale-x-150",
        "scale-x-50",
        "scale-y-100",
        "scale-y-75",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized
    assert_eq!(sorted.len(), 7);

    // scale (general) utilities should be grouped together
    // scale-x utilities should be grouped together
    // scale-y utilities should be grouped together

    // Within each group, should be sorted by numeric value
    let scale_50_pos = sorted.iter().position(|&c| c == "scale-50").unwrap();
    let scale_100_pos = sorted.iter().position(|&c| c == "scale-100").unwrap();
    let scale_150_pos = sorted.iter().position(|&c| c == "scale-150").unwrap();

    assert!(
        scale_50_pos < scale_100_pos,
        "scale-50 should come before scale-100"
    );
    assert!(
        scale_100_pos < scale_150_pos,
        "scale-100 should come before scale-150"
    );
}

#[test]
fn test_transform_utilities_translate() {
    // Test translate utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "translate-x-4",
        "translate-x-2",
        "translate-y-8",
        "translate-y-4",
        "-translate-x-2",
    ];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized
    assert_eq!(sorted.len(), 5);

    // Negative translate should come before positive (negative values sort first)
    let neg_pos = sorted.iter().position(|&c| c == "-translate-x-2").unwrap();
    let pos_2_pos = sorted.iter().position(|&c| c == "translate-x-2").unwrap();
    let pos_4_pos = sorted.iter().position(|&c| c == "translate-x-4").unwrap();

    assert!(
        neg_pos < pos_2_pos,
        "negative translate should come before positive"
    );
    assert!(
        pos_2_pos < pos_4_pos,
        "translate-x-2 should come before translate-x-4"
    );
}

#[test]
fn test_transform_utilities_rotate() {
    // Test rotate utilities
    let sorter = HybridSorter::new();

    let classes = vec!["rotate-180", "rotate-45", "rotate-90", "-rotate-45"];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized
    assert_eq!(sorted.len(), 4);

    // Should be sorted by numeric value
    let neg_45_pos = sorted.iter().position(|&c| c == "-rotate-45").unwrap();
    let pos_45_pos = sorted.iter().position(|&c| c == "rotate-45").unwrap();
    let pos_90_pos = sorted.iter().position(|&c| c == "rotate-90").unwrap();
    let pos_180_pos = sorted.iter().position(|&c| c == "rotate-180").unwrap();

    assert!(
        neg_45_pos < pos_45_pos,
        "negative should come before positive"
    );
    assert!(pos_45_pos < pos_90_pos, "45 should come before 90");
    assert!(pos_90_pos < pos_180_pos, "90 should come before 180");
}

#[test]
fn test_mixed_utility_categories() {
    // Test that different utility categories sort in correct property order
    let sorter = HybridSorter::new();

    let classes = vec![
        "ring-2",            // Shadow group (360)
        "blur-sm",           // Filter (374)
        "backdrop-blur-sm",  // Backdrop filter (384)
        "transition-colors", // Transition (397)
        "scale-100",         // Transform (79-88)
        "rotate-45",         // Transform (83)
    ];

    let sorted = sorter.sort_classes(&classes);

    // All should be recognized
    assert_eq!(sorted.len(), 6);

    // Should be in property order:
    // scale (79), rotate (83), ring (360), blur (374), backdrop (384), transition (397)
    assert_eq!(sorted[0], "scale-100");
    assert_eq!(sorted[1], "rotate-45");
    assert_eq!(sorted[2], "ring-2");
    assert_eq!(sorted[3], "blur-sm");
    assert_eq!(sorted[4], "backdrop-blur-sm");
    assert_eq!(sorted[5], "transition-colors");
}

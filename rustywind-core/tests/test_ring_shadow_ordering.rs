//! Tests for ring vs shadow utility ordering issues found in fuzz testing
//!
//! These tests verify that ring utilities are sorted in the correct position
//! relative to shadow utilities.
//!
//! From fuzz testing analysis (36 total failures):
//! - 25 shadow utility failures
//! - 11 ring utility failures
//!
//! Expected order (Prettier): ring → shadow
//! Bug: RustyWind was sorting shadow utilities BEFORE ring utilities
//!
//! Example failures:
//! - ring-0 should come BEFORE shadow-blue-500
//! - ring should come BEFORE shadow-gray-500
//! - ring-2 should come BEFORE shadow-gray-500

use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_ring_0_vs_shadow_with_color() {
    // ring-0 should come BEFORE shadow utilities with colors
    let sorter = HybridSorter::new();

    let classes = vec!["shadow-blue-500", "ring-0"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: ring-0 vs shadow-blue-500");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects: ring-0, shadow-blue-500
    assert_eq!(
        sorted[0], "ring-0",
        "ring-0 should come before shadow-blue-500"
    );
    assert_eq!(sorted[1], "shadow-blue-500");
}

#[test]
fn test_ring_vs_shadow_with_color() {
    // ring should come BEFORE shadow utilities with colors
    let sorter = HybridSorter::new();

    let classes = vec!["shadow-gray-500", "ring"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: ring vs shadow-gray-500");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects: ring, shadow-gray-500
    assert_eq!(sorted[0], "ring", "ring should come before shadow-gray-500");
    assert_eq!(sorted[1], "shadow-gray-500");
}

#[test]
fn test_ring_2_vs_shadow_with_color() {
    // ring-2 should come BEFORE shadow utilities with colors
    let sorter = HybridSorter::new();

    let classes = vec!["shadow-gray-500", "ring-2"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: ring-2 vs shadow-gray-500");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects: ring-2, shadow-gray-500
    assert_eq!(
        sorted[0], "ring-2",
        "ring-2 should come before shadow-gray-500"
    );
    assert_eq!(sorted[1], "shadow-gray-500");
}

#[test]
fn test_ring_utilities_vs_shadow_sizes() {
    // Shadow SIZE utilities come BEFORE ring SIZE utilities (per Prettier/Tailwind)
    let sorter = HybridSorter::new();

    let classes = vec![
        "shadow-sm",
        "ring-0",
        "shadow-lg",
        "ring",
        "shadow-xl",
        "ring-2",
        "shadow",
        "ring-1",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: ring utilities vs shadow size utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let ring_0_pos = sorted.iter().position(|&c| c == "ring-0").unwrap();
    let ring_pos = sorted.iter().position(|&c| c == "ring").unwrap();
    let ring_1_pos = sorted.iter().position(|&c| c == "ring-1").unwrap();
    let ring_2_pos = sorted.iter().position(|&c| c == "ring-2").unwrap();
    let shadow_pos = sorted.iter().position(|&c| c == "shadow").unwrap();
    let shadow_sm_pos = sorted.iter().position(|&c| c == "shadow-sm").unwrap();
    let shadow_lg_pos = sorted.iter().position(|&c| c == "shadow-lg").unwrap();
    let shadow_xl_pos = sorted.iter().position(|&c| c == "shadow-xl").unwrap();

    // Shadow SIZE utilities should come BEFORE ring utilities
    assert!(shadow_pos < ring_0_pos, "shadow should come before ring-0");
    assert!(
        shadow_sm_pos < ring_0_pos,
        "shadow-sm should come before ring-0"
    );
    assert!(
        shadow_lg_pos < ring_0_pos,
        "shadow-lg should come before ring-0"
    );
    assert!(
        shadow_xl_pos < ring_0_pos,
        "shadow-xl should come before ring-0"
    );

    assert!(shadow_pos < ring_pos, "shadow should come before ring");
    assert!(
        shadow_lg_pos < ring_pos,
        "shadow-lg should come before ring"
    );
    assert!(
        shadow_xl_pos < ring_pos,
        "shadow-xl should come before ring"
    );

    assert!(shadow_pos < ring_1_pos, "shadow should come before ring-1");
    assert!(
        shadow_sm_pos < ring_1_pos,
        "shadow-sm should come before ring-1"
    );
    assert!(
        shadow_lg_pos < ring_1_pos,
        "shadow-lg should come before ring-1"
    );
    assert!(
        shadow_xl_pos < ring_1_pos,
        "shadow-xl should come before ring-1"
    );

    assert!(shadow_pos < ring_2_pos, "shadow should come before ring-2");
    assert!(
        shadow_sm_pos < ring_2_pos,
        "shadow-sm should come before ring-2"
    );
    assert!(
        shadow_lg_pos < ring_2_pos,
        "shadow-lg should come before ring-2"
    );
    assert!(
        shadow_xl_pos < ring_2_pos,
        "shadow-xl should come before ring-2"
    );
}

#[test]
fn test_mixed_ring_shadow_with_other_utilities() {
    // Test ring and shadow utilities mixed with other utilities
    // This tests the complete ordering hierarchy
    let sorter = HybridSorter::new();

    let classes = vec![
        "shadow-blue-500",
        "border-2",
        "ring-0",
        "bg-white",
        "shadow-sm",
        "ring-2",
        "p-4",
        "shadow-gray-500",
        "ring",
        "text-gray-900",
        "shadow-lg",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: mixed ring and shadow with other utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let ring_0_pos = sorted.iter().position(|&c| c == "ring-0").unwrap();
    let ring_pos = sorted.iter().position(|&c| c == "ring").unwrap();
    let ring_2_pos = sorted.iter().position(|&c| c == "ring-2").unwrap();
    let shadow_sm_pos = sorted.iter().position(|&c| c == "shadow-sm").unwrap();
    let shadow_lg_pos = sorted.iter().position(|&c| c == "shadow-lg").unwrap();
    let shadow_blue_pos = sorted.iter().position(|&c| c == "shadow-blue-500").unwrap();
    let shadow_gray_pos = sorted.iter().position(|&c| c == "shadow-gray-500").unwrap();

    // Correct ordering per Prettier:
    // 1. Shadow SIZE utilities come before ring SIZE utilities
    // 2. Ring SIZE utilities come before shadow COLOR utilities
    assert!(
        shadow_sm_pos < ring_0_pos,
        "shadow-sm should come before ring-0"
    );
    assert!(
        shadow_lg_pos < ring_0_pos,
        "shadow-lg should come before ring-0"
    );
    assert!(
        ring_0_pos < shadow_blue_pos,
        "ring-0 should come before shadow-blue-500"
    );
    assert!(
        ring_0_pos < shadow_gray_pos,
        "ring-0 should come before shadow-gray-500"
    );

    assert!(
        shadow_sm_pos < ring_pos,
        "shadow-sm should come before ring"
    );
    assert!(
        shadow_lg_pos < ring_pos,
        "shadow-lg should come before ring"
    );
    assert!(
        ring_pos < shadow_blue_pos,
        "ring should come before shadow-blue-500"
    );
    assert!(
        ring_pos < shadow_gray_pos,
        "ring should come before shadow-gray-500"
    );

    assert!(
        shadow_sm_pos < ring_2_pos,
        "shadow-sm should come before ring-2"
    );
    assert!(
        shadow_lg_pos < ring_2_pos,
        "shadow-lg should come before ring-2"
    );
    assert!(
        ring_2_pos < shadow_blue_pos,
        "ring-2 should come before shadow-blue-500"
    );
    assert!(
        ring_2_pos < shadow_gray_pos,
        "ring-2 should come before shadow-gray-500"
    );
}

#[test]
fn test_all_ring_widths_vs_shadow_colors() {
    // Test multiple ring width values against shadow utilities with different colors
    let sorter = HybridSorter::new();

    let classes = vec![
        "shadow-blue-500",
        "ring-0",
        "shadow-red-400",
        "ring-1",
        "shadow-green-600",
        "ring-2",
        "shadow-yellow-300",
        "ring-4",
        "shadow-purple-700",
        "ring-8",
        "shadow-pink-500",
        "ring",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple ring widths vs shadow colors");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let ring_0_pos = sorted.iter().position(|&c| c == "ring-0").unwrap();
    let ring_1_pos = sorted.iter().position(|&c| c == "ring-1").unwrap();
    let ring_2_pos = sorted.iter().position(|&c| c == "ring-2").unwrap();
    let ring_4_pos = sorted.iter().position(|&c| c == "ring-4").unwrap();
    let ring_8_pos = sorted.iter().position(|&c| c == "ring-8").unwrap();
    let ring_pos = sorted.iter().position(|&c| c == "ring").unwrap();

    let shadow_blue_pos = sorted.iter().position(|&c| c == "shadow-blue-500").unwrap();
    let shadow_red_pos = sorted.iter().position(|&c| c == "shadow-red-400").unwrap();
    let shadow_green_pos = sorted
        .iter()
        .position(|&c| c == "shadow-green-600")
        .unwrap();
    let shadow_yellow_pos = sorted
        .iter()
        .position(|&c| c == "shadow-yellow-300")
        .unwrap();
    let shadow_purple_pos = sorted
        .iter()
        .position(|&c| c == "shadow-purple-700")
        .unwrap();
    let shadow_pink_pos = sorted.iter().position(|&c| c == "shadow-pink-500").unwrap();

    // Every ring utility should come before every shadow utility
    for ring_class_pos in [
        ring_0_pos, ring_1_pos, ring_2_pos, ring_4_pos, ring_8_pos, ring_pos,
    ] {
        assert!(
            ring_class_pos < shadow_blue_pos,
            "ring utilities should come before shadow-blue-500"
        );
        assert!(
            ring_class_pos < shadow_red_pos,
            "ring utilities should come before shadow-red-400"
        );
        assert!(
            ring_class_pos < shadow_green_pos,
            "ring utilities should come before shadow-green-600"
        );
        assert!(
            ring_class_pos < shadow_yellow_pos,
            "ring utilities should come before shadow-yellow-300"
        );
        assert!(
            ring_class_pos < shadow_purple_pos,
            "ring utilities should come before shadow-purple-700"
        );
        assert!(
            ring_class_pos < shadow_pink_pos,
            "ring utilities should come before shadow-pink-500"
        );
    }
}

#[test]
fn test_ring_inset_vs_shadow() {
    // Test ring-inset utility against shadow utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "shadow-lg",
        "ring-inset",
        "shadow-blue-500",
        "ring-0",
        "shadow",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: ring-inset vs shadow utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let ring_inset_pos = sorted.iter().position(|&c| c == "ring-inset").unwrap();
    let ring_0_pos = sorted.iter().position(|&c| c == "ring-0").unwrap();
    let shadow_pos = sorted.iter().position(|&c| c == "shadow").unwrap();
    let shadow_lg_pos = sorted.iter().position(|&c| c == "shadow-lg").unwrap();
    let shadow_blue_pos = sorted.iter().position(|&c| c == "shadow-blue-500").unwrap();

    // Shadow SIZE utilities come before ring utilities
    // Ring SIZE utilities (like ring-0) come before shadow COLOR utilities
    // ring-inset comes after shadow COLOR utilities (observed behavior)
    assert!(
        shadow_pos < ring_inset_pos,
        "shadow should come before ring-inset"
    );
    assert!(
        shadow_lg_pos < ring_inset_pos,
        "shadow-lg should come before ring-inset"
    );
    assert!(
        shadow_blue_pos < ring_inset_pos,
        "shadow-blue-500 should come before ring-inset"
    );
    assert!(shadow_pos < ring_0_pos, "shadow should come before ring-0");
    assert!(
        shadow_lg_pos < ring_0_pos,
        "shadow-lg should come before ring-0"
    );
    assert!(
        ring_0_pos < shadow_blue_pos,
        "ring-0 should come before shadow-blue-500"
    );
}

#[test]
fn test_ring_colors_vs_shadow_colors() {
    // Test ring utilities with colors against shadow utilities with colors
    let sorter = HybridSorter::new();

    let classes = vec![
        "shadow-gray-500",
        "ring-blue-500",
        "shadow-blue-500",
        "ring-gray-300",
        "shadow-red-400",
        "ring-red-600",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: ring colors vs shadow colors");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let ring_blue_pos = sorted.iter().position(|&c| c == "ring-blue-500").unwrap();
    let ring_gray_pos = sorted.iter().position(|&c| c == "ring-gray-300").unwrap();
    let ring_red_pos = sorted.iter().position(|&c| c == "ring-red-600").unwrap();
    let shadow_gray_pos = sorted.iter().position(|&c| c == "shadow-gray-500").unwrap();
    let shadow_blue_pos = sorted.iter().position(|&c| c == "shadow-blue-500").unwrap();
    let shadow_red_pos = sorted.iter().position(|&c| c == "shadow-red-400").unwrap();

    // Shadow COLOR utilities should come BEFORE ring COLOR utilities (per Prettier)
    assert!(
        shadow_gray_pos < ring_blue_pos,
        "shadow-gray-500 should come before ring-blue-500"
    );
    assert!(
        shadow_blue_pos < ring_blue_pos,
        "shadow-blue-500 should come before ring-blue-500"
    );
    assert!(
        shadow_red_pos < ring_blue_pos,
        "shadow-red-400 should come before ring-blue-500"
    );

    assert!(
        shadow_gray_pos < ring_gray_pos,
        "shadow-gray-500 should come before ring-gray-300"
    );
    assert!(
        shadow_blue_pos < ring_gray_pos,
        "shadow-blue-500 should come before ring-gray-300"
    );
    assert!(
        shadow_red_pos < ring_gray_pos,
        "shadow-red-400 should come before ring-gray-300"
    );

    assert!(
        shadow_gray_pos < ring_red_pos,
        "shadow-gray-500 should come before ring-red-600"
    );
    assert!(
        shadow_blue_pos < ring_red_pos,
        "shadow-blue-500 should come before ring-red-600"
    );
    assert!(
        shadow_red_pos < ring_red_pos,
        "shadow-red-400 should come before ring-red-600"
    );
}

#[test]
fn test_comprehensive_ring_shadow_ordering() {
    // Comprehensive test covering all ring and shadow utility types
    let sorter = HybridSorter::new();

    let classes = vec![
        "shadow",
        "ring-0",
        "shadow-sm",
        "ring-1",
        "shadow-md",
        "ring-2",
        "shadow-lg",
        "ring-4",
        "shadow-xl",
        "ring-8",
        "shadow-2xl",
        "ring",
        "shadow-inner",
        "ring-inset",
        "shadow-none",
        "ring-blue-500",
        "shadow-blue-500",
        "ring-gray-300",
        "shadow-gray-500",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: comprehensive ring and shadow ordering");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find all ring utility positions
    let ring_0_pos = sorted.iter().position(|&c| c == "ring-0").unwrap();
    let ring_1_pos = sorted.iter().position(|&c| c == "ring-1").unwrap();
    let ring_2_pos = sorted.iter().position(|&c| c == "ring-2").unwrap();
    let ring_4_pos = sorted.iter().position(|&c| c == "ring-4").unwrap();
    let ring_8_pos = sorted.iter().position(|&c| c == "ring-8").unwrap();
    let ring_pos = sorted.iter().position(|&c| c == "ring").unwrap();
    let ring_inset_pos = sorted.iter().position(|&c| c == "ring-inset").unwrap();
    let ring_blue_pos = sorted.iter().position(|&c| c == "ring-blue-500").unwrap();
    let ring_gray_pos = sorted.iter().position(|&c| c == "ring-gray-300").unwrap();

    // Find all shadow utility positions
    let shadow_pos = sorted.iter().position(|&c| c == "shadow").unwrap();
    let shadow_sm_pos = sorted.iter().position(|&c| c == "shadow-sm").unwrap();
    let shadow_md_pos = sorted.iter().position(|&c| c == "shadow-md").unwrap();
    let shadow_lg_pos = sorted.iter().position(|&c| c == "shadow-lg").unwrap();
    let shadow_xl_pos = sorted.iter().position(|&c| c == "shadow-xl").unwrap();
    let shadow_2xl_pos = sorted.iter().position(|&c| c == "shadow-2xl").unwrap();
    let shadow_inner_pos = sorted.iter().position(|&c| c == "shadow-inner").unwrap();
    let shadow_none_pos = sorted.iter().position(|&c| c == "shadow-none").unwrap();
    let shadow_blue_pos = sorted.iter().position(|&c| c == "shadow-blue-500").unwrap();
    let shadow_gray_pos = sorted.iter().position(|&c| c == "shadow-gray-500").unwrap();

    // Correct ordering pattern per Prettier:
    // 1. Shadow SIZE utilities
    // 2. Ring SIZE utilities
    // 3. Shadow COLOR utilities
    // 4. Ring COLOR utilities
    // 5. ring-inset (special)

    let shadow_size_positions = vec![
        shadow_pos,
        shadow_sm_pos,
        shadow_md_pos,
        shadow_lg_pos,
        shadow_xl_pos,
        shadow_2xl_pos,
        shadow_inner_pos,
        shadow_none_pos,
    ];
    let ring_size_positions = vec![
        ring_0_pos, ring_1_pos, ring_2_pos, ring_4_pos, ring_8_pos, ring_pos,
    ];
    let shadow_color_positions = vec![shadow_blue_pos, shadow_gray_pos];
    let ring_color_positions = vec![ring_blue_pos, ring_gray_pos];

    // Shadow SIZE before ring SIZE
    for &shadow_size in &shadow_size_positions {
        for &ring_size in &ring_size_positions {
            assert!(
                shadow_size < ring_size,
                "Shadow size at {} should come before ring size at {}",
                shadow_size,
                ring_size
            );
        }
    }

    // Ring SIZE before shadow COLOR
    for &ring_size in &ring_size_positions {
        for &shadow_color in &shadow_color_positions {
            assert!(
                ring_size < shadow_color,
                "Ring size at {} should come before shadow color at {}",
                ring_size,
                shadow_color
            );
        }
    }

    // Shadow COLOR before ring COLOR
    for &shadow_color in &shadow_color_positions {
        for &ring_color in &ring_color_positions {
            assert!(
                shadow_color < ring_color,
                "Shadow color at {} should come before ring color at {}",
                shadow_color,
                ring_color
            );
        }
    }

    // ring-inset comes last
    for &ring_color in &ring_color_positions {
        assert!(
            ring_color < ring_inset_pos,
            "Ring color at {} should come before ring-inset at {}",
            ring_color,
            ring_inset_pos
        );
    }
}

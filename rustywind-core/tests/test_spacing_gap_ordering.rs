//! Tests for spacing vs gap utility ordering issues found in fuzz testing
//!
//! This test suite covers 46 failures related to space utilities vs gap utilities being sorted incorrectly.
//! The main issue: space-x/space-y utilities should come BEFORE gap-x/gap-y utilities (cross-axis comparisons),
//! but RustyWind reverses this order.
//!
//! From fuzz testing analysis:
//! - space-y-2 should come BEFORE gap-x-0 (Prettier), but RustyWind puts gap-x-0 first
//! - space-x-4 should come BEFORE gap-y-2 (Prettier), but RustyWind puts gap-y-2 first
//! - space-x-reverse should come BEFORE gap-y-0 (Prettier), but RustyWind reverses this
//!
//! Pattern: space utilities (cross-axis) should precede gap utilities (cross-axis)

use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_space_y_vs_gap_x() {
    // space-y should come BEFORE gap-x according to Prettier
    let sorter = HybridSorter::new();

    let classes = vec!["gap-x-0", "space-y-2"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space-y-2 vs gap-x-0");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: space-y-2, gap-x-0
    assert_eq!(sorted[0], "space-y-2", "space-y should come before gap-x");
    assert_eq!(sorted[1], "gap-x-0");
}

#[test]
fn test_space_x_vs_gap_y() {
    // space-x should come BEFORE gap-y according to Prettier
    let sorter = HybridSorter::new();

    let classes = vec!["gap-y-2", "space-x-4"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space-x-4 vs gap-y-2");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: space-x-4, gap-y-2
    assert_eq!(sorted[0], "space-x-4", "space-x should come before gap-y");
    assert_eq!(sorted[1], "gap-y-2");
}

#[test]
fn test_space_x_reverse_vs_gap_y() {
    // space-x-reverse should come BEFORE gap-y according to Prettier
    let sorter = HybridSorter::new();

    let classes = vec!["gap-y-0", "space-x-reverse"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space-x-reverse vs gap-y-0");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: space-x-reverse, gap-y-0
    assert_eq!(sorted[0], "space-x-reverse", "space-x-reverse should come before gap-y");
    assert_eq!(sorted[1], "gap-y-0");
}

#[test]
fn test_space_y_reverse_vs_gap_x() {
    // space-y-reverse should come BEFORE gap-x according to Prettier
    let sorter = HybridSorter::new();

    let classes = vec!["gap-x-2", "space-y-reverse"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space-y-reverse vs gap-x-2");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: space-y-reverse, gap-x-2
    assert_eq!(sorted[0], "space-y-reverse", "space-y-reverse should come before gap-x");
    assert_eq!(sorted[1], "gap-x-2");
}

#[test]
fn test_multiple_space_values_vs_gap() {
    // Multiple space utilities with different values should all come BEFORE gap utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "gap-x-0",
        "gap-y-2",
        "space-y-0",
        "space-y-2",
        "space-x-1",
        "space-x-4",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple space utilities vs gap utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let space_y_0_pos = sorted.iter().position(|&c| c == "space-y-0").unwrap();
    let space_y_2_pos = sorted.iter().position(|&c| c == "space-y-2").unwrap();
    let space_x_1_pos = sorted.iter().position(|&c| c == "space-x-1").unwrap();
    let space_x_4_pos = sorted.iter().position(|&c| c == "space-x-4").unwrap();
    let gap_x_0_pos = sorted.iter().position(|&c| c == "gap-x-0").unwrap();
    let gap_y_2_pos = sorted.iter().position(|&c| c == "gap-y-2").unwrap();

    // All space-y utilities should come BEFORE gap-x
    assert!(space_y_0_pos < gap_x_0_pos, "space-y-0 should come before gap-x-0");
    assert!(space_y_2_pos < gap_x_0_pos, "space-y-2 should come before gap-x-0");

    // All space-x utilities should come BEFORE gap-y
    assert!(space_x_1_pos < gap_y_2_pos, "space-x-1 should come before gap-y-2");
    assert!(space_x_4_pos < gap_y_2_pos, "space-x-4 should come before gap-y-2");
}

#[test]
fn test_space_gap_with_other_utilities() {
    // Test space and gap utilities combined with other utilities
    // This mimics real-world usage where multiple utility types are mixed
    let sorter = HybridSorter::new();

    let classes = vec![
        "flex",
        "gap-x-0",
        "items-center",
        "gap-y-2",
        "justify-between",
        "space-y-2",
        "space-x-4",
        "p-4",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space and gap utilities with other utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let space_y_2_pos = sorted.iter().position(|&c| c == "space-y-2").unwrap();
    let space_x_4_pos = sorted.iter().position(|&c| c == "space-x-4").unwrap();
    let gap_x_0_pos = sorted.iter().position(|&c| c == "gap-x-0").unwrap();
    let gap_y_2_pos = sorted.iter().position(|&c| c == "gap-y-2").unwrap();

    // space-y should come BEFORE gap-x (cross-axis)
    assert!(space_y_2_pos < gap_x_0_pos, "space-y-2 should come before gap-x-0 even with other utilities");

    // space-x should come BEFORE gap-y (cross-axis)
    assert!(space_x_4_pos < gap_y_2_pos, "space-x-4 should come before gap-y-2 even with other utilities");
}

#[test]
fn test_space_gap_comprehensive_ordering() {
    // Comprehensive test with all combinations of space and gap utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "gap-0",
        "gap-x-0",
        "gap-x-2",
        "gap-y-0",
        "gap-y-4",
        "space-x-1",
        "space-x-4",
        "space-x-reverse",
        "space-y-0",
        "space-y-2",
        "space-y-reverse",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: comprehensive space and gap ordering");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let space_y_0_pos = sorted.iter().position(|&c| c == "space-y-0").unwrap();
    let space_y_2_pos = sorted.iter().position(|&c| c == "space-y-2").unwrap();
    let space_y_reverse_pos = sorted.iter().position(|&c| c == "space-y-reverse").unwrap();
    let space_x_1_pos = sorted.iter().position(|&c| c == "space-x-1").unwrap();
    let space_x_4_pos = sorted.iter().position(|&c| c == "space-x-4").unwrap();
    let space_x_reverse_pos = sorted.iter().position(|&c| c == "space-x-reverse").unwrap();
    let gap_x_0_pos = sorted.iter().position(|&c| c == "gap-x-0").unwrap();
    let gap_x_2_pos = sorted.iter().position(|&c| c == "gap-x-2").unwrap();
    let gap_y_0_pos = sorted.iter().position(|&c| c == "gap-y-0").unwrap();
    let gap_y_4_pos = sorted.iter().position(|&c| c == "gap-y-4").unwrap();

    // All space-y variants should come BEFORE gap-x variants (cross-axis)
    assert!(space_y_0_pos < gap_x_0_pos, "space-y-0 should come before gap-x-0");
    assert!(space_y_0_pos < gap_x_2_pos, "space-y-0 should come before gap-x-2");
    assert!(space_y_2_pos < gap_x_0_pos, "space-y-2 should come before gap-x-0");
    assert!(space_y_2_pos < gap_x_2_pos, "space-y-2 should come before gap-x-2");
    assert!(space_y_reverse_pos < gap_x_0_pos, "space-y-reverse should come before gap-x-0");
    assert!(space_y_reverse_pos < gap_x_2_pos, "space-y-reverse should come before gap-x-2");

    // All space-x variants should come BEFORE gap-y variants (cross-axis)
    assert!(space_x_1_pos < gap_y_0_pos, "space-x-1 should come before gap-y-0");
    assert!(space_x_1_pos < gap_y_4_pos, "space-x-1 should come before gap-y-4");
    assert!(space_x_4_pos < gap_y_0_pos, "space-x-4 should come before gap-y-0");
    assert!(space_x_4_pos < gap_y_4_pos, "space-x-4 should come before gap-y-4");
    assert!(space_x_reverse_pos < gap_y_0_pos, "space-x-reverse should come before gap-y-0");
    assert!(space_x_reverse_pos < gap_y_4_pos, "space-x-reverse should come before gap-y-4");
}

#[test]
fn test_space_gap_with_large_values() {
    // Test with larger spacing values to ensure the ordering holds
    let sorter = HybridSorter::new();

    let classes = vec![
        "gap-x-8",
        "gap-y-12",
        "space-x-8",
        "space-y-12",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space and gap with large values");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let space_y_12_pos = sorted.iter().position(|&c| c == "space-y-12").unwrap();
    let space_x_8_pos = sorted.iter().position(|&c| c == "space-x-8").unwrap();
    let gap_x_8_pos = sorted.iter().position(|&c| c == "gap-x-8").unwrap();
    let gap_y_12_pos = sorted.iter().position(|&c| c == "gap-y-12").unwrap();

    // space-y should come BEFORE gap-x
    assert!(space_y_12_pos < gap_x_8_pos, "space-y-12 should come before gap-x-8");

    // space-x should come BEFORE gap-y
    assert!(space_x_8_pos < gap_y_12_pos, "space-x-8 should come before gap-y-12");
}

#[test]
fn test_space_y_vs_gap_y_same_axis() {
    // Test same-axis comparison: space-y vs gap-y
    // From 100-run analysis: 4× space-y-2 vs gap-y-0, 3× space-y-4 vs gap-y-0
    let sorter = HybridSorter::new();

    let classes = vec![
        "gap-y-0",
        "space-y-2",
        "space-y-4",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space-y vs gap-y (same axis)");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let space_y_2_pos = sorted.iter().position(|&c| c == "space-y-2").unwrap();
    let space_y_4_pos = sorted.iter().position(|&c| c == "space-y-4").unwrap();
    let gap_y_0_pos = sorted.iter().position(|&c| c == "gap-y-0").unwrap();

    // space-y should come BEFORE gap-y (same axis)
    assert!(space_y_2_pos < gap_y_0_pos, "space-y-2 should come before gap-y-0");
    assert!(space_y_4_pos < gap_y_0_pos, "space-y-4 should come before gap-y-0");
}

#[test]
fn test_space_x_vs_gap_x_same_axis() {
    // Test same-axis comparison: space-x vs gap-x
    let sorter = HybridSorter::new();

    let classes = vec![
        "gap-x-0",
        "space-x-1",
        "space-x-2",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space-x vs gap-x (same axis)");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let space_x_1_pos = sorted.iter().position(|&c| c == "space-x-1").unwrap();
    let space_x_2_pos = sorted.iter().position(|&c| c == "space-x-2").unwrap();
    let gap_x_0_pos = sorted.iter().position(|&c| c == "gap-x-0").unwrap();

    // space-x should come BEFORE gap-x (same axis)
    assert!(space_x_1_pos < gap_x_0_pos, "space-x-1 should come before gap-x-0");
    assert!(space_x_2_pos < gap_x_0_pos, "space-x-2 should come before gap-x-0");
}

#[test]
fn test_space_x_vs_space_y_ordering() {
    // Test space-x vs space-y ordering
    // From 100-run analysis: 4× space-y-4 vs space-x-1, 4× space-y-0 vs space-x-0,
    // 3× space-y-1 vs space-x-4, 3× space-y-0 vs space-x-1
    let sorter = HybridSorter::new();

    let classes = vec![
        "space-y-4",
        "space-x-1",
        "space-y-0",
        "space-x-0",
        "space-y-1",
        "space-x-4",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space-x vs space-y ordering");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let space_x_0_pos = sorted.iter().position(|&c| c == "space-x-0").unwrap();
    let space_x_1_pos = sorted.iter().position(|&c| c == "space-x-1").unwrap();
    let space_x_4_pos = sorted.iter().position(|&c| c == "space-x-4").unwrap();
    let space_y_0_pos = sorted.iter().position(|&c| c == "space-y-0").unwrap();
    let space_y_1_pos = sorted.iter().position(|&c| c == "space-y-1").unwrap();
    let space_y_4_pos = sorted.iter().position(|&c| c == "space-y-4").unwrap();

    // space-x should come before space-y (alphabetically: x < y)
    assert!(space_x_0_pos < space_y_0_pos, "space-x-0 should come before space-y-0");
    assert!(space_x_1_pos < space_y_1_pos, "space-x-1 should come before space-y-1");
    assert!(space_x_4_pos < space_y_4_pos, "space-x-4 should come before space-y-4");
}

#[test]
fn test_specific_space_gap_failures_from_100run() {
    // This test covers all the specific failure cases from the 100-run analysis
    let sorter = HybridSorter::new();

    let classes = vec![
        // Cross-axis failures
        "space-x-1",
        "gap-y-2",
        "space-x-2",
        "gap-y-4",
        "space-x-4",
        "gap-y-0",
        "space-y-2",
        "gap-x-4",
        "space-y-1",
        "gap-x-0",
        // Same space utilities
        "space-y-4",
        "space-x-1",
        // Same-axis
        "gap-y-0",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: all specific space/gap failures from 100-run analysis");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Cross-axis ordering (most common failures)
    assert!(sorted.iter().position(|&c| c == "space-x-1").unwrap() < sorted.iter().position(|&c| c == "gap-y-2").unwrap());
    assert!(sorted.iter().position(|&c| c == "space-x-2").unwrap() < sorted.iter().position(|&c| c == "gap-y-4").unwrap());
    assert!(sorted.iter().position(|&c| c == "space-x-4").unwrap() < sorted.iter().position(|&c| c == "gap-y-0").unwrap());
    assert!(sorted.iter().position(|&c| c == "space-y-2").unwrap() < sorted.iter().position(|&c| c == "gap-x-4").unwrap());
    assert!(sorted.iter().position(|&c| c == "space-y-1").unwrap() < sorted.iter().position(|&c| c == "gap-x-0").unwrap());

    // Same-axis (space-y vs gap-y)
    let space_y_2_pos = sorted.iter().position(|&c| c == "space-y-2").unwrap();
    let space_y_4_pos = sorted.iter().position(|&c| c == "space-y-4").unwrap();
    let gap_y_0_first = sorted.iter().position(|&c| c == "gap-y-0").unwrap();

    assert!(space_y_2_pos < gap_y_0_first, "space-y-2 should come before gap-y-0");
    assert!(space_y_4_pos < gap_y_0_first, "space-y-4 should come before gap-y-0");
}

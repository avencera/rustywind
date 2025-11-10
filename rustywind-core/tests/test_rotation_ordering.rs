//! Tests for rotation utility ordering issues found in fuzz testing
//!
//! This test suite covers 10 failures related to rotation utilities being sorted incorrectly.
//! The main issue: rotation utilities with different numerical values are not being sorted
//! in the correct numerical order.
//!
//! From fuzz testing analysis:
//! - -rotate-45 should come BEFORE -rotate-90 (45 < 90)
//! - -rotate-1 should come BEFORE -rotate-180 (1 < 180)
//! - -rotate-1 should come BEFORE -rotate-90 (1 < 90)
//! - Expected order: smaller rotation → larger rotation (numerical ascending)
//! - RustyWind appears to be sorting them lexicographically instead of numerically

use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_rotate_1_vs_rotate_45() {
    // -rotate-1 should come BEFORE -rotate-45 (1 < 45)
    let sorter = HybridSorter::new();

    let classes = vec!["-rotate-45", "-rotate-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -rotate-1 vs -rotate-45");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -rotate-1, -rotate-45 (numerical order)
    assert_eq!(sorted[0], "-rotate-1", "-rotate-1 should come before -rotate-45");
    assert_eq!(sorted[1], "-rotate-45");
}

#[test]
fn test_rotate_45_vs_rotate_90() {
    // -rotate-45 should come BEFORE -rotate-90 (45 < 90)
    let sorter = HybridSorter::new();

    let classes = vec!["-rotate-90", "-rotate-45"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -rotate-45 vs -rotate-90");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -rotate-45, -rotate-90 (numerical order)
    assert_eq!(sorted[0], "-rotate-45", "-rotate-45 should come before -rotate-90");
    assert_eq!(sorted[1], "-rotate-90");
}

#[test]
fn test_rotate_1_vs_rotate_180() {
    // -rotate-1 should come BEFORE -rotate-180 (1 < 180)
    let sorter = HybridSorter::new();

    let classes = vec!["-rotate-180", "-rotate-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -rotate-1 vs -rotate-180");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -rotate-1, -rotate-180 (numerical order)
    assert_eq!(sorted[0], "-rotate-1", "-rotate-1 should come before -rotate-180");
    assert_eq!(sorted[1], "-rotate-180");
}

#[test]
fn test_rotate_1_vs_rotate_90() {
    // -rotate-1 should come BEFORE -rotate-90 (1 < 90)
    let sorter = HybridSorter::new();

    let classes = vec!["-rotate-90", "-rotate-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -rotate-1 vs -rotate-90");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -rotate-1, -rotate-90 (numerical order)
    assert_eq!(sorted[0], "-rotate-1", "-rotate-1 should come before -rotate-90");
    assert_eq!(sorted[1], "-rotate-90");
}

#[test]
fn test_multiple_rotation_values_together() {
    // Test multiple rotation utilities sorted in numerical ascending order
    let sorter = HybridSorter::new();

    let classes = vec![
        "-rotate-180",
        "-rotate-45",
        "-rotate-1",
        "-rotate-90",
        "-rotate-12",
        "-rotate-6",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple rotation values together");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expected order: numerical ascending (1, 6, 12, 45, 90, 180)
    let expected = vec![
        "-rotate-1",
        "-rotate-6",
        "-rotate-12",
        "-rotate-45",
        "-rotate-90",
        "-rotate-180",
    ];

    assert_eq!(sorted, expected, "Rotation values should be sorted in numerical ascending order");
}

#[test]
fn test_positive_rotation_values() {
    // Test positive rotation values (without minus prefix)
    let sorter = HybridSorter::new();

    let classes = vec![
        "rotate-180",
        "rotate-45",
        "rotate-1",
        "rotate-90",
        "rotate-12",
        "rotate-6",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: positive rotation values");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expected order: numerical ascending (1, 6, 12, 45, 90, 180)
    let expected = vec![
        "rotate-1",
        "rotate-6",
        "rotate-12",
        "rotate-45",
        "rotate-90",
        "rotate-180",
    ];

    assert_eq!(sorted, expected, "Positive rotation values should be sorted in numerical ascending order");
}

#[test]
fn test_mixed_positive_negative_rotation() {
    // Test mixed positive and negative rotation values
    let sorter = HybridSorter::new();

    let classes = vec![
        "rotate-45",
        "-rotate-45",
        "rotate-90",
        "-rotate-90",
        "rotate-1",
        "-rotate-1",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: mixed positive and negative rotation");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let rotate_1_pos = sorted.iter().position(|&c| c == "rotate-1").unwrap();
    let rotate_45_pos = sorted.iter().position(|&c| c == "rotate-45").unwrap();
    let rotate_90_pos = sorted.iter().position(|&c| c == "rotate-90").unwrap();
    let neg_rotate_1_pos = sorted.iter().position(|&c| c == "-rotate-1").unwrap();
    let neg_rotate_45_pos = sorted.iter().position(|&c| c == "-rotate-45").unwrap();
    let neg_rotate_90_pos = sorted.iter().position(|&c| c == "-rotate-90").unwrap();

    // Within positive rotations, numerical order should apply
    assert!(rotate_1_pos < rotate_45_pos, "rotate-1 should come before rotate-45");
    assert!(rotate_45_pos < rotate_90_pos, "rotate-45 should come before rotate-90");

    // Within negative rotations, numerical order should apply
    assert!(neg_rotate_1_pos < neg_rotate_45_pos, "-rotate-1 should come before -rotate-45");
    assert!(neg_rotate_45_pos < neg_rotate_90_pos, "-rotate-45 should come before -rotate-90");
}

#[test]
fn test_rotation_with_other_transform_utilities() {
    // Test rotation utilities mixed with other transform utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "-rotate-90",
        "scale-100",
        "-rotate-1",
        "translate-x-4",
        "-rotate-45",
        "skew-x-12",
        "scale-50",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: rotation with other transform utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find rotation positions
    let rotate_1_pos = sorted.iter().position(|&c| c == "-rotate-1").unwrap();
    let rotate_45_pos = sorted.iter().position(|&c| c == "-rotate-45").unwrap();
    let rotate_90_pos = sorted.iter().position(|&c| c == "-rotate-90").unwrap();

    // Rotation utilities should maintain numerical order among themselves
    assert!(rotate_1_pos < rotate_45_pos, "-rotate-1 should come before -rotate-45");
    assert!(rotate_45_pos < rotate_90_pos, "-rotate-45 should come before -rotate-90");
}

#[test]
fn test_rotation_edge_cases() {
    // Test edge cases like rotate-0, rotate-3, etc.
    let sorter = HybridSorter::new();

    let classes = vec![
        "rotate-180",
        "rotate-3",
        "rotate-0",
        "rotate-12",
        "rotate-6",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: rotation edge cases");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let rotate_0_pos = sorted.iter().position(|&c| c == "rotate-0").unwrap();
    let rotate_3_pos = sorted.iter().position(|&c| c == "rotate-3").unwrap();
    let rotate_6_pos = sorted.iter().position(|&c| c == "rotate-6").unwrap();
    let rotate_12_pos = sorted.iter().position(|&c| c == "rotate-12").unwrap();
    let rotate_180_pos = sorted.iter().position(|&c| c == "rotate-180").unwrap();

    // Numerical order: 0 < 3 < 6 < 12 < 180
    assert!(rotate_0_pos < rotate_3_pos, "rotate-0 should come before rotate-3");
    assert!(rotate_3_pos < rotate_6_pos, "rotate-3 should come before rotate-6");
    assert!(rotate_6_pos < rotate_12_pos, "rotate-6 should come before rotate-12");
    assert!(rotate_12_pos < rotate_180_pos, "rotate-12 should come before rotate-180");
}

#[test]
fn test_rotation_comprehensive() {
    // Comprehensive test combining all rotation patterns
    let sorter = HybridSorter::new();

    let classes = vec![
        "-rotate-180",
        "rotate-90",
        "-rotate-1",
        "rotate-45",
        "-rotate-90",
        "rotate-1",
        "-rotate-45",
        "rotate-180",
        "rotate-6",
        "-rotate-6",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: comprehensive rotation ordering");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Extract positive and negative rotations
    let positive_rotations: Vec<_> = sorted.iter()
        .filter(|c| c.starts_with("rotate-") && !c.starts_with("-rotate-"))
        .collect();
    let negative_rotations: Vec<_> = sorted.iter()
        .filter(|c| c.starts_with("-rotate-"))
        .collect();

    // Check that positive rotations are in numerical order
    if positive_rotations.len() >= 2 {
        for i in 0..positive_rotations.len() - 1 {
            let curr_val = positive_rotations[i].strip_prefix("rotate-").unwrap().parse::<i32>().unwrap();
            let next_val = positive_rotations[i + 1].strip_prefix("rotate-").unwrap().parse::<i32>().unwrap();
            assert!(curr_val <= next_val, "Positive rotations should be in numerical order: {} should come before or equal to {}", positive_rotations[i], positive_rotations[i + 1]);
        }
    }

    // Check that negative rotations are in numerical order
    if negative_rotations.len() >= 2 {
        for i in 0..negative_rotations.len() - 1 {
            let curr_val = negative_rotations[i].strip_prefix("-rotate-").unwrap().parse::<i32>().unwrap();
            let next_val = negative_rotations[i + 1].strip_prefix("-rotate-").unwrap().parse::<i32>().unwrap();
            assert!(curr_val <= next_val, "Negative rotations should be in numerical order: {} should come before or equal to {}", negative_rotations[i], negative_rotations[i + 1]);
        }
    }
}

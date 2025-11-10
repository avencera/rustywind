//! Tests for transform utility ordering issues found in fuzz testing
//!
//! This test suite covers failures related to transform utilities (skew and translate)
//! being sorted incorrectly. The main issue: negative transform utilities with different
//! numerical values are not being sorted in the correct numerical order.
//!
//! From fuzz testing analysis:
//! - -skew-x-1 should come BEFORE -skew-x-3 (1 < 3)
//! - -translate-x-1 should come BEFORE -translate-x-2 (1 < 2)
//! - Expected order: smaller value → larger value (numerical ascending)
//! - RustyWind appears to be sorting them in reverse order (descending)
//!
//! This is similar to the rotation bug: negative values are sorted in reverse order
//! instead of ascending numerical order.

use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_skew_x_1_vs_skew_x_3() {
    // -skew-x-1 should come BEFORE -skew-x-3 (1 < 3)
    let sorter = HybridSorter::new();

    let classes = vec!["-skew-x-3", "-skew-x-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -skew-x-1 vs -skew-x-3");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -skew-x-1, -skew-x-3 (numerical order)
    assert_eq!(sorted[0], "-skew-x-1", "-skew-x-1 should come before -skew-x-3");
    assert_eq!(sorted[1], "-skew-x-3");
}

#[test]
fn test_skew_x_1_vs_skew_x_6() {
    // -skew-x-1 should come BEFORE -skew-x-6 (1 < 6)
    let sorter = HybridSorter::new();

    let classes = vec!["-skew-x-6", "-skew-x-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -skew-x-1 vs -skew-x-6");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -skew-x-1, -skew-x-6 (numerical order)
    assert_eq!(sorted[0], "-skew-x-1", "-skew-x-1 should come before -skew-x-6");
    assert_eq!(sorted[1], "-skew-x-6");
}

#[test]
fn test_skew_x_1_vs_skew_x_12() {
    // -skew-x-1 should come BEFORE -skew-x-12 (1 < 12)
    let sorter = HybridSorter::new();

    let classes = vec!["-skew-x-12", "-skew-x-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -skew-x-1 vs -skew-x-12");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -skew-x-1, -skew-x-12 (numerical order)
    assert_eq!(sorted[0], "-skew-x-1", "-skew-x-1 should come before -skew-x-12");
    assert_eq!(sorted[1], "-skew-x-12");
}

#[test]
fn test_skew_y_1_vs_skew_y_3() {
    // -skew-y-1 should come BEFORE -skew-y-3 (1 < 3)
    let sorter = HybridSorter::new();

    let classes = vec!["-skew-y-3", "-skew-y-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -skew-y-1 vs -skew-y-3");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -skew-y-1, -skew-y-3 (numerical order)
    assert_eq!(sorted[0], "-skew-y-1", "-skew-y-1 should come before -skew-y-3");
    assert_eq!(sorted[1], "-skew-y-3");
}

#[test]
fn test_skew_y_1_vs_skew_y_6() {
    // -skew-y-1 should come BEFORE -skew-y-6 (1 < 6)
    let sorter = HybridSorter::new();

    let classes = vec!["-skew-y-6", "-skew-y-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -skew-y-1 vs -skew-y-6");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -skew-y-1, -skew-y-6 (numerical order)
    assert_eq!(sorted[0], "-skew-y-1", "-skew-y-1 should come before -skew-y-6");
    assert_eq!(sorted[1], "-skew-y-6");
}

#[test]
fn test_skew_y_1_vs_skew_y_12() {
    // -skew-y-1 should come BEFORE -skew-y-12 (1 < 12)
    let sorter = HybridSorter::new();

    let classes = vec!["-skew-y-12", "-skew-y-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -skew-y-1 vs -skew-y-12");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -skew-y-1, -skew-y-12 (numerical order)
    assert_eq!(sorted[0], "-skew-y-1", "-skew-y-1 should come before -skew-y-12");
    assert_eq!(sorted[1], "-skew-y-12");
}

#[test]
fn test_translate_x_1_vs_translate_x_2() {
    // -translate-x-1 should come BEFORE -translate-x-2 (1 < 2)
    let sorter = HybridSorter::new();

    let classes = vec!["-translate-x-2", "-translate-x-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -translate-x-1 vs -translate-x-2");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -translate-x-1, -translate-x-2 (numerical order)
    assert_eq!(sorted[0], "-translate-x-1", "-translate-x-1 should come before -translate-x-2");
    assert_eq!(sorted[1], "-translate-x-2");
}

#[test]
fn test_translate_x_1_vs_translate_x_4() {
    // -translate-x-1 should come BEFORE -translate-x-4 (1 < 4)
    let sorter = HybridSorter::new();

    let classes = vec!["-translate-x-4", "-translate-x-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -translate-x-1 vs -translate-x-4");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -translate-x-1, -translate-x-4 (numerical order)
    assert_eq!(sorted[0], "-translate-x-1", "-translate-x-1 should come before -translate-x-4");
    assert_eq!(sorted[1], "-translate-x-4");
}

#[test]
fn test_translate_y_1_vs_translate_y_2() {
    // -translate-y-1 should come BEFORE -translate-y-2 (1 < 2)
    let sorter = HybridSorter::new();

    let classes = vec!["-translate-y-2", "-translate-y-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -translate-y-1 vs -translate-y-2");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -translate-y-1, -translate-y-2 (numerical order)
    assert_eq!(sorted[0], "-translate-y-1", "-translate-y-1 should come before -translate-y-2");
    assert_eq!(sorted[1], "-translate-y-2");
}

#[test]
fn test_translate_y_1_vs_translate_y_4() {
    // -translate-y-1 should come BEFORE -translate-y-4 (1 < 4)
    let sorter = HybridSorter::new();

    let classes = vec!["-translate-y-4", "-translate-y-1"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: -translate-y-1 vs -translate-y-4");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: -translate-y-1, -translate-y-4 (numerical order)
    assert_eq!(sorted[0], "-translate-y-1", "-translate-y-1 should come before -translate-y-4");
    assert_eq!(sorted[1], "-translate-y-4");
}

#[test]
fn test_multiple_skew_x_values() {
    // Test multiple skew-x utilities sorted in numerical ascending order
    let sorter = HybridSorter::new();

    let classes = vec![
        "-skew-x-12",
        "-skew-x-3",
        "-skew-x-1",
        "-skew-x-6",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple skew-x values");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expected order: numerical ascending (1, 3, 6, 12)
    let expected = vec![
        "-skew-x-1",
        "-skew-x-3",
        "-skew-x-6",
        "-skew-x-12",
    ];

    assert_eq!(sorted, expected, "Skew-x values should be sorted in numerical ascending order");
}

#[test]
fn test_multiple_skew_y_values() {
    // Test multiple skew-y utilities sorted in numerical ascending order
    let sorter = HybridSorter::new();

    let classes = vec![
        "-skew-y-12",
        "-skew-y-3",
        "-skew-y-1",
        "-skew-y-6",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple skew-y values");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expected order: numerical ascending (1, 3, 6, 12)
    let expected = vec![
        "-skew-y-1",
        "-skew-y-3",
        "-skew-y-6",
        "-skew-y-12",
    ];

    assert_eq!(sorted, expected, "Skew-y values should be sorted in numerical ascending order");
}

#[test]
fn test_multiple_translate_x_values() {
    // Test multiple translate-x utilities sorted in numerical ascending order
    let sorter = HybridSorter::new();

    let classes = vec![
        "-translate-x-4",
        "-translate-x-1",
        "-translate-x-2",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple translate-x values");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expected order: numerical ascending (1, 2, 4)
    let expected = vec![
        "-translate-x-1",
        "-translate-x-2",
        "-translate-x-4",
    ];

    assert_eq!(sorted, expected, "Translate-x values should be sorted in numerical ascending order");
}

#[test]
fn test_multiple_translate_y_values() {
    // Test multiple translate-y utilities sorted in numerical ascending order
    let sorter = HybridSorter::new();

    let classes = vec![
        "-translate-y-4",
        "-translate-y-1",
        "-translate-y-2",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple translate-y values");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expected order: numerical ascending (1, 2, 4)
    let expected = vec![
        "-translate-y-1",
        "-translate-y-2",
        "-translate-y-4",
    ];

    assert_eq!(sorted, expected, "Translate-y values should be sorted in numerical ascending order");
}

#[test]
fn test_mixed_transform_values() {
    // Test mixed skew and translate utilities sorted together
    let sorter = HybridSorter::new();

    let classes = vec![
        "-skew-y-6",
        "-translate-x-2",
        "-skew-x-3",
        "-translate-y-4",
        "-skew-x-1",
        "-translate-x-1",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: mixed transform values");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions of skew-x values
    let skew_x_1_pos = sorted.iter().position(|&c| c == "-skew-x-1").unwrap();
    let skew_x_3_pos = sorted.iter().position(|&c| c == "-skew-x-3").unwrap();

    // Find positions of translate-x values
    let translate_x_1_pos = sorted.iter().position(|&c| c == "-translate-x-1").unwrap();
    let translate_x_2_pos = sorted.iter().position(|&c| c == "-translate-x-2").unwrap();

    // Skew-x utilities should maintain numerical order among themselves
    assert!(skew_x_1_pos < skew_x_3_pos, "-skew-x-1 should come before -skew-x-3");

    // Translate-x utilities should maintain numerical order among themselves
    assert!(translate_x_1_pos < translate_x_2_pos, "-translate-x-1 should come before -translate-x-2");
}

#[test]
fn test_transform_values_with_other_utilities() {
    // Test transform utilities mixed with other utilities (not just transforms)
    let sorter = HybridSorter::new();

    let classes = vec![
        "bg-blue-500",
        "-skew-x-3",
        "p-4",
        "-translate-x-2",
        "-skew-x-1",
        "rounded-lg",
        "-translate-x-1",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: transform values with other utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions of skew-x values
    let skew_x_1_pos = sorted.iter().position(|&c| c == "-skew-x-1").unwrap();
    let skew_x_3_pos = sorted.iter().position(|&c| c == "-skew-x-3").unwrap();

    // Find positions of translate-x values
    let translate_x_1_pos = sorted.iter().position(|&c| c == "-translate-x-1").unwrap();
    let translate_x_2_pos = sorted.iter().position(|&c| c == "-translate-x-2").unwrap();

    // Skew-x utilities should maintain numerical order among themselves
    assert!(skew_x_1_pos < skew_x_3_pos, "-skew-x-1 should come before -skew-x-3");

    // Translate-x utilities should maintain numerical order among themselves
    assert!(translate_x_1_pos < translate_x_2_pos, "-translate-x-1 should come before -translate-x-2");
}

#[test]
fn test_positive_transform_values() {
    // Test positive transform values (without minus prefix)
    let sorter = HybridSorter::new();

    let classes = vec![
        "skew-x-12",
        "translate-x-4",
        "skew-x-3",
        "translate-x-1",
        "skew-x-1",
        "translate-x-2",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: positive transform values");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions of skew-x values
    let skew_x_1_pos = sorted.iter().position(|&c| c == "skew-x-1").unwrap();
    let skew_x_3_pos = sorted.iter().position(|&c| c == "skew-x-3").unwrap();
    let skew_x_12_pos = sorted.iter().position(|&c| c == "skew-x-12").unwrap();

    // Find positions of translate-x values
    let translate_x_1_pos = sorted.iter().position(|&c| c == "translate-x-1").unwrap();
    let translate_x_2_pos = sorted.iter().position(|&c| c == "translate-x-2").unwrap();
    let translate_x_4_pos = sorted.iter().position(|&c| c == "translate-x-4").unwrap();

    // Skew-x utilities should maintain numerical order: 1 < 3 < 12
    assert!(skew_x_1_pos < skew_x_3_pos, "skew-x-1 should come before skew-x-3");
    assert!(skew_x_3_pos < skew_x_12_pos, "skew-x-3 should come before skew-x-12");

    // Translate-x utilities should maintain numerical order: 1 < 2 < 4
    assert!(translate_x_1_pos < translate_x_2_pos, "translate-x-1 should come before translate-x-2");
    assert!(translate_x_2_pos < translate_x_4_pos, "translate-x-2 should come before translate-x-4");
}

#[test]
fn test_mixed_positive_negative_transform_values() {
    // Test mixed positive and negative transform values
    let sorter = HybridSorter::new();

    let classes = vec![
        "skew-x-3",
        "-skew-x-3",
        "skew-x-1",
        "-skew-x-1",
        "translate-x-2",
        "-translate-x-2",
        "translate-x-1",
        "-translate-x-1",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: mixed positive and negative transform values");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions of positive skew-x
    let skew_x_1_pos = sorted.iter().position(|&c| c == "skew-x-1").unwrap();
    let skew_x_3_pos = sorted.iter().position(|&c| c == "skew-x-3").unwrap();

    // Find positions of negative skew-x
    let neg_skew_x_1_pos = sorted.iter().position(|&c| c == "-skew-x-1").unwrap();
    let neg_skew_x_3_pos = sorted.iter().position(|&c| c == "-skew-x-3").unwrap();

    // Find positions of positive translate-x
    let translate_x_1_pos = sorted.iter().position(|&c| c == "translate-x-1").unwrap();
    let translate_x_2_pos = sorted.iter().position(|&c| c == "translate-x-2").unwrap();

    // Find positions of negative translate-x
    let neg_translate_x_1_pos = sorted.iter().position(|&c| c == "-translate-x-1").unwrap();
    let neg_translate_x_2_pos = sorted.iter().position(|&c| c == "-translate-x-2").unwrap();

    // Within positive skew-x, numerical order should apply
    assert!(skew_x_1_pos < skew_x_3_pos, "skew-x-1 should come before skew-x-3");

    // Within negative skew-x, numerical order should apply
    assert!(neg_skew_x_1_pos < neg_skew_x_3_pos, "-skew-x-1 should come before -skew-x-3");

    // Within positive translate-x, numerical order should apply
    assert!(translate_x_1_pos < translate_x_2_pos, "translate-x-1 should come before translate-x-2");

    // Within negative translate-x, numerical order should apply
    assert!(neg_translate_x_1_pos < neg_translate_x_2_pos, "-translate-x-1 should come before -translate-x-2");
}

#[test]
fn test_comprehensive_transform_ordering() {
    // Comprehensive test combining all transform patterns from fuzz failures
    let sorter = HybridSorter::new();

    let classes = vec![
        "-skew-x-12",
        "skew-x-6",
        "-translate-x-4",
        "translate-x-1",
        "-skew-x-1",
        "skew-x-3",
        "-translate-x-1",
        "translate-x-2",
        "-skew-y-6",
        "skew-y-3",
        "-translate-y-2",
        "translate-y-1",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: comprehensive transform ordering");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Extract and verify skew-x ordering
    let skew_x_classes: Vec<_> = sorted.iter()
        .filter(|c| c.contains("skew-x") && !c.starts_with("-"))
        .collect();
    let neg_skew_x_classes: Vec<_> = sorted.iter()
        .filter(|c| c.starts_with("-skew-x"))
        .collect();

    // Verify positive skew-x numerical ordering
    if skew_x_classes.len() >= 2 {
        for i in 0..skew_x_classes.len() - 1 {
            let curr_val = skew_x_classes[i].strip_prefix("skew-x-").unwrap().parse::<i32>().unwrap();
            let next_val = skew_x_classes[i + 1].strip_prefix("skew-x-").unwrap().parse::<i32>().unwrap();
            assert!(curr_val <= next_val, "Positive skew-x should be in numerical order: {} <= {}", curr_val, next_val);
        }
    }

    // Verify negative skew-x numerical ordering
    if neg_skew_x_classes.len() >= 2 {
        for i in 0..neg_skew_x_classes.len() - 1 {
            let curr_val = neg_skew_x_classes[i].strip_prefix("-skew-x-").unwrap().parse::<i32>().unwrap();
            let next_val = neg_skew_x_classes[i + 1].strip_prefix("-skew-x-").unwrap().parse::<i32>().unwrap();
            assert!(curr_val <= next_val, "Negative skew-x should be in numerical order: {} <= {}", curr_val, next_val);
        }
    }

    // Extract and verify translate-x ordering
    let translate_x_classes: Vec<_> = sorted.iter()
        .filter(|c| c.contains("translate-x") && !c.starts_with("-translate-x"))
        .collect();
    let neg_translate_x_classes: Vec<_> = sorted.iter()
        .filter(|c| c.starts_with("-translate-x"))
        .collect();

    // Verify positive translate-x numerical ordering
    if translate_x_classes.len() >= 2 {
        for i in 0..translate_x_classes.len() - 1 {
            let curr_val = translate_x_classes[i].strip_prefix("translate-x-").unwrap().parse::<i32>().unwrap();
            let next_val = translate_x_classes[i + 1].strip_prefix("translate-x-").unwrap().parse::<i32>().unwrap();
            assert!(curr_val <= next_val, "Positive translate-x should be in numerical order: {} <= {}", curr_val, next_val);
        }
    }

    // Verify negative translate-x numerical ordering
    if neg_translate_x_classes.len() >= 2 {
        for i in 0..neg_translate_x_classes.len() - 1 {
            let curr_val = neg_translate_x_classes[i].strip_prefix("-translate-x-").unwrap().parse::<i32>().unwrap();
            let next_val = neg_translate_x_classes[i + 1].strip_prefix("-translate-x-").unwrap().parse::<i32>().unwrap();
            assert!(curr_val <= next_val, "Negative translate-x should be in numerical order: {} <= {}", curr_val, next_val);
        }
    }
}

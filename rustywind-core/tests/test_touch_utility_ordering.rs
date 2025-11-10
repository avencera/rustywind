//! Tests for touch utility ordering issues found in fuzz testing
//!
//! This test suite covers 4 failures related to touch utilities not being sorted
//! in the correct alphabetical order.
//!
//! From fuzz testing analysis:
//! - Prettier sorts touch utilities alphabetically
//! - RustyWind was using a different ordering
//! - Expected alphabetical order:
//!   touch-auto < touch-manipulation < touch-none < touch-pan-down < touch-pan-left
//!   < touch-pan-right < touch-pan-up < touch-pan-x < touch-pan-y < touch-pinch-zoom

use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_touch_manipulation_vs_touch_pan_left() {
    // touch-manipulation should come BEFORE touch-pan-left (alphabetically: m < p)
    let sorter = HybridSorter::new();

    let classes = vec!["touch-pan-left", "touch-manipulation"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: touch-manipulation vs touch-pan-left");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: touch-manipulation, touch-pan-left (alphabetical order)
    assert_eq!(
        sorted[0], "touch-manipulation",
        "touch-manipulation should come before touch-pan-left"
    );
    assert_eq!(sorted[1], "touch-pan-left");
}

#[test]
fn test_touch_pan_up_vs_touch_pan_x() {
    // touch-pan-up should come BEFORE touch-pan-x (alphabetically: u < x)
    let sorter = HybridSorter::new();

    let classes = vec!["touch-pan-x", "touch-pan-up"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: touch-pan-up vs touch-pan-x");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: touch-pan-up, touch-pan-x (alphabetical order)
    assert_eq!(
        sorted[0], "touch-pan-up",
        "touch-pan-up should come before touch-pan-x"
    );
    assert_eq!(sorted[1], "touch-pan-x");
}

#[test]
fn test_touch_none_vs_touch_pan_down() {
    // touch-none should come BEFORE touch-pan-down (alphabetically: n < p)
    let sorter = HybridSorter::new();

    let classes = vec!["touch-pan-down", "touch-none"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: touch-none vs touch-pan-down");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: touch-none, touch-pan-down (alphabetical order)
    assert_eq!(
        sorted[0], "touch-none",
        "touch-none should come before touch-pan-down"
    );
    assert_eq!(sorted[1], "touch-pan-down");
}

#[test]
fn test_touch_auto_vs_touch_manipulation() {
    // touch-auto should come BEFORE touch-manipulation (alphabetically: a < m)
    let sorter = HybridSorter::new();

    let classes = vec!["touch-manipulation", "touch-auto"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: touch-auto vs touch-manipulation");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: touch-auto, touch-manipulation (alphabetical order)
    assert_eq!(
        sorted[0], "touch-auto",
        "touch-auto should come before touch-manipulation"
    );
    assert_eq!(sorted[1], "touch-manipulation");
}

#[test]
fn test_multiple_touch_pan_utilities() {
    // Test multiple touch-pan-* utilities sorted alphabetically
    let sorter = HybridSorter::new();

    let classes = vec![
        "touch-pan-x",
        "touch-pan-left",
        "touch-pan-up",
        "touch-pan-down",
        "touch-pan-right",
        "touch-pan-y",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple touch-pan-* utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expected order: alphabetical (down < left < right < up < x < y)
    let expected = vec![
        "touch-pan-down",
        "touch-pan-left",
        "touch-pan-right",
        "touch-pan-up",
        "touch-pan-x",
        "touch-pan-y",
    ];

    assert_eq!(
        sorted, expected,
        "touch-pan-* utilities should be sorted alphabetically"
    );
}

#[test]
fn test_all_touch_utilities_alphabetically() {
    // Test all touch utilities sorted in complete alphabetical order
    let sorter = HybridSorter::new();

    let classes = vec![
        "touch-pinch-zoom",
        "touch-pan-x",
        "touch-manipulation",
        "touch-auto",
        "touch-pan-up",
        "touch-none",
        "touch-pan-left",
        "touch-pan-down",
        "touch-pan-right",
        "touch-pan-y",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: all touch utilities alphabetically");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expected order: complete alphabetical
    let expected = vec![
        "touch-auto",
        "touch-manipulation",
        "touch-none",
        "touch-pan-down",
        "touch-pan-left",
        "touch-pan-right",
        "touch-pan-up",
        "touch-pan-x",
        "touch-pan-y",
        "touch-pinch-zoom",
    ];

    assert_eq!(
        sorted, expected,
        "All touch utilities should be sorted in alphabetical order"
    );
}

#[test]
fn test_touch_utilities_mixed_with_other_utilities() {
    // Test touch utilities mixed with other pointer and user interaction utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "touch-pan-x",
        "pointer-events-none",
        "touch-manipulation",
        "cursor-pointer",
        "touch-pan-up",
        "select-none",
        "touch-auto",
        "user-select-none",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: touch utilities mixed with other utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find touch utility positions
    let touch_auto_pos = sorted.iter().position(|&c| c == "touch-auto").unwrap();
    let touch_manipulation_pos = sorted
        .iter()
        .position(|&c| c == "touch-manipulation")
        .unwrap();
    let touch_pan_up_pos = sorted.iter().position(|&c| c == "touch-pan-up").unwrap();
    let touch_pan_x_pos = sorted.iter().position(|&c| c == "touch-pan-x").unwrap();

    // Touch utilities should maintain alphabetical order among themselves
    assert!(
        touch_auto_pos < touch_manipulation_pos,
        "touch-auto should come before touch-manipulation"
    );
    assert!(
        touch_manipulation_pos < touch_pan_up_pos,
        "touch-manipulation should come before touch-pan-up"
    );
    assert!(
        touch_pan_up_pos < touch_pan_x_pos,
        "touch-pan-up should come before touch-pan-x"
    );
}

#[test]
fn test_touch_pan_left_vs_touch_pan_right() {
    // touch-pan-left should come BEFORE touch-pan-right (alphabetically: l < r)
    let sorter = HybridSorter::new();

    let classes = vec!["touch-pan-right", "touch-pan-left"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: touch-pan-left vs touch-pan-right");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: touch-pan-left, touch-pan-right (alphabetical order)
    assert_eq!(
        sorted[0], "touch-pan-left",
        "touch-pan-left should come before touch-pan-right"
    );
    assert_eq!(sorted[1], "touch-pan-right");
}

#[test]
fn test_touch_pan_y_vs_touch_pinch_zoom() {
    // touch-pan-y should come BEFORE touch-pinch-zoom (alphabetically: pan < pinch)
    let sorter = HybridSorter::new();

    let classes = vec!["touch-pinch-zoom", "touch-pan-y"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: touch-pan-y vs touch-pinch-zoom");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: touch-pan-y, touch-pinch-zoom (alphabetical order)
    assert_eq!(
        sorted[0], "touch-pan-y",
        "touch-pan-y should come before touch-pinch-zoom"
    );
    assert_eq!(sorted[1], "touch-pinch-zoom");
}

#[test]
fn test_touch_utilities_comprehensive() {
    // Comprehensive test combining all touch patterns with various orderings
    let sorter = HybridSorter::new();

    let classes = vec![
        "touch-pinch-zoom",
        "touch-auto",
        "touch-pan-right",
        "touch-manipulation",
        "touch-pan-x",
        "touch-none",
        "touch-pan-down",
        "touch-pan-y",
        "touch-pan-left",
        "touch-pan-up",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: comprehensive touch utilities ordering");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Verify the complete alphabetical order
    let expected = vec![
        "touch-auto",
        "touch-manipulation",
        "touch-none",
        "touch-pan-down",
        "touch-pan-left",
        "touch-pan-right",
        "touch-pan-up",
        "touch-pan-x",
        "touch-pan-y",
        "touch-pinch-zoom",
    ];

    assert_eq!(
        sorted, expected,
        "All touch utilities should maintain strict alphabetical order"
    );

    // Additional verification: check each consecutive pair
    for i in 0..sorted.len() - 1 {
        assert!(
            sorted[i] < sorted[i + 1],
            "Each touch utility should come before the next alphabetically: {} should be < {}",
            sorted[i],
            sorted[i + 1]
        );
    }
}

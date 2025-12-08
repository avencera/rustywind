//! Tests for touch utility ordering issues found in fuzz testing
//!
//! This test suite covers touch utility ordering based on CSS property grouping.
//!
//! Per Prettier, touch utilities are grouped by behavior:
//! 1. Horizontal pan: touch-pan-left, touch-pan-right, touch-pan-x
//! 2. Vertical pan: touch-pan-down, touch-pan-up, touch-pan-y
//! 3. Pinch zoom: touch-pinch-zoom
//! 4. General touch-action (alphabetical): touch-auto, touch-manipulation, touch-none

use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_touch_manipulation_vs_touch_pan_left() {
    // touch-pan-left (horizontal pan) should come BEFORE touch-manipulation (general)
    let sorter = HybridSorter::new();

    let classes = vec!["touch-pan-left", "touch-manipulation"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: touch-manipulation vs touch-pan-left");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Per Prettier: pan values come before general touch-action values
    assert_eq!(
        sorted[0], "touch-pan-left",
        "touch-pan-left (horizontal pan) should come before touch-manipulation (general)"
    );
    assert_eq!(sorted[1], "touch-manipulation");
}

#[test]
fn test_touch_pan_up_vs_touch_pan_x() {
    // touch-pan-x (horizontal) should come BEFORE touch-pan-up (vertical)
    let sorter = HybridSorter::new();

    let classes = vec!["touch-pan-x", "touch-pan-up"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: touch-pan-up vs touch-pan-x");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Per Prettier: horizontal pan comes before vertical pan
    assert_eq!(
        sorted[0], "touch-pan-x",
        "touch-pan-x (horizontal) should come before touch-pan-up (vertical)"
    );
    assert_eq!(sorted[1], "touch-pan-up");
}

#[test]
fn test_touch_none_vs_touch_pan_down() {
    // touch-pan-down (vertical pan) should come BEFORE touch-none (general)
    let sorter = HybridSorter::new();

    let classes = vec!["touch-pan-down", "touch-none"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: touch-none vs touch-pan-down");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Per Prettier: pan values come before general touch-action values
    assert_eq!(
        sorted[0], "touch-pan-down",
        "touch-pan-down (vertical pan) should come before touch-none (general)"
    );
    assert_eq!(sorted[1], "touch-none");
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

    // Per Prettier: grouped by direction (horizontal, then vertical)
    let expected = vec![
        "touch-pan-left",
        "touch-pan-right",
        "touch-pan-x",
        "touch-pan-down",
        "touch-pan-up",
        "touch-pan-y",
    ];

    assert_eq!(
        sorted, expected,
        "touch-pan-* utilities should be grouped by direction: horizontal (left, right, x), then vertical (down, up, y)"
    );
}

#[test]
fn test_all_touch_utilities_alphabetically() {
    // Test all touch utilities grouped by behavior
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

    // Per Prettier: grouped by behavior (horizontal pan, vertical pan, pinch, general)
    let expected = vec![
        "touch-pan-left",
        "touch-pan-right",
        "touch-pan-x",
        "touch-pan-down",
        "touch-pan-up",
        "touch-pan-y",
        "touch-pinch-zoom",
        "touch-auto",
        "touch-manipulation",
        "touch-none",
    ];

    assert_eq!(
        sorted, expected,
        "Touch utilities should be grouped: horizontal pan, vertical pan, pinch-zoom, then general (alphabetical)"
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

    // Per Prettier: pan utilities come before general touch-action values
    // Within pan utilities, horizontal comes before vertical
    assert!(
        touch_pan_x_pos < touch_pan_up_pos,
        "touch-pan-x (horizontal) should come before touch-pan-up (vertical)"
    );
    assert!(
        touch_pan_up_pos < touch_auto_pos,
        "touch-pan-up (pan) should come before touch-auto (general)"
    );
    assert!(
        touch_auto_pos < touch_manipulation_pos,
        "touch-auto should come before touch-manipulation (alphabetical within general values)"
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

    // Per Prettier: grouped by behavior (horizontal pan, vertical pan, pinch, general)
    let expected = vec![
        "touch-pan-left",
        "touch-pan-right",
        "touch-pan-x",
        "touch-pan-down",
        "touch-pan-up",
        "touch-pan-y",
        "touch-pinch-zoom",
        "touch-auto",
        "touch-manipulation",
        "touch-none",
    ];

    assert_eq!(
        sorted, expected,
        "Touch utilities should be grouped by behavior: horizontal pan, vertical pan, pinch-zoom, then general"
    );
}

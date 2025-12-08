//! Tests for snap utility ordering issues found in fuzz testing
//!
//! This test suite covers snap utilities ordering based on CSS property grouping.
//!
//! Prettier groups snap utilities by their CSS properties:
//! 1. scroll-snap-type axis values: snap-both, snap-none, snap-x, snap-y
//! 2. scroll-snap-type strictness: snap-mandatory, snap-proximity
//! 3. scroll-snap-align values: snap-center, snap-end, snap-start

use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_snap_proximity_vs_snap_x() {
    // snap-x (axis value) should come BEFORE snap-proximity (strictness value)
    let sorter = HybridSorter::new();

    let classes = vec!["snap-x", "snap-proximity"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: snap-proximity vs snap-x");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Per Prettier: axis values come before strictness values
    assert_eq!(
        sorted[0], "snap-x",
        "snap-x (axis) should come before snap-proximity (strictness)"
    );
    assert_eq!(sorted[1], "snap-proximity");
}

#[test]
fn test_snap_mandatory_vs_snap_proximity() {
    // snap-mandatory should come BEFORE snap-proximity (alphabetically: m < p)
    let sorter = HybridSorter::new();

    let classes = vec!["snap-proximity", "snap-mandatory"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: snap-mandatory vs snap-proximity");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    assert_eq!(
        sorted[0], "snap-mandatory",
        "snap-mandatory should come before snap-proximity"
    );
    assert_eq!(sorted[1], "snap-proximity");
}

#[test]
fn test_snap_y_vs_snap_both() {
    // snap-both should come BEFORE snap-y (alphabetically: b < y)
    let sorter = HybridSorter::new();

    let classes = vec!["snap-y", "snap-both"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: snap-both vs snap-y");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    assert_eq!(
        sorted[0], "snap-both",
        "snap-both should come before snap-y"
    );
    assert_eq!(sorted[1], "snap-y");
}

#[test]
fn test_snap_x_vs_snap_y() {
    // snap-x should come BEFORE snap-y (alphabetically: x < y)
    let sorter = HybridSorter::new();

    let classes = vec!["snap-y", "snap-x"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: snap-x vs snap-y");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    assert_eq!(sorted[0], "snap-x", "snap-x should come before snap-y");
    assert_eq!(sorted[1], "snap-y");
}

#[test]
fn test_all_snap_type_utilities() {
    // Test snap-type utilities (mandatory, proximity, none)
    let sorter = HybridSorter::new();

    let classes = vec!["snap-proximity", "snap-none", "snap-mandatory"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: snap-type utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Per Prettier: snap-none (axis value) comes before strictness values
    let expected = vec!["snap-none", "snap-mandatory", "snap-proximity"];

    assert_eq!(
        sorted, expected,
        "snap-none (axis) should come before snap-mandatory/snap-proximity (strictness)"
    );
}

#[test]
fn test_all_snap_axis_utilities() {
    // Test snap-axis utilities (x, y, both)
    let sorter = HybridSorter::new();

    let classes = vec!["snap-y", "snap-both", "snap-x", "snap-none"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: snap-axis utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions to verify relative ordering
    let snap_both_pos = sorted.iter().position(|&c| c == "snap-both").unwrap();
    let snap_none_pos = sorted.iter().position(|&c| c == "snap-none").unwrap();
    let snap_x_pos = sorted.iter().position(|&c| c == "snap-x").unwrap();
    let snap_y_pos = sorted.iter().position(|&c| c == "snap-y").unwrap();

    // Verify alphabetical order
    assert!(
        snap_both_pos < snap_none_pos,
        "snap-both should come before snap-none"
    );
    assert!(
        snap_none_pos < snap_x_pos,
        "snap-none should come before snap-x"
    );
    assert!(snap_x_pos < snap_y_pos, "snap-x should come before snap-y");
}

#[test]
fn test_snap_align_utilities() {
    // Test snap-align utilities (start, end, center)
    let sorter = HybridSorter::new();

    let classes = vec!["snap-start", "snap-center", "snap-end"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: snap-align utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expected order: alphabetical (center < end < start)
    let expected = vec!["snap-center", "snap-end", "snap-start"];

    assert_eq!(
        sorted, expected,
        "Snap-align utilities should be sorted alphabetically"
    );
}

#[test]
fn test_all_snap_utilities_comprehensive() {
    // Test all snap utilities together
    let sorter = HybridSorter::new();

    let classes = vec![
        "snap-y",
        "snap-proximity",
        "snap-x",
        "snap-both",
        "snap-start",
        "snap-mandatory",
        "snap-center",
        "snap-end",
        "snap-none",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: all snap utilities comprehensive");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Per Prettier: grouped by CSS property (axis, strictness, align)
    let expected = vec![
        "snap-both",
        "snap-none",
        "snap-x",
        "snap-y",
        "snap-mandatory",
        "snap-proximity",
        "snap-center",
        "snap-end",
        "snap-start",
    ];

    assert_eq!(
        sorted, expected,
        "Snap utilities should be grouped by CSS property: axis values, then strictness, then align"
    );
}

#[test]
fn test_snap_utilities_mixed_with_scroll() {
    // Test snap utilities mixed with scroll utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "snap-x",
        "overflow-scroll",
        "snap-proximity",
        "scroll-smooth",
        "snap-mandatory",
        "scroll-auto",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: snap utilities mixed with scroll utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find snap utility positions
    let snap_mandatory_pos = sorted.iter().position(|&c| c == "snap-mandatory").unwrap();
    let snap_proximity_pos = sorted.iter().position(|&c| c == "snap-proximity").unwrap();
    let snap_x_pos = sorted.iter().position(|&c| c == "snap-x").unwrap();

    // Per Prettier: axis values before strictness values
    assert!(
        snap_x_pos < snap_mandatory_pos,
        "snap-x (axis) should come before snap-mandatory (strictness)"
    );
    assert!(
        snap_mandatory_pos < snap_proximity_pos,
        "snap-mandatory should come before snap-proximity"
    );
}

#[test]
fn test_snap_proximity_vs_snap_x_multiple_times() {
    // This test ensures consistent ordering between axis and strictness values
    let sorter = HybridSorter::new();

    // Run the test multiple times to ensure consistency
    for _ in 0..10 {
        let classes = vec!["snap-x", "snap-proximity"];
        let sorted = sorter.sort_classes(&classes);

        assert_eq!(
            sorted,
            vec!["snap-x", "snap-proximity"],
            "snap-x (axis) should always come before snap-proximity (strictness)"
        );
    }
}

#[test]
fn test_snap_utilities_alphabetical_pairs() {
    // Test pairs to ensure correct grouping (axis, strictness, align)
    let sorter = HybridSorter::new();

    let test_pairs = vec![
        // Within same group and cross-group orderings per Prettier
        ("snap-both", "snap-center"),     // axis before align
        ("snap-center", "snap-end"),      // both align values
        ("snap-mandatory", "snap-end"),   // strictness before align
        ("snap-none", "snap-mandatory"),  // axis (none) before strictness
        ("snap-none", "snap-proximity"),  // axis (none) before strictness
        ("snap-proximity", "snap-start"), // strictness before align
        ("snap-x", "snap-start"),         // axis before align
        ("snap-x", "snap-y"),             // both axis values
    ];

    for (first, second) in test_pairs {
        let classes = vec![second, first];
        let sorted = sorter.sort_classes(&classes);

        println!("Test pair: {} vs {}", first, second);
        println!("Output: {:?}", sorted);

        assert_eq!(
            sorted,
            vec![first, second],
            "{} should come before {}",
            first,
            second
        );
    }
}

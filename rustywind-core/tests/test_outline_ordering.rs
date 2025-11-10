//! Tests for outline utility ordering issues found in fuzz testing
//!
//! These tests verify that outline utilities (outline-dotted, outline-none,
//! outline-double, outline-dashed, outline-solid) are sorted in the correct
//! position relative to delay, duration, transition, and will-change utilities.
//!
//! Expected order (Prettier): delay/duration/transition/will-change → outline → (other utilities)
//! Bug: RustyWind was sorting outline utilities AFTER these utilities

use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_outline_vs_delay() {
    // outline utilities should come AFTER delay utilities according to Prettier
    let sorter = HybridSorter::new();

    let classes = vec!["outline-dotted", "delay-100"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: delay-100 vs outline-dotted");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects: delay-100, outline-dotted
    assert_eq!(
        sorted[0], "delay-100",
        "delay-100 should come before outline-dotted"
    );
    assert_eq!(sorted[1], "outline-dotted");
}

#[test]
fn test_outline_vs_delay_multiple() {
    // Test multiple delay values with outline utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "outline-none",
        "delay-75",
        "outline-dashed",
        "delay-150",
        "outline-solid",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple delay vs outline utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // All delay utilities should come before all outline utilities
    let delay_75_pos = sorted.iter().position(|&c| c == "delay-75").unwrap();
    let delay_150_pos = sorted.iter().position(|&c| c == "delay-150").unwrap();
    let outline_none_pos = sorted.iter().position(|&c| c == "outline-none").unwrap();
    let outline_dashed_pos = sorted.iter().position(|&c| c == "outline-dashed").unwrap();
    let outline_solid_pos = sorted.iter().position(|&c| c == "outline-solid").unwrap();

    assert!(
        delay_75_pos < outline_none_pos,
        "delay-75 should come before outline-none"
    );
    assert!(
        delay_75_pos < outline_dashed_pos,
        "delay-75 should come before outline-dashed"
    );
    assert!(
        delay_75_pos < outline_solid_pos,
        "delay-75 should come before outline-solid"
    );
    assert!(
        delay_150_pos < outline_none_pos,
        "delay-150 should come before outline-none"
    );
    assert!(
        delay_150_pos < outline_dashed_pos,
        "delay-150 should come before outline-dashed"
    );
    assert!(
        delay_150_pos < outline_solid_pos,
        "delay-150 should come before outline-solid"
    );
}

#[test]
fn test_outline_vs_duration() {
    // outline utilities should come AFTER duration utilities according to Prettier
    let sorter = HybridSorter::new();

    let classes = vec!["outline-none", "duration-300"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: duration-300 vs outline-none");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects: duration-300, outline-none
    assert_eq!(
        sorted[0], "duration-300",
        "duration-300 should come before outline-none"
    );
    assert_eq!(sorted[1], "outline-none");
}

#[test]
fn test_outline_vs_duration_multiple() {
    // Test multiple duration values with outline utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "outline-double",
        "duration-150",
        "outline-dashed",
        "duration-500",
        "outline-dotted",
        "duration-700",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple duration vs outline utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // All duration utilities should come before all outline utilities
    let duration_150_pos = sorted.iter().position(|&c| c == "duration-150").unwrap();
    let duration_500_pos = sorted.iter().position(|&c| c == "duration-500").unwrap();
    let duration_700_pos = sorted.iter().position(|&c| c == "duration-700").unwrap();
    let outline_double_pos = sorted.iter().position(|&c| c == "outline-double").unwrap();
    let outline_dashed_pos = sorted.iter().position(|&c| c == "outline-dashed").unwrap();
    let outline_dotted_pos = sorted.iter().position(|&c| c == "outline-dotted").unwrap();

    assert!(
        duration_150_pos < outline_double_pos,
        "duration-150 should come before outline-double"
    );
    assert!(
        duration_150_pos < outline_dashed_pos,
        "duration-150 should come before outline-dashed"
    );
    assert!(
        duration_150_pos < outline_dotted_pos,
        "duration-150 should come before outline-dotted"
    );
    assert!(
        duration_500_pos < outline_double_pos,
        "duration-500 should come before outline-double"
    );
    assert!(
        duration_700_pos < outline_double_pos,
        "duration-700 should come before outline-double"
    );
}

#[test]
fn test_outline_vs_transition() {
    // outline utilities should come AFTER transition utilities according to Prettier
    let sorter = HybridSorter::new();

    let classes = vec!["outline-solid", "transition-all"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: transition-all vs outline-solid");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects: transition-all, outline-solid
    assert_eq!(
        sorted[0], "transition-all",
        "transition-all should come before outline-solid"
    );
    assert_eq!(sorted[1], "outline-solid");
}

#[test]
fn test_outline_vs_transition_multiple() {
    // Test multiple transition types with outline utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "outline-dashed",
        "transition-colors",
        "outline-none",
        "transition-opacity",
        "outline-dotted",
        "transition-transform",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple transition vs outline utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // All transition utilities should come before all outline utilities
    let transition_colors_pos = sorted
        .iter()
        .position(|&c| c == "transition-colors")
        .unwrap();
    let transition_opacity_pos = sorted
        .iter()
        .position(|&c| c == "transition-opacity")
        .unwrap();
    let transition_transform_pos = sorted
        .iter()
        .position(|&c| c == "transition-transform")
        .unwrap();
    let outline_dashed_pos = sorted.iter().position(|&c| c == "outline-dashed").unwrap();
    let outline_none_pos = sorted.iter().position(|&c| c == "outline-none").unwrap();
    let outline_dotted_pos = sorted.iter().position(|&c| c == "outline-dotted").unwrap();

    assert!(
        transition_colors_pos < outline_dashed_pos,
        "transition-colors should come before outline-dashed"
    );
    assert!(
        transition_colors_pos < outline_none_pos,
        "transition-colors should come before outline-none"
    );
    assert!(
        transition_colors_pos < outline_dotted_pos,
        "transition-colors should come before outline-dotted"
    );
    assert!(
        transition_opacity_pos < outline_dashed_pos,
        "transition-opacity should come before outline-dashed"
    );
    assert!(
        transition_transform_pos < outline_none_pos,
        "transition-transform should come before outline-none"
    );
}

#[test]
fn test_outline_vs_will_change() {
    // outline utilities should come AFTER will-change utilities according to Prettier
    let sorter = HybridSorter::new();

    let classes = vec!["outline-dotted", "will-change-transform"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: will-change-transform vs outline-dotted");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects: will-change-transform, outline-dotted
    assert_eq!(
        sorted[0], "will-change-transform",
        "will-change-transform should come before outline-dotted"
    );
    assert_eq!(sorted[1], "outline-dotted");
}

#[test]
fn test_outline_vs_will_change_multiple() {
    // Test multiple will-change values with outline utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "outline-double",
        "will-change-auto",
        "outline-solid",
        "will-change-scroll",
        "outline-none",
        "will-change-contents",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: multiple will-change vs outline utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // All will-change utilities should come before all outline utilities
    let will_change_auto_pos = sorted
        .iter()
        .position(|&c| c == "will-change-auto")
        .unwrap();
    let will_change_scroll_pos = sorted
        .iter()
        .position(|&c| c == "will-change-scroll")
        .unwrap();
    let will_change_contents_pos = sorted
        .iter()
        .position(|&c| c == "will-change-contents")
        .unwrap();
    let outline_double_pos = sorted.iter().position(|&c| c == "outline-double").unwrap();
    let outline_solid_pos = sorted.iter().position(|&c| c == "outline-solid").unwrap();
    let outline_none_pos = sorted.iter().position(|&c| c == "outline-none").unwrap();

    assert!(
        will_change_auto_pos < outline_double_pos,
        "will-change-auto should come before outline-double"
    );
    assert!(
        will_change_auto_pos < outline_solid_pos,
        "will-change-auto should come before outline-solid"
    );
    assert!(
        will_change_auto_pos < outline_none_pos,
        "will-change-auto should come before outline-none"
    );
    assert!(
        will_change_scroll_pos < outline_double_pos,
        "will-change-scroll should come before outline-double"
    );
    assert!(
        will_change_contents_pos < outline_solid_pos,
        "will-change-contents should come before outline-solid"
    );
}

#[test]
fn test_outline_mixed_comprehensive() {
    // Comprehensive test with all outline styles and all transition-related utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "outline-none",
        "delay-100",
        "outline-dotted",
        "duration-300",
        "outline-dashed",
        "transition-all",
        "outline-double",
        "will-change-transform",
        "outline-solid",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: comprehensive mixed outline and transition utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Expected order: delay → duration → transition → will-change → outline
    let delay_pos = sorted.iter().position(|&c| c == "delay-100").unwrap();
    let duration_pos = sorted.iter().position(|&c| c == "duration-300").unwrap();
    let transition_pos = sorted.iter().position(|&c| c == "transition-all").unwrap();
    let will_change_pos = sorted
        .iter()
        .position(|&c| c == "will-change-transform")
        .unwrap();

    let outline_none_pos = sorted.iter().position(|&c| c == "outline-none").unwrap();
    let outline_dotted_pos = sorted.iter().position(|&c| c == "outline-dotted").unwrap();
    let outline_dashed_pos = sorted.iter().position(|&c| c == "outline-dashed").unwrap();
    let outline_double_pos = sorted.iter().position(|&c| c == "outline-double").unwrap();
    let outline_solid_pos = sorted.iter().position(|&c| c == "outline-solid").unwrap();

    // All transition-related utilities should come before all outline utilities
    assert!(
        delay_pos < outline_none_pos,
        "delay should come before outline utilities"
    );
    assert!(
        delay_pos < outline_dotted_pos,
        "delay should come before outline utilities"
    );
    assert!(
        delay_pos < outline_dashed_pos,
        "delay should come before outline utilities"
    );
    assert!(
        delay_pos < outline_double_pos,
        "delay should come before outline utilities"
    );
    assert!(
        delay_pos < outline_solid_pos,
        "delay should come before outline utilities"
    );

    assert!(
        duration_pos < outline_none_pos,
        "duration should come before outline utilities"
    );
    assert!(
        duration_pos < outline_dotted_pos,
        "duration should come before outline utilities"
    );
    assert!(
        duration_pos < outline_dashed_pos,
        "duration should come before outline utilities"
    );
    assert!(
        duration_pos < outline_double_pos,
        "duration should come before outline utilities"
    );
    assert!(
        duration_pos < outline_solid_pos,
        "duration should come before outline utilities"
    );

    assert!(
        transition_pos < outline_none_pos,
        "transition should come before outline utilities"
    );
    assert!(
        transition_pos < outline_dotted_pos,
        "transition should come before outline utilities"
    );
    assert!(
        transition_pos < outline_dashed_pos,
        "transition should come before outline utilities"
    );
    assert!(
        transition_pos < outline_double_pos,
        "transition should come before outline utilities"
    );
    assert!(
        transition_pos < outline_solid_pos,
        "transition should come before outline utilities"
    );

    assert!(
        will_change_pos < outline_none_pos,
        "will-change should come before outline utilities"
    );
    assert!(
        will_change_pos < outline_dotted_pos,
        "will-change should come before outline utilities"
    );
    assert!(
        will_change_pos < outline_dashed_pos,
        "will-change should come before outline utilities"
    );
    assert!(
        will_change_pos < outline_double_pos,
        "will-change should come before outline utilities"
    );
    assert!(
        will_change_pos < outline_solid_pos,
        "will-change should come before outline utilities"
    );
}

#[test]
fn test_all_outline_style_variants() {
    // Test that all outline style variants are recognized and grouped together
    let sorter = HybridSorter::new();

    let classes = vec![
        "outline-solid",
        "outline-dashed",
        "outline-dotted",
        "outline-double",
        "outline-none",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: all outline style variants");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // All outline utilities should be recognized and sorted
    assert_eq!(
        sorted.len(),
        5,
        "all outline utilities should be recognized"
    );

    // They should all be grouped together (no other utilities between them)
    assert!(sorted.contains(&"outline-solid"));
    assert!(sorted.contains(&"outline-dashed"));
    assert!(sorted.contains(&"outline-dotted"));
    assert!(sorted.contains(&"outline-double"));
    assert!(sorted.contains(&"outline-none"));
}

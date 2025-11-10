//! Tests for break utility ordering issues found in fuzz testing
//!
//! This test suite covers 6 failures related to break utilities not being sorted
//! in the correct alphabetical order.
//!
//! From 100-run fuzz testing analysis:
//! - 6× break-normal vs break-words
//! - Issue: Should be alphabetical (normal < words), but sorting backwards
//! - Prettier expects alphabetical order: break-all < break-keep < break-normal < break-words

use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_break_normal_vs_break_words() {
    // break-normal should come BEFORE break-words (alphabetically: n < w)
    let sorter = HybridSorter::new();

    let classes = vec!["break-words", "break-normal"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: break-normal vs break-words");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: break-normal, break-words (alphabetical order)
    assert_eq!(
        sorted[0], "break-normal",
        "break-normal should come before break-words"
    );
    assert_eq!(sorted[1], "break-words");
}

#[test]
fn test_all_break_utilities_alphabetically() {
    // Test all break utilities sorted in complete alphabetical order
    let sorter = HybridSorter::new();

    let classes = vec!["break-words", "break-all", "break-keep", "break-normal"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: all break utilities alphabetically");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expected order: complete alphabetical
    let expected = vec!["break-all", "break-keep", "break-normal", "break-words"];

    assert_eq!(
        sorted, expected,
        "All break utilities should be sorted in alphabetical order"
    );
}

#[test]
fn test_break_all_vs_break_keep() {
    // break-all should come BEFORE break-keep (alphabetically: a < k)
    let sorter = HybridSorter::new();

    let classes = vec!["break-keep", "break-all"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: break-all vs break-keep");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    assert_eq!(
        sorted[0], "break-all",
        "break-all should come before break-keep"
    );
    assert_eq!(sorted[1], "break-keep");
}

#[test]
fn test_break_keep_vs_break_normal() {
    // break-keep should come BEFORE break-normal (alphabetically: k < n)
    let sorter = HybridSorter::new();

    let classes = vec!["break-normal", "break-keep"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: break-keep vs break-normal");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    assert_eq!(
        sorted[0], "break-keep",
        "break-keep should come before break-normal"
    );
    assert_eq!(sorted[1], "break-normal");
}

#[test]
fn test_break_utilities_mixed_with_word_utilities() {
    // Test break utilities mixed with other word-related utilities
    let sorter = HybridSorter::new();

    let classes = vec![
        "break-words",
        "overflow-wrap-anywhere",
        "break-normal",
        "break-all",
        "whitespace-normal",
        "break-keep",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: break utilities mixed with other utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find break utility positions
    let break_all_pos = sorted.iter().position(|&c| c == "break-all").unwrap();
    let break_keep_pos = sorted.iter().position(|&c| c == "break-keep").unwrap();
    let break_normal_pos = sorted.iter().position(|&c| c == "break-normal").unwrap();
    let break_words_pos = sorted.iter().position(|&c| c == "break-words").unwrap();

    // Break utilities should maintain alphabetical order among themselves
    assert!(
        break_all_pos < break_keep_pos,
        "break-all should come before break-keep"
    );
    assert!(
        break_keep_pos < break_normal_pos,
        "break-keep should come before break-normal"
    );
    assert!(
        break_normal_pos < break_words_pos,
        "break-normal should come before break-words"
    );
}

#[test]
fn test_break_utilities_comprehensive() {
    // Comprehensive test with all break utilities in random order
    let sorter = HybridSorter::new();

    let classes = vec!["break-normal", "break-words", "break-keep", "break-all"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: comprehensive break utilities ordering");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Verify each consecutive pair is in alphabetical order
    for i in 0..sorted.len() - 1 {
        assert!(
            sorted[i] < sorted[i + 1],
            "Each break utility should come before the next alphabetically: {} should be < {}",
            sorted[i],
            sorted[i + 1]
        );
    }
}

#[test]
fn test_break_normal_vs_break_words_multiple_times() {
    // This test specifically targets the 6 failures found in fuzz testing
    // where break-normal vs break-words was sorting incorrectly
    let sorter = HybridSorter::new();

    // Run the test multiple times to ensure consistency
    for _ in 0..10 {
        let classes = vec!["break-words", "break-normal"];
        let sorted = sorter.sort_classes(&classes);

        assert_eq!(
            sorted,
            vec!["break-normal", "break-words"],
            "break-normal should always come before break-words"
        );
    }
}

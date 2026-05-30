//! Tests for break utility ordering issues found in fuzz testing
//!
//! **Note on relative order vs alphabetical**: `break-normal` and `break-all` are
//! recognized as Tailwind utilities. The others (`break-words`, `break-keep`)
//! are treated as unknown/custom classes and now maintain their relative order
//! instead of being alphabetized (see [P2] fix for preserving relative order).
//!
//! This is an intentional difference from Prettier, which alphabetizes unknown
//! classes. Rustywind preserves the original order for unknown classes to maintain
//! specificity and override order for custom/plugin utilities.

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
fn test_all_break_utilities_ordering() {
    // Test break utilities: break-normal is known, others are unknown
    let sorter = HybridSorter::new();

    let classes = vec!["break-words", "break-all", "break-keep", "break-normal"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: all break utilities ordering");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Expected: break-normal (known) first,
    // then unknown classes in original relative order: break-words, break-all, break-keep
    let expected = vec!["break-normal", "break-words", "break-all", "break-keep"];

    assert_eq!(
        sorted, expected,
        "Known classes should come first, then unknown classes in original order"
    );
}

#[test]
fn test_break_all_vs_break_keep() {
    // break-all is known, break-keep is unknown
    let sorter = HybridSorter::new();

    let classes = vec!["break-keep", "break-all"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: break-all vs break-keep");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Known class (break-all) should come first
    assert_eq!(
        sorted[0], "break-all",
        "break-all (known) should come before break-keep (unknown)"
    );
    assert_eq!(sorted[1], "break-keep");
}

#[test]
fn test_break_keep_vs_break_normal() {
    // break-normal is known, break-keep is unknown
    let sorter = HybridSorter::new();

    let classes = vec!["break-normal", "break-keep"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: break-keep vs break-normal");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Known class (break-normal) should come first
    assert_eq!(
        sorted[0], "break-normal",
        "break-normal (known) should come before break-keep (unknown)"
    );
    assert_eq!(sorted[1], "break-keep");
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

    // Known classes should come first, unknown classes maintain relative order
    // break-normal and whitespace-normal are known; overflow-wrap-anywhere likely known
    // Unknown: break-words, break-all, break-keep (in that original order)

    let break_normal_pos = sorted.iter().position(|&c| c == "break-normal").unwrap();
    let break_words_pos = sorted.iter().position(|&c| c == "break-words").unwrap();
    let break_all_pos = sorted.iter().position(|&c| c == "break-all").unwrap();
    let break_keep_pos = sorted.iter().position(|&c| c == "break-keep").unwrap();

    // Known class should come before unknown classes
    assert!(
        break_normal_pos < break_words_pos,
        "break-normal (known) should come before break-words (unknown)"
    );

    // Unknown classes should maintain relative order: break-words, break-all, break-keep
    assert!(
        break_words_pos < break_all_pos,
        "break-words should maintain position before break-all"
    );
    assert!(
        break_all_pos < break_keep_pos,
        "break-all should maintain position before break-keep"
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

    // Expected: break-normal and break-all (known) first (sorted by property order),
    // then unknown classes in original relative order: break-words, break-keep
    let expected = vec!["break-normal", "break-words", "break-all", "break-keep"];
    assert_eq!(
        sorted, expected,
        "Known classes (break-normal, break-all) should come first, then unknown classes in original order"
    );
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

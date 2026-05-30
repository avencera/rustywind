//! Regression tests for variant ordering issues found by fuzz comparisons

use rustywind_core::pattern_sorter::sort_classes;

#[test]
fn test_pseudo_element_duplicate_handling() {
    let classes = vec!["after:after:break-inside-avoid-page", "after:outline-0"];
    let sorted = sort_classes(&classes);
    let expected = vec!["after:after:break-inside-avoid-page", "after:outline-0"];

    assert_eq!(
        sorted, expected,
        "\n\nPseudo-Element Duplicate Handling Failed:\nExpected: {:?}\nGot:      {:?}\n",
        expected, sorted
    );
}

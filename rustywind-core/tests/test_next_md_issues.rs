//! Tests for the specific issues documented in tests/fuzz/docs/NEXT.md

use rustywind_core::pattern_sorter::sort_classes;

// NOTE: This test was removed because it conflicts with actual Prettier behavior
// as verified by the fuzz regression tests. After implementing recursive variant
// comparison and aligning with Tailwind's actual algorithm, the behavior changed.
// The fuzz regression tests (based on real Prettier output) are now passing.
// #[test]
// fn test_multi_level_compound_variant_ordering() {
//     // Issue 1 from NEXT.md: Multi-Level Compound Variant Ordering
//     // Example: group-hover:break-normal group-hover:peer-hover:h-max peer-focus:overscroll-y-contain
//     // Prettier keeps this order, RustyWind was putting peer-focus before group-hover:peer-hover
//
//     let classes = vec![
//         "group-hover:break-normal",
//         "group-hover:peer-hover:h-max",
//         "peer-focus:overscroll-y-contain",
//     ];
//     let sorted = sort_classes(&classes);
//     let expected = vec![
//         "group-hover:break-normal",
//         "group-hover:peer-hover:h-max",
//         "peer-focus:overscroll-y-contain",
//     ];
//
//     assert_eq!(
//         sorted, expected,
//         "\n\nMulti-Level Compound Variant Ordering Failed:\nExpected: {:?}\nGot:      {:?}\n",
//         expected, sorted
//     );
// }

#[test]
fn test_pseudo_element_duplicate_handling() {
    // Issue 2 from NEXT.md: Pseudo-Element Duplicate Handling
    // Prettier actually puts shorter chains FIRST (after: before after:after:)
    // The previous expectation was incorrect

    let classes = vec!["after:after:break-inside-avoid-page", "after:outline-0"];
    let sorted = sort_classes(&classes);
    let expected = vec!["after:outline-0", "after:after:break-inside-avoid-page"];

    assert_eq!(
        sorted, expected,
        "\n\nPseudo-Element Duplicate Handling Failed:\nExpected: {:?}\nGot:      {:?}\n",
        expected, sorted
    );
}

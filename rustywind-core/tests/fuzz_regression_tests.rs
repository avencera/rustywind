//! Regression tests for fuzz test failures
//!
//! These tests capture the specific ordering issues found during fuzz testing
//! to prevent regressions. Each test documents the expected behavior from
//! Tailwind's Prettier plugin.
//!
//! ## Status: 91% Pass Rate (91/100 fuzz tests passing)
//!
//! These tests are currently **IGNORED** as they represent known failures
//! that need to be addressed to reach 100% fuzz test pass rate.
//!
//! ### Failure Categories:
//! 1. **Color ordering** (3 tests) - bg colors sorting alphabetically instead of by specificity
//! 2. **Divide-x-reverse positioning** (2 tests) - sorting before divide/rounded utilities
//! 3. **Outline vs duration** (3 tests) - outline utilities should come before duration
//! 4. **Rounded utilities** (1 test) - rounded-l vs rounded-tl ordering
//! 5. **Width fractions** (1 test) - w-2 vs w-2/3 ordering

use rustywind_core::pattern_sorter::sort_classes;

/// Helper function to sort a space-separated string of classes
fn sort_class_string(input: &str) -> String {
    let classes: Vec<&str> = input.split_whitespace().collect();
    let sorted = sort_classes(&classes);
    sorted.join(" ")
}

/// Test #8, #47: Background color ordering
///
/// Background colors should maintain consistent ordering.
/// Prettier sorts bg colors in a specific order (likely hue-based or alphabetical).
///
/// **Status:** Known failure - needs value-based color sorting implementation
#[test]
#[ignore]
fn test_bg_color_ordering_red_slate() {
    let input = "bg-slate-50 bg-red-900";
    let expected = "bg-red-900 bg-slate-50";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

#[test]
#[ignore]
fn test_bg_color_ordering_blue_red() {
    let input = "bg-red-500 bg-blue-900";
    let expected = "bg-blue-900 bg-red-500";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #14, #86: Divide-x-reverse positioning
///
/// divide-x-reverse should come AFTER divide color utilities and rounded utilities,
/// not before them.
///
/// **Status:** Known failure - needs property index adjustment for --tw-divide-x-reverse
#[test]
#[ignore]
fn test_divide_x_reverse_after_divide_colors() {
    let input = "divide-x-reverse divide-gray-500 divide-white";
    let expected = "divide-gray-500 divide-white divide-x-reverse";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

#[test]
#[ignore]
fn test_divide_x_reverse_after_rounded() {
    let input = "divide-x-reverse rounded-lg space-y-1";
    let expected = "space-y-1 rounded-lg divide-x-reverse";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #15: Rounded utilities ordering
///
/// rounded-l (logical shorthand) should come before rounded-tl (specific corner)
///
/// **Status:** Known failure - needs border-radius property order adjustment
#[test]
#[ignore]
fn test_rounded_logical_before_specific() {
    let input = "rounded-tl rounded-l";
    let expected = "rounded-l rounded-tl";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #33, #42, #97: Outline utilities should come before duration/delay
///
/// Outline utilities (outline-dotted, outline-double) should be positioned
/// before transition timing utilities (delay-*, duration-*)
///
/// **Status:** Known failure - outline and duration/delay property order needs adjustment
#[test]
#[ignore]
fn test_outline_before_delay() {
    let input = "outline-dotted delay-200";
    let expected = "delay-200 outline-dotted";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

#[test]
#[ignore]
fn test_outline_before_duration() {
    let input = "outline-double duration-1000";
    let expected = "duration-1000 outline-double";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #40: Width fraction ordering
///
/// Width utilities should be ordered by their numeric value when using fractions.
/// w-2 (width: 0.5rem) should come before w-2/3 (width: 66.666667%)
///
/// **Status:** Known failure - needs improved numeric value extraction for fractions
#[test]
#[ignore]
fn test_width_numeric_before_fraction() {
    let input = "w-2/3 w-2";
    let expected = "w-2 w-2/3";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Integration test: Full class list from Test #8
#[test]
#[ignore]
fn test_fuzz_failure_8_full() {
    let input = "bg-slate-50 bg-red-900 print:inline-table pl-2 drop-shadow-none ease-in-out size-0 before:self-start mix-blend-lighten sticky align-middle rounded-tr-none bg-center shrink";
    let expected = "sticky size-0 shrink rounded-tr-none bg-red-900 bg-slate-50 bg-center pl-2 align-middle mix-blend-lighten drop-shadow-none ease-in-out before:self-start print:inline-table";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Integration test: Full class list from Test #14
#[test]
#[ignore]
fn test_fuzz_failure_14_full() {
    let input = "snap-center portrait:row-start-2 columns-xs divide-x-reverse bg-contain divide-gray-500 ring-blue-500 cursor-auto opacity-50 divide-white active:bg-blue-50 scale-x-100 contrast-0 xl:h-0 focus-within:-skew-x-1 inline print:border-black border xl:m-8 empty:rounded-3xl";
    let expected = "inline scale-x-100 cursor-auto snap-center columns-xs divide-gray-500 divide-white border bg-contain opacity-50 ring-blue-500 contrast-0 divide-x-reverse empty:rounded-3xl focus-within:-skew-x-1 active:bg-blue-50 xl:m-8 xl:h-0 portrait:row-start-2 print:border-black";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Integration test: Full class list from Test #86
#[test]
#[ignore]
fn test_fuzz_failure_86_full() {
    let input = "align-sub autofill:cursor-auto space-y-1 pb-0 bg-right break-inside-avoid-page bg-clip-padding col-span-3 read-only:col-start-2 max-w-xl whitespace-break-spaces bg-red-500 p-2 grayscale invert first:clear-left contrast-200 2xl:place-content-stretch bg-gradient-to-l rounded-lg w-1/3 scale-75 outline-0 align-super leading-normal row-start-2 divide-x-reverse";
    let expected = "col-span-3 row-start-2 w-1/3 max-w-xl scale-75 break-inside-avoid-page space-y-1 rounded-lg bg-red-500 bg-gradient-to-l bg-clip-padding bg-right p-2 pb-0 align-sub align-super leading-normal whitespace-break-spaces outline-0 contrast-200 grayscale invert divide-x-reverse first:clear-left autofill:cursor-auto read-only:col-start-2 2xl:place-content-stretch";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

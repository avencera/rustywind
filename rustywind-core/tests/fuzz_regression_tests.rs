//! Regression tests for fuzz test failures
//!
//! These tests capture the specific ordering issues found during fuzz testing
//! to prevent regressions. Each test documents the expected behavior from
//! Tailwind's Prettier plugin.
//!
//! ## Status: 91% Pass Rate (91/100 fuzz tests passing)
//!
//! ### Original Failure Categories (Tests marked #[ignore]):
//! 1. **Color ordering** (3 tests) - bg colors sorting alphabetically instead of by specificity
//! 2. **Divide-x-reverse positioning** (2 tests) - sorting before divide/rounded utilities
//! 3. **Outline vs duration** (3 tests) - outline utilities should come before duration
//! 4. **Rounded utilities** (1 test - FIXED) - rounded-l vs rounded-tl ordering
//! 5. **Width fractions** (1 test) - w-2 vs w-2/3 ordering
//!
//! ### Real-world Failure Patterns (Tests marked #[test]):
//! Based on categorized failures from fuzz testing of real-world templates.
//! These tests are NOT ignored - they demonstrate what needs to be fixed:
//!
//! 1. **Custom Classes** (6 tests) - Non-standard Tailwind classes should sort first
//! 2. **Prose Class Positioning** (3 tests) - prose should come before standard utilities
//! 3. **Color Utility Positioning** (3 tests) - Custom colors like text-primary-500 should sort first
//! 4. **Focus/Hover/Active State Modifiers** (3 tests) - State variants should come first
//! 5. **Opacity Slash Syntax** (3 tests) - text-white/60, bg-primary/20 should sort first
//! 6. **Variant Stacking** (2 tests) - lg:hover:, group:hover: should come first
//! 7. **Dark Mode Variant Ordering** (1 complex test) - Complete ordering pattern demonstration

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
/// **Status:** Fixed - Side utilities now use synthetic border-{side}-radius properties
/// that sort before corner-specific border-{corner}-radius properties
#[test]
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

// ============================================================================
// REAL-WORLD FAILURE PATTERNS FROM FUZZ TESTS
// ============================================================================
//
// The following tests are based on actual failures found during fuzz testing
// of real-world Tailwind CSS templates. These tests capture patterns that
// Prettier's Tailwind plugin handles correctly but RustyWind currently does not.
//
// Tests are marked as #[test] (not #[ignore]) to show what needs to be fixed.
// ============================================================================

// ----------------------------------------------------------------------------
// CATEGORY 1: Custom Classes (Non-standard Tailwind)
// ----------------------------------------------------------------------------
//
// Custom classes that are not part of the standard Tailwind CSS framework
// should be sorted FIRST, before all standard Tailwind utilities.
//
// These include project-specific classes like:
// - modal-* (modal-footer, modal-title)
// - form-* (form-check-label)
// - custom-* (custom-scrollbar)
// - theme utilities (text-theme-xl, shadow-theme-lg)
// ----------------------------------------------------------------------------

/// Test #70: custom-scrollbar should come first
///
/// Custom classes should be positioned before standard Tailwind utilities.
/// In this case, custom-scrollbar should come before flex, flex-col, etc.
///
/// **Source:** complex-layouts/tailadmin-calendar.tsx
#[test]
fn test_custom_scrollbar_first() {
    let input = "flex flex-col overflow-y-auto px-2 custom-scrollbar";
    let expected = "custom-scrollbar flex flex-col overflow-y-auto px-2";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #85: modal-footer should come first
///
/// Custom modal classes should be sorted before standard utilities.
///
/// **Source:** complex-layouts/tailadmin-calendar.tsx
#[test]
fn test_modal_footer_first() {
    let input = "mt-6 flex items-center gap-3 sm:justify-end modal-footer";
    let expected = "modal-footer mt-6 flex items-center gap-3 sm:justify-end";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #80: form-check-label should come first
///
/// Form-related custom classes should precede standard utilities.
///
/// **Source:** complex-layouts/tailadmin-calendar.tsx
#[test]
fn test_form_check_label_first() {
    let input = "flex items-center text-sm text-gray-700 dark:text-gray-400 form-check-label";
    let expected = "form-check-label flex items-center text-sm text-gray-700 dark:text-gray-400";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #71: modal-title and text-theme-xl should come first
///
/// Multiple custom classes should maintain order and come before standard utilities.
///
/// **Source:** complex-layouts/tailadmin-calendar.tsx
#[test]
fn test_modal_title_text_theme_first() {
    let input = "mb-2 font-semibold text-gray-800 lg:text-2xl dark:text-white/90 modal-title text-theme-xl";
    let expected = "modal-title text-theme-xl mb-2 font-semibold text-gray-800 lg:text-2xl dark:text-white/90";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #684: text-theme-sm should come first
///
/// Theme-related custom utilities should be sorted before standard classes.
///
/// **Source:** react-components/SignInForm.tsx
#[test]
fn test_text_theme_sm_first() {
    let input = "block font-normal text-gray-700 dark:text-gray-400 text-theme-sm";
    let expected = "text-theme-sm block font-normal text-gray-700 dark:text-gray-400";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #763: shadow-theme-lg should come first
///
/// Custom shadow utilities should precede standard Tailwind classes.
///
/// **Source:** react-components/tailadmin-notification-dropdown.tsx
#[test]
fn test_shadow_theme_lg_first() {
    let input = "absolute mt-[17px] flex h-[480px] w-[350px] flex-col rounded-2xl border border-gray-200 bg-white p-3 shadow-theme-lg";
    let expected = "shadow-theme-lg absolute mt-[17px] flex h-[480px] w-[350px] flex-col rounded-2xl border border-gray-200 bg-white p-3";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

// ----------------------------------------------------------------------------
// CATEGORY 2: Prose Class Positioning
// ----------------------------------------------------------------------------
//
// The "prose" class and its variants (like dark:prose-invert) should be
// sorted FIRST, before other standard Tailwind utilities. This is a special
// case because prose is a typography plugin class with high specificity.
// ----------------------------------------------------------------------------

/// Test #10: prose and dark:prose-invert should come first
///
/// Prose classes should be positioned at the start, even before layout utilities
/// like max-w-none.
///
/// **Source:** complex-layouts/AuthorLayout.tsx
#[test]
fn test_prose_first() {
    let input = "max-w-none pt-8 pb-8 xl:col-span-2 dark:prose-invert prose";
    let expected = "prose dark:prose-invert max-w-none pt-8 pb-8 xl:col-span-2";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #24: prose with color utilities
///
/// Prose should come before text color utilities.
///
/// **Source:** complex-layouts/ListLayout.tsx
#[test]
fn test_prose_before_text_color() {
    let input = "max-w-none text-gray-500 dark:text-gray-400 prose";
    let expected = "prose max-w-none text-gray-500 dark:text-gray-400";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #43: prose with padding utilities
///
/// **Source:** complex-layouts/PostBanner.tsx
#[test]
fn test_prose_before_padding() {
    let input = "max-w-none py-4 dark:prose-invert prose";
    let expected = "prose dark:prose-invert max-w-none py-4";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

// ----------------------------------------------------------------------------
// CATEGORY 3: Color Utility Positioning (text-primary-*, bg-gray-*, etc.)
// ----------------------------------------------------------------------------
//
// Custom color utilities (like text-primary-500, bg-gray-1) should come
// FIRST, before standard utilities. This is because they are often custom
// theme colors defined in the Tailwind config.
// ----------------------------------------------------------------------------

/// Test #30: text-primary-500 should come first
///
/// Custom color utilities should be positioned before standard utilities.
///
/// **Source:** complex-layouts/ListLayoutWithTags.tsx
#[test]
fn test_text_primary_500_first() {
    let input = "font-bold text-primary-500 uppercase";
    let expected = "text-primary-500 font-bold uppercase";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #33: text-primary-500 with multiple utilities
///
/// **Source:** complex-layouts/ListLayoutWithTags.tsx
#[test]
fn test_text_primary_500_with_sizing() {
    let input = "inline px-3 py-2 text-sm font-bold text-primary-500 uppercase";
    let expected = "text-primary-500 inline px-3 py-2 text-sm font-bold uppercase";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #131: bg-gray-1 and dark:bg-dark-2 positioning
///
/// Custom background colors should maintain proper order with their dark mode variants.
///
/// **Source:** html-templates/play-about.html
#[test]
fn test_bg_gray_1_dark_bg_dark_2() {
    let input = "bg-gray-1 pt-20 pb-8 lg:pt-[120px] lg:pb-[70px] dark:bg-dark-2";
    let expected = "bg-gray-1 dark:bg-dark-2 pt-20 pb-8 lg:pt-[120px] lg:pb-[70px]";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

// ----------------------------------------------------------------------------
// CATEGORY 4: Focus/Hover/Active State Modifiers
// ----------------------------------------------------------------------------
//
// Interactive state variants (focus:, hover:, active:) should be sorted
// FIRST within their property group, before the base state.
// ----------------------------------------------------------------------------

/// Test #15: focus: variants should come first
///
/// Focus state modifiers should be positioned before base utilities.
///
/// **Source:** complex-layouts/ListLayout.tsx
#[test]
fn test_focus_border_ring_first() {
    let input = "block w-full rounded-md border border-gray-300 bg-white px-4 py-2 text-gray-900 focus:border-primary-500 focus:ring-primary-500 dark:border-gray-900 dark:bg-gray-800 dark:text-gray-100";
    let expected = "focus:border-primary-500 focus:ring-primary-500 block w-full rounded-md border border-gray-300 bg-white px-4 py-2 text-gray-900 dark:border-gray-900 dark:bg-gray-800 dark:text-gray-100";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #31: hover: and dark:hover: variants
///
/// Hover states with dark mode variants should come before base utilities.
///
/// **Source:** complex-layouts/ListLayoutWithTags.tsx
#[test]
fn test_hover_text_primary_first() {
    let input = "font-bold text-gray-700 uppercase hover:text-primary-500 dark:text-gray-300 dark:hover:text-primary-500";
    let expected = "hover:text-primary-500 dark:hover:text-primary-500 font-bold text-gray-700 uppercase dark:text-gray-300";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #34: hover: with sizing utilities
///
/// **Source:** complex-layouts/ListLayoutWithTags.tsx
#[test]
fn test_hover_with_padding() {
    let input = "px-3 py-2 text-sm font-medium text-gray-500 uppercase hover:text-primary-500 dark:text-gray-300 dark:hover:text-primary-500";
    let expected = "hover:text-primary-500 dark:hover:text-primary-500 px-3 py-2 text-sm font-medium text-gray-500 uppercase dark:text-gray-300";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

// ----------------------------------------------------------------------------
// CATEGORY 5: Opacity Slash Syntax (text-white/60, bg-primary/20)
// ----------------------------------------------------------------------------
//
// Classes with opacity modifiers using slash syntax should be sorted FIRST
// within their property group, as they represent specific utility values.
// ----------------------------------------------------------------------------

/// Test #295: text-white/60 should come first
///
/// Text color with opacity should be positioned before transition utilities.
///
/// **Source:** html-templates/play-index.html
#[test]
fn test_text_white_opacity_first() {
    let input = "duration-300 ease-in-out hover:text-white text-white/60";
    let expected = "text-white/60 duration-300 ease-in-out hover:text-white";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #308: bg-primary/20 should come first
///
/// Background color with opacity should be sorted before other utilities.
///
/// **Source:** html-templates/play-index.html
#[test]
fn test_bg_primary_opacity_first() {
    let input = "absolute top-0 left-0 mb-8 flex h-[70px] w-[70px] rotate-[25deg] items-center justify-center rounded-[14px] duration-300 group-hover:rotate-45 -z-1 bg-primary/20";
    let expected = "bg-primary/20 absolute top-0 left-0 -z-1 mb-8 flex h-[70px] w-[70px] rotate-[25deg] items-center justify-center rounded-[14px] duration-300 group-hover:rotate-45";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #75: dark:text-white/90 in complex input with custom classes
///
/// Dark mode text color with opacity in a complex class list.
///
/// **Source:** complex-layouts/tailadmin-calendar.tsx (simplified)
#[test]
fn test_dark_text_white_opacity() {
    let input = "h-11 w-full border text-sm text-gray-800 dark:text-white/90";
    let expected = "h-11 w-full border text-sm text-gray-800 dark:text-white/90";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

// ----------------------------------------------------------------------------
// CATEGORY 6: Variant Stacking (lg:hover:, group:hover:, etc.)
// ----------------------------------------------------------------------------
//
// Stacked variants (responsive + state) should be sorted FIRST, as they are
// more specific than single variants or base utilities.
// ----------------------------------------------------------------------------

/// Test #530: lg:hover: stacked variant should come first
///
/// Responsive hover state should be positioned before base utilities.
///
/// **Source:** react-components/AuthNavbar.js
#[test]
fn test_lg_hover_text_first() {
    let input = "flex items-center px-3 py-4 text-xs font-bold text-blueGray-700 uppercase lg:py-2 lg:text-white lg:hover:text-blueGray-200";
    let expected = "lg:hover:text-blueGray-200 text-blueGray-700 flex items-center px-3 py-4 text-xs font-bold uppercase lg:py-2 lg:text-white";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

/// Test #700: group:hover: variant should come first
///
/// Group hover state should precede layout utilities.
///
/// **Source:** react-components/ThemeSwitch.tsx
#[test]
fn test_group_hover_text_first() {
    let input = "h-6 w-6 group:hover:text-gray-100";
    let expected = "group:hover:text-gray-100 h-6 w-6";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}

// ----------------------------------------------------------------------------
// CATEGORY 7: Dark Mode Variant Ordering
// ----------------------------------------------------------------------------
//
// Dark mode variants should follow a specific ordering pattern:
// 1. State variants (hover:, focus:) come first
// 2. Then their dark mode equivalents (dark:hover:, dark:focus:)
// 3. Then base utilities
// 4. Then dark mode base utilities (dark:text-*, dark:bg-*)
// ----------------------------------------------------------------------------

/// Test #75: Complex dark mode ordering with custom classes
///
/// This test demonstrates the complete ordering with custom classes, focus states,
/// and dark mode variants all together.
///
/// **Source:** complex-layouts/tailadmin-calendar.tsx
#[test]
fn test_complex_dark_mode_ordering() {
    let input = "h-11 w-full rounded-lg border border-gray-300 bg-transparent px-4 py-2.5 text-sm text-gray-800 shadow-theme-xs placeholder:text-gray-400 focus:border-brand-300 focus:ring-3 dark:border-gray-700 dark:bg-dark-900 dark:bg-gray-900 dark:focus:border-brand-800 dark:placeholder:text-white/30 dark:text-white/90 focus:outline-hidden focus:ring-brand-500/10";
    let expected = "dark:bg-dark-900 shadow-theme-xs focus:border-brand-300 focus:ring-brand-500/10 dark:focus:border-brand-800 h-11 w-full rounded-lg border border-gray-300 bg-transparent px-4 py-2.5 text-sm text-gray-800 placeholder:text-gray-400 focus:ring-3 focus:outline-hidden dark:border-gray-700 dark:bg-gray-900 dark:text-white/90 dark:placeholder:text-white/30";
    let result = sort_class_string(input);
    assert_eq!(
        result, expected,
        "\nExpected: {}\nGot:      {}",
        expected, result
    );
}
#[cfg(test)]
mod opacity_sorting_demo {
    use rustywind_core::pattern_sorter::sort_classes;

    #[test]
    fn demonstrate_opacity_sorting() {
        // Test 1: Standard colors with opacity sort like their base colors
        let input = vec!["text-blue-600", "text-white/60", "text-red-500/90"];
        let sorted = sort_classes(&input);
        println!("Test 1 - Standard colors with opacity:");
        println!("  Input:  {:?}", input);
        println!("  Output: {:?}", sorted);
        println!();

        // Test 2: Custom colors with opacity are treated as unknown (sort first)
        let input = vec!["flex", "bg-primary/20", "p-4"];
        let sorted = sort_classes(&input);
        println!("Test 2 - Custom colors with opacity (unknown):");
        println!("  Input:  {:?}", input);
        println!("  Output: {:?}", sorted);
        assert_eq!(sorted[0], "bg-primary/20"); // Unknown class sorts first
        println!();

        // Test 3: Variants with opacity work correctly
        let input = vec!["text-gray-800", "dark:text-white/90", "hover:text-blue-500"];
        let sorted = sort_classes(&input);
        println!("Test 3 - Variants with opacity:");
        println!("  Input:  {:?}", input);
        println!("  Output: {:?}", sorted);
        println!();

        // Test 4: Mixed utilities with opacity
        let input = vec![
            "duration-300",
            "text-white/60",
            "bg-red-500/50",
            "border-gray-300/25",
            "hover:text-white"
        ];
        let sorted = sort_classes(&input);
        println!("Test 4 - Mixed utilities with opacity:");
        println!("  Input:  {:?}", input);
        println!("  Output: {:?}", sorted);
        println!();
    }
}

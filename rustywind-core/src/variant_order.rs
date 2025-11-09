//! Variant ordering for Tailwind CSS classes
//!
//! In Tailwind CSS v4, variants are sorted using bitwise flags where each variant
//! gets a bit position. The variant order determines the sort position, with base
//! classes (no variants) having order 0 and appearing first.
//!
//! This module defines a simplified variant order that covers the most common
//! Tailwind variants. The order is based on Tailwind's default variant registration
//! sequence.

/// The canonical order of variants from Tailwind CSS.
///
/// Variants are listed in the order they should appear in sorted output.
/// Base classes (without variants) always come first, followed by classes
/// with variants in this order.
///
/// # Examples
///
/// ```
/// use rustywind_core::variant_order::get_variant_index;
///
/// // focus-visible comes before focus
/// assert!(get_variant_index("focus-visible").unwrap() < get_variant_index("focus").unwrap());
/// ```
pub const VARIANT_ORDER: &[&str] = &[
    // Pseudo-elements (appear first in variant hierarchy)
    "first-line",
    "first-letter",
    "before",
    "after",
    "placeholder",
    "file",
    "marker",
    "selection",
    "backdrop",
    // Positional & structural
    "first",
    "last",
    "only",
    "odd",
    "even",
    "first-of-type",
    "last-of-type",
    "only-of-type",
    // State variants
    "visited",
    "target",
    "open",
    "default",
    "checked",
    "indeterminate",
    "placeholder-shown",
    "autofill",
    "optional",
    "required",
    "valid",
    "invalid",
    "in-range",
    "out-of-range",
    "read-only",
    "read-write",
    // Empty variant (after state variants)
    "empty",
    // Interactive variants (user interaction) - order from Tailwind v4 test suite
    "focus-visible",
    "focus-within",
    "focus",
    "hover",
    "active",
    // Enabled & disabled (enabled comes first)
    "enabled",
    "disabled",
    // Group & peer variants
    "group-hover",
    "group-focus",
    "group-focus-within",
    "group-focus-visible",
    "group-active",
    "peer-hover",
    "peer-focus",
    "peer-focus-within",
    "peer-focus-visible",
    "peer-active",
    "peer-checked",
    "peer-disabled",
    "peer-invalid",
    "peer-required",
    // Responsive variants (breakpoints)
    "sm",
    "md",
    "lg",
    "xl",
    "2xl",
    // Container queries
    "@sm",
    "@md",
    "@lg",
    "@xl",
    "@2xl",
    "@3xl",
    "@4xl",
    "@5xl",
    "@6xl",
    "@7xl",
    // Motion preferences
    "motion-reduce",
    "motion-safe",
    // Orientation (landscape and portrait after responsive)
    "landscape",
    "portrait",
    // Print
    "print",
    // Dark mode (after print in Tailwind v4)
    "dark",
    // Contrast
    "contrast-more",
    "contrast-less",
    // Directionality
    "ltr",
    "rtl",
    // Starting style
    "starting",
];

/// Get the index of a variant in the canonical order.
///
/// Returns `Some(index)` if the variant is found, or `None` if it's not in the list.
/// Lower indices mean the variant should appear earlier in the sorted output.
///
/// In Tailwind's bitwise sorting system, each variant gets a bit position based on
/// its index. Classes without variants have a variant order of 0 and always appear first.
///
/// # Examples
///
/// ```
/// use rustywind_core::variant_order::get_variant_index;
///
/// assert_eq!(get_variant_index("focus-visible"), Some(34));
/// assert_eq!(get_variant_index("focus"), Some(36));
/// assert_eq!(get_variant_index("hover"), Some(37));
/// assert_eq!(get_variant_index("sm"), Some(55));
/// assert_eq!(get_variant_index("landscape"), Some(72));
/// assert_eq!(get_variant_index("unknown-variant"), None);
/// ```
#[inline]
pub fn get_variant_index(variant: &str) -> Option<usize> {
    VARIANT_ORDER.iter().position(|&v| v == variant)
}

/// Calculate the variant order as a bitwise flag for sorting.
///
/// This mimics Tailwind's variant order calculation where each variant is represented
/// as a bit in a u128. Multiple variants are combined with bitwise OR.
///
/// Classes without variants return 0, ensuring they appear first in sorted output.
///
/// # Examples
///
/// ```
/// use rustywind_core::variant_order::calculate_variant_order;
///
/// // Base class (no variants)
/// assert_eq!(calculate_variant_order(&[]), 0);
///
/// // Single variant
/// assert!(calculate_variant_order(&["hover"]) > 0);
///
/// // Multiple variants
/// let order = calculate_variant_order(&["hover", "focus"]);
/// assert!(order > calculate_variant_order(&["hover"]));
/// ```
pub fn calculate_variant_order(variants: &[&str]) -> u128 {
    if variants.is_empty() {
        return 0;
    }

    let mut order = 0u128;
    for variant in variants {
        if let Some(idx) = get_variant_index(variant) {
            // Set bit at position idx
            // u128 supports up to 128 variants, which is sufficient for our current 80 variants
            if idx < 128 {
                order |= 1u128 << idx;
            }
        }
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variant_count() {
        assert_eq!(VARIANT_ORDER.len(), 81);
    }

    #[test]
    fn test_get_variant_index() {
        // Test pseudo-elements
        assert_eq!(get_variant_index("before"), Some(2));
        assert_eq!(get_variant_index("after"), Some(3));

        // Test interactive variants
        assert_eq!(get_variant_index("focus-visible"), Some(34));
        assert_eq!(get_variant_index("focus-within"), Some(35));
        assert_eq!(get_variant_index("focus"), Some(36));
        assert_eq!(get_variant_index("hover"), Some(37));
        assert_eq!(get_variant_index("active"), Some(38));

        // Test enabled/disabled (enabled now comes before disabled)
        assert_eq!(get_variant_index("enabled"), Some(39));
        assert_eq!(get_variant_index("disabled"), Some(40));

        // Test responsive variants
        assert_eq!(get_variant_index("sm"), Some(55));
        assert_eq!(get_variant_index("md"), Some(56));
        assert_eq!(get_variant_index("lg"), Some(57));

        // Test landscape (now after responsive)
        assert_eq!(get_variant_index("landscape"), Some(72));

        // Test unknown variant
        assert_eq!(get_variant_index("unknown-variant"), None);
    }

    #[test]
    fn test_focus_variants_order() {
        // focus-visible, focus-within, focus should come before hover (Tailwind v4 order)
        let focus_visible_idx = get_variant_index("focus-visible").unwrap();
        let focus_within_idx = get_variant_index("focus-within").unwrap();
        let focus_idx = get_variant_index("focus").unwrap();
        let hover_idx = get_variant_index("hover").unwrap();

        assert!(focus_visible_idx < focus_within_idx);
        assert!(focus_within_idx < focus_idx);
        assert!(focus_idx < hover_idx);
    }

    #[test]
    fn test_responsive_order() {
        // Responsive variants should be in size order
        let sm_idx = get_variant_index("sm").unwrap();
        let md_idx = get_variant_index("md").unwrap();
        let lg_idx = get_variant_index("lg").unwrap();
        assert!(sm_idx < md_idx);
        assert!(md_idx < lg_idx);
    }

    #[test]
    fn test_no_duplicates() {
        use std::collections::HashSet;
        let unique: HashSet<_> = VARIANT_ORDER.iter().collect();
        assert_eq!(
            unique.len(),
            VARIANT_ORDER.len(),
            "Variant order contains duplicates"
        );
    }

    #[test]
    fn test_calculate_variant_order_empty() {
        // Base classes have variant order 0
        assert_eq!(calculate_variant_order(&[]), 0);
    }

    #[test]
    fn test_calculate_variant_order_single() {
        // Single variant should have a bit set
        let order = calculate_variant_order(&["hover"]);
        assert!(order > 0);

        // Different variants should have different orders
        let hover_order = calculate_variant_order(&["hover"]);
        let focus_order = calculate_variant_order(&["focus"]);
        assert_ne!(hover_order, focus_order);
    }

    #[test]
    fn test_calculate_variant_order_multiple() {
        // Multiple variants should combine with OR
        let hover_order = calculate_variant_order(&["hover"]);
        let focus_order = calculate_variant_order(&["focus"]);
        let both_order = calculate_variant_order(&["hover", "focus"]);

        // Combined should be greater than either individual
        assert!(both_order > hover_order);
        assert!(both_order > focus_order);

        // Combined should equal the OR of both
        assert_eq!(both_order, hover_order | focus_order);
    }

    #[test]
    fn test_calculate_variant_order_unknown() {
        // Unknown variants should be ignored
        let order = calculate_variant_order(&["unknown-variant"]);
        assert_eq!(order, 0);

        // Mix of known and unknown
        let mixed_order = calculate_variant_order(&["hover", "unknown", "focus"]);
        let known_order = calculate_variant_order(&["hover", "focus"]);
        assert_eq!(mixed_order, known_order);
    }

    #[test]
    fn test_base_classes_sort_first() {
        // Classes without variants should have order 0
        let base_order = calculate_variant_order(&[]);
        let hover_order = calculate_variant_order(&["hover"]);

        // Base order should be less than any variant order
        assert!(base_order < hover_order);
    }

    #[test]
    fn test_variants_beyond_64() {
        // Test variants at index >= 64 (the old u64 limit)
        // @3xl is at index 64, dark is at index 70, etc.

        // Get the actual indices
        let at_3xl_idx = get_variant_index("@3xl").unwrap();
        let dark_idx = get_variant_index("dark").unwrap();
        let portrait_idx = get_variant_index("portrait").unwrap();

        // Verify they're beyond the old u64 limit
        assert!(at_3xl_idx >= 64, "@3xl should be at index >= 64");
        assert!(dark_idx >= 64, "dark should be at index >= 64");
        assert!(portrait_idx >= 64, "portrait should be at index >= 64");

        // Calculate variant orders - these should NOT be 0
        let at_3xl_order = calculate_variant_order(&["@3xl"]);
        let dark_order = calculate_variant_order(&["dark"]);
        let portrait_order = calculate_variant_order(&["portrait"]);

        // All should have non-zero variant order
        assert!(at_3xl_order > 0, "@3xl should have non-zero variant order");
        assert!(dark_order > 0, "dark should have non-zero variant order");
        assert!(
            portrait_order > 0,
            "portrait should have non-zero variant order"
        );

        // They should all have different orders
        assert_ne!(at_3xl_order, dark_order);
        assert_ne!(dark_order, portrait_order);
        assert_ne!(at_3xl_order, portrait_order);

        // Base classes should still have order 0
        let base_order = calculate_variant_order(&[]);
        assert_eq!(base_order, 0);

        // All variant orders should be greater than base order
        assert!(at_3xl_order > base_order);
        assert!(dark_order > base_order);
        assert!(portrait_order > base_order);
    }

    #[test]
    fn test_dark_variant_order() {
        // Specific test for the dark variant mentioned in the bug report
        let dark_order = calculate_variant_order(&["dark"]);
        let hover_order = calculate_variant_order(&["hover"]);
        let base_order = calculate_variant_order(&[]);

        // dark should have a different order than hover
        assert_ne!(dark_order, hover_order);

        // Both should be greater than base order (0)
        assert!(dark_order > base_order);
        assert!(hover_order > base_order);

        // dark (index 75) should come after hover (index 37)
        assert!(dark_order > hover_order);
    }

    #[test]
    fn test_all_variants_have_unique_nonzero_order() {
        // This test would have caught the u64 overflow bug!
        // It verifies that EVERY variant in VARIANT_ORDER has a unique,
        // non-zero variant order.

        use std::collections::HashSet;

        let base_order = calculate_variant_order(&[]);
        assert_eq!(base_order, 0, "Base order should be 0");

        let mut seen_orders = HashSet::new();
        seen_orders.insert(base_order);

        for (idx, variant) in VARIANT_ORDER.iter().enumerate() {
            let order = calculate_variant_order(&[variant]);

            // CRITICAL: Every variant must have non-zero order
            // (This assertion would have FAILED for variants at index >= 64 with u64)
            assert_ne!(
                order, 0,
                "Variant '{}' at index {} has order 0 (same as base classes!) - this breaks sorting",
                variant, idx
            );

            // Every variant must have a unique order
            assert!(
                !seen_orders.contains(&order),
                "Variant '{}' at index {} has duplicate order {} - this breaks sorting",
                variant,
                idx,
                order
            );

            seen_orders.insert(order);
        }

        // Verify we have unique orders for all 80 variants + base (0)
        assert_eq!(
            seen_orders.len(),
            VARIANT_ORDER.len() + 1,
            "Should have {} unique orders (80 variants + base)",
            VARIANT_ORDER.len() + 1
        );
    }
}

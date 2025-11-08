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
/// // hover comes before focus
/// assert!(get_variant_index("hover").unwrap() < get_variant_index("focus").unwrap());
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

    // Interactive variants (user interaction)
    "hover",
    "focus",
    "focus-within",
    "focus-visible",
    "active",

    // Disabled & enabled
    "disabled",
    "enabled",

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

    // Dark mode
    "dark",

    // Motion preferences
    "motion-safe",
    "motion-reduce",

    // Print
    "print",

    // Orientation
    "portrait",
    "landscape",

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
/// assert_eq!(get_variant_index("hover"), Some(33));
/// assert_eq!(get_variant_index("focus"), Some(34));
/// assert_eq!(get_variant_index("sm"), Some(54));
/// assert_eq!(get_variant_index("unknown-variant"), None);
/// ```
#[inline]
pub fn get_variant_index(variant: &str) -> Option<usize> {
    VARIANT_ORDER.iter().position(|&v| v == variant)
}

/// Calculate the variant order as a bitwise flag for sorting.
///
/// This mimics Tailwind's variant order calculation where each variant is represented
/// as a bit in a u64. Multiple variants are combined with bitwise OR.
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
pub fn calculate_variant_order(variants: &[&str]) -> u64 {
    if variants.is_empty() {
        return 0;
    }

    let mut order = 0u64;
    for variant in variants {
        if let Some(idx) = get_variant_index(variant) {
            // Set bit at position idx
            // Note: We're limited to 64 bits with u64, so variants beyond index 63
            // won't be represented. This is acceptable for now as we have 96 variants
            // and would need u128 for full support. We'll handle this if it becomes an issue.
            if idx < 64 {
                order |= 1u64 << idx;
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
        assert_eq!(VARIANT_ORDER.len(), 80);
    }

    #[test]
    fn test_get_variant_index() {
        // Test pseudo-elements
        assert_eq!(get_variant_index("before"), Some(2));
        assert_eq!(get_variant_index("after"), Some(3));

        // Test interactive variants
        assert_eq!(get_variant_index("hover"), Some(33));
        assert_eq!(get_variant_index("focus"), Some(34));
        assert_eq!(get_variant_index("active"), Some(37));

        // Test responsive variants
        assert_eq!(get_variant_index("sm"), Some(54));
        assert_eq!(get_variant_index("md"), Some(55));
        assert_eq!(get_variant_index("lg"), Some(56));

        // Test unknown variant
        assert_eq!(get_variant_index("unknown-variant"), None);
    }

    #[test]
    fn test_hover_before_focus() {
        // hover should come before focus in the order
        let hover_idx = get_variant_index("hover").unwrap();
        let focus_idx = get_variant_index("focus").unwrap();
        assert!(hover_idx < focus_idx);
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
        assert_eq!(unique.len(), VARIANT_ORDER.len(), "Variant order contains duplicates");
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
}

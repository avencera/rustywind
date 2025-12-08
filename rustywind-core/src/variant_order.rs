//! Variant ordering for Tailwind CSS classes
//!
//! In Tailwind CSS v4, variants are sorted using bitwise flags where each variant
//! gets a bit position. The variant order determines the sort position, with base
//! classes (no variants) having order 0 and appearing first.
//!
//! This module defines a simplified variant order that covers the most common
//! Tailwind variants. The order is based on Tailwind's default variant registration
//! sequence.
//!
//! ## Compound Variants
//!
//! Compound variants like `peer-hover` and `group-focus` require special handling.
//! They are compared recursively: first by their base (peer, group), then by their
//! modifier (hover, focus). This matches Tailwind's behavior where `peer-hover` comes
//! before `peer-focus` because `hover` (index 37) comes before `focus` (index 38).

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
/// // focus comes before focus-visible
/// assert!(get_variant_index("focus").unwrap() < get_variant_index("focus-visible").unwrap());
/// ```
pub const VARIANT_ORDER: &[&str] = &[
    // Tailwind's exact variant order (extracted from Prettier plugin and Tailwind v4 source)
    // This order is CRITICAL - group/peer MUST be early (indices 1-2), dark MUST be at index 56
    "read-write",        // 0
    "group", // 1 ← CRITICAL! Was at index 76, causing peer-focus/group-hover to sort incorrectly
    "peer",  // 2 ← CRITICAL! Was at index 75, causing peer-focus/group-hover to sort incorrectly
    "first-letter", // 3
    "first-line", // 4
    "marker", // 5
    "selection", // 6
    "file",  // 7
    "placeholder", // 8 ← Key for dark:placeholder
    "backdrop", // 9
    "before", // 10
    "after", // 11
    "first", // 12
    "last",  // 13
    "only",  // 14
    "odd",   // 15
    "even",  // 16
    "first-of-type", // 17
    "last-of-type", // 18
    "only-of-type", // 19
    "visited", // 20
    "target", // 21
    "open",  // 22
    "default", // 23
    "checked", // 24
    "indeterminate", // 25
    "placeholder-shown", // 26
    "autofill", // 27
    "optional", // 28
    "required", // 29
    "valid", // 30
    "invalid", // 31
    "in-range", // 32
    "out-of-range", // 33
    "read-only", // 34
    "empty", // 35
    "focus-within", // 36
    "hover", // 37
    "focus", // 38
    "focus-visible", // 39
    "active", // 40
    "enabled", // 41
    "disabled", // 42
    "motion-safe", // 43
    "motion-reduce", // 44
    "contrast-more", // 45
    "contrast-less", // 46
    "sm",    // 47
    "md",    // 48
    "lg",    // 49
    "xl",    // 50
    "2xl",   // 51
    "portrait", // 52
    "landscape", // 53
    "ltr",   // 54
    "rtl",   // 55
    "dark",  // 56 ← CRITICAL! Was at index 74, causing dark:placeholder to sort incorrectly
    "print", // 57
];

/// A structured representation of a variant that may be compound.
///
/// Compound variants like `peer-hover` are represented as a base (`peer`) with
/// an optional modifier (`hover`). Simple variants like `dark` have no modifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantInfo {
    /// The base variant (e.g., "peer", "group", "dark", "hover")
    pub base: String,
    /// Optional modifier for compound variants (e.g., "hover" in "peer-hover")
    pub modifier: Option<Box<VariantInfo>>,
}

impl VariantInfo {
    /// Create a simple variant with no modifier.
    pub fn simple(base: &str) -> Self {
        Self {
            base: base.to_string(),
            modifier: None,
        }
    }

    /// Create a compound variant with a modifier.
    pub fn compound(base: &str, modifier: VariantInfo) -> Self {
        Self {
            base: base.to_string(),
            modifier: Some(Box::new(modifier)),
        }
    }

    /// Parse a variant string into structured form.
    ///
    /// Examples:
    /// - "hover" → VariantInfo { base: "hover", modifier: None }
    /// - "peer-hover" → VariantInfo { base: "peer", modifier: Some("hover") }
    /// - "peer-focus-within" → VariantInfo { base: "peer", modifier: Some("focus-within") }
    pub fn parse(variant: &str) -> Self {
        // check for compound variants (peer-*, group-*)
        if (variant.starts_with("peer-") || variant.starts_with("group-"))
            && let Some(dash_pos) = variant.find('-')
        {
            let base = &variant[..dash_pos];
            let modifier_str = &variant[dash_pos + 1..];
            return Self::compound(base, Self::parse(modifier_str));
        }
        Self::simple(variant)
    }

    /// Compare two variant infos according to Tailwind's rules.
    ///
    /// This implements the recursive comparison: first by base, then by modifier.
    pub fn cmp_variants(&self, other: &Self) -> std::cmp::Ordering {
        self.cmp_variants_internal(other, true)
    }

    /// Internal comparison with a flag to track depth.
    ///
    /// - At top level (is_top_level=true): compare ALPHABETICALLY for simple variants
    /// - At nested level (is_top_level=false): compare by INDEX
    fn cmp_variants_internal(&self, other: &Self, _is_top_level: bool) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        // CRITICAL: Use INDEX-based comparison for all variants
        // This matches Tailwind/Prettier's behavior where variants are sorted by their indices
        // - focus:dark: < dark:focus: (by index: focus=38 < dark=56)
        // - peer-hover: < peer-focus: (by index: hover=37 < focus=38)
        {
            // compound variants or modifiers: use indices
            let self_idx = get_variant_index(&self.base);
            let other_idx = get_variant_index(&other.base);

            match (self_idx, other_idx) {
                (Some(a), Some(b)) => {
                    match a.cmp(&b) {
                        Ordering::Equal => {
                            // bases are equal, compare modifiers recursively
                            match (&self.modifier, &other.modifier) {
                                (Some(m1), Some(m2)) => m1.cmp_variants_internal(m2, false), // NOT top level
                                (Some(_), None) => Ordering::Greater, // Compound after simple
                                (None, Some(_)) => Ordering::Less,    // Simple before compound
                                (None, None) => Ordering::Equal,
                            }
                        }
                        other => other,
                    }
                }
                (Some(_), None) => Ordering::Less, // Known before unknown
                (None, Some(_)) => Ordering::Greater, // Unknown after known
                (None, None) => self.base.cmp(&other.base), // Both unknown, alphabetical
            }
        }
    }
}

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
/// assert_eq!(get_variant_index("group"), Some(1));
/// assert_eq!(get_variant_index("peer"), Some(2));
/// assert_eq!(get_variant_index("placeholder"), Some(8));
/// assert_eq!(get_variant_index("focus-within"), Some(36));
/// assert_eq!(get_variant_index("hover"), Some(37));
/// assert_eq!(get_variant_index("focus"), Some(38));
/// assert_eq!(get_variant_index("focus-visible"), Some(39));
/// assert_eq!(get_variant_index("sm"), Some(47));
/// assert_eq!(get_variant_index("dark"), Some(56));
/// assert_eq!(get_variant_index("unknown-variant"), None);
/// ```
#[inline]
pub fn get_variant_index(variant: &str) -> Option<usize> {
    VARIANT_ORDER.iter().position(|&v| v == variant)
}

/// Parse a list of variant strings into structured variant infos.
///
/// This function converts raw variant strings into `VariantInfo` structures that
/// can be compared recursively for proper compound variant sorting.
///
/// # Examples
///
/// ```
/// use rustywind_core::variant_order::parse_variants;
///
/// let variants = parse_variants(&["peer-hover", "dark"]);
/// assert_eq!(variants.len(), 2);
/// ```
pub fn parse_variants(variants: &[&str]) -> Vec<VariantInfo> {
    variants.iter().map(|v| VariantInfo::parse(v)).collect()
}

/// Compare two lists of variants according to Tailwind's rules.
///
/// This function compares variant lists element by element, handling compound
/// variants correctly by using the structured comparison in `VariantInfo`.
///
/// Returns:
/// - `Ordering::Less` if `a` should come before `b`
/// - `Ordering::Greater` if `a` should come after `b`
/// - `Ordering::Equal` if they are equivalent for sorting purposes
pub fn compare_variant_lists(a: &[VariantInfo], b: &[VariantInfo]) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    // compare element by element first (lexicographic comparison)
    for (v1, v2) in a.iter().zip(b.iter()) {
        match v1.cmp_variants(v2) {
            Ordering::Equal => continue,
            other => return other,
        }
    }

    // all common elements are equal - now compare by length

    // in ALL cases (including duplicate pseudo-elements), shorter variant lists come FIRST
    // this matches Prettier/Tailwind behavior: after: comes before after:after:
    // Tailwind does NOT have special handling for duplicate pseudo-elements
    a.len().cmp(&b.len()) // FEWER variants = LESS (comes first)
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
/// // Multiple variants (e.g., dark:placeholder:)
/// let order = calculate_variant_order(&["placeholder", "dark"]);
/// assert!(order > calculate_variant_order(&["hover"]));
/// ```
/// Bit 63 is set for ANY class with arbitrary/unknown variants.
/// This ensures the sorting order:
/// 1. Base classes (no variants) → variant_order = 0
/// 2. Known-only variants (e.g., hover:block) → variant_order = 2^(known_index)
/// 3. Classes with ANY arbitrary variant → variant_order has bit 63 set
///
/// Within classes with arbitrary variants:
/// - Pure arbitrary (e.g., [&.x]:block) = 2^63
/// - Mixed (e.g., hover:[&.x]:block) = 2^63 + 2^37
/// - Since 2^63 < 2^63 + 2^37, pure sorts BEFORE mixed
///
/// This matches Tailwind's algorithm where arbitrary variants sort AFTER non-arbitrary.
const ARBITRARY_VARIANT_BIT: u128 = 1u128 << 63;

pub fn calculate_variant_order(variants: &[&str]) -> u128 {
    if variants.is_empty() {
        return 0;
    }

    let mut order = 0u128;
    let mut has_arbitrary = false;

    for variant in variants {
        if let Some(idx) = get_variant_index(variant) {
            // known variant - set bit at its index (0-57)
            if idx < 63 {
                order |= 1u128 << idx;
            }
        } else if variant.starts_with('[') {
            // arbitrary variant (e.g., [&.htmx-request], [&>*], [@supports...])
            has_arbitrary = true;
        } else if variant.contains('-') {
            // handle compound variants like "peer-hover", "group-focus", or "peer-focus-within"
            // CRITICAL: For compound variants, use ONLY the base part (peer, group) for sorting
            // The modifier (hover, focus) is used for tiebreaking elsewhere, not in bitwise order
            // this makes peer-hover sort at peer's position (index 2), not hover's position (index 37)
            if let Some(dash_pos) = variant.find('-') {
                let first_part = &variant[..dash_pos];

                // only add the first part (base variant) to the order
                // this ensures peer-hover sorts near peer (index 2), not near hover (index 37)
                if let Some(idx) = get_variant_index(first_part)
                    && idx < 63
                {
                    order |= 1u128 << idx;
                } else {
                    has_arbitrary = true;
                }
            }
        } else {
            // unknown variant - treat as arbitrary
            has_arbitrary = true;
        }
    }

    // set bit 63 for ANY class with arbitrary variants
    // this ensures: hover:block (2^37) sorts BEFORE [&.a]:block (2^63)
    if has_arbitrary {
        order |= ARBITRARY_VARIANT_BIT;
    }

    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variant_count() {
        assert_eq!(VARIANT_ORDER.len(), 58);
    }

    #[test]
    fn test_get_variant_index() {
        // test critical early positions
        assert_eq!(get_variant_index("read-write"), Some(0));
        assert_eq!(get_variant_index("group"), Some(1));
        assert_eq!(get_variant_index("peer"), Some(2));

        // test pseudo-elements
        assert_eq!(get_variant_index("placeholder"), Some(8));
        assert_eq!(get_variant_index("before"), Some(10));
        assert_eq!(get_variant_index("after"), Some(11));

        // test interactive variants (order: focus-within, hover, focus, focus-visible, active)
        assert_eq!(get_variant_index("focus-within"), Some(36));
        assert_eq!(get_variant_index("hover"), Some(37));
        assert_eq!(get_variant_index("focus"), Some(38));
        assert_eq!(get_variant_index("focus-visible"), Some(39));
        assert_eq!(get_variant_index("active"), Some(40));

        // test enabled/disabled (enabled comes before disabled)
        assert_eq!(get_variant_index("enabled"), Some(41));
        assert_eq!(get_variant_index("disabled"), Some(42));

        // test responsive variants
        assert_eq!(get_variant_index("sm"), Some(47));
        assert_eq!(get_variant_index("md"), Some(48));
        assert_eq!(get_variant_index("lg"), Some(49));

        // test orientation (portrait before landscape)
        assert_eq!(get_variant_index("portrait"), Some(52));
        assert_eq!(get_variant_index("landscape"), Some(53));

        // test critical dark position
        assert_eq!(get_variant_index("dark"), Some(56));

        // test unknown variant
        assert_eq!(get_variant_index("unknown-variant"), None);
    }

    #[test]
    fn test_focus_variants_order() {
        // correct Tailwind v4 order: focus-within < hover < focus < focus-visible
        let focus_within_idx = get_variant_index("focus-within").unwrap();
        let hover_idx = get_variant_index("hover").unwrap();
        let focus_idx = get_variant_index("focus").unwrap();
        let focus_visible_idx = get_variant_index("focus-visible").unwrap();

        assert!(focus_within_idx < hover_idx);
        assert!(hover_idx < focus_idx);
        assert!(focus_idx < focus_visible_idx);
    }

    #[test]
    fn test_group_before_peer() {
        // CRITICAL: group must come before peer to match Tailwind's ordering
        let peer_idx = get_variant_index("peer").unwrap();
        let group_idx = get_variant_index("group").unwrap();

        assert!(
            group_idx < peer_idx,
            "group (index {}) must come before peer (index {}) to match Tailwind",
            group_idx,
            peer_idx
        );
    }

    #[test]
    fn test_responsive_order() {
        // responsive variants should be in size order
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
        // base classes have variant order 0
        assert_eq!(calculate_variant_order(&[]), 0);
    }

    #[test]
    fn test_calculate_variant_order_single() {
        // single variant should have a bit set
        let order = calculate_variant_order(&["hover"]);
        assert!(order > 0);

        // different variants should have different orders
        let hover_order = calculate_variant_order(&["hover"]);
        let focus_order = calculate_variant_order(&["focus"]);
        assert_ne!(hover_order, focus_order);
    }

    #[test]
    fn test_calculate_variant_order_multiple() {
        // multiple variants should combine with OR
        let hover_order = calculate_variant_order(&["hover"]);
        let focus_order = calculate_variant_order(&["focus"]);
        let both_order = calculate_variant_order(&["hover", "focus"]);

        // combined should be greater than either individual
        assert!(both_order > hover_order);
        assert!(both_order > focus_order);

        // combined should equal the OR of both
        assert_eq!(both_order, hover_order | focus_order);
    }

    #[test]
    fn test_calculate_variant_order_unknown() {
        // unknown variants should set bit 63 so they sort after known-only
        let order = calculate_variant_order(&["unknown-variant"]);
        assert!(order > 0, "unknown variants should have non-zero order");
        assert!(
            order & ARBITRARY_VARIANT_BIT != 0,
            "unknown variants should set bit 63"
        );

        // mix of known and unknown SHOULD set bit 63 (has arbitrary)
        let mixed_order = calculate_variant_order(&["hover", "unknown", "focus"]);
        let known_order = calculate_variant_order(&["hover", "focus"]);
        // mixed should have known bits PLUS bit 63
        assert_eq!(
            mixed_order,
            known_order | ARBITRARY_VARIANT_BIT,
            "mixed order should equal known bits + arbitrary bit"
        );
        assert!(
            mixed_order & ARBITRARY_VARIANT_BIT != 0,
            "mixed order should have bit 63 set"
        );
    }

    #[test]
    fn test_arbitrary_variants_sort_last() {
        // pure arbitrary variants like [&.htmx-request] should sort AFTER all known-only variants
        let arbitrary_order = calculate_variant_order(&["[&.htmx-request]"]);
        let dark_order = calculate_variant_order(&["dark"]);
        let print_order = calculate_variant_order(&["print"]); // highest known variant

        // pure arbitrary should be greater than any known-only variant
        assert!(
            arbitrary_order > dark_order,
            "pure arbitrary variants should sort after dark"
        );
        assert!(
            arbitrary_order > print_order,
            "pure arbitrary variants should sort after print"
        );

        // pure arbitrary variants should have bit 63 set
        assert!(arbitrary_order & ARBITRARY_VARIANT_BIT != 0);

        // different pure arbitrary variants should all sort after known-only variants
        let arbitrary2_order = calculate_variant_order(&["[&>*]"]);
        let arbitrary3_order = calculate_variant_order(&["[@supports(display:grid)]"]);
        assert!(arbitrary2_order > print_order);
        assert!(arbitrary3_order > print_order);

        // mixed variants (known + arbitrary) SHOULD have bit 63 set
        // they sort AFTER known-only, but pure arbitrary sorts BEFORE mixed
        // because pure has only bit 63, while mixed has bit 63 + known bits
        let focus_order = calculate_variant_order(&["focus"]);
        let mixed_order = calculate_variant_order(&["focus", "[&.collapsed]"]);

        // mixed SHOULD have bit 63 (has arbitrary variant)
        assert!(
            mixed_order & ARBITRARY_VARIANT_BIT != 0,
            "mixed variants should have bit 63 set"
        );
        // mixed should have known bits + bit 63
        assert_eq!(
            mixed_order,
            focus_order | ARBITRARY_VARIANT_BIT,
            "mixed order should equal focus bit + arbitrary bit"
        );
        // pure arbitrary (just bit 63) should sort BEFORE mixed (bit 63 + focus bit)
        assert!(
            arbitrary_order < mixed_order,
            "pure arbitrary should sort before mixed (2^63 < 2^63 + 2^focus)"
        );
    }

    #[test]
    fn test_base_classes_sort_first() {
        // classes without variants should have order 0
        let base_order = calculate_variant_order(&[]);
        let hover_order = calculate_variant_order(&["hover"]);

        // base order should be less than any variant order
        assert!(base_order < hover_order);
    }

    #[test]
    fn test_high_index_variants() {
        // test variants at higher indices to ensure they work correctly
        // dark is at index 56, portrait at 52, print at 57

        // get the actual indices
        let dark_idx = get_variant_index("dark").unwrap();
        let portrait_idx = get_variant_index("portrait").unwrap();
        let print_idx = get_variant_index("print").unwrap();

        // verify expected indices
        assert_eq!(dark_idx, 56, "dark should be at index 56");
        assert_eq!(portrait_idx, 52, "portrait should be at index 52");
        assert_eq!(print_idx, 57, "print should be at index 57");

        // calculate variant orders - these should NOT be 0
        let dark_order = calculate_variant_order(&["dark"]);
        let portrait_order = calculate_variant_order(&["portrait"]);
        let print_order = calculate_variant_order(&["print"]);

        // all should have non-zero variant order
        assert!(dark_order > 0, "dark should have non-zero variant order");
        assert!(
            portrait_order > 0,
            "portrait should have non-zero variant order"
        );
        assert!(print_order > 0, "print should have non-zero variant order");

        // they should all have different orders
        assert_ne!(dark_order, portrait_order);
        assert_ne!(dark_order, print_order);
        assert_ne!(portrait_order, print_order);

        // base classes should still have order 0
        let base_order = calculate_variant_order(&[]);
        assert_eq!(base_order, 0);

        // all variant orders should be greater than base order
        assert!(dark_order > base_order);
        assert!(portrait_order > base_order);
        assert!(print_order > base_order);
    }

    #[test]
    fn test_dark_variant_order() {
        // specific test for the dark variant - critical for dark:placeholder sorting
        let dark_order = calculate_variant_order(&["dark"]);
        let hover_order = calculate_variant_order(&["hover"]);
        let base_order = calculate_variant_order(&[]);

        // dark should have a different order than hover
        assert_ne!(dark_order, hover_order);

        // both should be greater than base order (0)
        assert!(dark_order > base_order);
        assert!(hover_order > base_order);

        // dark (index 56) should come after hover (index 37)
        assert!(dark_order > hover_order);
    }

    #[test]
    fn test_compound_variants() {
        // test that compound variants use ONLY the base part for ordering
        // this is critical for proper sorting where peer-hover sorts at peer's position (index 2)
        let peer_hover_order = calculate_variant_order(&["peer-hover"]);
        let peer_order = calculate_variant_order(&["peer"]);

        // peer-hover should equal peer (not peer | hover)
        // this makes it sort at peer's early position (index 2), not hover's later position (index 37)
        assert_eq!(
            peer_hover_order, peer_order,
            "peer-hover should sort at peer's position"
        );

        // test group-focus
        let group_focus_order = calculate_variant_order(&["group-focus"]);
        let group_order = calculate_variant_order(&["group"]);

        assert_eq!(
            group_focus_order, group_order,
            "group-focus should sort at group's position"
        );

        // test multi-dash compound (peer-focus-within)
        let peer_focus_within_order = calculate_variant_order(&["peer-focus-within"]);

        assert_eq!(
            peer_focus_within_order, peer_order,
            "peer-focus-within should sort at peer's position"
        );

        // test that compound variants sort correctly relative to simple variants
        // peer-hover uses peer's index (2), so it sorts BEFORE after (index 11)
        let after_order = calculate_variant_order(&["after"]);
        assert!(
            peer_hover_order < after_order,
            "peer-hover (index 2) should sort before after (index 11)"
        );

        // peer-hover also sorts before dark (index 56)
        let dark_order = calculate_variant_order(&["dark"]);
        assert!(
            peer_hover_order < dark_order,
            "peer-hover (index 2) should sort before dark (index 56)"
        );

        // but peer-hover sorts after group (index 1) since peer is at index 2
        let group_hover_order = calculate_variant_order(&["group-hover"]);
        assert!(
            group_hover_order < peer_hover_order,
            "group-hover (index 1) should sort before peer-hover (index 2)"
        );
    }

    #[test]
    fn test_all_variants_have_unique_nonzero_order() {
        // this test would have caught the u64 overflow bug!
        // it verifies that EVERY variant in VARIANT_ORDER has a unique,
        // non-zero variant order.

        use std::collections::HashSet;

        let base_order = calculate_variant_order(&[]);
        assert_eq!(base_order, 0, "Base order should be 0");

        let mut seen_orders = HashSet::new();
        seen_orders.insert(base_order);

        for (idx, variant) in VARIANT_ORDER.iter().enumerate() {
            let order = calculate_variant_order(&[variant]);

            // CRITICAL: every variant must have non-zero order
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

        // Verify we have unique orders for all 58 variants + base (0)
        assert_eq!(
            seen_orders.len(),
            VARIANT_ORDER.len() + 1,
            "Should have {} unique orders ({} variants + base)",
            VARIANT_ORDER.len() + 1,
            VARIANT_ORDER.len()
        );
    }
}

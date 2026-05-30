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
//! Compound variants like `peer-hover`, `group-focus`, and `not-hover` require
//! special handling. They are compared recursively: first by their base, then by
//! their modifier. This matches Tailwind's behavior where `peer-hover` comes
//! before `peer-focus` because `hover` comes before `focus`.

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
    // Tailwind's variant order, aligned with the current Prettier plugin defaults
    "read-write",        // 0
    "*",                 // 1
    "**",                // 2
    "not",               // 3
    "not-sm",            // 4
    "not-md",            // 5
    "not-lg",            // 6
    "not-xl",            // 7
    "not-2xl",           // 8
    "not-dark",          // 9
    "group",             // 10
    "peer",              // 11
    "first-letter",      // 12
    "first-line",        // 13
    "marker",            // 14
    "selection",         // 15
    "file",              // 16
    "placeholder",       // 17
    "backdrop",          // 18
    "before",            // 19
    "after",             // 20
    "first",             // 21
    "last",              // 22
    "only",              // 23
    "odd",               // 24
    "even",              // 25
    "first-of-type",     // 26
    "last-of-type",      // 27
    "only-of-type",      // 28
    "visited",           // 29
    "target",            // 30
    "open",              // 31
    "default",           // 32
    "checked",           // 33
    "indeterminate",     // 34
    "placeholder-shown", // 35
    "autofill",          // 36
    "optional",          // 37
    "required",          // 38
    "valid",             // 39
    "invalid",           // 40
    "in-range",          // 41
    "out-of-range",      // 42
    "read-only",         // 43
    "empty",             // 44
    "focus-within",      // 45
    "hover",             // 46
    "focus",             // 47
    "focus-visible",     // 48
    "active",            // 49
    "enabled",           // 50
    "disabled",          // 51
    "in",                // 52
    "has",               // 53
    "aria",              // 54
    "data",              // 55
    "nth",               // 56
    "nth-last",          // 57
    "nth-of-type",       // 58
    "nth-last-of-type",  // 59
    "motion-safe",       // 60
    "motion-reduce",     // 61
    "contrast-more",     // 62
    "contrast-less",     // 63
    "max-[]",            // 64
    "max-2xl",           // 65
    "max-xl",            // 66
    "max-lg",            // 67
    "max-md",            // 68
    "max-sm",            // 69
    "min-[]",            // 70
    "sm",                // 71
    "md",                // 72
    "lg",                // 73
    "xl",                // 74
    "2xl",               // 75
    "portrait",          // 76
    "landscape",         // 77
    "ltr",               // 78
    "rtl",               // 79
    "dark",              // 80
    "print",             // 81
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
    /// - "hover" -> VariantInfo { base: "hover", modifier: None }
    /// - "peer-hover" -> VariantInfo { base: "peer", modifier: Some("hover") }
    /// - "not-focus" -> VariantInfo { base: "not", modifier: Some("focus") }
    pub fn parse(variant: &str) -> Self {
        // check for compound variants (peer-*, group-*, not-*)
        if (variant.starts_with("peer-")
            || variant.starts_with("group-")
            || variant.starts_with("not-"))
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
        // - focus:dark: < dark:focus: (focus comes before dark)
        // - peer-hover: < peer-focus: (hover comes before focus)
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
                                (None, None) => {
                                    compare_dynamic_variant_bases(&self.base, &other.base)
                                }
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

fn compare_dynamic_variant_bases(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (dynamic_variant_sort_key(a), dynamic_variant_sort_key(b)) {
        (Some((a_kind, a_value)), Some((b_kind, b_value))) => {
            a_kind.cmp(&b_kind).then_with(|| a_value.cmp(b_value))
        }
        _ => {
            if a == b {
                Ordering::Equal
            } else {
                a.cmp(b)
            }
        }
    }
}

fn dynamic_variant_sort_key(variant: &str) -> Option<(u8, &str)> {
    if let Some(value) = variant.strip_prefix("nth-last-of-type-") {
        return Some(arbitrary_value_sort_key(value));
    }

    if let Some(value) = variant.strip_prefix("nth-of-type-") {
        return Some(arbitrary_value_sort_key(value));
    }

    if let Some(value) = variant.strip_prefix("nth-last-") {
        return Some(arbitrary_value_sort_key(value));
    }

    if let Some(value) = variant.strip_prefix("nth-") {
        return Some(arbitrary_value_sort_key(value));
    }

    if let Some(value) = variant.strip_prefix("data-") {
        return Some(arbitrary_value_sort_key(value));
    }

    if let Some(value) = variant.strip_prefix("aria-") {
        return Some(arbitrary_value_sort_key(value));
    }

    if let Some(value) = variant.strip_prefix("in-") {
        return Some(arbitrary_value_sort_key(value));
    }

    if let Some(value) = variant.strip_prefix("has-") {
        return Some(has_variant_sort_key(value));
    }

    None
}

fn arbitrary_value_sort_key(value: &str) -> (u8, &str) {
    if let Some(inner) = bracket_inner(value) {
        (1, inner)
    } else {
        (0, value)
    }
}

fn has_variant_sort_key(value: &str) -> (u8, &str) {
    let Some(inner) = bracket_inner(value) else {
        return (0, value);
    };

    let kind = match inner.as_bytes().first().copied() {
        Some(b'.') | Some(b'#') => 1,
        Some(b'+') => 2,
        Some(b'~') => 3,
        Some(b'[') => 4,
        _ => 5,
    };

    (kind, inner)
}

fn bracket_inner(value: &str) -> Option<&str> {
    value.strip_prefix('[')?.split(']').next()
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
/// assert_eq!(get_variant_index("group"), Some(10));
/// assert_eq!(get_variant_index("peer"), Some(11));
/// assert_eq!(get_variant_index("placeholder"), Some(17));
/// assert_eq!(get_variant_index("focus-within"), Some(45));
/// assert_eq!(get_variant_index("hover"), Some(46));
/// assert_eq!(get_variant_index("focus"), Some(47));
/// assert_eq!(get_variant_index("focus-visible"), Some(48));
/// assert_eq!(get_variant_index("sm"), Some(71));
/// assert_eq!(get_variant_index("dark"), Some(80));
/// assert_eq!(get_variant_index("unknown-variant"), None);
/// ```
#[inline]
pub fn get_variant_index(variant: &str) -> Option<usize> {
    let variant = variant.split_once('/').map_or(variant, |(base, _)| base);

    if let Some(index) = VARIANT_ORDER.iter().position(|&v| v == variant) {
        return Some(index);
    }

    if variant.starts_with("group-") {
        return VARIANT_ORDER.iter().position(|&v| v == "group");
    }

    if variant.starts_with("peer-") {
        return VARIANT_ORDER.iter().position(|&v| v == "peer");
    }

    if variant.starts_with("not-") {
        return VARIANT_ORDER.iter().position(|&v| v == "not");
    }

    if variant.starts_with("max-[") {
        return VARIANT_ORDER.iter().position(|&v| v == "max-[]");
    }

    if variant.starts_with("min-[") {
        return VARIANT_ORDER.iter().position(|&v| v == "min-[]");
    }

    if variant.starts_with("in-") {
        return VARIANT_ORDER.iter().position(|&v| v == "in");
    }

    if variant.starts_with("has-") {
        return VARIANT_ORDER.iter().position(|&v| v == "has");
    }

    if variant.starts_with("aria-") {
        return VARIANT_ORDER.iter().position(|&v| v == "aria");
    }

    if variant.starts_with("data-") {
        return VARIANT_ORDER.iter().position(|&v| v == "data");
    }

    if variant.starts_with("nth-last-of-type-") {
        return VARIANT_ORDER.iter().position(|&v| v == "nth-last-of-type");
    }

    if variant.starts_with("nth-of-type-") {
        return VARIANT_ORDER.iter().position(|&v| v == "nth-of-type");
    }

    if variant.starts_with("nth-last-") {
        return VARIANT_ORDER.iter().position(|&v| v == "nth-last");
    }

    if variant.starts_with("nth-") {
        return VARIANT_ORDER.iter().position(|&v| v == "nth");
    }

    None
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

    let (Some((a_head, a_tail)), Some((b_head, b_tail))) = (a.split_first(), b.split_first())
    else {
        return a.len().cmp(&b.len());
    };

    match compare_variant_base_order(a_head, b_head) {
        Ordering::Equal => {}
        other => return other,
    }

    if a.len() != b.len() {
        return a.len().cmp(&b.len());
    }

    match compare_variant_lists(a_tail, b_tail) {
        Ordering::Equal => {}
        other => return other,
    }

    a_head.cmp_variants(b_head)
}

fn compare_variant_base_order(a: &VariantInfo, b: &VariantInfo) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (get_variant_index(&a.base), get_variant_index(&b.base)) {
        (Some(a_idx), Some(b_idx)) => a_idx.cmp(&b_idx),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.base.cmp(&b.base),
    }
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
/// A high bit is set for ANY class with arbitrary/unknown variants.
/// This ensures the sorting order:
/// 1. Base classes (no variants) → variant_order = 0
/// 2. Known-only variants (e.g., hover:block) → variant_order = 2^(known_index)
/// 3. Classes with ANY arbitrary variant → variant_order has a high arbitrary bit set
///
/// Within classes with arbitrary variants:
/// - Pure arbitrary (e.g., [&.x]:block) = `ARBITRARY_VARIANT_BIT`
/// - Mixed (e.g., hover:[&.x]:block) = `ARBITRARY_VARIANT_BIT + 2^37`
/// - Since the pure arbitrary value has fewer known bits, it sorts before mixed
///
/// This matches Tailwind's algorithm where arbitrary variants sort AFTER non-arbitrary.
pub(crate) const ARBITRARY_VARIANT_BIT: u128 = 1u128 << 120;

pub fn calculate_variant_order(variants: &[&str]) -> u128 {
    if variants.is_empty() {
        return 0;
    }

    let mut order = 0u128;
    let mut has_arbitrary = false;

    for variant in variants {
        if let Some(idx) = get_variant_index(variant) {
            // known variant - set bit at its index
            if idx < 120 {
                order |= 1u128 << idx;
            }
        } else if variant.starts_with('[') {
            // arbitrary variant (e.g., [&.htmx-request], [&>*], [@supports...])
            has_arbitrary = true;
        } else if variant.contains('-') {
            // handle compound variants like "peer-hover", "group-focus", or "peer-focus-within"
            // CRITICAL: For compound variants, use ONLY the base part (peer, group) for sorting
            // The modifier (hover, focus) is used for tiebreaking elsewhere, not in bitwise order
            // this makes peer-hover sort at peer's position, not hover's position
            if let Some(dash_pos) = variant.find('-') {
                let first_part = &variant[..dash_pos];

                // only add the first part (base variant) to the order
                // this ensures peer-hover sorts near peer, not near hover
                if let Some(idx) = get_variant_index(first_part)
                    && idx < 120
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

    // set a high bit for any class with arbitrary variants
    // this ensures: hover:block sorts before [&.a]:block
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
        assert_eq!(VARIANT_ORDER.len(), 82);
    }

    #[test]
    fn test_get_variant_index() {
        // test critical early positions
        assert_eq!(get_variant_index("read-write"), Some(0));
        assert_eq!(get_variant_index("group"), Some(10));
        assert_eq!(get_variant_index("peer"), Some(11));

        // test pseudo-elements
        assert_eq!(get_variant_index("placeholder"), Some(17));
        assert_eq!(get_variant_index("before"), Some(19));
        assert_eq!(get_variant_index("after"), Some(20));

        // test interactive variants (order: focus-within, hover, focus, focus-visible, active)
        assert_eq!(get_variant_index("focus-within"), Some(45));
        assert_eq!(get_variant_index("hover"), Some(46));
        assert_eq!(get_variant_index("focus"), Some(47));
        assert_eq!(get_variant_index("focus-visible"), Some(48));
        assert_eq!(get_variant_index("active"), Some(49));

        // test enabled/disabled (enabled comes before disabled)
        assert_eq!(get_variant_index("enabled"), Some(50));
        assert_eq!(get_variant_index("disabled"), Some(51));

        // test responsive variants
        assert_eq!(get_variant_index("max-xl"), Some(66));
        assert_eq!(get_variant_index("min-[900px]"), Some(70));
        assert_eq!(get_variant_index("sm"), Some(71));
        assert_eq!(get_variant_index("md"), Some(72));
        assert_eq!(get_variant_index("lg"), Some(73));

        // test orientation (portrait before landscape)
        assert_eq!(get_variant_index("portrait"), Some(76));
        assert_eq!(get_variant_index("landscape"), Some(77));

        // test critical dark position
        assert_eq!(get_variant_index("dark"), Some(80));

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
        // unknown variants should set the arbitrary bit so they sort after known-only
        let order = calculate_variant_order(&["unknown-variant"]);
        assert!(order > 0, "unknown variants should have non-zero order");
        assert!(
            order & ARBITRARY_VARIANT_BIT != 0,
            "unknown variants should set the arbitrary bit"
        );

        // mix of known and unknown should set the arbitrary bit
        let mixed_order = calculate_variant_order(&["hover", "unknown", "focus"]);
        let known_order = calculate_variant_order(&["hover", "focus"]);
        // mixed should have known bits plus the arbitrary bit
        assert_eq!(
            mixed_order,
            known_order | ARBITRARY_VARIANT_BIT,
            "mixed order should equal known bits + arbitrary bit"
        );
        assert!(
            mixed_order & ARBITRARY_VARIANT_BIT != 0,
            "mixed order should have the arbitrary bit set"
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

        // pure arbitrary variants should have the arbitrary bit set
        assert!(arbitrary_order & ARBITRARY_VARIANT_BIT != 0);

        // different pure arbitrary variants should all sort after known-only variants
        let arbitrary2_order = calculate_variant_order(&["[&>*]"]);
        let arbitrary3_order = calculate_variant_order(&["[@supports(display:grid)]"]);
        assert!(arbitrary2_order > print_order);
        assert!(arbitrary3_order > print_order);

        // mixed variants (known + arbitrary) should have the arbitrary bit set
        // they sort AFTER known-only, but pure arbitrary sorts BEFORE mixed
        // because pure has only the arbitrary bit, while mixed also has known bits
        let focus_order = calculate_variant_order(&["focus"]);
        let mixed_order = calculate_variant_order(&["focus", "[&.collapsed]"]);

        // mixed should have the arbitrary bit
        assert!(
            mixed_order & ARBITRARY_VARIANT_BIT != 0,
            "mixed variants should have the arbitrary bit set"
        );
        // mixed should have known bits plus the arbitrary bit
        assert_eq!(
            mixed_order,
            focus_order | ARBITRARY_VARIANT_BIT,
            "mixed order should equal focus bit + arbitrary bit"
        );
        // pure arbitrary should sort before mixed arbitrary plus focus
        assert!(
            arbitrary_order < mixed_order,
            "pure arbitrary should sort before mixed arbitrary plus focus"
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
        // dark, portrait, and print are high enough to catch bit-width regressions

        // get the actual indices
        let dark_idx = get_variant_index("dark").unwrap();
        let portrait_idx = get_variant_index("portrait").unwrap();
        let print_idx = get_variant_index("print").unwrap();

        // verify expected indices
        assert_eq!(dark_idx, 80, "dark should be at index 80");
        assert_eq!(portrait_idx, 76, "portrait should be at index 76");
        assert_eq!(print_idx, 81, "print should be at index 81");

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

        // dark should come after hover
        assert!(dark_order > hover_order);
    }

    #[test]
    fn test_compound_variants() {
        // test that compound variants use their group/peer variant buckets
        let peer_hover_order = calculate_variant_order(&["peer-hover"]);
        let peer_order = calculate_variant_order(&["peer"]);

        assert_eq!(
            peer_hover_order, peer_order,
            "peer-hover should sort at peer position"
        );

        // test group-focus
        let group_focus_order = calculate_variant_order(&["group-focus"]);
        let group_order = calculate_variant_order(&["group"]);

        assert_eq!(
            group_focus_order, group_order,
            "group-focus should sort at group position"
        );

        // test multi-dash compound (peer-focus-within)
        let peer_focus_within_order = calculate_variant_order(&["peer-focus-within"]);

        assert_eq!(
            peer_focus_within_order, peer_order,
            "peer-focus-within should sort at peer position"
        );

        // test that compound variants sort correctly relative to simple variants
        // peer-hover sorts before first-letter and after not-dark in the official plugin
        let after_order = calculate_variant_order(&["after"]);
        assert!(
            peer_hover_order < after_order,
            "peer-hover should sort before after"
        );
        assert!(peer_hover_order > calculate_variant_order(&["not-dark"]));

        // peer-hover also sorts before dark
        let dark_order = calculate_variant_order(&["dark"]);
        assert!(
            peer_hover_order < dark_order,
            "peer-hover should sort before dark"
        );

        // but peer-hover sorts after group since peer is after group
        let group_hover_order = calculate_variant_order(&["group-hover"]);
        assert!(
            group_hover_order < peer_hover_order,
            "group-hover should sort before peer-hover"
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

        // verify we have unique orders for all variants plus base
        assert_eq!(
            seen_orders.len(),
            VARIANT_ORDER.len() + 1,
            "Should have {} unique orders ({} variants + base)",
            VARIANT_ORDER.len() + 1,
            VARIANT_ORDER.len()
        );
    }
}

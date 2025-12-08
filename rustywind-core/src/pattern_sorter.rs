//! Pattern-based sorting implementation matching Tailwind CSS v4's algorithm
//!
//! This module implements the core sorting logic that matches Tailwind's canonical
//! class ordering. It uses pattern matching rather than hardcoded lists to determine
//! the sort order of classes.
//!
//! # Algorithm
//!
//! Classes are sorted using a six-tier comparison:
//! 1. **Variant Order** - Classes without variants come first, then variants in order
//! 2. **Property Index** - Based on the CSS properties the utility generates
//! 3. **Numeric Value** - When both classes have numeric values (e.g., p-4 vs p-8)
//! 4. **Property Count** - More properties = earlier (multi-property utilities sort first)
//! 5. **Utility Prefix Priority** - space-* before gap-* when properties match
//! 6. **Alphabetical** - Final tiebreaker
//!
//! # Examples
//!
//! ```
//! use rustywind_core::pattern_sorter::sort_classes;
//!
//! let classes = vec!["focus:hover:p-3", "hover:p-1", "m-4", "p-4"];
//! let sorted = sort_classes(&classes);
//!
//! // Base classes first (margin before padding), then variants
//! assert_eq!(sorted, vec!["m-4", "p-4", "hover:p-1", "focus:hover:p-3"]);
//! ```

use std::cmp::Ordering;

use crate::class_parser::parse_class;
use crate::property_order::get_property_index;
use crate::variant_order::{
    VariantInfo, calculate_variant_order, compare_variant_lists, parse_variants,
};

/// Check if a variant chain contains bare group/peer variants (without modifiers).
/// In Tailwind CSS, group and peer must be used as compound variants (e.g., group-hover, peer-focus).
/// Bare group: or peer: without modifiers are invalid and should sort first (matching Prettier's behavior).
fn has_bare_group_or_peer(variant_chain: &[VariantInfo]) -> bool {
    variant_chain
        .iter()
        .any(|v| (v.base == "group" || v.base == "peer") && v.modifier.is_none())
}

/// Compare two strings alphanumerically (like Tailwind CSS does).
/// Numbers within strings are compared numerically rather than lexicographically.
fn compare_alphanumeric(a: &str, z: &str) -> Ordering {
    let a_bytes = a.as_bytes();
    let z_bytes = z.as_bytes();
    let min_len = a.len().min(z.len());

    let mut i = 0;
    while i < min_len {
        let a_char = a_bytes[i];
        let z_char = z_bytes[i];

        // if both are digits, compare them as numbers
        if a_char.is_ascii_digit() && z_char.is_ascii_digit() {
            // find the end of the number in both strings
            let mut a_end = i + 1;
            while a_end < a.len() && a_bytes[a_end].is_ascii_digit() {
                a_end += 1;
            }

            let mut z_end = i + 1;
            while z_end < z.len() && z_bytes[z_end].is_ascii_digit() {
                z_end += 1;
            }

            // parse and compare numerically
            if let (Ok(a_num), Ok(z_num)) = (a[i..a_end].parse::<i64>(), z[i..z_end].parse::<i64>())
            {
                match a_num.cmp(&z_num) {
                    Ordering::Equal => {
                        i = a_end.max(z_end);
                        continue;
                    }
                    other => return other,
                }
            }

            // fallback to string comparison if parsing fails
            match a[i..a_end].cmp(&z[i..z_end]) {
                Ordering::Equal => {
                    i = a_end.max(z_end);
                    continue;
                }
                other => return other,
            }
        }

        // compare characters
        match a_char.cmp(&z_char) {
            Ordering::Equal => {
                i += 1;
                continue;
            }
            other => return other,
        }
    }

    // shorter string comes first
    a.len().cmp(&z.len())
}

/// Extract the base name from a utility class, removing size modifiers.
///
/// This function extracts the base name for utilities with size modifiers:
/// - `rounded-t-lg` → `rounded-t`
/// - `rounded-tl-none` → `rounded-tl`
/// - `rounded-t` → `rounded-t`
///
/// This is used for proper alphabetical comparison when properties match.
fn extract_base_name(utility: &str) -> &str {
    // strip variants first to get just the utility part
    let utility_base = utility.split(':').next_back().unwrap_or(utility);

    // extract base for rounded utilities
    if let Some(after_rounded) = utility_base.strip_prefix("rounded-") {
        let parts: Vec<&str> = after_rounded.split('-').collect();
        if parts.len() >= 2 {
            // check if first part is a side or corner indicator
            match parts[0] {
                "t" | "r" | "b" | "l" | "s" | "e" => {
                    return &utility[..("rounded-".len() + parts[0].len())];
                }
                "tl" | "tr" | "br" | "bl" | "ss" | "se" | "ee" | "es" => {
                    return &utility[..("rounded-".len() + parts[0].len())];
                }
                _ => {}
            }
        }
    }

    utility // Return full name if no modifier
}

///
/// This function extracts numeric values from utilities like:
/// - `p-4` → Some(4.0)
/// - `scale-110` → Some(110.0)
/// - `w-1/2` → Some(0.5)
/// - `text-lg` → None
///
/// Utilities with the same property are sorted by their numeric value when available.
fn extract_numeric_value(utility: &str) -> Option<f64> {
    // remove variants to get just the utility part
    let utility = utility.split(':').next_back()?;

    // handle arbitrary values first (e.g., h-[120px], bg-white/30, max-w-[485px])
    // check for brackets [...] or opacity /number
    if let Some(bracket_start) = utility.find('[')
        && let Some(bracket_end) = utility.find(']')
    {
        // extract content within brackets: h-[120px] -> "120px"
        let value_str = &utility[bracket_start + 1..bracket_end];

        // try to extract number from the start of the string
        // handles: "120px", "2rem", "0.5", "50%", etc.
        let mut num_str = String::new();
        let mut seen_dot = false;

        for ch in value_str.chars() {
            if ch.is_numeric() {
                num_str.push(ch);
            } else if ch == '.' && !seen_dot {
                num_str.push(ch);
                seen_dot = true;
            } else {
                // stop at first non-numeric, non-dot character
                break;
            }
        }

        if let Ok(num) = num_str.parse::<f64>() {
            return Some(num);
        }
    }

    // handle opacity syntax: bg-white/30 -> extract 30
    // distinguish from fractions like w-1/2
    if let Some(slash_pos) = utility.rfind('/') {
        let after_slash = &utility[slash_pos + 1..];
        let before_slash = &utility[..slash_pos];

        // count dashes to distinguish opacity from fractions:
        // - bg-blue-500/75 (2 dashes) = color-shade/opacity
        // - bg-white/30 (1 dash, non-numeric last part) = color/opacity
        // - w-1/2 (1 dash, numeric last part) = utility-fraction
        let dash_count = before_slash.matches('-').count();

        if dash_count >= 2 {
            // multiple dashes before slash = color-shade/opacity like bg-blue-500/75
            if let Ok(num) = after_slash.parse::<f64>() {
                return Some(num);
            }
        } else if dash_count == 1 {
            // single dash: check if last part is a number
            let parts: Vec<&str> = before_slash.split('-').collect();
            if let Some(last_part) = parts.last() {
                // if last part is NOT a number, it's opacity like bg-white/30
                // if last part IS a number, it's a fraction like w-1/2 - skip to fraction logic
                if last_part.parse::<f64>().is_err()
                    && let Ok(num) = after_slash.parse::<f64>()
                {
                    return Some(num);
                }
            }
        }
    }

    // split by dash to get potential numeric parts
    let parts: Vec<&str> = utility.split('-').collect();

    // look for the last part which is usually the value
    let value_part = parts.last()?;

    // handle negative values (e.g., -translate-x-4 → value is "4" with negative prefix)
    let (_is_negative, value_str) = if parts.len() > 1 && parts[0].is_empty() {
        // negative utility like -translate-x-4
        (true, value_part)
    } else {
        (false, value_part)
    };

    // try to parse as integer
    if let Ok(num) = value_str.parse::<i32>() {
        return Some(num as f64);
    }

    // try to parse as fraction (e.g., "1/2") - check this BEFORE extracting leading digits
    // this ensures w-1/2 returns 0.5, not 1.0
    if let Some((numerator_str, denominator_str)) = value_str.split_once('/')
        && let (Ok(numerator), Ok(denominator)) =
            (numerator_str.parse::<f64>(), denominator_str.parse::<f64>())
        && denominator != 0.0
    {
        let result = numerator / denominator;
        return Some(result);
    }

    // try to extract leading digits from values like "4xl", "2xl", etc.
    // this allows numeric comparison between max-w-4xl and max-w-[485px]
    if !value_str.is_empty() {
        let mut num_str = String::new();
        for ch in value_str.chars() {
            if ch.is_numeric() {
                num_str.push(ch);
            } else {
                break; // Stop at first non-numeric character
            }
        }
        if !num_str.is_empty()
            && let Ok(num) = num_str.parse::<f64>()
        {
            return Some(num);
        }
    }

    // try to parse as decimal (e.g., "0.5")
    if let Ok(num) = value_str.parse::<f64>() {
        return Some(num);
    }

    None
}

/// A sort key for a Tailwind CSS class.
///
/// This struct encapsulates all the information needed to sort a class according
/// to Tailwind's algorithm. It implements `Ord` to provide the exact comparison
/// logic used by Tailwind CSS.
#[derive(Debug, Clone, PartialEq)]
pub struct SortKey {
    /// Variant order as bitwise flags (0 for no variants)
    pub variant_order: u128,

    /// Structured variant information for recursive comparison
    /// This is used to properly sort compound variants like peer-hover vs peer-focus
    pub variant_chain: Vec<VariantInfo>,

    /// Arbitrary variant selectors for tiebreaking when variant_order is equal
    /// e.g., for `[&.x]:block` this would be `["[&.x]"]`
    /// Used to sort different arbitrary variants lexicographically (with `_` decoded as space)
    pub arbitrary_variants: Vec<compact_str::CompactString>,

    /// Property indices from PROPERTY_ORDER (lower = earlier)
    /// When utilities have multiple properties (e.g., rounded-t), ALL property indices
    /// are stored and compared in order for proper tiebreaking.
    pub property_indices: Vec<usize>,

    /// Numeric value for value-based sub-sorting (e.g., p-4 → 4.0)
    /// Classes with the same property are sorted by numeric value when available
    pub numeric_value: Option<f64>,

    /// Whether this utility has a negative value (e.g., -rotate-1, -translate-x-4)
    /// Negative values sort BEFORE positive values for the same utility
    pub is_negative: bool,

    /// Number of properties this utility generates
    pub property_count: usize,

    /// Original class string (for alphabetical tiebreaker)
    /// Uses CompactString for memory efficiency (24 bytes inline, no heap for typical classes)
    pub class: compact_str::CompactString,

    /// Whether this class is unparseable (e.g., bare group:/peer: without modifiers)
    /// Unparseable classes sort first (matching Prettier's behavior)
    pub is_unparseable: bool,
}

impl Eq for SortKey {}

/// Check if a class contains an arbitrary value (e.g., h-[120px], border-[1.5px])
fn has_arbitrary_value(class: &str) -> bool {
    class.contains('[') && class.contains(']')
}

/// Check if a utility uses opacity syntax (has a slash like bg-white/20)
/// Returns true for classes like: bg-white/20, text-black/75, border-gray-500/50
/// Returns false for fractions like: w-1/4, h-1/2 (these are not opacity)
fn has_opacity_syntax(class: &str) -> bool {
    // strip variants to get the utility part
    let utility = class.split(':').next_back().unwrap_or(class);

    if let Some(slash_pos) = utility.rfind('/') {
        let before_slash = &utility[..slash_pos];

        // count dashes to distinguish opacity from fractions:
        // - bg-blue-500/75 (2 dashes) = color-shade/opacity
        // - bg-white/30 (1 dash, non-numeric last part) = color/opacity
        // - w-1/4 (1 dash, numeric last part) = utility-fraction
        let dash_count = before_slash.matches('-').count();

        if dash_count >= 2 {
            // multiple dashes before slash = color-shade/opacity like bg-blue-500/75
            return true;
        } else if dash_count == 1 {
            // single dash: check if last part before slash is a number
            let parts: Vec<&str> = before_slash.split('-').collect();
            if let Some(last_part) = parts.last() {
                // if last part is NOT a number, it's opacity like bg-white/30
                // if last part IS a number, it's a fraction like w-1/4
                return last_part.parse::<f64>().is_err();
            }
        }
    }

    false
}

/// Extract the base number for width/height utilities to enable proper grouping
///
/// For Tailwind width/height classes, the "base number" is:
/// - For fractions (w-1/2): the numerator (1)
/// - For whole numbers (w-2): the number itself (2)
///
/// This enables grouping: w-1, w-1/2, w-1/3 all have base 1
///
/// Returns (base_number, denominator) where denominator is None for whole numbers
fn extract_base_number(class: &str) -> Option<(i32, Option<i32>)> {
    // strip variants to get the utility part
    let utility = class.split(':').next_back().unwrap_or(class);

    // only process width/height utilities with numeric values
    if !utility.starts_with("w-") && !utility.starts_with("h-") {
        return None;
    }

    // split by dash to get the value part
    let parts: Vec<&str> = utility.split('-').collect();
    if parts.len() < 2 {
        return None;
    }

    let value_part = parts.last()?;

    // check if it's a fraction
    if let Some((numerator_str, denominator_str)) = value_part.split_once('/') {
        // it's a fraction like w-1/2
        let numerator = numerator_str.parse::<i32>().ok()?;
        let denominator = denominator_str.parse::<i32>().ok()?;
        Some((numerator, Some(denominator)))
    } else {
        // it's a whole number like w-2
        let number = value_part.parse::<i32>().ok()?;
        Some((number, None))
    }
}

/// Check if a utility class has a negative value (e.g., -rotate-1, -translate-x-4)
/// Returns true for classes like: -rotate-1, -skew-y-3, -translate-x-4
/// Returns false for positive values: rotate-0, skew-y-1, translate-x-2
fn is_negative_value(class: &str) -> bool {
    // strip variants first to get just the utility part
    let utility = class.split(':').next_back().unwrap_or(class);

    // check if the utility starts with a dash followed by a letter
    // this handles cases like: -rotate-1, -translate-x-4, -skew-y-3
    // but not arbitrary values like: [--spacing-4] or bg-[#fff]
    if let Some(rest) = utility.strip_prefix('-') {
        // make sure it's not an arbitrary value or a regular dash in a color name
        // negative utilities start with dash followed by a letter (e.g., -rotate, -translate)
        rest.chars().next().is_some_and(|c| c.is_alphabetic())
    } else {
        false
    }
}

/// Extract the color name from a Tailwind color utility.
///
/// Examples:
/// - `bg-blue-500` → Some("blue")
/// - `text-red-50` → Some("red")
/// - `border-gray-500/50` → Some("gray")
/// - `bg-white` → Some("white")
/// - `p-4` → None (not a color utility)
///
/// This is used to ensure colors sort alphabetically by color name first,
/// then by shade number when color names match (matching Prettier's behavior).
fn extract_color_name(utility: &str) -> Option<&str> {
    // strip variants first to get just the utility part
    let utility_base = utility.split(':').next_back().unwrap_or(utility);

    // remove opacity suffix if present (e.g., bg-blue-500/50 → bg-blue-500)
    let utility_without_opacity = utility_base.split('/').next().unwrap_or(utility_base);

    // known Tailwind color names (in alphabetical order)
    const COLOR_NAMES: &[&str] = &[
        "amber",
        "black",
        "blue",
        "current",
        "cyan",
        "emerald",
        "fuchsia",
        "gray",
        "green",
        "indigo",
        "inherit",
        "lime",
        "neutral",
        "orange",
        "pink",
        "purple",
        "red",
        "rose",
        "sky",
        "slate",
        "stone",
        "teal",
        "transparent",
        "violet",
        "white",
        "yellow",
        "zinc",
    ];

    // color utilities follow patterns like:
    // bg-{color}-{shade}, text-{color}-{shade}, border-{color}-{shade}, etc.
    // or: bg-{color} (for white, black, transparent, etc.)

    // split by dash to extract parts
    let parts: Vec<&str> = utility_without_opacity.split('-').collect();

    // need at least 2 parts: prefix-color or prefix-color-shade
    if parts.len() < 2 {
        return None;
    }

    // check common color property prefixes
    let color_prefixes = &[
        "bg",
        "text",
        "border",
        "ring",
        "divide",
        "outline",
        "decoration",
        "accent",
        "caret",
        "fill",
        "stroke",
        "shadow",
        "from",
        "via",
        "to",
    ];

    if color_prefixes.contains(&parts[0]) {
        // second part should be the color name
        let potential_color = parts[1];

        // check if it's a known color name
        if COLOR_NAMES.contains(&potential_color) {
            return Some(potential_color);
        }
    }

    None
}

/// Check if a utility property should have arbitrary values sort BEFORE regular values
///
/// Tailwind/Prettier uses property-specific ordering:
/// - max-*, w, h, size, rounded, leading: arbitrary BEFORE keyword (more specific first)
/// - min-*, spacing, text, etc.: keyword BEFORE arbitrary (semantic first)
fn should_arbitrary_come_first(class: &str) -> bool {
    // strip variants to get the base utility
    let utility = class.split(':').next_back().unwrap_or(class);

    // properties where arbitrary values come BEFORE regular values
    utility.starts_with("max-w-")
        || utility.starts_with("max-h-")
        || (utility.starts_with("w-") && !utility.starts_with("will-"))
        || (utility.starts_with("h-") && !utility.starts_with("hue-"))
        || utility.starts_with("size-")
        || utility.starts_with("rounded-")
        || utility.starts_with("leading-")
        || utility.starts_with("z-")
        // spacing utilities: margin, padding, gap, space
        || utility.starts_with("m-") || utility.starts_with("mx-") || utility.starts_with("my-")
        || utility.starts_with("mt-") || utility.starts_with("mr-") || utility.starts_with("mb-") || utility.starts_with("ml-")
        || utility.starts_with("ms-") || utility.starts_with("me-")
        || utility.starts_with("p-") || utility.starts_with("px-") || utility.starts_with("py-")
        || utility.starts_with("pt-") || utility.starts_with("pr-") || utility.starts_with("pb-") || utility.starts_with("pl-")
        || utility.starts_with("ps-") || utility.starts_with("pe-")
        || utility.starts_with("gap-") || utility.starts_with("gap-x-") || utility.starts_with("gap-y-")
        || utility.starts_with("space-x-") || utility.starts_with("space-y-")
}

/// Get the utility prefix priority for tiebreaking when properties match.
///
/// This handles cases where utilities map to the same CSS property but represent
/// different semantic concepts (e.g., space-x and gap-y both map to row-gap).
///
/// Lower number = higher priority (sorts first)
/// - space-* utilities get priority 1 (sort first)
/// - gap-* utilities get priority 2 (sort after space-*)
/// - all other utilities get priority 100 (default)
fn get_utility_prefix_priority(utility: &str) -> u32 {
    // extract the base utility name without variants
    let utility_base = utility.split(':').next_back().unwrap_or(utility);

    if utility_base.starts_with("space-") {
        return 1;
    }
    if utility_base.starts_with("gap-") {
        return 2;
    }
    100 // default for other utilities
}

impl Ord for SortKey {
    /// Compare sort keys using Tailwind's exact algorithm with value-based sub-sorting.
    ///
    /// Order of comparison:
    /// 1. Unparseable classes (bare group:/peer:) sort first
    /// 2. Base classes (no variants) sort next
    /// 3. Coarse variant order (bitwise OR of variant indices)
    /// 4. Fine-grained variant chain comparison (recursive, handles multi-level variants)
    /// 5. Property indices (compare ALL properties in order for proper tiebreaking)
    /// 6. Utility prefix priority (space-* before gap-* when properties match)
    /// 7. Property count (MORE properties first - utilities with more properties sort earlier)
    /// 8. Color name alphabetical (for color utilities)
    /// 9. Negative value priority (negatives before positives)
    /// 10. Numeric value (when both present - lower value first, e.g., p-4 before p-8)
    /// 11. Alphabetical (final tiebreaker)
    fn cmp(&self, other: &Self) -> Ordering {
        // 1. unparseable classes sort FIRST (before everything else)
        //    when BOTH are unparseable, continue with normal comparison but skip base class check
        match (self.is_unparseable, other.is_unparseable) {
            (true, false) => return Ordering::Less, // unparseable before parseable
            (false, true) => return Ordering::Greater, // parseable after unparseable
            (true, true) => {
                // both unparseable: use normal comparison (variant_order, then variant_chain, then properties)
                // this replaces the previous alphabetical comparison
                return self
                    .variant_order
                    .cmp(&other.variant_order)
                    .then_with(|| compare_variant_lists(&self.variant_chain, &other.variant_chain))
                    .then_with(|| {
                        for (a_idx, b_idx) in self
                            .property_indices
                            .iter()
                            .zip(other.property_indices.iter())
                        {
                            match a_idx.cmp(b_idx) {
                                Ordering::Equal => continue,
                                other => return other,
                            }
                        }
                        other
                            .property_indices
                            .len()
                            .cmp(&self.property_indices.len())
                    })
                    .then_with(|| compare_alphanumeric(&self.class, &other.class));
            }
            (false, false) => {} // both parseable, continue with normal comparison
        }

        // 2. base classes (variant_order=0) come first
        match (self.variant_order == 0, other.variant_order == 0) {
            (true, false) => return Ordering::Less, // base class before variant
            (false, true) => return Ordering::Greater, // variant after base class
            (true, true) => {} // both base classes, continue to property comparison
            (false, false) => {
                // both have variants - continue with comparison below
            }
        }

        // bit 63 indicates presence of arbitrary variants
        const ARBITRARY_BIT: u128 = 1u128 << 63;
        let self_has_arbitrary = self.variant_order & ARBITRARY_BIT != 0;
        let other_has_arbitrary = other.variant_order & ARBITRARY_BIT != 0;

        // 2. compare by arbitrary variant presence and selectors
        // classes without arbitrary variants sort BEFORE classes with arbitrary variants
        // when both have arbitrary, compare selectors FIRST, then known variant bits
        match (self_has_arbitrary, other_has_arbitrary) {
            (false, true) => return Ordering::Less, // no arbitrary before arbitrary
            (true, false) => return Ordering::Greater,
            (true, true) => {
                // both have arbitrary variants - compare selectors FIRST
                let decode = |s: &str| s.replace('_', " ");
                let a: Vec<_> = self.arbitrary_variants.iter().map(|s| decode(s)).collect();
                let b: Vec<_> = other.arbitrary_variants.iter().map(|s| decode(s)).collect();
                match a.cmp(&b) {
                    Ordering::Equal => {
                        // same arbitrary selectors - compare known variant bits
                        // (mask out the arbitrary bit for comparison)
                        let self_known = self.variant_order & !ARBITRARY_BIT;
                        let other_known = other.variant_order & !ARBITRARY_BIT;
                        if self_known != other_known {
                            return self_known.cmp(&other_known);
                        }
                        // fall through to fine-grained comparison
                    }
                    other => return other,
                }
            }
            (false, false) => {
                // neither has arbitrary - compare by known variant bits
                if self.variant_order != other.variant_order {
                    return self.variant_order.cmp(&other.variant_order);
                }
                // fall through to fine-grained comparison
            }
        }

        // 3. fine-grained recursive variant chain comparison
        // when coarse variant_order ties, compare the actual variant chains
        // this handles multi-level variants like focus:dark: vs dark:focus:
        compare_variant_lists(&self.variant_chain, &other.variant_chain)
            // then compare by property indices - compare ALL properties in order
            // this is crucial for utilities like rounded-t vs rounded-l that tie on first property
            .then_with(|| {
                (|| {
                    for (a_idx, b_idx) in self
                        .property_indices
                        .iter()
                        .zip(other.property_indices.iter())
                    {
                        match a_idx.cmp(b_idx) {
                            Ordering::Equal => continue, // tie on this property, check next
                            other => return other,       // found difference
                        }
                    }
                    // all common properties are equal, compare by length (MORE properties = earlier)
                    other
                        .property_indices
                        .len()
                        .cmp(&self.property_indices.len())
                })()
            })
            // CRITICAL FIX: when property indices match, check utility prefix priority
            // this fixes space-x vs gap-y ordering (both map to row-gap, but space-* has priority)
            // must happen BEFORE numeric value comparison to prevent gap-y-0 sorting before space-x-4
            .then_with(|| {
                // only apply prefix priority when property indices are identical
                if self.property_indices == other.property_indices {
                    return get_utility_prefix_priority(&self.class)
                        .cmp(&get_utility_prefix_priority(&other.class));
                }
                Ordering::Equal
            })
            // then by property count (MORE properties = earlier, matching Tailwind v4)
            // Tailwind's: zSorting.properties.count - aSorting.properties.count
            // means if z (other) has MORE properties, result is positive, so a (self) comes first
            // therefore: compare other.count vs self.count (reversed)
            .then(other.property_count.cmp(&self.property_count))
            // then by color name alphabetically (when both are color utilities)
            // this ensures bg-blue-500 comes before bg-red-50 (blue < red alphabetically)
            // rather than sorting by shade number (50 < 500)
            .then_with(|| {
                match (
                    extract_color_name(&self.class),
                    extract_color_name(&other.class),
                ) {
                    (Some(self_color), Some(other_color)) => {
                        // both are color utilities - compare by color name first
                        self_color.cmp(other_color)
                    }
                    _ => Ordering::Equal, // at least one is not a color utility, continue
                }
            })
            // then handle negative value priority
            // negative values (-rotate-1, -skew-y-3) should sort BEFORE positive values
            .then_with(|| {
                match (self.is_negative, other.is_negative) {
                    (true, false) => Ordering::Less,    // negative before positive
                    (false, true) => Ordering::Greater, // positive after negative
                    _ => Ordering::Equal, // both negative or both positive, continue to numeric comparison
                }
            })
            // then handle numeric and arbitrary value comparison
            // CRITICAL FIX: check arbitrary status FIRST, before numeric comparison!
            // this fixes the fraction vs arbitrary ordering issue (Issue 2 from FAILURE_ANALYSIS.md)
            //
            // ordering rules:
            // 1. non-arbitrary numerics/fractions (w-1/2, w-4) come BEFORE arbitrary values (w-[50px])
            // 2. arbitrary values come before/after keywords based on property (should_arbitrary_come_first)
            // 3. within non-arbitrary numerics/fractions, sort by numeric value (w-0 < w-1/2 < w-4)
            // 4. within arbitrary values, sort by extracted numeric value (w-[10px] < w-[50px])
            //
            // examples:
            // - w-1/2 w-4 → w-1/2 w-4 (both non-arbitrary, compare numerically: 0.5 < 4)
            // - w-4 w-[50px] → w-4 w-[50px] (non-arbitrary before arbitrary, even though 4 < 50)
            // - w-2/3 w-[50px] → w-2/3 w-[50px] (fraction before arbitrary)
            // - z-40 z-[-1] → z-40 z-[-1] (non-arbitrary before arbitrary)
            // - w-full w-[50px] → w-[50px] w-full (for w-*, arbitrary before keyword)
            .then_with(|| {
                // check arbitrary and opacity status
                let self_has_arbitrary = has_arbitrary_value(&self.class);
                let other_has_arbitrary = has_arbitrary_value(&other.class);
                let self_has_opacity = has_opacity_syntax(&self.class);
                let other_has_opacity = has_opacity_syntax(&other.class);

                // FIRST: check arbitrary vs non-arbitrary status
                // fractions (w-1/2) are NOT arbitrary (no brackets)
                // numerics (w-4) are NOT arbitrary
                // arbitrary values (w-[50px]) ARE arbitrary (have brackets)
                match (self_has_arbitrary, other_has_arbitrary) {
                    (true, false) => {
                        // self is arbitrary, other is not
                        if other.numeric_value.is_some() {
                            // other has numeric value (fraction or numeric like w-4, w-1/2)
                            // non-arbitrary numerics/fractions ALWAYS come before arbitrary
                            return Ordering::Greater; // arbitrary AFTER non-arbitrary numeric
                        } else {
                            // other is a keyword (w-full, w-auto, etc.)
                            // use property-specific rule for arbitrary vs keyword ordering
                            if should_arbitrary_come_first(&self.class) {
                                return Ordering::Less; // arbitrary BEFORE keyword (e.g., w-[50px] before w-full)
                            } else {
                                return Ordering::Greater; // arbitrary AFTER keyword
                            }
                        }
                    }
                    (false, true) => {
                        // other is arbitrary, self is not
                        if self.numeric_value.is_some() {
                            // self has numeric value (fraction or numeric)
                            // non-arbitrary numerics/fractions ALWAYS come before arbitrary
                            return Ordering::Less; // non-arbitrary numeric BEFORE arbitrary
                        } else {
                            // self is a keyword
                            // use property-specific rule for keyword vs arbitrary ordering
                            if should_arbitrary_come_first(&other.class) {
                                return Ordering::Greater; // keyword AFTER arbitrary
                            } else {
                                return Ordering::Less; // keyword BEFORE arbitrary
                            }
                        }
                    }
                    _ => {
                        // both arbitrary OR both non-arbitrary - continue to numeric comparison
                    }
                }

                // SECOND: compare numeric values (for same arbitrary status)
                // this applies to:
                // 1. both non-arbitrary: fractions and numerics compared together (w-1/2 vs w-4)
                // 2. both arbitrary: compare extracted numeric values (w-[50px] vs w-[100px])
                // DON'T compare numerically if one has opacity syntax and the other doesn't
                match (self.numeric_value, other.numeric_value) {
                    (Some(a), Some(b)) => {
                        // only compare numerically if both have same opacity status
                        // this prevents comparing shade values (gray-500) with opacity values (white/20)
                        if self_has_opacity == other_has_opacity {
                            // check if both are width/height utilities with base numbers
                            let self_base = extract_base_number(&self.class);
                            let other_base = extract_base_number(&other.class);

                            match (self_base, other_base) {
                                (
                                    Some((self_base_num, self_denom)),
                                    Some((other_base_num, other_denom)),
                                ) => {
                                    // both have base numbers (w-1, w-1/2, w-2, etc.)
                                    // rule 1: compare by base number first (ascending)
                                    // example: w-1/3 (base 1) before w-2 (base 2)
                                    if self_base_num != other_base_num {
                                        return self_base_num.cmp(&other_base_num);
                                    }

                                    // rule 2: within same base number, whole numbers before fractions
                                    // example: w-1 before w-1/2
                                    match (self_denom, other_denom) {
                                        (None, Some(_)) => return Ordering::Less, // whole before fraction
                                        (Some(_), None) => return Ordering::Greater, // fraction after whole
                                        (Some(self_d), Some(other_d)) => {
                                            // rule 3: both fractions with same numerator, sort by denominator ascending
                                            // example: w-1/2 (denom 2) before w-1/3 (denom 3)
                                            // smaller denominator = larger fraction value = comes first
                                            if self_d != other_d {
                                                return self_d.cmp(&other_d);
                                            }
                                        }
                                        (None, None) => {
                                            // both whole numbers with same base, equal
                                        }
                                    }
                                }
                                _ => {
                                    // at least one doesn't have a base number, fall back to standard numeric comparison
                                    match a.partial_cmp(&b).unwrap_or(Ordering::Equal) {
                                        Ordering::Equal => {
                                            // numeric values are equal, continue to next tier
                                        }
                                        ordering => return ordering, // different numeric values
                                    }
                                }
                            }
                        }
                        // different opacity status, continue to next tier
                    }
                    _ => {
                        // at least one doesn't have a numeric value, continue
                    }
                }

                Ordering::Equal // fall through to next comparison tier
            })
            // then by alphanumeric comparison for utilities with numeric values
            // (space-* prefix priority is handled here)
            .then_with(|| {
                match (self.numeric_value, other.numeric_value) {
                    (Some(_), Some(_)) => {
                        // first check prefix priority (space-* before gap-*)
                        let prefix_cmp = get_utility_prefix_priority(&self.class)
                            .cmp(&get_utility_prefix_priority(&other.class));
                        if prefix_cmp != Ordering::Equal {
                            return prefix_cmp;
                        }
                        // then use alphanumeric comparison of full class names
                        compare_alphanumeric(&self.class, &other.class)
                    }
                    // if only one has a numeric value, no preference (continue to next comparison)
                    _ => Ordering::Equal,
                }
            })
            // then by utility prefix priority (space-* before gap-* when properties match)
            .then_with(|| {
                get_utility_prefix_priority(&self.class)
                    .cmp(&get_utility_prefix_priority(&other.class))
            })
            // compare base names (extracts modifiers)
            .then_with(|| {
                let base_self = extract_base_name(&self.class);
                let base_other = extract_base_name(&other.class);
                base_self.cmp(base_other)
            })
            // finally alphabetically on full name
            .then(self.class.cmp(&other.class))
    }
}

impl PartialOrd for SortKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Pattern-based sorter for Tailwind CSS classes.
///
/// This struct provides methods to generate sort keys for classes and sort
/// collections of classes according to Tailwind's canonical ordering.
pub struct PatternSorter;

impl PatternSorter {
    /// Create a new pattern sorter.
    pub fn new() -> Self {
        Self
    }

    /// Get the sort key for a class string.
    ///
    /// Returns `None` if the class cannot be parsed or its properties are unknown.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustywind_core::pattern_sorter::PatternSorter;
    ///
    /// let sorter = PatternSorter::new();
    ///
    /// // Base class
    /// let key = sorter.get_sort_key("flex").unwrap();
    /// assert_eq!(key.variant_order, 0);
    ///
    /// // Class with variant
    /// let key = sorter.get_sort_key("md:flex").unwrap();
    /// assert!(key.variant_order > 0);
    /// ```
    pub fn get_sort_key(&self, class: &str) -> Option<SortKey> {
        // parse the class
        let parsed = parse_class(class)?;

        // calculate variant order using bitwise flags
        let variant_order = calculate_variant_order(&parsed.variants);

        // parse variants into structured form for recursive comparison
        let variant_chain = parse_variants(&parsed.variants);

        // extract arbitrary variants for lexicographic tiebreaking
        // these are variants that start with '[' (e.g., [&.htmx-request], [&>*])
        let arbitrary_variants: Vec<compact_str::CompactString> = parsed
            .variants
            .iter()
            .filter(|v| v.starts_with('['))
            .map(|v| compact_str::CompactString::new(*v))
            .collect();

        // get the CSS properties this utility generates
        let properties = parsed.get_properties()?;

        // get ALL property indices (not just minimum) for proper multi-property tiebreaking
        // this is crucial for utilities like rounded-t vs rounded-l that share the first property
        // but differ on the second property (e.g., border-top-left-radius ties, but
        // border-top-right-radius (190) < border-bottom-left-radius (192))
        let property_indices: Vec<usize> = properties
            .iter()
            .filter_map(|&prop| get_property_index(prop))
            .collect();

        // ensure we have at least one valid property index
        if property_indices.is_empty() {
            return None;
        }

        // count how many CSS declarations this utility generates
        // use the real declaration count from Tailwind (not just property count)
        let property_count = crate::utility_map::get_declaration_count(class);

        // extract numeric value for value-based sub-sorting
        let numeric_value = extract_numeric_value(class);

        // check if this is a negative value utility
        let is_negative = is_negative_value(class);

        // use CompactString for memory efficiency (24 bytes inline storage)
        // most Tailwind classes fit within 24 bytes avoiding heap allocation entirely
        let class_compact = compact_str::CompactString::new(class);

        // check if this class contains bare group/peer variants (invalid in Tailwind)
        let is_unparseable = has_bare_group_or_peer(&variant_chain);

        Some(SortKey {
            variant_order,
            variant_chain,
            arbitrary_variants,
            property_indices,
            numeric_value,
            is_negative,
            property_count,
            class: class_compact,
            is_unparseable,
        })
    }
}

impl Default for PatternSorter {
    fn default() -> Self {
        Self::new()
    }
}

/// Sort a list of Tailwind CSS classes according to the canonical ordering.
///
/// This function sorts classes using Tailwind's exact algorithm:
/// 1. Unknown/custom classes come first (sorted by variant order, then alphabetically)
/// 2. Known Tailwind base classes (no variants) come next
/// 3. Known classes with variants come after, sorted by variant order
/// 4. Within each group, sort by property order
/// 5. Tiebreak by property count, then alphabetically
///
/// Unknown classes are those that cannot be parsed or have unknown properties.
/// This matches the Prettier plugin behavior where getClassOrder() returns null
/// for unknown classes, which are sorted to the front.
///
/// # Examples
///
/// ```
/// use rustywind_core::pattern_sorter::sort_classes;
///
/// // Unknown/custom classes first, then known classes
/// let classes = vec!["flex", "custom-class", "p-4"];
/// let sorted = sort_classes(&classes);
/// assert_eq!(sorted, vec!["custom-class", "flex", "p-4"]);
///
/// // Base classes before variants (among known classes)
/// let classes = vec!["md:flex", "flex", "sm:grid", "grid"];
/// let sorted = sort_classes(&classes);
/// assert_eq!(sorted, vec!["flex", "grid", "sm:grid", "md:flex"]);
///
/// // Property order within base classes
/// let classes = vec!["p-4", "m-4"]; // margin before padding
/// let sorted = sort_classes(&classes);
/// assert_eq!(sorted, vec!["m-4", "p-4"]);
/// ```
pub fn sort_classes<'a>(classes: &[&'a str]) -> Vec<&'a str> {
    let sorter = PatternSorter::new();

    // generate sort keys for all classes
    // for unknown classes, we still need variant order for proper sorting
    let mut with_keys: Vec<(Option<SortKey>, u128, &str)> = classes
        .iter()
        .map(|&class| {
            let key = sorter.get_sort_key(class);
            // for unknown classes, calculate variant order manually
            let variant_order = if key.is_none() {
                if let Some(parsed) = parse_class(class) {
                    calculate_variant_order(&parsed.variants)
                } else {
                    0
                }
            } else {
                0 // not needed for known classes
            };
            (key, variant_order, class)
        })
        .collect();

    // sort by keys
    // classes without valid keys (unknown/custom) come first, sorted by variant order then alphabetically
    // classes with valid keys (known Tailwind utilities) come after, sorted by key
    // this matches prettier-plugin-tailwindcss behavior where getClassOrder() returns
    // null for unknown classes, which are sorted to the front.
    with_keys.sort_by(
        |(a_key, a_variant_order, a_class), (z_key, z_variant_order, z_class)| {
            match (a_key, z_key) {
                (Some(a), Some(z)) => a.cmp(z),
                (Some(_), None) => Ordering::Greater, // known classes after unknown
                (None, Some(_)) => Ordering::Less,    // unknown classes before known
                (None, None) => {
                    // unknown classes: sort by variant order first, then alphabetically
                    // lower variant order values come first (0 for no variants, then increasing)
                    a_variant_order
                        .cmp(z_variant_order)
                        .then_with(|| a_class.cmp(z_class))
                }
            }
        },
    );

    // extract the sorted classes
    with_keys.iter().map(|(_, _, class)| *class).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_classes_before_variants() {
        let classes = vec!["md:flex", "flex", "sm:grid", "grid"];
        let sorted = sort_classes(&classes);

        // base classes should come first
        assert_eq!(sorted[0], "flex");
        assert_eq!(sorted[1], "grid");
        // then variant classes
        assert!(sorted[2] == "sm:grid" || sorted[2] == "md:flex");
        assert!(sorted[3] == "sm:grid" || sorted[3] == "md:flex");
    }

    #[test]
    fn test_property_order() {
        let classes = vec!["p-4", "m-4"];
        let sorted = sort_classes(&classes);

        // margin (index 25) comes before padding (index 252)
        assert_eq!(sorted, vec!["m-4", "p-4"]);
    }

    #[test]
    fn test_property_order_complex() {
        let classes = vec!["px-3", "py-4", "bg-red-500"];
        let sorted = sort_classes(&classes);

        // background-color (180) < padding-left (258) < padding-top (257)
        // so bg should be first
        assert_eq!(sorted[0], "bg-red-500");
    }

    #[test]
    fn test_variant_order() {
        let classes = vec!["focus:p-1", "hover:p-1"];
        let sorted = sort_classes(&classes);

        // tailwind v4: focus-within (34) < hover (35) < focus (36) < focus-visible (37)
        assert_eq!(sorted, vec!["hover:p-1", "focus:p-1"]);
    }

    #[test]
    fn test_matches_tailwind_example() {
        // from Tailwind's sort.test.ts:22
        let classes = vec!["px-3", "focus:hover:p-3", "hover:p-1", "py-3"];
        let sorted = sort_classes(&classes);

        // debug output
        eprintln!("Sorted: {:?}", sorted);

        // expected: base classes first, then variants
        // note: px and py might be in either order depending on property indices
        // let's just check they're both in the first two positions
        assert!(sorted[0] == "px-3" || sorted[0] == "py-3");
        assert!(sorted[1] == "px-3" || sorted[1] == "py-3");
        assert_eq!(sorted[2], "hover:p-1");
        assert_eq!(sorted[3], "focus:hover:p-3");
    }

    #[test]
    fn test_arbitrary_values() {
        let classes = vec!["m-[10px]", "p-4", "bg-[#abc]"];
        let sorted = sort_classes(&classes);

        // margin < background-color < padding in property order
        assert_eq!(sorted[0], "m-[10px]");
        assert_eq!(sorted[1], "bg-[#abc]");
        assert_eq!(sorted[2], "p-4");
    }

    #[test]
    fn test_unknown_classes() {
        let classes = vec!["flex", "unknown-class", "grid", "fake-utility"];
        let sorted = sort_classes(&classes);

        // unknown classes first, alphabetically
        assert_eq!(sorted[0], "fake-utility");
        assert_eq!(sorted[1], "unknown-class");
        // known classes after
        assert_eq!(sorted[2], "flex");
        assert_eq!(sorted[3], "grid");
    }

    #[test]
    fn test_sort_key_ordering() {
        // create sort keys manually to test comparison
        let key1 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: None,
            is_negative: false,
            property_count: 1,
            class: "flex".into(),
            is_unparseable: false,
        };

        let key2 = SortKey {
            variant_order: 1,
            variant_chain: parse_variants(&["md"]),
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: None,
            is_negative: false,
            property_count: 1,
            class: "md:flex".into(),
            is_unparseable: false,
        };

        // base class (variant_order=0) should come before variant class
        assert!(key1 < key2);
    }

    #[test]
    fn test_sort_key_property_index() {
        let key1 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![50],
            numeric_value: None,
            is_negative: false,
            property_count: 1,
            class: "a".into(),
            is_unparseable: false,
        };

        let key2 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: None,
            is_negative: false,
            property_count: 1,
            class: "b".into(),
            is_unparseable: false,
        };

        // lower property index comes first
        assert!(key1 < key2);
    }

    #[test]
    fn test_sort_key_property_count() {
        let key1 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: None,
            is_negative: false,
            property_count: 1,
            class: "a".into(),
            is_unparseable: false,
        };

        let key2 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: None,
            is_negative: false,
            property_count: 2,
            class: "b".into(),
            is_unparseable: false,
        };

        // more properties come first (key2 has 2, key1 has 1, so key2 < key1)
        assert!(key2 < key1);
    }

    #[test]
    fn test_sort_key_alphabetical() {
        let key1 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: None,
            is_negative: false,
            property_count: 1,
            class: "aaa".into(),
            is_unparseable: false,
        };

        let key2 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: None,
            is_negative: false,
            property_count: 1,
            class: "bbb".into(),
            is_unparseable: false,
        };

        // alphabetical tiebreaker
        assert!(key1 < key2);
    }

    #[test]
    fn test_get_sort_key() {
        let sorter = PatternSorter::new();

        // Simple utility
        let key = sorter.get_sort_key("flex").unwrap();
        assert_eq!(key.variant_order, 0);
        assert!(!key.property_indices.is_empty());
        assert_eq!(key.class.as_str(), "flex");

        // With variant
        let key = sorter.get_sort_key("md:flex").unwrap();
        assert!(key.variant_order > 0);

        // Unknown utility
        assert!(sorter.get_sort_key("unknown-utility").is_none());
    }

    #[test]
    fn test_multiple_variants() {
        let classes = vec!["hover:focus:p-4", "hover:p-4", "focus:p-4", "p-4"];
        let sorted = sort_classes(&classes);

        // Base class first
        assert_eq!(sorted[0], "p-4");
        // Then single variants, then multiple variants
        // The exact order depends on bitwise combination
        assert!(sorted[1] == "hover:p-4" || sorted[1] == "focus:p-4");
    }

    #[test]
    fn test_important_modifier() {
        let classes = vec!["p-4!", "p-4", "m-4!"];
        let sorted = sort_classes(&classes);

        // important modifier is part of the class string, affects alphabetical sort
        assert_eq!(sorted[0], "m-4!");
        assert_eq!(sorted[1], "p-4");
        assert_eq!(sorted[2], "p-4!");
    }

    #[test]
    fn test_realistic_class_list() {
        let classes = vec![
            "flex",
            "items-center",
            "justify-between",
            "p-4",
            "bg-white",
            "hover:bg-gray-100",
            "rounded-lg",
            "shadow-md",
        ];

        let sorted = sort_classes(&classes);

        // Debug output
        eprintln!("Realistic sorted: {:?}", sorted);

        // All base classes (no :) should come before variant classes (with :)
        let base_classes: Vec<_> = sorted.iter().filter(|c| !c.contains(':')).collect();
        let variant_classes: Vec<_> = sorted.iter().filter(|c| c.contains(':')).collect();

        // Should have 7 base classes and 1 variant class
        assert_eq!(base_classes.len(), 7);
        assert_eq!(variant_classes.len(), 1);

        // Last class should be the variant class
        assert_eq!(sorted[sorted.len() - 1], "hover:bg-gray-100");
    }

    #[test]
    fn test_dark_variant_beyond_u64_limit() {
        // Regression test for the bug where dark (index 70) was treated
        // as having variant_order = 0 due to u64 overflow
        let classes = vec!["flex", "dark:flex", "hover:flex"];
        let sorted = sort_classes(&classes);

        // Base class MUST come first
        assert_eq!(sorted[0], "flex");

        // Variant classes MUST come after base class
        // hover (index 33) should come before dark (index 70)
        assert_eq!(sorted[1], "hover:flex");
        assert_eq!(sorted[2], "dark:flex");
    }

    #[test]
    fn test_high_index_variants_all_work() {
        // Test multiple variants including higher-indexed ones
        // With the corrected VARIANT_ORDER, we have 58 variants total
        let classes = vec![
            "flex",
            "hover:flex",    // index 37
            "sm:flex",       // index 47
            "portrait:flex", // index 52
            "dark:flex",     // index 56
            "print:flex",    // index 57
        ];
        let sorted = sort_classes(&classes);

        // Base class first
        assert_eq!(sorted[0], "flex");

        // Then variants in index order:
        // hover (37) < sm (47) < portrait (52) < dark (56) < print (57)
        assert_eq!(sorted[1], "hover:flex");
        assert_eq!(sorted[2], "sm:flex");
        assert_eq!(sorted[3], "portrait:flex");
        assert_eq!(sorted[4], "dark:flex");
        assert_eq!(sorted[5], "print:flex");
    }

    #[test]
    fn test_sort_key_numeric_value() {
        // Test that utilities with same property but different numeric values sort correctly
        // p-4 should come before p-8 (4 < 8)
        let key1 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: Some(4.0),
            is_negative: false,
            property_count: 1,
            class: "p-4".into(),
            is_unparseable: false,
        };
        let key2 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: Some(8.0),
            is_negative: false,
            property_count: 1,
            class: "p-8".into(),
            is_unparseable: false,
        };
        assert!(key1 < key2);

        // scale-50 should come before scale-110 (50 < 110)
        let key3 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: Some(50.0),
            is_negative: false,
            property_count: 1,
            class: "scale-50".into(),
            is_unparseable: false,
        };
        let key4 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: Some(110.0),
            is_negative: false,
            property_count: 1,
            class: "scale-110".into(),
            is_unparseable: false,
        };
        assert!(key3 < key4);

        // When one has numeric value and other doesn't, they should be equal (fall through to next tier)
        let key5 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: Some(4.0),
            is_negative: false,
            property_count: 1,
            class: "p-4".into(),
            is_unparseable: false,
        };
        let key6 = SortKey {
            variant_order: 0,
            variant_chain: vec![],
            arbitrary_variants: vec![],
            property_indices: vec![100],
            numeric_value: None,
            is_negative: false,
            property_count: 1,
            class: "p-auto".into(),
            is_unparseable: false,
        };
        // They should differ only by alphabetical order
        assert!(key5 < key6); // "p-4" < "p-auto" alphabetically
    }

    #[test]
    fn test_rounded_corner_tiebreaking() {
        // Test that rounded-t and rounded-l are properly ordered using secondary property
        // Both share border-top-left-radius (189), but:
        // - rounded-t: [189, 190] (border-top-right-radius)
        // - rounded-l: [189, 192] (border-bottom-left-radius)
        // Since 190 < 192, rounded-t should come BEFORE rounded-l
        let classes = vec!["rounded-l", "rounded-t"];
        let sorted = sort_classes(&classes);
        assert_eq!(sorted, vec!["rounded-t", "rounded-l"]);

        // Test with modifiers too
        let classes = vec!["rounded-l-lg", "rounded-t-none"];
        let sorted = sort_classes(&classes);
        assert_eq!(sorted, vec!["rounded-t-none", "rounded-l-lg"]);

        // Test all four side-rounded utilities
        let classes = vec!["rounded-l", "rounded-b", "rounded-r", "rounded-t"];
        let sorted = sort_classes(&classes);
        // Expected order by minimum property:
        // rounded-t: (189, 190) and rounded-l: (189, 192) - rounded-t first due to 190 < 192
        // rounded-r: (190, 191)
        // rounded-b: (191, 192)
        assert_eq!(
            sorted,
            vec!["rounded-t", "rounded-l", "rounded-r", "rounded-b"]
        );
    }

    #[test]
    fn test_fraction_sorting_order() {
        let classes = vec!["w-1/2", "w-1/4", "w-1/3", "w-2/3"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["w-1/2", "w-1/3", "w-1/4", "w-2/3"],
            "fractions should be grouped by numerator (base number), then sorted by denominator within group",
        );
    }

    #[test]
    fn test_extract_numeric_value() {
        // Basic integer values
        assert_eq!(extract_numeric_value("p-4"), Some(4.0));
        assert_eq!(extract_numeric_value("p-8"), Some(8.0));
        assert_eq!(extract_numeric_value("scale-110"), Some(110.0));
        assert_eq!(extract_numeric_value("brightness-50"), Some(50.0));

        // Fraction values
        assert_eq!(extract_numeric_value("w-1/2"), Some(0.5));
        assert_eq!(extract_numeric_value("w-1/3"), Some(1.0 / 3.0));
        assert_eq!(extract_numeric_value("w-3/4"), Some(0.75));
        assert_eq!(extract_numeric_value("w-1/4"), Some(0.25));
        assert_eq!(extract_numeric_value("w-2/3"), Some(2.0 / 3.0));

        // Decimal values
        assert_eq!(extract_numeric_value("opacity-50"), Some(50.0));
        assert_eq!(extract_numeric_value("scale-95"), Some(95.0));

        // Negative values (e.g., -translate-x-4) - now returns absolute values
        assert_eq!(extract_numeric_value("-translate-x-4"), Some(4.0));
        assert_eq!(extract_numeric_value("-m-2"), Some(2.0));

        // With variants (should extract from utility part)
        assert_eq!(extract_numeric_value("md:p-8"), Some(8.0));
        assert_eq!(extract_numeric_value("hover:scale-110"), Some(110.0));
        assert_eq!(extract_numeric_value("dark:w-1/2"), Some(0.5));

        // Non-numeric utilities should return None
        assert_eq!(extract_numeric_value("flex"), None);
        assert_eq!(extract_numeric_value("p-auto"), None);
        assert_eq!(extract_numeric_value("rounded-lg"), None);

        // Color shades are numeric and get extracted (bg-blue-500 → 500)
        assert_eq!(extract_numeric_value("bg-blue-500"), Some(500.0));

        // Arbitrary values with brackets
        assert_eq!(extract_numeric_value("h-[120px]"), Some(120.0));
        assert_eq!(extract_numeric_value("h-[2px]"), Some(2.0));
        assert_eq!(extract_numeric_value("w-[50px]"), Some(50.0));
        assert_eq!(extract_numeric_value("w-[120px]"), Some(120.0));
        assert_eq!(extract_numeric_value("max-w-[485px]"), Some(485.0));
        assert_eq!(extract_numeric_value("text-[14px]"), Some(14.0));

        // Arbitrary values with different units
        assert_eq!(extract_numeric_value("h-[2rem]"), Some(2.0));
        assert_eq!(extract_numeric_value("w-[50%]"), Some(50.0));
        assert_eq!(extract_numeric_value("h-[0.5rem]"), Some(0.5));

        // Opacity syntax (color/opacity)
        assert_eq!(extract_numeric_value("bg-white/5"), Some(5.0));
        assert_eq!(extract_numeric_value("bg-white/30"), Some(30.0));
        assert_eq!(extract_numeric_value("text-black/50"), Some(50.0));
        assert_eq!(extract_numeric_value("bg-blue-500/75"), Some(75.0));

        // Arbitrary values with variants
        assert_eq!(extract_numeric_value("md:h-[120px]"), Some(120.0));
        assert_eq!(extract_numeric_value("dark:bg-white/30"), Some(30.0));

        // Edge cases
        assert_eq!(extract_numeric_value("p-0"), Some(0.0));
        assert_eq!(extract_numeric_value("w-1/4"), Some(0.25));
    }

    #[test]
    fn test_space_vs_gap_prefix_priority() {
        // Test space-x-reverse vs gap-y-4
        // Both use row-gap (index 153), but space-* has higher priority than gap-*
        // space-x-reverse should come BEFORE gap-y-4 (prefix priority)
        let classes = vec!["gap-y-4", "space-x-reverse"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["space-x-reverse", "gap-y-4"],
            "space-x-reverse should come before gap-y-4 (both at row-gap index, prefix priority)"
        );

        // Test space-y-reverse vs gap-x-0
        // Both use column-gap (index 152), but space-* has higher priority than gap-*
        // space-y-reverse should come BEFORE gap-x-0 (prefix priority)
        let classes = vec!["gap-x-0", "space-y-reverse"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["space-y-reverse", "gap-x-0"],
            "space-y-reverse should come before gap-x-0 (both at column-gap index, prefix priority)"
        );

        // Test multiple combinations with cross-axis conflicts
        // Expected order:
        // 1. space-y-reverse (column-gap, 152) - space-* prefix priority
        // 2. gap-x-2 (column-gap, 152) - gap-* comes after space-*
        // 3. space-x-reverse (row-gap, 153) - space-* prefix priority
        // 4. gap-y-4 (row-gap, 153) - gap-* comes after space-*
        let classes = vec!["gap-y-4", "space-x-reverse", "gap-x-2", "space-y-reverse"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["space-y-reverse", "gap-x-2", "space-x-reverse", "gap-y-4"],
            "Should sort by property index first, then by prefix priority within same index"
        );
    }

    #[test]
    fn test_base_peer_group_still_work() {
        // Base peer and group (without compounds) should still work
        let classes = vec!["first:p-4", "peer:p-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["peer:p-4", "first:p-4"],
            "peer should sort before first"
        );

        let classes = vec!["last:p-4", "group:p-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["group:p-4", "last:p-4"],
            "group should sort before last"
        );
    }

    #[test]
    fn test_base_classes_before_compound_variants() {
        // Base classes (no variants) should come before compound peer/group variants
        let classes = vec!["group-hover:leading-tight", "my-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["my-4", "group-hover:leading-tight"],
            "base class should come before compound variant"
        );

        let classes = vec!["peer-focus:lowercase", "mb-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["mb-4", "peer-focus:lowercase"],
            "base class should come before compound variant"
        );
    }

    #[test]
    fn test_compound_variants_among_themselves() {
        // Compound variants sort by their BASE, then by MODIFIER INDEX (not alphabetically)
        // peer is at index 2, group is at index 1
        // hover is at index 37, focus is at index 38

        // Both peer-hover and peer-focus sort at peer's position (index 2)
        // Tiebreaking is by modifier index: hover (37) < focus (38)
        let classes = vec!["peer-hover:p-4", "peer-focus:p-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["peer-hover:p-4", "peer-focus:p-4"],
            "peer compounds sort by modifier index when base is same"
        );

        // Both group-hover and group-focus sort at group's position (index 1)
        // Tiebreaking is by modifier index: hover (37) < focus (38)
        let classes = vec!["group-hover:p-4", "group-focus:p-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["group-hover:p-4", "group-focus:p-4"],
            "group compounds sort by modifier index when base is same"
        );

        // Mix of group and peer compounds
        // group (index 1) < peer (index 2)
        let classes = vec!["peer-hover:p-4", "group-hover:p-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["group-hover:p-4", "peer-hover:p-4"],
            "group compounds (index 1) sort before peer compounds (index 2)"
        );
    }

    // NOTE: This test was removed because it conflicts with actual Prettier behavior
    // as verified by the fuzz regression tests. The expectation here may have been incorrect.
    // #[test]
    // fn test_nested_group_peer_compound_order() {
    //     // Tailwind keeps the longer nested group→peer chain ahead of the shorter peer-only variant
    //     let classes = vec![
    //         "group-hover:break-normal",
    //         "group-hover:peer-hover:h-max",
    //         "peer-focus:overscroll-y-contain",
    //     ];
    //     let sorted = sort_classes(&classes);
    //     assert_eq!(
    //         sorted,
    //         vec![
    //             "group-hover:break-normal",
    //             "group-hover:peer-hover:h-max",
    //             "peer-focus:overscroll-y-contain",
    //         ],
    //         "nested group→peer compounds should outrank shorter peer-only chains",
    //     );
    // }

    #[test]
    fn test_pseudo_element_duplicate_ordering() {
        // Prettier/Tailwind puts single pseudo-elements BEFORE duplicates (shorter chains first)
        let classes = vec!["after:after:break-inside-avoid-page", "after:outline-0"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["after:outline-0", "after:after:break-inside-avoid-page"],
            "single pseudo-element should sort before duplicate pseudo-elements (shorter chains first)",
        );
    }

    // NOTE: This test was removed because it conflicts with actual Prettier behavior
    // as verified by the fuzz regression tests. The expectation here may have been incorrect.
    // #[test]
    // fn test_multi_level_compound_variant_ordering() {
    //     // From NEXT.md: Multi-level compound variants should compare recursively
    //     // group-hover:peer-hover: should come before peer-focus:
    //     let classes = vec![
    //         "group-hover:break-normal",
    //         "group-hover:peer-hover:h-max",
    //         "peer-focus:overscroll-y-contain",
    //     ];
    //     let sorted = sort_classes(&classes);
    //     assert_eq!(
    //         sorted,
    //         vec![
    //             "group-hover:break-normal",
    //             "group-hover:peer-hover:h-max",
    //             "peer-focus:overscroll-y-contain",
    //         ],
    //         "multi-level compound variants should be compared recursively",
    //     );
    // }

    #[test]
    fn test_complex_stacked_variants() {
        // Test complex stacking scenarios
        // dark:hover:p-4 vs hover:p-4
        let classes = vec!["hover:p-4", "dark:hover:p-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["hover:p-4", "dark:hover:p-4"],
            "single variant before stacked variants"
        );

        // lg:hover:p-4 vs md:hover:p-4
        let classes = vec!["lg:hover:p-4", "md:hover:p-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["md:hover:p-4", "lg:hover:p-4"],
            "md (index 58) before lg (index 59)"
        );
    }

    #[test]
    fn test_responsive_plus_interactive_stacking() {
        // Test responsive breakpoints with interactive variants
        let classes = vec!["dark:focus:p-4", "dark:hover:p-4", "dark:md:p-4"];
        let sorted = sort_classes(&classes);
        // dark is at index 77
        // hover is at index 35, focus is at index 36, md is at index 58
        // Combined order: dark|hover < dark|focus < dark|md
        assert_eq!(
            sorted,
            vec!["dark:hover:p-4", "dark:focus:p-4", "dark:md:p-4"],
            "stacked variants should combine bitwise"
        );
    }

    #[test]
    fn test_all_peer_compound_variants() {
        // All peer-* compound variants sort at peer's position (index 2)
        // They are tiebroken by MODIFIER INDEX (not alphabetically)
        // Variant indices: checked=24, required=29, invalid=31, focus-within=36,
        // hover=37, focus=38, focus-visible=39, active=40, disabled=42
        let classes = vec![
            "peer-required:p-4",
            "peer-invalid:p-4",
            "peer-disabled:p-4",
            "peer-checked:p-4",
            "peer-active:p-4",
            "peer-focus-visible:p-4",
            "peer-focus-within:p-4",
            "peer-focus:p-4",
            "peer-hover:p-4",
        ];
        let sorted = sort_classes(&classes);
        // Should sort by modifier variant index since all use peer (index 2)
        assert_eq!(
            sorted,
            vec![
                "peer-checked:p-4",       // checked = 24
                "peer-required:p-4",      // required = 29
                "peer-invalid:p-4",       // invalid = 31
                "peer-focus-within:p-4",  // focus-within = 36
                "peer-hover:p-4",         // hover = 37
                "peer-focus:p-4",         // focus = 38
                "peer-focus-visible:p-4", // focus-visible = 39
                "peer-active:p-4",        // active = 40
                "peer-disabled:p-4",      // disabled = 42
            ],
            "peer-* variants should sort by modifier index when all at peer's position"
        );
    }

    #[test]
    fn test_all_group_compound_variants() {
        // All group-* compound variants sort at group's position (index 1)
        // They are tiebroken by MODIFIER INDEX (not alphabetically)
        // Variant indices: focus-within=36, hover=37, focus=38, focus-visible=39, active=40
        let classes = vec![
            "group-active:p-4",
            "group-focus-visible:p-4",
            "group-focus-within:p-4",
            "group-focus:p-4",
            "group-hover:p-4",
        ];
        let sorted = sort_classes(&classes);
        // Should sort by modifier variant index since all use group (index 1)
        assert_eq!(
            sorted,
            vec![
                "group-focus-within:p-4",  // focus-within = 36
                "group-hover:p-4",         // hover = 37
                "group-focus:p-4",         // focus = 38
                "group-focus-visible:p-4", // focus-visible = 39
                "group-active:p-4",        // active = 40
            ],
            "group-* variants should sort by modifier index when all at group's position"
        );
    }
}

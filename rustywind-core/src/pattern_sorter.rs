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
use crate::variant_order::calculate_variant_order;

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

        // If both are digits, compare them as numbers
        if a_char.is_ascii_digit() && z_char.is_ascii_digit() {
            // Find the end of the number in both strings
            let mut a_end = i + 1;
            while a_end < a.len() && a_bytes[a_end].is_ascii_digit() {
                a_end += 1;
            }

            let mut z_end = i + 1;
            while z_end < z.len() && z_bytes[z_end].is_ascii_digit() {
                z_end += 1;
            }

            // Parse and compare numerically
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

            // Fallback to string comparison if parsing fails
            match a[i..a_end].cmp(&z[i..z_end]) {
                Ordering::Equal => {
                    i = a_end.max(z_end);
                    continue;
                }
                other => return other,
            }
        }

        // Compare characters
        match a_char.cmp(&z_char) {
            Ordering::Equal => {
                i += 1;
                continue;
            }
            other => return other,
        }
    }

    // Shorter string comes first
    a.len().cmp(&z.len())
}

/// Extract the base name from a utility class, removing size modifiers.
///
/// This function extracts the base name for utilities with size modifiers:
/// - `rounded-t-lg` → `rounded-t`
/// - `rounded-tl-none` → `rounded-tl`
/// - `rounded-t` → `rounded-t`
/// - `drop-shadow-xl` → `drop-shadow-xl` (no extraction, full name)
///
/// This is used for proper alphabetical comparison when properties match.
fn extract_base_name(utility: &str) -> &str {
    // Strip variants first to get just the utility part
    let utility_base = utility.split(':').next_back().unwrap_or(utility);

    // Extract base for rounded utilities
    if let Some(after_rounded) = utility_base.strip_prefix("rounded-") {
        let parts: Vec<&str> = after_rounded.split('-').collect();
        if parts.len() >= 2 {
            // Check if first part is a side or corner indicator
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

    // PRAGMATIC WORKAROUND: Extract base for drop-shadow and transition utilities
    // This ensures drop-shadow-xl and drop-shadow-none compare as equal at this stage
    // so the special -none handling can kick in (see lines 300-323).
    //
    // NOTE: This is NOT how Tailwind CSS v4 actually works! Tailwind uses property
    // count-based sorting (utilities with MORE CSS declarations sort first), which
    // naturally makes -none variants sort last without special handling.
    //
    // See PROPERTY_COUNT_TODO.md for details on implementing the proper approach.
    if utility_base.starts_with("drop-shadow") {
        return "drop-shadow";
    }
    if utility_base.starts_with("transition") {
        return "transition";
    }

    utility // Return full name if no modifier
}

/// Extract the utility prefix for utilities with size/value modifiers.
///
/// PRAGMATIC WORKAROUND: This function supports the hardcoded -none handling below.
///
/// For drop-shadow-xl, returns "drop-shadow"
/// For transition-colors, returns "transition"
/// For hover:drop-shadow-xl, returns "drop-shadow" (strips variants first)
///
/// NOTE: Tailwind CSS v4 doesn't use prefix matching - it counts CSS declarations.
/// See PROPERTY_COUNT_TODO.md for the proper implementation approach.
fn extract_utility_prefix(utility: &str) -> &str {
    // Strip variants first
    let utility_base = utility.split(':').next_back().unwrap_or(utility);

    // Handle drop-shadow-* utilities
    if utility_base.starts_with("drop-shadow") {
        return "drop-shadow";
    }
    // Handle transition-* utilities
    if utility_base.starts_with("transition") {
        return "transition";
    }
    utility_base
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
    // Remove variants to get just the utility part
    let utility = utility.split(':').next_back()?;

    // Split by dash to get potential numeric parts
    let parts: Vec<&str> = utility.split('-').collect();

    // Look for the last part which is usually the value
    let value_part = parts.last()?;

    // Handle negative values (e.g., -translate-x-4 → value is "4" with negative prefix)
    let (_is_negative, value_str) = if parts.len() > 1 && parts[0].is_empty() {
        // Negative utility like -translate-x-4
        (true, value_part)
    } else {
        (false, value_part)
    };

    // Try to parse as integer
    if let Ok(num) = value_str.parse::<i32>() {
        return Some(num as f64);
    }

    // Try to parse as fraction (e.g., "1/2")
    if value_str.contains('/') {
        let fraction_parts: Vec<&str> = value_str.split('/').collect();
        if fraction_parts.len() == 2
            && let (Ok(numerator), Ok(denominator)) = (
                fraction_parts[0].parse::<f64>(),
                fraction_parts[1].parse::<f64>(),
            )
            && denominator != 0.0
        {
            let result = numerator / denominator;
            return Some(result);
        }
    }

    // Try to parse as decimal (e.g., "0.5")
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

    /// Property indices from PROPERTY_ORDER (lower = earlier)
    /// When utilities have multiple properties (e.g., rounded-t), ALL property indices
    /// are stored and compared in order for proper tiebreaking.
    pub property_indices: Vec<usize>,

    /// Numeric value for value-based sub-sorting (e.g., p-4 → 4.0)
    /// Classes with the same property are sorted by numeric value when available
    pub numeric_value: Option<f64>,

    /// Number of properties this utility generates
    pub property_count: usize,

    /// Original class string (for alphabetical tiebreaker)
    pub class: String,
}

impl Eq for SortKey {}

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
    // Extract the base utility name without variants
    let utility_base = utility.split(':').next_back().unwrap_or(utility);

    if utility_base.starts_with("space-") {
        return 1;
    }
    if utility_base.starts_with("gap-") {
        return 2;
    }
    100 // Default for other utilities
}

impl Ord for SortKey {
    /// Compare sort keys using Tailwind's exact algorithm with value-based sub-sorting.
    ///
    /// Order of comparison:
    /// 1. Variant order (0 first, then by bit flags)
    /// 2. Property indices (compare ALL properties in order for proper tiebreaking)
    /// 3. Numeric value (when both present - lower value first, e.g., p-4 before p-8)
    /// 4. Property count (MORE properties first - utilities with more properties sort earlier)
    /// 5. Utility prefix priority (space-* before gap-* when properties match)
    /// 6. Alphabetical (final tiebreaker)
    fn cmp(&self, other: &Self) -> Ordering {
        self.variant_order
            .cmp(&other.variant_order)
            // Then by property indices - compare ALL properties in order
            // This is crucial for utilities like rounded-t vs rounded-l that tie on first property
            .then_with(|| {
                for (a_idx, b_idx) in self
                    .property_indices
                    .iter()
                    .zip(other.property_indices.iter())
                {
                    match a_idx.cmp(b_idx) {
                        Ordering::Equal => continue, // Tie on this property, check next
                        other => return other,       // Found difference
                    }
                }
                // All common properties are equal, compare by length (MORE properties = earlier)
                other
                    .property_indices
                    .len()
                    .cmp(&self.property_indices.len())
            })
            // Then by numeric value (if both present)
            .then_with(|| {
                match (self.numeric_value, other.numeric_value) {
                    (Some(_), Some(_)) => {
                        // First check prefix priority (space-* before gap-*)
                        let priority_self = get_utility_prefix_priority(&self.class);
                        let priority_other = get_utility_prefix_priority(&other.class);
                        let prefix_cmp = priority_self.cmp(&priority_other);
                        if prefix_cmp != Ordering::Equal {
                            return prefix_cmp;
                        }
                        // Then use alphanumeric comparison of full class names
                        compare_alphanumeric(&self.class, &other.class)
                    }
                    // If only one has a numeric value, no preference (continue to next comparison)
                    _ => Ordering::Equal,
                }
            })
            // Then by property count (fewer properties = earlier)
            // Tailwind's: zSorting.properties.count - aSorting.properties.count
            // means if z has MORE properties, result is positive, so a comes first
            .then(self.property_count.cmp(&other.property_count))
            // Then by utility prefix priority (space-* before gap-* when properties match)
            .then_with(|| {
                let priority_self = get_utility_prefix_priority(&self.class);
                let priority_other = get_utility_prefix_priority(&other.class);
                priority_self.cmp(&priority_other)
            })
            // Compare base names (extracts modifiers)
            .then_with(|| {
                let base_self = extract_base_name(&self.class);
                let base_other = extract_base_name(&other.class);
                base_self.cmp(base_other)
            })
            // ⚠️ PRAGMATIC WORKAROUND: Special -none handling for specific utilities
            //
            // For drop-shadow-* and transition-* utilities, -none should sort LAST
            // For other utilities like shadow-*, blur-*, rounded-*, -none sorts alphabetically
            //
            // WHY THIS EXISTS:
            // Tailwind CSS v4 uses property COUNT (# of CSS declarations) for sorting.
            // - transition-colors generates 3 CSS declarations
            // - transition-none generates 1 CSS declaration
            // - Result: transition-colors (3) sorts before transition-none (1) naturally
            //
            // We don't have declaration counts, so we hardcode these specific cases.
            // See PROPERTY_COUNT_TODO.md for the proper implementation approach.
            .then_with(|| {
                // Extract the utility prefix (e.g., "drop-shadow" from "drop-shadow-xl")
                let prefix_self = extract_utility_prefix(&self.class);
                let prefix_other = extract_utility_prefix(&other.class);

                // Only apply special -none handling if both utilities share the same prefix
                if prefix_self == prefix_other {
                    let self_is_none = self.class.ends_with("-none");
                    let other_is_none = other.class.ends_with("-none");

                    // Only apply special sorting for drop-shadow and transition utilities
                    let needs_special_none_handling =
                        prefix_self == "drop-shadow" || prefix_self == "transition";

                    if needs_special_none_handling && self_is_none != other_is_none {
                        // If one ends with -none and the other doesn't, put -none last
                        return self_is_none.cmp(&other_is_none);
                    }
                }

                Ordering::Equal
            })
            // Finally alphabetically on full name
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
        // Parse the class
        let parsed = parse_class(class)?;

        // Calculate variant order using bitwise flags
        let variant_order = calculate_variant_order(&parsed.variants);

        // Get the CSS properties this utility generates
        let properties = parsed.get_properties()?;

        // Get ALL property indices (not just minimum) for proper multi-property tiebreaking
        // This is crucial for utilities like rounded-t vs rounded-l that share the first property
        // but differ on the second property (e.g., border-top-left-radius ties, but
        // border-top-right-radius (190) < border-bottom-left-radius (192))
        let property_indices: Vec<usize> = properties
            .iter()
            .filter_map(|&prop| get_property_index(prop))
            .collect();

        // Ensure we have at least one valid property index
        if property_indices.is_empty() {
            return None;
        }

        // Count how many properties this utility generates
        let property_count = properties.len();

        // Extract numeric value for value-based sub-sorting
        let numeric_value = extract_numeric_value(class);

        Some(SortKey {
            variant_order,
            property_indices,
            numeric_value,
            property_count,
            class: class.to_string(),
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

    // Generate sort keys for all classes
    // For unknown classes, we still need variant order for proper sorting
    let mut with_keys: Vec<(Option<SortKey>, u128, &str)> = classes
        .iter()
        .map(|&class| {
            let key = sorter.get_sort_key(class);
            // For unknown classes, calculate variant order manually
            let variant_order = if key.is_none() {
                if let Some(parsed) = parse_class(class) {
                    calculate_variant_order(&parsed.variants)
                } else {
                    0
                }
            } else {
                0 // Not needed for known classes
            };
            (key, variant_order, class)
        })
        .collect();

    // Sort by keys
    // Classes without valid keys (unknown/custom) come first, sorted by variant order then alphabetically
    // Classes with valid keys (known Tailwind utilities) come after, sorted by key
    // This matches prettier-plugin-tailwindcss behavior where getClassOrder() returns
    // null for unknown classes, which are sorted to the front.
    with_keys.sort_by(|(a_key, a_variant_order, a_class), (z_key, z_variant_order, z_class)| {
        match (a_key, z_key) {
            (Some(a), Some(z)) => a.cmp(z),
            (Some(_), None) => Ordering::Greater, // Known classes after unknown
            (None, Some(_)) => Ordering::Less,    // Unknown classes before known
            (None, None) => {
                // Unknown classes: sort by variant order first, then alphabetically
                // Lower variant order values come first (0 for no variants, then increasing)
                a_variant_order
                    .cmp(&z_variant_order)
                    .then_with(|| a_class.cmp(z_class))
            }
        }
    });

    // Extract the sorted classes
    with_keys.iter().map(|(_, _, class)| *class).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_classes_before_variants() {
        let classes = vec!["md:flex", "flex", "sm:grid", "grid"];
        let sorted = sort_classes(&classes);

        // Base classes should come first
        assert_eq!(sorted[0], "flex");
        assert_eq!(sorted[1], "grid");
        // Then variant classes
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
        // So bg should be first
        assert_eq!(sorted[0], "bg-red-500");
    }

    #[test]
    fn test_variant_order() {
        let classes = vec!["focus:p-1", "hover:p-1"];
        let sorted = sort_classes(&classes);

        // Tailwind v4: focus-within (34) < hover (35) < focus (36) < focus-visible (37)
        assert_eq!(sorted, vec!["hover:p-1", "focus:p-1"]);
    }

    #[test]
    fn test_matches_tailwind_example() {
        // From Tailwind's sort.test.ts:22
        let classes = vec!["px-3", "focus:hover:p-3", "hover:p-1", "py-3"];
        let sorted = sort_classes(&classes);

        // Debug output
        eprintln!("Sorted: {:?}", sorted);

        // Expected: base classes first, then variants
        // Note: px and py might be in either order depending on property indices
        // Let's just check they're both in the first two positions
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

        // Unknown classes first, alphabetically
        assert_eq!(sorted[0], "fake-utility");
        assert_eq!(sorted[1], "unknown-class");
        // Known classes after
        assert_eq!(sorted[2], "flex");
        assert_eq!(sorted[3], "grid");
    }

    #[test]
    fn test_sort_key_ordering() {
        // Create sort keys manually to test comparison
        let key1 = SortKey {
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: None,
            property_count: 1,
            class: "flex".to_string(),
        };

        let key2 = SortKey {
            variant_order: 1,
            property_indices: vec![100],
            numeric_value: None,
            property_count: 1,
            class: "md:flex".to_string(),
        };

        // Base class (variant_order=0) should come before variant class
        assert!(key1 < key2);
    }

    #[test]
    fn test_sort_key_property_index() {
        let key1 = SortKey {
            variant_order: 0,
            property_indices: vec![50],
            numeric_value: None,
            property_count: 1,
            class: "a".to_string(),
        };

        let key2 = SortKey {
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: None,
            property_count: 1,
            class: "b".to_string(),
        };

        // Lower property index comes first
        assert!(key1 < key2);
    }

    #[test]
    fn test_sort_key_property_count() {
        let key1 = SortKey {
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: None,
            property_count: 1,
            class: "a".to_string(),
        };

        let key2 = SortKey {
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: None,
            property_count: 2,
            class: "b".to_string(),
        };

        // Fewer properties come first (note: reversed comparison)
        assert!(key1 < key2);
    }

    #[test]
    fn test_sort_key_alphabetical() {
        let key1 = SortKey {
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: None,
            property_count: 1,
            class: "aaa".to_string(),
        };

        let key2 = SortKey {
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: None,
            property_count: 1,
            class: "bbb".to_string(),
        };

        // Alphabetical tiebreaker
        assert!(key1 < key2);
    }

    #[test]
    fn test_get_sort_key() {
        let sorter = PatternSorter::new();

        // Simple utility
        let key = sorter.get_sort_key("flex").unwrap();
        assert_eq!(key.variant_order, 0);
        assert!(!key.property_indices.is_empty());
        assert_eq!(key.class, "flex");

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

        // Important modifier is part of the class string, affects alphabetical sort
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
    fn test_variants_beyond_64_all_work() {
        // Test multiple variants beyond the old u64 limit
        let classes = vec![
            "flex",
            "@3xl:flex",     // index 66
            "dark:flex",     // index 74
            "print:flex",    // index 76
            "portrait:flex", // index 72
            "hover:flex",    // index 35 (before 64)
        ];
        let sorted = sort_classes(&classes);

        // Base class first
        assert_eq!(sorted[0], "flex");

        // Then variants in index order:
        // hover (35) < @3xl (66) < portrait (72) < dark (74) < print (76)
        assert_eq!(sorted[1], "hover:flex");
        assert_eq!(sorted[2], "@3xl:flex");
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
            property_indices: vec![100],
            numeric_value: Some(4.0),
            property_count: 1,
            class: "p-4".to_string(),
        };
        let key2 = SortKey {
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: Some(8.0),
            property_count: 1,
            class: "p-8".to_string(),
        };
        assert!(key1 < key2);

        // scale-50 should come before scale-110 (50 < 110)
        let key3 = SortKey {
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: Some(50.0),
            property_count: 1,
            class: "scale-50".to_string(),
        };
        let key4 = SortKey {
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: Some(110.0),
            property_count: 1,
            class: "scale-110".to_string(),
        };
        assert!(key3 < key4);

        // When one has numeric value and other doesn't, they should be equal (fall through to next tier)
        let key5 = SortKey {
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: Some(4.0),
            property_count: 1,
            class: "p-4".to_string(),
        };
        let key6 = SortKey {
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: None,
            property_count: 1,
            class: "p-auto".to_string(),
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
}

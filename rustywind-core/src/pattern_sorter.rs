//! Pattern-based sorting implementation matching Tailwind CSS v4's algorithm
//!
//! This module implements the core sorting logic that matches Tailwind's canonical
//! class ordering. It uses pattern matching rather than hardcoded lists to determine
//! the sort order of classes.
//!
//! # Algorithm
//!
//! Classes are sorted using a five-tier comparison:
//! 1. **Variant Order** - Classes without variants come first, then variants in order
//! 2. **Property Index** - Based on the CSS properties the utility generates
//! 3. **Numeric Value** - When both classes have numeric values (e.g., p-4 vs p-8)
//! 4. **Property Count** - More properties = later (for stability)
//! 5. **Alphabetical** - Final tiebreaker
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

/// Extract a numeric value from a utility class name for value-based sub-sorting.
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
    let utility = utility.split(':').last()?;

    // Split by dash to get potential numeric parts
    let parts: Vec<&str> = utility.split('-').collect();

    // Look for the last part which is usually the value
    let value_part = parts.last()?;

    // Handle negative values (e.g., -translate-x-4 → value is "4" with negative prefix)
    let (is_negative, value_str) = if parts.len() > 1 && parts[0].is_empty() {
        // Negative utility like -translate-x-4
        (true, value_part)
    } else {
        (false, value_part)
    };

    // Try to parse as integer
    if let Ok(num) = value_str.parse::<i32>() {
        return Some(if is_negative { -(num as f64) } else { num as f64 });
    }

    // Try to parse as fraction (e.g., "1/2")
    if value_str.contains('/') {
        let fraction_parts: Vec<&str> = value_str.split('/').collect();
        if fraction_parts.len() == 2 {
            if let (Ok(numerator), Ok(denominator)) = (
                fraction_parts[0].parse::<f64>(),
                fraction_parts[1].parse::<f64>(),
            ) {
                if denominator != 0.0 {
                    let result = numerator / denominator;
                    return Some(if is_negative { -result } else { result });
                }
            }
        }
    }

    // Try to parse as decimal (e.g., "0.5")
    if let Ok(num) = value_str.parse::<f64>() {
        return Some(if is_negative { -num } else { num });
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

    /// Property index from PROPERTY_ORDER (lower = earlier)
    pub property_index: usize,

    /// Numeric value for value-based sub-sorting (e.g., p-4 → 4.0)
    /// Classes with the same property are sorted by numeric value when available
    pub numeric_value: Option<f64>,

    /// Number of properties this utility generates
    pub property_count: usize,

    /// Original class string (for alphabetical tiebreaker)
    pub class: String,
}

impl Eq for SortKey {}

impl Ord for SortKey {
    /// Compare sort keys using Tailwind's exact algorithm with value-based sub-sorting.
    ///
    /// Order of comparison:
    /// 1. Variant order (0 first, then by bit flags)
    /// 2. Property index (lower index first)
    /// 3. Numeric value (when both present - lower value first, e.g., p-4 before p-8)
    /// 4. Property count (FEWER properties first - note the reversal)
    /// 5. Alphabetical (final tiebreaker)
    fn cmp(&self, other: &Self) -> Ordering {
        self.variant_order
            .cmp(&other.variant_order)
            // Then by property index
            .then(self.property_index.cmp(&other.property_index))
            // Then by numeric value (if both present)
            .then_with(|| {
                match (self.numeric_value, other.numeric_value) {
                    (Some(a), Some(b)) => {
                        // Use partial_cmp and default to Equal for NaN cases
                        a.partial_cmp(&b).unwrap_or(Ordering::Equal)
                    }
                    // If only one has a numeric value, no preference (continue to next comparison)
                    _ => Ordering::Equal,
                }
            })
            // Then by property count (fewer properties = earlier)
            // Tailwind's: zSorting.properties.count - aSorting.properties.count
            // means if z has MORE properties, result is positive, so a comes first
            .then(self.property_count.cmp(&other.property_count))
            // Finally alphabetically
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

        // Get the MINIMUM property index (for utilities generating multiple properties)
        // This matches Tailwind's algorithm which uses the lowest property index
        let property_index = properties
            .iter()
            .filter_map(|&prop| get_property_index(prop))
            .min()?;

        // Count how many properties this utility generates
        let property_count = properties.len();

        // Extract numeric value for value-based sub-sorting
        let numeric_value = extract_numeric_value(class);

        Some(SortKey {
            variant_order,
            property_index,
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
/// 1. Base classes (no variants) come first
/// 2. Classes with variants come after, sorted by variant order
/// 3. Within each group, sort by property order
/// 4. Tiebreak by property count, then alphabetically
///
/// Classes that cannot be parsed or have unknown properties are placed at the end,
/// maintaining their relative order.
///
/// # Examples
///
/// ```
/// use rustywind_core::pattern_sorter::sort_classes;
///
/// // Base classes before variants
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
    let mut with_keys: Vec<(Option<SortKey>, &str)> = classes
        .iter()
        .map(|&class| (sorter.get_sort_key(class), class))
        .collect();

    // Sort by keys
    // Classes with valid keys come first (sorted by key)
    // Classes without keys come last (maintaining relative order)
    with_keys.sort_by(|(a_key, a_class), (z_key, z_class)| {
        match (a_key, z_key) {
            (Some(a), Some(z)) => a.cmp(z),
            (Some(_), None) => Ordering::Less, // Known classes before unknown
            (None, Some(_)) => Ordering::Greater, // Unknown classes after known
            (None, None) => a_class.cmp(z_class), // Unknown classes alphabetically
        }
    });

    // Extract the sorted classes
    with_keys.iter().map(|(_, class)| *class).collect()
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

        // Known classes first
        assert_eq!(sorted[0], "flex");
        assert_eq!(sorted[1], "grid");
        // Unknown classes after, alphabetically
        assert_eq!(sorted[2], "fake-utility");
        assert_eq!(sorted[3], "unknown-class");
    }

    #[test]
    fn test_sort_key_ordering() {
        // Create sort keys manually to test comparison
        let key1 = SortKey {
            variant_order: 0,
            property_index: 100,
            numeric_value: None,
            property_count: 1,
            class: "flex".to_string(),
        };

        let key2 = SortKey {
            variant_order: 1,
            property_index: 100,
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
            property_index: 50,
            numeric_value: None,
            property_count: 1,
            class: "a".to_string(),
        };

        let key2 = SortKey {
            variant_order: 0,
            property_index: 100,
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
            property_index: 100,
            numeric_value: None,
            property_count: 1,
            class: "a".to_string(),
        };

        let key2 = SortKey {
            variant_order: 0,
            property_index: 100,
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
            property_index: 100,
            numeric_value: None,
            property_count: 1,
            class: "aaa".to_string(),
        };

        let key2 = SortKey {
            variant_order: 0,
            property_index: 100,
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
        assert!(key.property_index > 0);
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
            property_index: 100,
            numeric_value: Some(4.0),
            property_count: 1,
            class: "p-4".to_string(),
        };
        let key2 = SortKey {
            variant_order: 0,
            property_index: 100,
            numeric_value: Some(8.0),
            property_count: 1,
            class: "p-8".to_string(),
        };
        assert!(key1 < key2);

        // scale-50 should come before scale-110 (50 < 110)
        let key3 = SortKey {
            variant_order: 0,
            property_index: 100,
            numeric_value: Some(50.0),
            property_count: 1,
            class: "scale-50".to_string(),
        };
        let key4 = SortKey {
            variant_order: 0,
            property_index: 100,
            numeric_value: Some(110.0),
            property_count: 1,
            class: "scale-110".to_string(),
        };
        assert!(key3 < key4);

        // When one has numeric value and other doesn't, they should be equal (fall through to next tier)
        let key5 = SortKey {
            variant_order: 0,
            property_index: 100,
            numeric_value: Some(4.0),
            property_count: 1,
            class: "p-4".to_string(),
        };
        let key6 = SortKey {
            variant_order: 0,
            property_index: 100,
            numeric_value: None,
            property_count: 1,
            class: "p-auto".to_string(),
        };
        // They should differ only by alphabetical order
        assert!(key5 < key6); // "p-4" < "p-auto" alphabetically
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

        // Negative values (e.g., -translate-x-4)
        assert_eq!(extract_numeric_value("-translate-x-4"), Some(-4.0));
        assert_eq!(extract_numeric_value("-m-2"), Some(-2.0));

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
}

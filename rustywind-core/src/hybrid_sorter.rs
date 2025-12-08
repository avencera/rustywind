//! Hybrid sorting implementation combining LRU cache and pattern-based sorting
//!
//! This module optimizes sorting performance by using a two-tier approach:
//! 1. **LRU cache** - Runtime cache of previously computed sort keys (quick_cache)
//! 2. **Pattern sorter** - Fallback for uncached classes
//!
//! # Performance
//!
//! - Cached classes: O(1) LRU cache lookup
//! - Uncached classes: O(1) pattern matching + cache insert
//!
//! # Examples
//!
//! ```
//! use rustywind_core::hybrid_sorter::HybridSorter;
//!
//! let sorter = HybridSorter::new();
//! let classes = vec!["flex", "p-4", "hover:bg-blue-500", "m-4"];
//! let sorted = sorter.sort_classes(&classes);
//! ```

use std::sync::Arc;

use quick_cache::sync::Cache;

use crate::pattern_sorter::{PatternSorter, SortKey};

pub const DEFAULT_CACHE_SIZE: usize = 7500;

/// Hybrid sorter combining LRU cache and pattern-based sorting
///
/// This provides optimal performance for sorting Tailwind CSS classes by:
/// - Caching computed sort keys for recently seen classes
/// - Falling back to pattern matching for uncached classes
pub struct HybridSorter {
    /// Pattern-based sorter for computing new sort keys
    pattern_sorter: PatternSorter,

    /// LRU cache for dynamically computed sort keys
    /// Capacity: DEFAULT_CACHE_SIZE entries (covers most real-world usage)
    /// Uses CompactString keys for memory efficiency (24 bytes inline, no heap for typical classes)
    cache: Arc<Cache<compact_str::CompactString, SortKey>>,
}

impl HybridSorter {
    /// Create a new hybrid sorter with default cache size (5000 entries)
    pub fn new() -> Self {
        Self::with_cache_size(DEFAULT_CACHE_SIZE)
    }

    /// Create a new hybrid sorter with custom cache size
    ///
    /// # Arguments
    ///
    /// * `cache_size` - Maximum number of entries to store in the LRU cache
    ///
    /// # Examples
    ///
    /// ```
    /// use rustywind_core::hybrid_sorter::HybridSorter;
    ///
    /// // Create sorter with larger cache for big projects
    /// let sorter = HybridSorter::with_cache_size(25_000);
    /// ```
    pub fn with_cache_size(cache_size: usize) -> Self {
        Self {
            pattern_sorter: PatternSorter::new(),
            cache: Arc::new(Cache::new(cache_size)),
        }
    }

    /// Get the sort key for a class string
    ///
    /// Uses two-tier lookup:
    /// 1. LRU cache (fastest)
    /// 2. Pattern sorter (fallback, result gets cached)
    ///
    /// Returns `None` if the class cannot be parsed or its properties are unknown.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustywind_core::hybrid_sorter::HybridSorter;
    ///
    /// let sorter = HybridSorter::new();
    ///
    /// // First lookup - pattern matched and cached
    /// let key = sorter.get_sort_key("flex").unwrap();
    ///
    /// // Second lookup - cache hit
    /// let key = sorter.get_sort_key("flex").unwrap();
    ///
    /// // Arbitrary values supported
    /// let key = sorter.get_sort_key("m-[10px]").unwrap();
    /// ```
    pub fn get_sort_key(&self, class: &str) -> Option<SortKey> {
        // Tier 1: Check LRU cache for previously computed classes (fast)
        // CompactString has efficient conversion from &str
        let class_compact = compact_str::CompactString::new(class);
        if let Some(cached_key) = self.cache.get(&class_compact) {
            return Some(cached_key);
        }

        // Tier 2: Compute using pattern sorter and cache the result
        if let Some(sort_key) = self.pattern_sorter.get_sort_key(class) {
            // Cache the computed result for future lookups
            // CompactString stores most classes inline (24 bytes) avoiding heap allocations
            self.cache.insert(sort_key.class.clone(), sort_key.clone());
            return Some(sort_key);
        }

        None
    }

    /// Sort a list of Tailwind CSS classes according to the canonical ordering
    ///
    /// This function sorts classes using the hybrid approach with caching.
    /// Classes that cannot be parsed or have unknown properties are placed at the end,
    /// maintaining their relative order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustywind_core::hybrid_sorter::HybridSorter;
    ///
    /// let sorter = HybridSorter::new();
    ///
    /// // Base classes before variants
    /// let classes = vec!["md:flex", "flex", "sm:grid", "grid"];
    /// let sorted = sorter.sort_classes(&classes);
    /// assert_eq!(sorted, vec!["flex", "grid", "sm:grid", "md:flex"]);
    ///
    /// // Property order within base classes
    /// let classes = vec!["p-4", "m-4"]; // margin before padding
    /// let sorted = sorter.sort_classes(&classes);
    /// assert_eq!(sorted, vec!["m-4", "p-4"]);
    /// ```
    pub fn sort_classes<'a>(&self, classes: &[&'a str]) -> Vec<&'a str> {
        use std::cmp::Ordering;

        // Pre-allocate with exact capacity to avoid reallocations
        let mut with_keys: Vec<(Option<SortKey>, &str)> = Vec::with_capacity(classes.len());

        // Generate sort keys for all classes
        for &class in classes {
            with_keys.push((self.get_sort_key(class), class));
        }

        // Sort by keys
        // Classes without keys (unknown/custom) come first (maintaining relative order)
        // Classes with valid keys come after (sorted by key)
        // This matches prettier-plugin-tailwindcss behavior where unknown classes sort first
        with_keys.sort_by(
            |(a_key, _a_class), (z_key, _z_class)| match (a_key, z_key) {
                (Some(a), Some(z)) => a.cmp(z),
                (Some(_), None) => Ordering::Greater, // Known classes after unknown
                (None, Some(_)) => Ordering::Less,    // Unknown classes before known
                (None, None) => Ordering::Equal,      // Unknown classes maintain relative order
            },
        );

        // Extract the sorted classes (pre-allocated for efficiency)
        let mut result = Vec::with_capacity(with_keys.len());
        for (_, class) in with_keys {
            result.push(class);
        }
        result
    }

    /// Get cache statistics
    ///
    /// Returns (entries, capacity) for monitoring cache performance
    ///
    /// Note: This is a simplified interface. The actual quick_cache doesn't
    /// track hits/misses by default, so this returns current usage.
    pub fn cache_stats(&self) -> (usize, usize) {
        // quick_cache capacity() returns u64, convert to usize
        (self.cache.len(), self.cache.capacity() as usize)
    }

    /// Clear the LRU cache
    ///
    /// Useful for testing or memory management.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

impl Default for HybridSorter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_classes() {
        let sorter = HybridSorter::new();

        // These should be computed via pattern matching and cached
        let key = sorter.get_sort_key("flex").unwrap();
        assert_eq!(key.variant_order, 0);
        assert_eq!(key.class.as_str(), "flex");

        let key = sorter.get_sort_key("relative").unwrap();
        assert_eq!(key.variant_order, 0);
        assert_eq!(key.class.as_str(), "relative");
    }

    #[test]
    fn test_pattern_matching_and_caching() {
        let sorter = HybridSorter::new();

        // First lookup - pattern matching, result gets cached
        let key = sorter.get_sort_key("m-4").unwrap();
        assert_eq!(key.variant_order, 0);
        assert_eq!(key.class.as_str(), "m-4");

        // Should be cached now
        let (entries, _) = sorter.cache_stats();
        assert_eq!(entries, 1);
    }

    #[test]
    fn test_lru_cache() {
        let sorter = HybridSorter::new();

        // First lookup - cache miss, will compute and cache
        let key1 = sorter.get_sort_key("m-4").unwrap();

        // Second lookup - cache hit
        let key2 = sorter.get_sort_key("m-4").unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_sort_classes() {
        let sorter = HybridSorter::new();

        let classes = vec!["flex", "p-4", "m-4", "grid"];
        let sorted = sorter.sort_classes(&classes);

        // All should be recognized
        assert_eq!(sorted.len(), 4);

        // All classes will be pattern matched on first pass
        // Should maintain proper order
        assert!(sorted.contains(&"flex"));
        assert!(sorted.contains(&"grid"));
        assert!(sorted.contains(&"m-4"));
        assert!(sorted.contains(&"p-4"));
    }

    #[test]
    fn test_base_classes_before_variants() {
        let sorter = HybridSorter::new();

        let classes = vec!["md:flex", "flex", "sm:grid", "grid"];
        let sorted = sorter.sort_classes(&classes);

        // Base classes should come first
        assert_eq!(sorted[0], "flex");
        assert_eq!(sorted[1], "grid");
        // Then variant classes
        assert!(sorted[2] == "sm:grid" || sorted[2] == "md:flex");
        assert!(sorted[3] == "sm:grid" || sorted[3] == "md:flex");
    }

    #[test]
    fn test_property_order() {
        let sorter = HybridSorter::new();

        let classes = vec!["p-4", "m-4"];
        let sorted = sorter.sort_classes(&classes);

        // margin (index 25) comes before padding (index 252)
        assert_eq!(sorted, vec!["m-4", "p-4"]);
    }

    #[test]
    fn test_variant_order() {
        let sorter = HybridSorter::new();

        let classes = vec!["focus:p-1", "hover:p-1"];
        let sorted = sorter.sort_classes(&classes);

        // Tailwind v4: focus-within (34) < hover (35) < focus (36) < focus-visible (37)
        assert_eq!(sorted, vec!["hover:p-1", "focus:p-1"]);
    }

    #[test]
    fn test_arbitrary_values() {
        let sorter = HybridSorter::new();

        let classes = vec!["m-[10px]", "p-4", "bg-[#abc]"];
        let sorted = sorter.sort_classes(&classes);

        // All should be recognized and sorted
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], "m-[10px]");
        assert_eq!(sorted[1], "bg-[#abc]");
        assert_eq!(sorted[2], "p-4");
    }

    #[test]
    fn test_unknown_classes() {
        let sorter = HybridSorter::new();

        let classes = vec!["flex", "unknown-class", "grid", "fake-utility"];
        let sorted = sorter.sort_classes(&classes);

        // Unknown classes first, maintaining relative order
        assert_eq!(sorted[0], "unknown-class");
        assert_eq!(sorted[1], "fake-utility");
        // Known classes after
        assert_eq!(sorted[2], "flex");
        assert_eq!(sorted[3], "grid");
    }

    #[test]
    fn test_relative_order_preserved_for_unknown_classes() {
        // Test that unknown classes maintain their relative order
        // instead of being alphabetized
        let sorter = HybridSorter::new();

        // Test multiple unknown classes in various orders
        let classes = vec![
            "flex",           // Known: should be 5th
            "zebra-class",    // Unknown: should be 1st (original position)
            "grid",           // Known: should be 6th
            "apple-class",    // Unknown: should be 2nd (original position)
            "m-4",            // Known: should be 4th (by property order)
            "[custom:value]", // Unknown: should be 3rd (original position)
            "banana-class",   // Unknown: should be 7th (original position)
        ];
        let sorted = sorter.sort_classes(&classes);

        // Verify unknown classes come first and maintain relative order (not alphabetized)
        // Original order: zebra-class, apple-class, [custom:value], banana-class
        // If alphabetized it would be: [custom:value], apple-class, banana-class, zebra-class
        // But we want to preserve original order
        assert_eq!(
            sorted[0], "zebra-class",
            "First unknown class should maintain position"
        );
        assert_eq!(
            sorted[1], "apple-class",
            "Second unknown class should maintain position"
        );
        assert_eq!(
            sorted[2], "[custom:value]",
            "Third unknown class should maintain position"
        );
        assert_eq!(
            sorted[3], "banana-class",
            "Fourth unknown class should maintain position"
        );

        // Verify known classes are sorted last by their sort keys
        assert!(sorted[4] == "flex" || sorted[4] == "grid" || sorted[4] == "m-4");
        assert!(sorted[5] == "flex" || sorted[5] == "grid" || sorted[5] == "m-4");
        assert!(sorted[6] == "flex" || sorted[6] == "grid" || sorted[6] == "m-4");
    }

    #[test]
    fn test_clear_cache() {
        let sorter = HybridSorter::new();

        // Add some entries to cache
        sorter.get_sort_key("m-4");
        sorter.get_sort_key("p-4");

        let (entries_before, _) = sorter.cache_stats();
        assert_eq!(entries_before, 2);

        // Clear cache
        sorter.clear_cache();

        let (entries_after, _) = sorter.cache_stats();
        assert_eq!(entries_after, 0);
    }

    #[test]
    fn test_realistic_class_list() {
        let sorter = HybridSorter::new();

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

        let sorted = sorter.sort_classes(&classes);

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
    fn test_custom_cache_size() {
        let sorter = HybridSorter::with_cache_size(10);

        // Add entries
        for i in 0..15 {
            sorter.get_sort_key(&format!("m-{}", i));
        }

        let (entries, capacity) = sorter.cache_stats();
        // Should not exceed capacity (though exact behavior depends on LRU)
        assert!(entries <= capacity);
    }

    #[test]
    fn test_opacity_slash_standard_colors_sort_by_property() {
        // Standard colors with opacity (like text-white/60, bg-black/25) should be
        // treated as known and sort according to property order
        let sorter = HybridSorter::new();

        let classes = vec!["text-white/60", "flex"];
        let sorted = sorter.sort_classes(&classes);
        // flex (display) vs text-white/60 (color)
        // display has lower property index than color, so flex comes first
        assert_eq!(
            sorted,
            vec!["flex", "text-white/60"],
            "flex should sort BEFORE text-white/60 (by property order)"
        );

        let classes = vec!["bg-black/25", "sticky"];
        let sorted = sorter.sort_classes(&classes);
        // sticky (position) vs bg-black/25 (background-color)
        // position has lower property index than background-color, so sticky comes first
        assert_eq!(
            sorted,
            vec!["sticky", "bg-black/25"],
            "sticky should sort BEFORE bg-black/25 (by property order)"
        );
    }
}

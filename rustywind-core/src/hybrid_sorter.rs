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

/// Hybrid sorter combining LRU cache and pattern-based sorting
///
/// This provides optimal performance for sorting Tailwind CSS classes by:
/// - Caching computed sort keys for recently seen classes
/// - Falling back to pattern matching for uncached classes
pub struct HybridSorter {
    /// Pattern-based sorter for computing new sort keys
    pattern_sorter: PatternSorter,

    /// LRU cache for dynamically computed sort keys
    /// Capacity: 1000 entries (covers most real-world usage)
    cache: Arc<Cache<String, SortKey>>,
}

impl HybridSorter {
    /// Create a new hybrid sorter with default cache size (1000 entries)
    pub fn new() -> Self {
        Self::with_cache_size(1000)
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
    /// let sorter = HybridSorter::with_cache_size(5000);
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
        if let Some(cached_key) = self.cache.get(class) {
            return Some(cached_key);
        }

        // Tier 2: Compute using pattern sorter and cache the result
        if let Some(sort_key) = self.pattern_sorter.get_sort_key(class) {
            // Cache the computed result for future lookups
            self.cache.insert(class.to_string(), sort_key.clone());
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

        // Generate sort keys for all classes
        let mut with_keys: Vec<(Option<SortKey>, &str)> = classes
            .iter()
            .map(|&class| (self.get_sort_key(class), class))
            .collect();

        // Sort by keys
        // Classes with valid keys come first (sorted by key)
        // Classes without keys come last (maintaining relative order)
        with_keys.sort_by(|(a_key, a_class), (z_key, z_class)| match (a_key, z_key) {
            (Some(a), Some(z)) => a.cmp(z),
            (Some(_), None) => Ordering::Less, // Known classes before unknown
            (None, Some(_)) => Ordering::Greater, // Unknown classes after known
            (None, None) => a_class.cmp(z_class), // Unknown classes alphabetically
        });

        // Extract the sorted classes
        with_keys.iter().map(|(_, class)| *class).collect()
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
        assert_eq!(key.class, "flex");

        let key = sorter.get_sort_key("relative").unwrap();
        assert_eq!(key.variant_order, 0);
        assert_eq!(key.class, "relative");
    }

    #[test]
    fn test_pattern_matching_and_caching() {
        let sorter = HybridSorter::new();

        // First lookup - pattern matching, result gets cached
        let key = sorter.get_sort_key("m-4").unwrap();
        assert_eq!(key.variant_order, 0);
        assert_eq!(key.class, "m-4");

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

        // Known classes first
        assert_eq!(sorted[0], "flex");
        assert_eq!(sorted[1], "grid");
        // Unknown classes after, alphabetically
        assert_eq!(sorted[2], "fake-utility");
        assert_eq!(sorted[3], "unknown-class");
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
}

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

    // Handle arbitrary values first (e.g., h-[120px], bg-white/30, max-w-[485px])
    // Check for brackets [...] or opacity /number
    if let Some(bracket_start) = utility.find('[')
        && let Some(bracket_end) = utility.find(']')
    {
        // Extract content within brackets: h-[120px] -> "120px"
        let value_str = &utility[bracket_start + 1..bracket_end];

        // Try to extract number from the start of the string
        // Handles: "120px", "2rem", "0.5", "50%", etc.
        let mut num_str = String::new();
        let mut seen_dot = false;

        for ch in value_str.chars() {
            if ch.is_numeric() {
                num_str.push(ch);
            } else if ch == '.' && !seen_dot {
                num_str.push(ch);
                seen_dot = true;
            } else {
                // Stop at first non-numeric, non-dot character
                break;
            }
        }

        if let Ok(num) = num_str.parse::<f64>() {
            return Some(num);
        }
    }

    // Handle opacity syntax: bg-white/30 -> extract 30
    // Distinguish from fractions like w-1/2
    if let Some(slash_pos) = utility.rfind('/') {
        let after_slash = &utility[slash_pos + 1..];
        let before_slash = &utility[..slash_pos];

        // Count dashes to distinguish opacity from fractions:
        // - bg-blue-500/75 (2 dashes) = color-shade/opacity
        // - bg-white/30 (1 dash, non-numeric last part) = color/opacity
        // - w-1/2 (1 dash, numeric last part) = utility-fraction
        let dash_count = before_slash.matches('-').count();

        if dash_count >= 2 {
            // Multiple dashes before slash = color-shade/opacity like bg-blue-500/75
            if let Ok(num) = after_slash.parse::<f64>() {
                return Some(num);
            }
        } else if dash_count == 1 {
            // Single dash: check if last part is a number
            let parts: Vec<&str> = before_slash.split('-').collect();
            if let Some(last_part) = parts.last() {
                // If last part is NOT a number, it's opacity like bg-white/30
                // If last part IS a number, it's a fraction like w-1/2 - skip to fraction logic
                if last_part.parse::<f64>().is_err()
                    && let Ok(num) = after_slash.parse::<f64>()
                {
                    return Some(num);
                }
            }
        }
    }

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

    // Try to extract leading digits from values like "4xl", "2xl", etc.
    // This allows numeric comparison between max-w-4xl and max-w-[485px]
    if !value_str.is_empty() {
        let mut num_str = String::new();
        for ch in value_str.chars() {
            if ch.is_numeric() {
                num_str.push(ch);
            } else {
                break; // Stop at first non-numeric character
            }
        }
        if !num_str.is_empty() {
            if let Ok(num) = num_str.parse::<f64>() {
                return Some(num);
            }
        }
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
    /// Whether this class has BASE `group` or `peer` variants (not compounds)
    /// Base group/peer sort FIRST (before base classes), matching Prettier's behavior
    pub has_base_group_or_peer: bool,

    /// Variant order as bitwise flags (0 for no variants)
    pub variant_order: u128,

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
    pub class: String,
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
    // Strip variants to get the utility part
    let utility = class.split(':').next_back().unwrap_or(class);

    if let Some(slash_pos) = utility.rfind('/') {
        let before_slash = &utility[..slash_pos];

        // Count dashes to distinguish opacity from fractions:
        // - bg-blue-500/75 (2 dashes) = color-shade/opacity
        // - bg-white/30 (1 dash, non-numeric last part) = color/opacity
        // - w-1/4 (1 dash, numeric last part) = utility-fraction
        let dash_count = before_slash.matches('-').count();

        if dash_count >= 2 {
            // Multiple dashes before slash = color-shade/opacity like bg-blue-500/75
            return true;
        } else if dash_count == 1 {
            // Single dash: check if last part before slash is a number
            let parts: Vec<&str> = before_slash.split('-').collect();
            if let Some(last_part) = parts.last() {
                // If last part is NOT a number, it's opacity like bg-white/30
                // If last part IS a number, it's a fraction like w-1/4
                return last_part.parse::<f64>().is_err();
            }
        }
    }

    false
}

/// Check if a utility class has a negative value (e.g., -rotate-1, -translate-x-4)
/// Returns true for classes like: -rotate-1, -skew-y-3, -translate-x-4
/// Returns false for positive values: rotate-0, skew-y-1, translate-x-2
fn is_negative_value(class: &str) -> bool {
    // Strip variants first to get just the utility part
    let utility = class.split(':').next_back().unwrap_or(class);

    // Check if the utility starts with a dash followed by a letter
    // This handles cases like: -rotate-1, -translate-x-4, -skew-y-3
    // But not arbitrary values like: [--spacing-4] or bg-[#fff]
    if let Some(rest) = utility.strip_prefix('-') {
        // Make sure it's not an arbitrary value or a regular dash in a color name
        // Negative utilities start with dash followed by a letter (e.g., -rotate, -translate)
        rest.chars().next().map_or(false, |c| c.is_alphabetic())
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
    // Strip variants first to get just the utility part
    let utility_base = utility.split(':').next_back().unwrap_or(utility);

    // Remove opacity suffix if present (e.g., bg-blue-500/50 → bg-blue-500)
    let utility_without_opacity = utility_base.split('/').next().unwrap_or(utility_base);

    // Known Tailwind color names (in alphabetical order)
    const COLOR_NAMES: &[&str] = &[
        "amber", "black", "blue", "current", "cyan", "emerald", "fuchsia", "gray",
        "green", "indigo", "inherit", "lime", "neutral", "orange", "pink", "purple",
        "red", "rose", "sky", "slate", "stone", "teal", "transparent", "violet",
        "white", "yellow", "zinc",
    ];

    // Color utilities follow patterns like:
    // bg-{color}-{shade}, text-{color}-{shade}, border-{color}-{shade}, etc.
    // Or: bg-{color} (for white, black, transparent, etc.)

    // Split by dash to extract parts
    let parts: Vec<&str> = utility_without_opacity.split('-').collect();

    // Need at least 2 parts: prefix-color or prefix-color-shade
    if parts.len() < 2 {
        return None;
    }

    // Check common color property prefixes
    let color_prefixes = &[
        "bg", "text", "border", "ring", "divide", "outline", "decoration",
        "accent", "caret", "fill", "stroke", "shadow", "from", "via", "to",
    ];

    if color_prefixes.contains(&parts[0]) {
        // Second part should be the color name
        let potential_color = parts[1];

        // Check if it's a known color name
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
    // Strip variants to get the base utility
    let utility = class.split(':').next_back().unwrap_or(class);

    // Properties where arbitrary values come BEFORE regular values
    utility.starts_with("max-w-")
        || utility.starts_with("max-h-")
        || (utility.starts_with("w-") && !utility.starts_with("will-"))
        || (utility.starts_with("h-") && !utility.starts_with("hue-"))
        || utility.starts_with("size-")
        || utility.starts_with("rounded-")
        || utility.starts_with("leading-")
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
    /// 1. Base group/peer variants (group:, peer:) sort FIRST
    /// 2. Base classes (no variants) sort next
    /// 3. Other variants sort by variant_order (bitwise OR of variant indices)
    /// 4. Property indices (compare ALL properties in order for proper tiebreaking)
    /// 5. Numeric value (when both present - lower value first, e.g., p-4 before p-8)
    /// 6. Property count (MORE properties first - utilities with more properties sort earlier)
    /// 7. Utility prefix priority (space-* before gap-* when properties match)
    /// 8. Alphabetical (final tiebreaker)
    fn cmp(&self, other: &Self) -> Ordering {
        // 1. Base group/peer variants handling
        // When BOTH have group/peer (simple OR compound), treat as EQUAL
        // This matches Tailwind v4 behavior where group/peer variants don't affect class sorting
        match (self.has_base_group_or_peer, other.has_base_group_or_peer) {
            (true, true) => {
                // Both have group/peer - treat as EQUAL, which triggers stable sort
                // Stable sort preserves original order, matching Tailwind v4 behavior
                // This applies to ALL combinations:
                // - peer: vs group:
                // - peer:hover: vs group:visited:
                // - even:peer: vs group:focus:
                return Ordering::Equal;
            }
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
            _ => {} // Neither has group/peer, continue with normal sorting
        }

        // 2. Base classes (variant_order=0) come next
        match (self.variant_order == 0, other.variant_order == 0) {
            (true, false) => return Ordering::Less, // Base class before variant
            (false, true) => return Ordering::Greater, // Variant after base class
            (true, true) => {} // Both base classes, continue to property comparison
            (false, false) => {
                // Both have variants - compare by variant_order (unless simple group/peer handled above)
            }
        }

        // 3. Compare by variant order (for compound variants and non-group/peer variants)
        // Simple group/peer variants are already handled above and skip this comparison
        self.variant_order
            .cmp(&other.variant_order)
            // Then compare by property indices - compare ALL properties in order
            // This is crucial for utilities like rounded-t vs rounded-l that tie on first property
            .then_with(|| (|| {
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
        })())
            // Then by property count (MORE properties = earlier, matching Tailwind v4)
            // Tailwind's: zSorting.properties.count - aSorting.properties.count
            // means if z (other) has MORE properties, result is positive, so a (self) comes first
            // Therefore: compare other.count vs self.count (reversed)
            .then(other.property_count.cmp(&self.property_count))
            // Then by color name alphabetically (when both are color utilities)
            // This ensures bg-blue-500 comes before bg-red-50 (blue < red alphabetically)
            // rather than sorting by shade number (50 < 500)
            .then_with(|| {
                match (extract_color_name(&self.class), extract_color_name(&other.class)) {
                    (Some(self_color), Some(other_color)) => {
                        // Both are color utilities - compare by color name first
                        self_color.cmp(other_color)
                    }
                    _ => Ordering::Equal, // At least one is not a color utility, continue
                }
            })
            // Then handle negative value priority
            // Negative values (-rotate-1, -skew-y-3) should sort BEFORE positive values
            .then_with(|| {
                match (self.is_negative, other.is_negative) {
                    (true, false) => return Ordering::Less,    // Negative before positive
                    (false, true) => return Ordering::Greater, // Positive after negative
                    _ => Ordering::Equal, // Both negative or both positive, continue to numeric comparison
                }
            })
            // Then handle numeric and arbitrary value comparison
            // CRITICAL FIX: Check arbitrary status FIRST, before numeric comparison!
            // This fixes the fraction vs arbitrary ordering issue (Issue 2 from FAILURE_ANALYSIS.md)
            //
            // Ordering rules:
            // 1. Non-arbitrary numerics/fractions (w-1/2, w-4) come BEFORE arbitrary values (w-[50px])
            // 2. Arbitrary values come before/after keywords based on property (should_arbitrary_come_first)
            // 3. Within non-arbitrary numerics/fractions, sort by numeric value (w-0 < w-1/2 < w-4)
            // 4. Within arbitrary values, sort by extracted numeric value (w-[10px] < w-[50px])
            //
            // Examples:
            // - w-1/2 w-4 → w-1/2 w-4 (both non-arbitrary, compare numerically: 0.5 < 4)
            // - w-4 w-[50px] → w-4 w-[50px] (non-arbitrary before arbitrary, even though 4 < 50)
            // - w-2/3 w-[50px] → w-2/3 w-[50px] (fraction before arbitrary)
            // - z-40 z-[-1] → z-40 z-[-1] (non-arbitrary before arbitrary)
            // - w-full w-[50px] → w-[50px] w-full (for w-*, arbitrary before keyword)
            .then_with(|| {
                let self_has_arbitrary = has_arbitrary_value(&self.class);
                let other_has_arbitrary = has_arbitrary_value(&other.class);
                let self_has_opacity = has_opacity_syntax(&self.class);
                let other_has_opacity = has_opacity_syntax(&other.class);

                // FIRST: Check arbitrary vs non-arbitrary status
                // Fractions (w-1/2) are NOT arbitrary (no brackets)
                // Numerics (w-4) are NOT arbitrary
                // Arbitrary values (w-[50px]) ARE arbitrary (have brackets)
                match (self_has_arbitrary, other_has_arbitrary) {
                    (true, false) => {
                        // self is arbitrary, other is not
                        if other.numeric_value.is_some() {
                            // other has numeric value (fraction or numeric like w-4, w-1/2)
                            // Non-arbitrary numerics/fractions ALWAYS come before arbitrary
                            return Ordering::Greater; // Arbitrary AFTER non-arbitrary numeric
                        } else {
                            // other is a keyword (w-full, w-auto, etc.)
                            // Use property-specific rule for arbitrary vs keyword ordering
                            if should_arbitrary_come_first(&self.class) {
                                return Ordering::Less; // Arbitrary BEFORE keyword (e.g., w-[50px] before w-full)
                            } else {
                                return Ordering::Greater; // Arbitrary AFTER keyword
                            }
                        }
                    }
                    (false, true) => {
                        // other is arbitrary, self is not
                        if self.numeric_value.is_some() {
                            // self has numeric value (fraction or numeric)
                            // Non-arbitrary numerics/fractions ALWAYS come before arbitrary
                            return Ordering::Less; // Non-arbitrary numeric BEFORE arbitrary
                        } else {
                            // self is a keyword
                            // Use property-specific rule for keyword vs arbitrary ordering
                            if should_arbitrary_come_first(&other.class) {
                                return Ordering::Greater; // Keyword AFTER arbitrary
                            } else {
                                return Ordering::Less; // Keyword BEFORE arbitrary
                            }
                        }
                    }
                    _ => {
                        // Both arbitrary OR both non-arbitrary - continue to numeric comparison
                    }
                }

                // SECOND: Compare numeric values (for same arbitrary status)
                // This applies to:
                // 1. Both non-arbitrary: fractions and numerics compared together (w-1/2 vs w-4)
                // 2. Both arbitrary: compare extracted numeric values (w-[50px] vs w-[100px])
                // DON'T compare numerically if one has opacity syntax and the other doesn't
                match (self.numeric_value, other.numeric_value) {
                    (Some(a), Some(b)) => {
                        // Only compare numerically if both have same opacity status
                        // This prevents comparing shade values (gray-500) with opacity values (white/20)
                        if self_has_opacity == other_has_opacity {
                            match a.partial_cmp(&b).unwrap_or(Ordering::Equal) {
                                Ordering::Equal => {
                                    // Numeric values are equal, continue to next tier
                                }
                                ordering => return ordering, // Different numeric values
                            }
                        }
                        // Different opacity status, continue to next tier
                    }
                    _ => {
                        // At least one doesn't have a numeric value, continue
                    }
                }

                Ordering::Equal // Fall through to next comparison tier
            })
            // Then by alphanumeric comparison for utilities with numeric values
            // (space-* prefix priority is handled here)
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

        // Check if this is a negative value utility
        let is_negative = is_negative_value(class);

        // Check if this class has BASE group or peer variants (not compounds)
        // Base group/peer sort FIRST (before base classes), matching Prettier's behavior
        // Compound variants (group-hover, peer-focus, etc.) do NOT get this special treatment
        let has_base_group_or_peer = parsed
            .variants
            .iter()
            .any(|v| *v == "group" || *v == "peer");

        Some(SortKey {
            has_base_group_or_peer,
            variant_order,
            property_indices,
            numeric_value,
            is_negative,
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
    with_keys.sort_by(
        |(a_key, a_variant_order, a_class), (z_key, z_variant_order, z_class)| {
            match (a_key, z_key) {
                (Some(a), Some(z)) => a.cmp(z),
                (Some(_), None) => Ordering::Greater, // Known classes after unknown
                (None, Some(_)) => Ordering::Less,    // Unknown classes before known
                (None, None) => {
                    // Unknown classes: sort by variant order first, then alphabetically
                    // Lower variant order values come first (0 for no variants, then increasing)
                    a_variant_order
                        .cmp(z_variant_order)
                        .then_with(|| a_class.cmp(z_class))
                }
            }
        },
    );

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
            has_base_group_or_peer: false,
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: None,
            property_count: 1,
            class: "flex".to_string(),
        };

        let key2 = SortKey {
            has_base_group_or_peer: false,
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
            has_base_group_or_peer: false,
            variant_order: 0,
            property_indices: vec![50],
            numeric_value: None,
            property_count: 1,
            class: "a".to_string(),
        };

        let key2 = SortKey {
            has_base_group_or_peer: false,
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
            has_base_group_or_peer: false,
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: None,
            property_count: 1,
            class: "a".to_string(),
        };

        let key2 = SortKey {
            has_base_group_or_peer: false,
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
            has_base_group_or_peer: false,
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: None,
            property_count: 1,
            class: "aaa".to_string(),
        };

        let key2 = SortKey {
            has_base_group_or_peer: false,
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
            has_base_group_or_peer: false,
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: Some(4.0),
            property_count: 1,
            class: "p-4".to_string(),
        };
        let key2 = SortKey {
            has_base_group_or_peer: false,
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: Some(8.0),
            property_count: 1,
            class: "p-8".to_string(),
        };
        assert!(key1 < key2);

        // scale-50 should come before scale-110 (50 < 110)
        let key3 = SortKey {
            has_base_group_or_peer: false,
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: Some(50.0),
            property_count: 1,
            class: "scale-50".to_string(),
        };
        let key4 = SortKey {
            has_base_group_or_peer: false,
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: Some(110.0),
            property_count: 1,
            class: "scale-110".to_string(),
        };
        assert!(key3 < key4);

        // When one has numeric value and other doesn't, they should be equal (fall through to next tier)
        let key5 = SortKey {
            has_base_group_or_peer: false,
            variant_order: 0,
            property_indices: vec![100],
            numeric_value: Some(4.0),
            property_count: 1,
            class: "p-4".to_string(),
        };
        let key6 = SortKey {
            has_base_group_or_peer: false,
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
    fn test_group_before_peer() {
        // CRITICAL: group:p-4 must come before peer:p-4 to match Tailwind's ordering
        // In the corrected VARIANT_ORDER, group is at index 1, peer is at index 2
        let classes = vec!["peer:p-4", "group:p-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["group:p-4", "peer:p-4"],
            "group: must sort before peer: to match Tailwind"
        );

        // Also test with different properties to ensure it's not property-dependent
        let classes = vec!["peer:translate-x-full", "group:min-w-max"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["group:min-w-max", "peer:translate-x-full"],
            "group: must sort before peer: regardless of property"
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
        // Compound variants now sort by their BASE only (peer at index 2, group at index 1)
        // The modifier (hover, focus) is used for alphabetical tiebreaking

        // Both peer-hover and peer-focus sort at peer's position (index 2)
        // Tiebreaking is alphabetical: "peer-focus" < "peer-hover"
        let classes = vec!["peer-hover:p-4", "peer-focus:p-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["peer-focus:p-4", "peer-hover:p-4"],
            "peer compounds sort alphabetically when base is same"
        );

        // Both group-hover and group-focus sort at group's position (index 1)
        // Tiebreaking is alphabetical: "group-focus" < "group-hover"
        let classes = vec!["group-hover:p-4", "group-focus:p-4"];
        let sorted = sort_classes(&classes);
        assert_eq!(
            sorted,
            vec!["group-focus:p-4", "group-hover:p-4"],
            "group compounds sort alphabetically when base is same"
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
        // All peer-* compound variants now sort at peer's position (index 2)
        // They are tiebroken alphabetically by the full variant name
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
        // Should sort alphabetically since all use peer (index 2)
        assert_eq!(
            sorted,
            vec![
                "peer-active:p-4",
                "peer-checked:p-4",
                "peer-disabled:p-4",
                "peer-focus-visible:p-4",
                "peer-focus-within:p-4",
                "peer-focus:p-4",
                "peer-hover:p-4",
                "peer-invalid:p-4",
                "peer-required:p-4",
            ],
            "peer-* variants should sort alphabetically when all at peer's position"
        );
    }

    #[test]
    fn test_all_group_compound_variants() {
        // All group-* compound variants now sort at group's position (index 1)
        // They are tiebroken alphabetically by the full variant name
        let classes = vec![
            "group-active:p-4",
            "group-focus-visible:p-4",
            "group-focus-within:p-4",
            "group-focus:p-4",
            "group-hover:p-4",
        ];
        let sorted = sort_classes(&classes);
        // Should sort alphabetically since all use group (index 1)
        assert_eq!(
            sorted,
            vec![
                "group-active:p-4",
                "group-focus-visible:p-4",
                "group-focus-within:p-4",
                "group-focus:p-4",
                "group-hover:p-4",
            ],
            "group-* variants should sort alphabetically when all at group's position"
        );
    }
}

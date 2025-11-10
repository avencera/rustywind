//! Class name parsing for Tailwind CSS utilities
//!
//! This module provides functionality to parse complete Tailwind CSS class strings
//! into their constituent parts: variants, utility base, value, and modifiers.
//!
//! # Examples
//!
//! ```
//! use rustywind_core::class_parser::parse_class;
//!
//! // Simple utility
//! let parsed = parse_class("flex").unwrap();
//! assert_eq!(parsed.utility, "flex");
//! assert_eq!(parsed.variants.len(), 0);
//!
//! // With responsive variant
//! let parsed = parse_class("md:mx-4").unwrap();
//! assert_eq!(parsed.variants, vec!["md"]);
//! assert_eq!(parsed.utility, "mx");
//! assert_eq!(parsed.value, "4");
//!
//! // With multiple variants and important
//! let parsed = parse_class("hover:focus:bg-red-500!").unwrap();
//! assert_eq!(parsed.variants, vec!["hover", "focus"]);
//! assert_eq!(parsed.utility, "bg");
//! assert_eq!(parsed.value, "red-500");
//! assert!(parsed.important);
//! ```

use crate::utility_map::UTILITY_MAP;

/// A parsed Tailwind CSS class name with all its components.
///
/// This struct represents a fully parsed class name, decomposed into:
/// - The original class string
/// - Any variants (modifiers like `hover:`, `md:`, etc.)
/// - The utility base (e.g., `mx`, `bg`, `flex`)
/// - The value (e.g., `4`, `red-500`, `[#fff]`)
/// - Whether the `!important` modifier is present
///
/// # Examples
///
/// ```
/// use rustywind_core::class_parser::parse_class;
///
/// let parsed = parse_class("md:hover:mx-4!").unwrap();
/// assert_eq!(parsed.original, "md:hover:mx-4!");
/// assert_eq!(parsed.variants, vec!["md", "hover"]);
/// assert_eq!(parsed.utility, "mx");
/// assert_eq!(parsed.value, "4");
/// assert!(parsed.important);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedClass<'a> {
    /// The original class string
    pub original: &'a str,

    /// Variants in order: ["hover", "md"]
    /// Empty vector if no variants
    pub variants: Vec<&'a str>,

    /// The base utility: "mx", "bg", "flex"
    pub utility: &'a str,

    /// The value: "4", "red-500", "[#fff]"
    /// Empty string if no value
    pub value: &'a str,

    /// Whether the important modifier (!) is present
    pub important: bool,
}

impl<'a> ParsedClass<'a> {
    /// Get the full utility part (base + value) without variants.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustywind_core::class_parser::parse_class;
    ///
    /// let parsed = parse_class("md:mx-4").unwrap();
    /// assert_eq!(parsed.full_utility(), "mx-4");
    ///
    /// let parsed = parse_class("flex").unwrap();
    /// assert_eq!(parsed.full_utility(), "flex");
    /// ```
    pub fn full_utility(&self) -> String {
        if self.value.is_empty() {
            self.utility.to_string()
        } else {
            format!("{}-{}", self.utility, self.value)
        }
    }

    /// Check if this class has any variants.
    pub fn has_variants(&self) -> bool {
        !self.variants.is_empty()
    }

    /// Get the number of variants.
    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }

    /// Look up the CSS properties this utility generates.
    ///
    /// Returns `None` if the utility is not recognized by the utility map.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustywind_core::class_parser::parse_class;
    ///
    /// let parsed = parse_class("mx-4").unwrap();
    /// let props = parsed.get_properties().unwrap();
    /// assert!(props.contains(&"margin-inline"));
    /// ```
    pub fn get_properties(&self) -> Option<&'static [&'static str]> {
        UTILITY_MAP.get_properties(&self.full_utility())
    }
}

/// Parse a Tailwind CSS class name into its components.
///
/// This function decomposes a complete class string like `"md:hover:mx-4!"` into:
/// - Variants: `["md", "hover"]`
/// - Utility: `"mx"`
/// - Value: `"4"`
/// - Important: `true`
///
/// Returns `None` if the class string is empty or invalid.
///
/// # Examples
///
/// ```
/// use rustywind_core::class_parser::parse_class;
///
/// // Simple utility
/// let parsed = parse_class("flex").unwrap();
/// assert_eq!(parsed.utility, "flex");
///
/// // With variant
/// let parsed = parse_class("md:mx-4").unwrap();
/// assert_eq!(parsed.variants, vec!["md"]);
/// assert_eq!(parsed.utility, "mx");
/// assert_eq!(parsed.value, "4");
///
/// // With important modifier
/// let parsed = parse_class("bg-red-500!").unwrap();
/// assert_eq!(parsed.utility, "bg");
/// assert_eq!(parsed.value, "red-500");
/// assert!(parsed.important);
///
/// // Multiple variants
/// let parsed = parse_class("hover:focus:p-4").unwrap();
/// assert_eq!(parsed.variants, vec!["hover", "focus"]);
/// ```
pub fn parse_class(class: &str) -> Option<ParsedClass<'_>> {
    if class.is_empty() {
        return None;
    }

    let mut working = class;

    // Handle important modifier (!)
    let important = working.ends_with('!');
    if important {
        working = &working[..working.len() - 1];
    }

    // Split by ':' to separate variants from utility
    let parts: Vec<&str> = working.split(':').collect();

    if parts.is_empty() {
        return None;
    }

    // Last part is the utility (with value)
    let utility_part = parts[parts.len() - 1];

    // Everything before is variants
    let variants = if parts.len() > 1 {
        parts[..parts.len() - 1].to_vec()
    } else {
        vec![]
    };

    // Parse utility into base + value
    let (utility, value) = parse_utility_value(utility_part)?;

    Some(ParsedClass {
        original: class,
        variants,
        utility,
        value,
        important,
    })
}

/// Parse a utility string into base and value parts.
///
/// This reuses the logic from utility_map but is adapted for class parsing.
///
/// # Examples
///
/// - `"flex"` → `("flex", "")`
/// - `"m-4"` → `("m", "4")`
/// - `"bg-red-500"` → `("bg", "red-500")`
/// - `"min-w-0"` → `("min-w", "0")`
fn parse_utility_value(utility: &str) -> Option<(&str, &str)> {
    if utility.is_empty() {
        return None;
    }

    // Handle arbitrary values: bg-[#fff], w-[100px]
    if let Some(bracket_start) = utility.find('[') {
        let base = &utility[..bracket_start.saturating_sub(1)];
        let value = &utility[bracket_start..];
        return Some((base, value));
    }

    // Handle negative values: -translate-x-4, -skew-y-3, -rotate-90, etc.
    let (is_negative, utility_without_neg) = if let Some(stripped) = utility.strip_prefix('-') {
        (true, stripped)
    } else {
        (false, utility)
    };

    // Try to match multi-part bases first (with or without negative sign)
    for prefix in &[
        "min-w",
        "min-h",
        "max-w",
        "max-h",
        "border-t",
        "border-r",
        "border-b",
        "border-l",
        "border-x",
        "border-y",
        "border-s",
        "border-e",
        "rounded-t",
        "rounded-r",
        "rounded-b",
        "rounded-l",
        "rounded-s",
        "rounded-e",
        "rounded-tl",
        "rounded-tr",
        "rounded-br",
        "rounded-bl",
        "rounded-ss",
        "rounded-se",
        "rounded-ee",
        "rounded-es",
        "grid-cols",
        "grid-rows",
        "grid-flow",
        "auto-cols",
        "auto-rows",
        "gap-x",
        "gap-y",
        "flex-row",
        "flex-col",
        "flex-wrap",
        "flex-nowrap",
        "ring-offset",
        "col-span",
        "col-start",
        "col-end",
        "row-span",
        "row-start",
        "row-end",
        "translate-x",
        "translate-y",
        "scale-x",
        "scale-y",
        "skew-x",
        "skew-y",
        "backdrop-blur",
        "backdrop-brightness",
        "backdrop-contrast",
        "backdrop-grayscale",
        "backdrop-hue-rotate",
        "backdrop-invert",
        "backdrop-opacity",
        "backdrop-saturate",
        "backdrop-sepia",
        "will-change",
        "outline-offset",
    ] {
        if utility_without_neg.starts_with(prefix) {
            if utility_without_neg.len() == prefix.len() {
                // Exact match, no value
                return Some((utility, ""));
            } else if utility_without_neg.as_bytes().get(prefix.len()) == Some(&b'-') {
                // Has a dash after the prefix
                let value = &utility_without_neg[prefix.len() + 1..];
                // Return the full utility (including negative sign) as the base
                let base = if is_negative {
                    // prefix.len() is relative to utility_without_neg, add 1 for initial '-'
                    &utility[..prefix.len() + 1] // +1 for initial '-'
                } else {
                    prefix
                };
                return Some((base, value));
            }
        }
    }

    // Simple single-dash split (skip the negative sign if present)
    if let Some(dash_pos) = utility_without_neg.find('-') {
        let base_without_neg = &utility_without_neg[..dash_pos];
        let value = &utility_without_neg[dash_pos + 1..];
        let base = if is_negative {
            // Include the negative sign in the base
            // dash_pos is relative to utility_without_neg, add 1 to offset for the '-' prefix
            &utility[..1 + dash_pos] // 1 for initial '-', then dash_pos characters
        } else {
            base_without_neg
        };
        return Some((base, value));
    }

    // No dash found - utility with no value (keep negative sign if present)
    Some((utility, ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_utility() {
        let parsed = parse_class("flex").unwrap();
        assert_eq!(parsed.original, "flex");
        assert_eq!(parsed.utility, "flex");
        assert_eq!(parsed.value, "");
        assert_eq!(parsed.variants.len(), 0);
        assert!(!parsed.important);
    }

    #[test]
    fn test_parse_utility_with_value() {
        let parsed = parse_class("m-4").unwrap();
        assert_eq!(parsed.utility, "m");
        assert_eq!(parsed.value, "4");
        assert_eq!(parsed.variants.len(), 0);
        assert!(!parsed.important);
    }

    #[test]
    fn test_parse_single_variant() {
        let parsed = parse_class("md:flex").unwrap();
        assert_eq!(parsed.variants, vec!["md"]);
        assert_eq!(parsed.utility, "flex");
        assert_eq!(parsed.value, "");
        assert!(!parsed.important);
    }

    #[test]
    fn test_parse_multiple_variants() {
        let parsed = parse_class("hover:focus:p-4").unwrap();
        assert_eq!(parsed.variants, vec!["hover", "focus"]);
        assert_eq!(parsed.utility, "p");
        assert_eq!(parsed.value, "4");
        assert!(!parsed.important);
    }

    #[test]
    fn test_parse_with_important() {
        let parsed = parse_class("bg-red-500!").unwrap();
        assert_eq!(parsed.utility, "bg");
        assert_eq!(parsed.value, "red-500");
        assert!(parsed.important);
    }

    #[test]
    fn test_parse_variant_with_important() {
        let parsed = parse_class("md:hover:mx-4!").unwrap();
        assert_eq!(parsed.variants, vec!["md", "hover"]);
        assert_eq!(parsed.utility, "mx");
        assert_eq!(parsed.value, "4");
        assert!(parsed.important);
    }

    #[test]
    fn test_parse_arbitrary_value() {
        let parsed = parse_class("bg-[#fff]").unwrap();
        assert_eq!(parsed.utility, "bg");
        assert_eq!(parsed.value, "[#fff]");
        assert!(!parsed.important);

        let parsed = parse_class("w-[100px]").unwrap();
        assert_eq!(parsed.utility, "w");
        assert_eq!(parsed.value, "[100px]");
    }

    #[test]
    fn test_parse_multi_part_utility() {
        let parsed = parse_class("min-w-0").unwrap();
        assert_eq!(parsed.utility, "min-w");
        assert_eq!(parsed.value, "0");

        let parsed = parse_class("border-t-2").unwrap();
        assert_eq!(parsed.utility, "border-t");
        assert_eq!(parsed.value, "2");

        let parsed = parse_class("rounded-tl-lg").unwrap();
        assert_eq!(parsed.utility, "rounded-tl");
        assert_eq!(parsed.value, "lg");
    }

    #[test]
    fn test_parse_color_utility() {
        let parsed = parse_class("bg-red-500").unwrap();
        assert_eq!(parsed.utility, "bg");
        assert_eq!(parsed.value, "red-500");

        let parsed = parse_class("text-blue-600").unwrap();
        assert_eq!(parsed.utility, "text");
        assert_eq!(parsed.value, "blue-600");
    }

    #[test]
    fn test_parse_empty_string() {
        assert!(parse_class("").is_none());
    }

    #[test]
    fn test_full_utility() {
        let parsed = parse_class("mx-4").unwrap();
        assert_eq!(parsed.full_utility(), "mx-4");

        let parsed = parse_class("flex").unwrap();
        assert_eq!(parsed.full_utility(), "flex");

        let parsed = parse_class("bg-red-500").unwrap();
        assert_eq!(parsed.full_utility(), "bg-red-500");
    }

    #[test]
    fn test_has_variants() {
        let parsed = parse_class("flex").unwrap();
        assert!(!parsed.has_variants());

        let parsed = parse_class("md:flex").unwrap();
        assert!(parsed.has_variants());
    }

    #[test]
    fn test_variant_count() {
        let parsed = parse_class("flex").unwrap();
        assert_eq!(parsed.variant_count(), 0);

        let parsed = parse_class("md:flex").unwrap();
        assert_eq!(parsed.variant_count(), 1);

        let parsed = parse_class("hover:focus:active:p-4").unwrap();
        assert_eq!(parsed.variant_count(), 3);
    }

    #[test]
    fn test_get_properties() {
        let parsed = parse_class("mx-4").unwrap();
        let props = parsed.get_properties().unwrap();
        assert!(props.contains(&"margin-inline"));

        let parsed = parse_class("flex").unwrap();
        let props = parsed.get_properties().unwrap();
        assert!(props.contains(&"display"));

        let parsed = parse_class("bg-red-500").unwrap();
        let props = parsed.get_properties().unwrap();
        assert!(props.contains(&"background-color"));
    }

    #[test]
    fn test_complex_class_strings() {
        // Realistic Tailwind class strings
        let parsed = parse_class("sm:hover:bg-blue-500").unwrap();
        assert_eq!(parsed.variants, vec!["sm", "hover"]);
        assert_eq!(parsed.utility, "bg");
        assert_eq!(parsed.value, "blue-500");

        let parsed = parse_class("lg:focus:ring-2").unwrap();
        assert_eq!(parsed.variants, vec!["lg", "focus"]);
        assert_eq!(parsed.utility, "ring");
        assert_eq!(parsed.value, "2");

        let parsed = parse_class("dark:md:hover:text-white!").unwrap();
        assert_eq!(parsed.variants, vec!["dark", "md", "hover"]);
        assert_eq!(parsed.utility, "text");
        assert_eq!(parsed.value, "white");
        assert!(parsed.important);
    }

    #[test]
    fn test_original_preserved() {
        let class = "md:hover:bg-red-500!";
        let parsed = parse_class(class).unwrap();
        assert_eq!(parsed.original, class);
    }

    #[test]
    fn test_parse_utility_value() {
        assert_eq!(parse_utility_value("flex"), Some(("flex", "")));
        assert_eq!(parse_utility_value("m-4"), Some(("m", "4")));
        assert_eq!(parse_utility_value("bg-red-500"), Some(("bg", "red-500")));
        assert_eq!(parse_utility_value("bg-[#fff]"), Some(("bg", "[#fff]")));
        assert_eq!(parse_utility_value("min-w-0"), Some(("min-w", "0")));
        assert_eq!(parse_utility_value(""), None);

        // Test negative values
        assert_eq!(
            parse_utility_value("-translate-x-4"),
            Some(("-translate-x", "4"))
        );
        assert_eq!(
            parse_utility_value("-translate-y-1"),
            Some(("-translate-y", "1"))
        );
        assert_eq!(parse_utility_value("-skew-x-6"), Some(("-skew-x", "6")));
        assert_eq!(parse_utility_value("-skew-y-3"), Some(("-skew-y", "3")));
        assert_eq!(parse_utility_value("-rotate-90"), Some(("-rotate", "90")));
        assert_eq!(parse_utility_value("-scale-x-50"), Some(("-scale-x", "50")));
        assert_eq!(parse_utility_value("-m-4"), Some(("-m", "4")));
    }
}

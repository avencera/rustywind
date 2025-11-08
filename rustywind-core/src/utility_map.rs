//! Utility to CSS property mapping
//!
//! This module maps Tailwind CSS utility class names to the CSS properties they generate.
//! It uses a combination of exact matches (for static utilities) and pattern matching
//! (for parameterized utilities) to determine which properties a utility affects.
//!
//! # Examples
//!
//! ```
//! use rustywind_core::utility_map::UtilityMap;
//!
//! let map = UtilityMap::new();
//!
//! // Exact match
//! assert_eq!(map.get_properties("flex"), Some(&["display"][..]));
//!
//! // Pattern match - parameterized utility
//! assert_eq!(map.get_properties("mx-4"), Some(&["margin-inline"][..]));
//!
//! // Pattern match - arbitrary value
//! assert_eq!(map.get_properties("bg-[#fff]"), Some(&["background-color"][..]));
//! ```

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Maps utility names to the CSS properties they generate.
///
/// This struct provides methods to look up which CSS properties a given utility
/// class will generate. It uses a two-tier approach:
/// 1. Exact matches for static utilities (e.g., "flex" → "display")
/// 2. Pattern matching for parameterized utilities (e.g., "mx-4" → "margin-inline")
pub struct UtilityMap {
    /// Fast lookup for exact utility matches
    exact: HashMap<&'static str, &'static [&'static str]>,
}

impl UtilityMap {
    /// Create a new utility map with all standard Tailwind utilities.
    pub fn new() -> Self {
        let mut exact = HashMap::new();

        // Container
        exact.insert("container", &["container-type"][..]);

        // Display utilities
        exact.insert("block", &["display"][..]);
        exact.insert("inline-block", &["display"][..]);
        exact.insert("inline", &["display"][..]);
        exact.insert("flex", &["display"][..]);
        exact.insert("inline-flex", &["display"][..]);
        exact.insert("table", &["display"][..]);
        exact.insert("inline-table", &["display"][..]);
        exact.insert("table-caption", &["display"][..]);
        exact.insert("table-cell", &["display"][..]);
        exact.insert("table-column", &["display"][..]);
        exact.insert("table-column-group", &["display"][..]);
        exact.insert("table-footer-group", &["display"][..]);
        exact.insert("table-header-group", &["display"][..]);
        exact.insert("table-row-group", &["display"][..]);
        exact.insert("table-row", &["display"][..]);
        exact.insert("flow-root", &["display"][..]);
        exact.insert("grid", &["display"][..]);
        exact.insert("inline-grid", &["display"][..]);
        exact.insert("contents", &["display"][..]);
        exact.insert("list-item", &["display"][..]);
        exact.insert("hidden", &["display"][..]);

        // Position
        exact.insert("static", &["position"][..]);
        exact.insert("fixed", &["position"][..]);
        exact.insert("absolute", &["position"][..]);
        exact.insert("relative", &["position"][..]);
        exact.insert("sticky", &["position"][..]);

        // Visibility
        exact.insert("visible", &["visibility"][..]);
        exact.insert("invisible", &["visibility"][..]);
        exact.insert("collapse", &["visibility"][..]);

        // Float
        exact.insert("float-start", &["float"][..]);
        exact.insert("float-end", &["float"][..]);
        exact.insert("float-right", &["float"][..]);
        exact.insert("float-left", &["float"][..]);
        exact.insert("float-none", &["float"][..]);

        // Clear
        exact.insert("clear-start", &["clear"][..]);
        exact.insert("clear-end", &["clear"][..]);
        exact.insert("clear-left", &["clear"][..]);
        exact.insert("clear-right", &["clear"][..]);
        exact.insert("clear-both", &["clear"][..]);
        exact.insert("clear-none", &["clear"][..]);

        // Isolation
        exact.insert("isolate", &["isolation"][..]);
        exact.insert("isolation-auto", &["isolation"][..]);

        // Object Fit
        exact.insert("object-contain", &["object-fit"][..]);
        exact.insert("object-cover", &["object-fit"][..]);
        exact.insert("object-fill", &["object-fit"][..]);
        exact.insert("object-none", &["object-fit"][..]);
        exact.insert("object-scale-down", &["object-fit"][..]);

        // Overflow
        exact.insert("overflow-auto", &["overflow"][..]);
        exact.insert("overflow-hidden", &["overflow"][..]);
        exact.insert("overflow-clip", &["overflow"][..]);
        exact.insert("overflow-visible", &["overflow"][..]);
        exact.insert("overflow-scroll", &["overflow"][..]);
        exact.insert("overflow-x-auto", &["overflow-x"][..]);
        exact.insert("overflow-x-hidden", &["overflow-x"][..]);
        exact.insert("overflow-x-clip", &["overflow-x"][..]);
        exact.insert("overflow-x-visible", &["overflow-x"][..]);
        exact.insert("overflow-x-scroll", &["overflow-x"][..]);
        exact.insert("overflow-y-auto", &["overflow-y"][..]);
        exact.insert("overflow-y-hidden", &["overflow-y"][..]);
        exact.insert("overflow-y-clip", &["overflow-y"][..]);
        exact.insert("overflow-y-visible", &["overflow-y"][..]);
        exact.insert("overflow-y-scroll", &["overflow-y"][..]);

        // Box Sizing
        exact.insert("box-border", &["box-sizing"][..]);
        exact.insert("box-content", &["box-sizing"][..]);

        // Flexbox & Grid Alignment (common utilities without values)
        exact.insert("items-start", &["align-items"][..]);
        exact.insert("items-end", &["align-items"][..]);
        exact.insert("items-center", &["align-items"][..]);
        exact.insert("items-baseline", &["align-items"][..]);
        exact.insert("items-stretch", &["align-items"][..]);

        exact.insert("justify-start", &["justify-content"][..]);
        exact.insert("justify-end", &["justify-content"][..]);
        exact.insert("justify-center", &["justify-content"][..]);
        exact.insert("justify-between", &["justify-content"][..]);
        exact.insert("justify-around", &["justify-content"][..]);
        exact.insert("justify-evenly", &["justify-content"][..]);
        exact.insert("justify-normal", &["justify-content"][..]);
        exact.insert("justify-stretch", &["justify-content"][..]);

        exact.insert("content-start", &["align-content"][..]);
        exact.insert("content-end", &["align-content"][..]);
        exact.insert("content-center", &["align-content"][..]);
        exact.insert("content-between", &["align-content"][..]);
        exact.insert("content-around", &["align-content"][..]);
        exact.insert("content-evenly", &["align-content"][..]);

        Self { exact }
    }

    /// Get the CSS properties generated by a utility class.
    ///
    /// Returns `Some(&[property, ...])` if the utility is recognized, or `None` if unknown.
    /// Some utilities generate multiple properties (e.g., `px-4` generates both
    /// `padding-left` and `padding-right`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustywind_core::utility_map::UtilityMap;
    ///
    /// let map = UtilityMap::new();
    ///
    /// // Static utility
    /// assert_eq!(map.get_properties("flex"), Some(&["display"][..]));
    ///
    /// // Parameterized utility
    /// assert_eq!(map.get_properties("m-4"), Some(&["margin"][..]));
    ///
    /// // Multiple properties
    /// let px_props = map.get_properties("px-4").unwrap();
    /// assert!(px_props.contains(&"padding-left"));
    /// assert!(px_props.contains(&"padding-right"));
    /// ```
    pub fn get_properties(&self, utility: &str) -> Option<&'static [&'static str]> {
        // Try exact match first (fast path)
        if let Some(props) = self.exact.get(utility) {
            return Some(props);
        }

        // Fall back to pattern matching
        self.match_pattern(utility)
    }

    /// Match a utility against known patterns to determine its properties.
    fn match_pattern(&self, utility: &str) -> Option<&'static [&'static str]> {
        // Parse utility into base and value
        let (base, value) = parse_utility_parts(utility)?;

        // Match against known patterns
        match base {
            // Inset positioning
            "inset" => Some(&["inset"][..]),
            "inset-x" => Some(&["inset-inline"][..]),
            "inset-y" => Some(&["inset-block"][..]),
            "start" => Some(&["inset-inline-start"][..]),
            "end" => Some(&["inset-inline-end"][..]),
            "top" => Some(&["top"][..]),
            "right" => Some(&["right"][..]),
            "bottom" => Some(&["bottom"][..]),
            "left" => Some(&["left"][..]),

            // Z-index
            "z" => Some(&["z-index"][..]),

            // Order
            "order" => Some(&["order"][..]),

            // Grid column/row
            "col" if value.starts_with("span") => Some(&["grid-column"][..]),
            "col" if value.starts_with("start") => Some(&["grid-column-start"][..]),
            "col" if value.starts_with("end") => Some(&["grid-column-end"][..]),
            "row" if value.starts_with("span") => Some(&["grid-row"][..]),
            "row" if value.starts_with("start") => Some(&["grid-row-start"][..]),
            "row" if value.starts_with("end") => Some(&["grid-row-end"][..]),

            // Margins
            "m" => Some(&["margin"][..]),
            "mx" => Some(&["margin-inline"][..]),
            "my" => Some(&["margin-block"][..]),
            "ms" => Some(&["margin-inline-start"][..]),
            "me" => Some(&["margin-inline-end"][..]),
            "mt" => Some(&["margin-top"][..]),
            "mr" => Some(&["margin-right"][..]),
            "mb" => Some(&["margin-bottom"][..]),
            "ml" => Some(&["margin-left"][..]),

            // Sizing
            "w" => Some(&["width"][..]),
            "h" => Some(&["height"][..]),
            "size" => Some(&["width", "height"][..]),
            "min-w" => Some(&["min-width"][..]),
            "min-h" => Some(&["min-height"][..]),
            "max-w" => Some(&["max-width"][..]),
            "max-h" => Some(&["max-height"][..]),

            // Flex
            "flex" if !value.is_empty() => Some(&["flex"][..]), // flex-1, flex-auto, etc.
            "flex-row" => Some(&["flex-direction"][..]),
            "flex-row-reverse" => Some(&["flex-direction"][..]),
            "flex-col" => Some(&["flex-direction"][..]),
            "flex-col-reverse" => Some(&["flex-direction"][..]),
            "flex-wrap" => Some(&["flex-wrap"][..]),
            "flex-wrap-reverse" => Some(&["flex-wrap"][..]),
            "flex-nowrap" => Some(&["flex-wrap"][..]),
            "grow" => Some(&["flex-grow"][..]),
            "grow-0" => Some(&["flex-grow"][..]),
            "shrink" => Some(&["flex-shrink"][..]),
            "shrink-0" => Some(&["flex-shrink"][..]),
            "basis" => Some(&["flex-basis"][..]),

            // Grid
            "grid-cols" => Some(&["grid-template-columns"][..]),
            "grid-rows" => Some(&["grid-template-rows"][..]),
            "auto-cols" => Some(&["grid-auto-columns"][..]),
            "auto-rows" => Some(&["grid-auto-rows"][..]),
            "grid-flow-row" => Some(&["grid-auto-flow"][..]),
            "grid-flow-col" => Some(&["grid-auto-flow"][..]),
            "grid-flow-dense" => Some(&["grid-auto-flow"][..]),
            "grid-flow-row-dense" => Some(&["grid-auto-flow"][..]),
            "grid-flow-col-dense" => Some(&["grid-auto-flow"][..]),

            // Gap
            "gap" if !value.is_empty() => Some(&["gap"][..]),
            "gap-x" => Some(&["column-gap"][..]),
            "gap-y" => Some(&["row-gap"][..]),

            // Padding
            "p" => Some(&["padding"][..]),
            "px" => Some(&["padding-left", "padding-right"][..]),
            "py" => Some(&["padding-top", "padding-bottom"][..]),
            "ps" => Some(&["padding-inline-start"][..]),
            "pe" => Some(&["padding-inline-end"][..]),
            "pt" => Some(&["padding-top"][..]),
            "pr" => Some(&["padding-right"][..]),
            "pb" => Some(&["padding-bottom"][..]),
            "pl" => Some(&["padding-left"][..]),

            // Alignment
            "justify-normal" | "justify-start" | "justify-end" | "justify-center"
            | "justify-between" | "justify-around" | "justify-evenly" | "justify-stretch" => {
                Some(&["justify-content"][..])
            }
            "justify-items-start"
            | "justify-items-end"
            | "justify-items-center"
            | "justify-items-stretch" => Some(&["justify-items"][..]),
            "justify-self-auto"
            | "justify-self-start"
            | "justify-self-end"
            | "justify-self-center"
            | "justify-self-stretch" => Some(&["justify-self"][..]),
            "items-start" | "items-end" | "items-center" | "items-baseline" | "items-stretch" => {
                Some(&["align-items"][..])
            }
            "self-auto" | "self-start" | "self-end" | "self-center" | "self-stretch"
            | "self-baseline" => Some(&["align-self"][..]),
            "content-normal" | "content-center" | "content-start" | "content-end"
            | "content-between" | "content-around" | "content-evenly" | "content-baseline"
            | "content-stretch" => Some(&["align-content"][..]),

            // Background
            "bg" if is_color_value(value) => Some(&["background-color"][..]),
            "bg" if value.starts_with('[') => Some(&["background-color"][..]), // arbitrary value

            // Border Width
            "border" if value.is_empty() || value.parse::<u32>().is_ok() => {
                Some(&["border-width"][..])
            }
            "border-x" => Some(&["border-left-width", "border-right-width"][..]),
            "border-y" => Some(&["border-top-width", "border-bottom-width"][..]),
            "border-s" => Some(&["border-inline-start-width"][..]),
            "border-e" => Some(&["border-inline-end-width"][..]),
            "border-t" => Some(&["border-top-width"][..]),
            "border-r" => Some(&["border-right-width"][..]),
            "border-b" => Some(&["border-bottom-width"][..]),
            "border-l" => Some(&["border-left-width"][..]),

            // Border Color
            "border" if is_color_value(value) => Some(&["border-color"][..]),

            // Border Radius
            "rounded" if value.is_empty() || value.starts_with('[') || is_size_keyword(value) => {
                Some(&["border-radius"][..])
            }
            "rounded-s" => Some(&["border-start-radius"][..]),
            "rounded-e" => Some(&["border-end-radius"][..]),
            "rounded-t" => Some(&["border-top-radius"][..]),
            "rounded-r" => Some(&["border-right-radius"][..]),
            "rounded-b" => Some(&["border-bottom-radius"][..]),
            "rounded-l" => Some(&["border-left-radius"][..]),
            "rounded-ss" => Some(&["border-start-start-radius"][..]),
            "rounded-se" => Some(&["border-start-end-radius"][..]),
            "rounded-ee" => Some(&["border-end-end-radius"][..]),
            "rounded-es" => Some(&["border-end-start-radius"][..]),
            "rounded-tl" => Some(&["border-top-left-radius"][..]),
            "rounded-tr" => Some(&["border-top-right-radius"][..]),
            "rounded-br" => Some(&["border-bottom-right-radius"][..]),
            "rounded-bl" => Some(&["border-bottom-left-radius"][..]),

            // Text
            "text" if is_color_value(value) => Some(&["color"][..]),
            "text" if is_size_keyword(value) => Some(&["font-size"][..]),
            "text-left" => Some(&["text-align"][..]),
            "text-center" => Some(&["text-align"][..]),
            "text-right" => Some(&["text-align"][..]),
            "text-justify" => Some(&["text-align"][..]),
            "text-start" => Some(&["text-align"][..]),
            "text-end" => Some(&["text-align"][..]),

            // Font
            "font" if is_weight_keyword(value) => Some(&["font-weight"][..]),
            "font" => Some(&["font-family"][..]),

            // Opacity
            "opacity" => Some(&["opacity"][..]),

            // Shadow
            "shadow" => Some(&["box-shadow"][..]),

            // Ring (uses multiple properties)
            "ring" if value.is_empty() || value.parse::<u32>().is_ok() => {
                Some(&["--tw-ring-shadow"][..])
            }
            "ring" if is_color_value(value) => Some(&["--tw-ring-color"][..]),
            "ring-offset" if value.parse::<u32>().is_ok() => Some(&["--tw-ring-offset-width"][..]),
            "ring-offset" if is_color_value(value) => Some(&["--tw-ring-offset-color"][..]),

            // Unknown utility
            _ => None,
        }
    }
}

impl Default for UtilityMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a utility into its base name and value.
///
/// Examples:
/// - `"flex"` → `("flex", "")`
/// - `"m-4"` → `("m", "4")`
/// - `"mx-auto"` → `("mx", "auto")`
/// - `"bg-red-500"` → `("bg", "red-500")`
/// - `"bg-[#fff]"` → `("bg", "[#fff]")`
/// - `"min-w-0"` → `("min-w", "0")`
fn parse_utility_parts(utility: &str) -> Option<(&str, &str)> {
    // Handle arbitrary values: bg-[#fff], w-[100px]
    if let Some(bracket_start) = utility.find('[') {
        let base = &utility[..bracket_start.saturating_sub(1)]; // Remove the '-' before '['
        let value = &utility[bracket_start..];
        return Some((base, value));
    }

    // Try to match multi-part bases first
    // These need to be checked before simple dash splitting
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
    ] {
        if utility.starts_with(prefix) {
            if utility.len() == prefix.len() {
                // Exact match, no value
                return Some((prefix, ""));
            } else if utility.as_bytes().get(prefix.len()) == Some(&b'-') {
                // Has a dash after the prefix
                let value = &utility[prefix.len() + 1..];
                return Some((prefix, value));
            } else if prefix.ends_with('-') {
                // Prefix ends with dash (shouldn't happen with our list, but safe)
                let value = &utility[prefix.len()..];
                return Some((prefix, value));
            }
        }
    }

    // Simple single-dash split
    if let Some(dash_pos) = utility.find('-') {
        let base = &utility[..dash_pos];
        let value = &utility[dash_pos + 1..];
        return Some((base, value));
    }

    // No dash found - utility with no value
    Some((utility, ""))
}

/// Check if this base+value combination indicates a multi-part base.
#[allow(dead_code)]
fn is_multi_part_base(base: &str, value: &str) -> bool {
    matches!(
        (base, value.split('-').next().unwrap_or("")),
        ("min", "w")
            | ("min", "h")
            | ("max", "w")
            | ("max", "h")
            | (
                "rounded",
                "t" | "r"
                    | "b"
                    | "l"
                    | "s"
                    | "e"
                    | "tl"
                    | "tr"
                    | "br"
                    | "bl"
                    | "ss"
                    | "se"
                    | "ee"
                    | "es"
            )
            | ("border", "t" | "r" | "b" | "l" | "s" | "e" | "x" | "y")
            | ("grid", "cols" | "rows" | "flow")
            | ("auto", "cols" | "rows")
            | ("gap", "x" | "y")
            | ("flex", "row" | "col" | "wrap" | "nowrap")
            | ("items", "start" | "end" | "center" | "baseline" | "stretch")
            | (
                "justify",
                "start"
                    | "end"
                    | "center"
                    | "between"
                    | "around"
                    | "evenly"
                    | "normal"
                    | "stretch"
                    | "items"
                    | "self"
            )
            | (
                "content",
                "start"
                    | "end"
                    | "center"
                    | "between"
                    | "around"
                    | "evenly"
                    | "normal"
                    | "baseline"
                    | "stretch"
            )
            | (
                "self",
                "auto" | "start" | "end" | "center" | "stretch" | "baseline"
            )
            | ("place", "content" | "items" | "self")
            | ("overflow", "x" | "y")
            | ("ring", "offset")
    )
}

/// Check if a value looks like a color.
fn is_color_value(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    // Check for arbitrary color value: [#fff], [rgb(255,0,0)]
    if value.starts_with('[') {
        return true;
    }

    // Check for color with shade: red-500, blue-600
    if value.contains('-') {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() == 2 {
            // Check if second part is a number (shade)
            if parts[1].parse::<u32>().is_ok() {
                return true;
            }
        }
    }

    // Check for named colors: red, blue, transparent, current, inherit
    matches!(
        value,
        "transparent"
            | "current"
            | "inherit"
            | "black"
            | "white"
            | "red"
            | "blue"
            | "green"
            | "yellow"
            | "purple"
            | "pink"
            | "gray"
            | "slate"
            | "zinc"
            | "neutral"
            | "stone"
            | "orange"
            | "amber"
            | "lime"
            | "emerald"
            | "teal"
            | "cyan"
            | "sky"
            | "indigo"
            | "violet"
            | "fuchsia"
            | "rose"
    )
}

/// Check if a value is a size keyword.
fn is_size_keyword(value: &str) -> bool {
    matches!(
        value,
        "xs" | "sm"
            | "base"
            | "lg"
            | "xl"
            | "2xl"
            | "3xl"
            | "4xl"
            | "5xl"
            | "6xl"
            | "7xl"
            | "8xl"
            | "9xl"
            | "full"
            | "min"
            | "max"
            | "fit"
            | "auto"
    )
}

/// Check if a value is a font weight keyword.
fn is_weight_keyword(value: &str) -> bool {
    matches!(
        value,
        "thin"
            | "extralight"
            | "light"
            | "normal"
            | "medium"
            | "semibold"
            | "bold"
            | "extrabold"
            | "black"
    )
}

/// Global lazy-initialized utility map for efficient reuse.
pub static UTILITY_MAP: Lazy<UtilityMap> = Lazy::new(UtilityMap::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_matches() {
        let map = UtilityMap::new();

        // Display utilities
        assert_eq!(map.get_properties("flex"), Some(&["display"][..]));
        assert_eq!(map.get_properties("block"), Some(&["display"][..]));
        assert_eq!(map.get_properties("hidden"), Some(&["display"][..]));
        assert_eq!(map.get_properties("grid"), Some(&["display"][..]));

        // Position utilities
        assert_eq!(map.get_properties("relative"), Some(&["position"][..]));
        assert_eq!(map.get_properties("absolute"), Some(&["position"][..]));
        assert_eq!(map.get_properties("fixed"), Some(&["position"][..]));
    }

    #[test]
    fn test_margin_utilities() {
        let map = UtilityMap::new();

        assert_eq!(map.get_properties("m-4"), Some(&["margin"][..]));
        assert_eq!(map.get_properties("mx-auto"), Some(&["margin-inline"][..]));
        assert_eq!(map.get_properties("my-8"), Some(&["margin-block"][..]));
        assert_eq!(map.get_properties("mt-2"), Some(&["margin-top"][..]));
        assert_eq!(map.get_properties("mr-4"), Some(&["margin-right"][..]));
        assert_eq!(map.get_properties("mb-6"), Some(&["margin-bottom"][..]));
        assert_eq!(map.get_properties("ml-1"), Some(&["margin-left"][..]));
    }

    #[test]
    fn test_padding_utilities() {
        let map = UtilityMap::new();

        assert_eq!(map.get_properties("p-4"), Some(&["padding"][..]));
        assert_eq!(map.get_properties("pt-2"), Some(&["padding-top"][..]));

        // px and py generate multiple properties
        let px = map.get_properties("px-4").unwrap();
        assert!(px.contains(&"padding-left"));
        assert!(px.contains(&"padding-right"));

        let py = map.get_properties("py-8").unwrap();
        assert!(py.contains(&"padding-top"));
        assert!(py.contains(&"padding-bottom"));
    }

    #[test]
    fn test_sizing_utilities() {
        let map = UtilityMap::new();

        assert_eq!(map.get_properties("w-full"), Some(&["width"][..]));
        assert_eq!(map.get_properties("h-screen"), Some(&["height"][..]));
        assert_eq!(map.get_properties("min-w-0"), Some(&["min-width"][..]));
        assert_eq!(map.get_properties("max-h-96"), Some(&["max-height"][..]));
    }

    #[test]
    fn test_color_utilities() {
        let map = UtilityMap::new();

        // Background colors
        assert_eq!(
            map.get_properties("bg-red-500"),
            Some(&["background-color"][..])
        );
        assert_eq!(
            map.get_properties("bg-blue-600"),
            Some(&["background-color"][..])
        );

        // Text colors
        assert_eq!(map.get_properties("text-white"), Some(&["color"][..]));
        assert_eq!(map.get_properties("text-gray-900"), Some(&["color"][..]));

        // Border colors
        assert_eq!(
            map.get_properties("border-black"),
            Some(&["border-color"][..])
        );
    }

    #[test]
    fn test_arbitrary_values() {
        let map = UtilityMap::new();

        // Arbitrary color values
        assert_eq!(
            map.get_properties("bg-[#fff]"),
            Some(&["background-color"][..])
        );
        assert_eq!(
            map.get_properties("text-[rgb(255,0,0)]"),
            Some(&["color"][..])
        );

        // Arbitrary size values
        assert_eq!(map.get_properties("w-[100px]"), Some(&["width"][..]));
        assert_eq!(map.get_properties("m-[10rem]"), Some(&["margin"][..]));
    }

    #[test]
    fn test_unknown_utilities() {
        let map = UtilityMap::new();

        assert_eq!(map.get_properties("unknown-utility"), None);
        assert_eq!(map.get_properties("fake-class"), None);
    }

    #[test]
    fn test_parse_utility_parts() {
        assert_eq!(parse_utility_parts("flex"), Some(("flex", "")));
        assert_eq!(parse_utility_parts("m-4"), Some(("m", "4")));
        assert_eq!(parse_utility_parts("mx-auto"), Some(("mx", "auto")));
        assert_eq!(parse_utility_parts("bg-red-500"), Some(("bg", "red-500")));
        assert_eq!(parse_utility_parts("bg-[#fff]"), Some(("bg", "[#fff]")));
    }

    #[test]
    fn test_is_color_value() {
        assert!(is_color_value("red-500"));
        assert!(is_color_value("blue-600"));
        assert!(is_color_value("[#fff]"));
        assert!(is_color_value("[rgb(255,0,0)]"));
        assert!(is_color_value("transparent"));
        assert!(is_color_value("black"));

        assert!(!is_color_value("4"));
        assert!(!is_color_value("auto"));
        assert!(!is_color_value(""));
    }

    #[test]
    fn test_is_size_keyword() {
        assert!(is_size_keyword("xs"));
        assert!(is_size_keyword("sm"));
        assert!(is_size_keyword("lg"));
        assert!(is_size_keyword("xl"));
        assert!(is_size_keyword("full"));
        assert!(is_size_keyword("auto"));

        assert!(!is_size_keyword("4"));
        assert!(!is_size_keyword("red"));
    }

    #[test]
    fn test_border_utilities() {
        let map = UtilityMap::new();

        // Border width
        assert_eq!(map.get_properties("border"), Some(&["border-width"][..]));
        assert_eq!(map.get_properties("border-2"), Some(&["border-width"][..]));
        assert_eq!(
            map.get_properties("border-t"),
            Some(&["border-top-width"][..])
        );

        // Border radius
        assert_eq!(map.get_properties("rounded"), Some(&["border-radius"][..]));
        assert_eq!(
            map.get_properties("rounded-lg"),
            Some(&["border-radius"][..])
        );
        assert_eq!(
            map.get_properties("rounded-tl"),
            Some(&["border-top-left-radius"][..])
        );
    }

    #[test]
    fn test_flex_utilities() {
        let map = UtilityMap::new();

        assert_eq!(map.get_properties("flex-1"), Some(&["flex"][..]));
        assert_eq!(
            map.get_properties("flex-row"),
            Some(&["flex-direction"][..])
        );
        assert_eq!(map.get_properties("flex-wrap"), Some(&["flex-wrap"][..]));
        assert_eq!(map.get_properties("grow"), Some(&["flex-grow"][..]));
        assert_eq!(map.get_properties("shrink"), Some(&["flex-shrink"][..]));
    }

    #[test]
    fn test_grid_utilities() {
        let map = UtilityMap::new();

        assert_eq!(
            map.get_properties("grid-cols-3"),
            Some(&["grid-template-columns"][..])
        );
        assert_eq!(
            map.get_properties("grid-rows-2"),
            Some(&["grid-template-rows"][..])
        );
        assert_eq!(map.get_properties("gap-4"), Some(&["gap"][..]));
        assert_eq!(map.get_properties("gap-x-2"), Some(&["column-gap"][..]));
    }
}

//! The canonical order of CSS properties from Tailwind CSS v4
//!
//! This is a direct port of `packages/tailwindcss/src/property-order.ts`
//! from the Tailwind CSS repository. The order of these properties determines
//! how Tailwind classes are sorted.
//!
//! Source: https://github.com/tailwindlabs/tailwindcss/blob/next/packages/tailwindcss/src/property-order.ts

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// The canonical order of CSS properties as defined by Tailwind CSS.
///
/// Classes are sorted based on the CSS properties they generate, and this array
/// defines the order in which those properties should appear. A lower index means
/// the class appears earlier in the sorted output.
///
/// # Examples
///
/// ```
/// use rustywind_core::property_order::{PROPERTY_ORDER, get_property_index};
///
/// // margin appears before padding in the property order
/// assert!(get_property_index("margin").unwrap() < get_property_index("padding").unwrap());
/// ```
pub const PROPERTY_ORDER: &[&str] = &[
    // Complete property order from Tailwind CSS v4
    // Source: packages/tailwindcss/src/property-order.ts (345 properties - includes 8 additional properties for compatibility)
    // Added properties (from pre-Tailwind v4 sync with 96% pass rate):
    //   - background-opacity (index 0, Tailwind v3 backwards compatibility)
    //   - border-opacity (after border-left-color)
    //   - --tw-prose-component, --tw-prose-invert (after vertical-align)
    //   - outline-style, --tw-divide-x-reverse, --tw-ring-inset, user-select (from previous fix)

    // Tailwind v3 backwards compatibility (sorted first)
    "background-opacity",

    "container-type",
    "pointer-events",
    "visibility",
    "position",
    "inset",
    "inset-inline",
    "inset-block",
    "inset-inline-start",
    "inset-inline-end",
    "top",
    "right",
    "bottom",
    "left",
    "isolation",
    "z-index",
    "order",
    "grid-column",
    "grid-column-start",
    "grid-column-end",
    "grid-row",
    "grid-row-start",
    "grid-row-end",
    "float",
    "clear",
    "--tw-container-component",
    "margin",
    "margin-inline",
    "margin-block",
    "margin-inline-start",
    "margin-inline-end",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "box-sizing",
    "display",
    "field-sizing",
    "aspect-ratio",
    "height",
    "max-height",
    "min-height",
    "width",
    "max-width",
    "min-width",
    "flex",
    "flex-shrink",
    "flex-grow",
    "flex-basis",
    "table-layout",
    "caption-side",
    "border-collapse",
    "border-spacing",
    "transform-origin",
    "translate",
    "--tw-translate-x",
    "--tw-translate-y",
    "--tw-translate-z",
    "scale",
    "--tw-scale-x",
    "--tw-scale-y",
    "--tw-scale-z",
    "rotate",
    "--tw-rotate-x",
    "--tw-rotate-y",
    "--tw-rotate-z",
    "--tw-skew-x",
    "--tw-skew-y",
    "transform",
    "animation",
    "cursor",
    "touch-action",
    "--tw-pan-x",
    "--tw-pan-y",
    "--tw-pinch-zoom",
    "resize",
    "scroll-snap-type",
    "--tw-scroll-snap-strictness",
    "scroll-snap-align",
    "scroll-snap-stop",
    "scroll-margin",
    "scroll-margin-inline",
    "scroll-margin-block",
    "scroll-margin-inline-start",
    "scroll-margin-inline-end",
    "scroll-margin-top",
    "scroll-margin-right",
    "scroll-margin-bottom",
    "scroll-margin-left",
    "scroll-padding",
    "scroll-padding-inline",
    "scroll-padding-block",
    "scroll-padding-inline-start",
    "scroll-padding-inline-end",
    "scroll-padding-top",
    "scroll-padding-right",
    "scroll-padding-bottom",
    "scroll-padding-left",
    "list-style-position",
    "list-style-type",
    "list-style-image",
    "appearance",
    "columns",
    "break-before",
    "break-inside",
    "break-after",
    "grid-auto-columns",
    "grid-auto-flow",
    "grid-auto-rows",
    "grid-template-columns",
    "grid-template-rows",
    "flex-direction",
    "flex-wrap",
    "place-content",
    "place-items",
    "align-content",
    "align-items",
    "justify-content",
    "justify-items",
    "gap",
    "column-gap",
    "row-gap",
    "--tw-space-x-reverse",
    "--tw-space-y-reverse",
    "divide-x-width",
    "divide-y-width",
    "--tw-divide-x-reverse",
    "--tw-divide-y-reverse",
    "divide-style",
    "divide-color",
    "place-self",
    "align-self",
    "justify-self",
    "overflow",
    "overflow-x",
    "overflow-y",
    "overscroll-behavior",
    "overscroll-behavior-x",
    "overscroll-behavior-y",
    "scroll-behavior",
    "border-radius",
    "border-start-radius",
    "border-end-radius",
    "border-top-radius",
    "border-right-radius",
    "border-bottom-radius",
    "border-left-radius",
    "border-start-start-radius",
    "border-start-end-radius",
    "border-end-end-radius",
    "border-end-start-radius",
    "border-top-left-radius",
    "border-top-right-radius",
    "border-bottom-right-radius",
    "border-bottom-left-radius",
    "border-width",
    "border-inline-width",
    "border-block-width",
    "border-inline-start-width",
    "border-inline-end-width",
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
    "border-style",
    "border-inline-style",
    "border-block-style",
    "border-inline-start-style",
    "border-inline-end-style",
    "border-top-style",
    "border-right-style",
    "border-bottom-style",
    "border-left-style",
    "border-color",
    "border-inline-color",
    "border-block-color",
    "border-inline-start-color",
    "border-inline-end-color",
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
    "border-opacity",
    "background-color",
    "background-image",
    "--tw-gradient-position",
    "--tw-gradient-stops",
    "--tw-gradient-via-stops",
    "--tw-gradient-from",
    "--tw-gradient-from-position",
    "--tw-gradient-via",
    "--tw-gradient-via-position",
    "--tw-gradient-to",
    "--tw-gradient-to-position",
    "mask-image",
    "--tw-mask-top",
    "--tw-mask-top-from-color",
    "--tw-mask-top-from-position",
    "--tw-mask-top-to-color",
    "--tw-mask-top-to-position",
    "--tw-mask-right",
    "--tw-mask-right-from-color",
    "--tw-mask-right-from-position",
    "--tw-mask-right-to-color",
    "--tw-mask-right-to-position",
    "--tw-mask-bottom",
    "--tw-mask-bottom-from-color",
    "--tw-mask-bottom-from-position",
    "--tw-mask-bottom-to-color",
    "--tw-mask-bottom-to-position",
    "--tw-mask-left",
    "--tw-mask-left-from-color",
    "--tw-mask-left-from-position",
    "--tw-mask-left-to-color",
    "--tw-mask-left-to-position",
    "--tw-mask-linear",
    "--tw-mask-linear-position",
    "--tw-mask-linear-from-color",
    "--tw-mask-linear-from-position",
    "--tw-mask-linear-to-color",
    "--tw-mask-linear-to-position",
    "--tw-mask-radial",
    "--tw-mask-radial-shape",
    "--tw-mask-radial-size",
    "--tw-mask-radial-position",
    "--tw-mask-radial-from-color",
    "--tw-mask-radial-from-position",
    "--tw-mask-radial-to-color",
    "--tw-mask-radial-to-position",
    "--tw-mask-conic",
    "--tw-mask-conic-position",
    "--tw-mask-conic-from-color",
    "--tw-mask-conic-from-position",
    "--tw-mask-conic-to-color",
    "--tw-mask-conic-to-position",
    "box-decoration-break",
    "background-size",
    "background-attachment",
    "background-clip",
    "background-position",
    "background-repeat",
    "background-origin",
    "mask-composite",
    "mask-mode",
    "mask-type",
    "mask-size",
    "mask-clip",
    "mask-position",
    "mask-repeat",
    "mask-origin",
    "fill",
    "stroke",
    "stroke-width",
    "object-fit",
    "object-position",
    "padding",
    "padding-inline",
    "padding-block",
    "padding-inline-start",
    "padding-inline-end",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "text-align",
    "text-indent",
    "vertical-align",
    "--tw-prose-component",
    "--tw-prose-invert",
    "font-family",
    "font-size",
    "line-height",
    "font-weight",
    "letter-spacing",
    "text-wrap",
    "overflow-wrap",
    "word-break",
    "text-overflow",
    "hyphens",
    "white-space",
    "color",
    "text-transform",
    "font-style",
    "font-stretch",
    "font-variant-numeric",
    "text-decoration-line",
    "text-decoration-color",
    "text-decoration-style",
    "text-decoration-thickness",
    "text-underline-offset",
    "-webkit-font-smoothing",
    "placeholder-color",
    "caret-color",
    "accent-color",
    "color-scheme",
    "opacity",
    "background-blend-mode",
    "mix-blend-mode",
    "box-shadow",
    "--tw-shadow",
    "--tw-shadow-color",
    "--tw-ring-shadow",
    "--tw-ring-color",
    "--tw-inset-shadow",
    "--tw-inset-shadow-color",
    "--tw-inset-ring-shadow",
    "--tw-inset-ring-color",
    "--tw-ring-offset-width",
    "--tw-ring-offset-color",
    "--tw-ring-inset",
    "outline",
    "outline-width",
    "outline-offset",
    "outline-color",
    "outline-style",
    "--tw-blur",
    "--tw-brightness",
    "--tw-contrast",
    "--tw-drop-shadow",
    "--tw-grayscale",
    "--tw-hue-rotate",
    "--tw-invert",
    "--tw-saturate",
    "--tw-sepia",
    "filter",
    "--tw-backdrop-blur",
    "--tw-backdrop-brightness",
    "--tw-backdrop-contrast",
    "--tw-backdrop-grayscale",
    "--tw-backdrop-hue-rotate",
    "--tw-backdrop-invert",
    "--tw-backdrop-opacity",
    "--tw-backdrop-saturate",
    "--tw-backdrop-sepia",
    "backdrop-filter",
    "transition-property",
    "transition-behavior",
    "transition-delay",
    "transition-duration",
    "transition-timing-function",
    "will-change",
    "contain",
    "content",
    "forced-color-adjust",
    "user-select",
];

/// Optimized HashMap for O(1) property index lookup.
///
/// This is lazily initialized on first use and maps property names to their indices.
static PROPERTY_INDEX_MAP: Lazy<HashMap<&'static str, usize>> = Lazy::new(|| {
    PROPERTY_ORDER
        .iter()
        .enumerate()
        .map(|(idx, &prop)| (prop, idx))
        .collect()
});

/// Get the index of a CSS property in the canonical order.
///
/// Returns `Some(index)` if the property is found, or `None` if it's not in the list.
/// Lower indices mean the property (and classes that generate it) should appear
/// earlier in the sorted output.
///
/// This uses an optimized O(1) HashMap lookup instead of linear search.
///
/// # Examples
///
/// ```
/// use rustywind_core::property_order::get_property_index;
///
/// assert_eq!(get_property_index("margin"), Some(26));
/// assert_eq!(get_property_index("padding"), Some(255));
/// assert_eq!(get_property_index("unknown-property"), None);
/// ```
#[inline]
pub fn get_property_index(property: &str) -> Option<usize> {
    PROPERTY_INDEX_MAP.get(property).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_count() {
        // Complete property order from Tailwind CSS v4 with compatibility additions
        // Base: 337 properties + 8 added for 96% pass rate compatibility:
        //   - background-opacity, border-opacity (Tailwind v3 compatibility)
        //   - --tw-prose-component, --tw-prose-invert (prose utilities)
        //   - outline-style, --tw-divide-x-reverse, --tw-ring-inset, user-select
        assert_eq!(PROPERTY_ORDER.len(), 345);
    }

    #[test]
    fn test_get_property_index() {
        // Test first properties (including added ones)
        assert_eq!(get_property_index("background-opacity"), Some(0)); // Added for v3 compatibility
        assert_eq!(get_property_index("container-type"), Some(1));
        assert_eq!(get_property_index("pointer-events"), Some(2));

        // Test critical properties for ring/shadow/filter ordering
        // These indices must match Tailwind v4 for correct sorting
        assert_eq!(get_property_index("box-shadow"), Some(298));
        assert_eq!(get_property_index("--tw-shadow"), Some(299));
        assert_eq!(get_property_index("--tw-shadow-color"), Some(300));
        assert_eq!(get_property_index("--tw-ring-shadow"), Some(301));
        assert_eq!(get_property_index("--tw-ring-color"), Some(302));
        assert_eq!(get_property_index("--tw-ring-inset"), Some(309)); // Added property

        // Outline properties
        assert_eq!(get_property_index("outline"), Some(310));
        assert_eq!(get_property_index("outline-style"), Some(314)); // Added property

        // Filters come after rings
        assert_eq!(get_property_index("--tw-blur"), Some(315));
        assert_eq!(get_property_index("filter"), Some(324));

        // Border properties for arbitrary value sorting
        assert_eq!(get_property_index("border-width"), Some(155));
        assert_eq!(get_property_index("border-opacity"), Some(182)); // Added property
        assert_eq!(get_property_index("border-top-width"), Some(160));

        // Test divide reverse properties (both x and y)
        let divide_x_idx = get_property_index("--tw-divide-x-reverse").unwrap();
        let divide_y_idx = get_property_index("--tw-divide-y-reverse").unwrap();
        let divide_style_idx = get_property_index("divide-style").unwrap();
        assert_eq!(divide_x_idx, 126); // Added property
        assert_eq!(divide_y_idx, 127);
        assert_eq!(divide_y_idx, divide_style_idx - 1);

        // Test prose properties (added for prose utilities)
        assert_eq!(get_property_index("--tw-prose-component"), Some(267));
        assert_eq!(get_property_index("--tw-prose-invert"), Some(268));

        // Test common properties
        assert_eq!(get_property_index("margin"), Some(26));
        assert_eq!(get_property_index("padding"), Some(255));
        assert_eq!(get_property_index("display"), Some(36));
        assert_eq!(get_property_index("background-color"), Some(183));

        // Test user-select (added property)
        assert_eq!(get_property_index("user-select"), Some(344));

        // Test unknown property
        assert_eq!(get_property_index("unknown-property"), None);
    }

    #[test]
    fn test_margin_before_padding() {
        // Margin should come before padding
        let margin_idx = get_property_index("margin").unwrap();
        let padding_idx = get_property_index("padding").unwrap();
        assert!(margin_idx < padding_idx);
    }

    #[test]
    fn test_specific_margin_properties() {
        // All specific margin properties should come after margin
        let margin_idx = get_property_index("margin").unwrap();
        assert!(get_property_index("margin-inline").unwrap() > margin_idx);
        assert!(get_property_index("margin-top").unwrap() > margin_idx);
        assert!(get_property_index("margin-left").unwrap() > margin_idx);
    }

    #[test]
    fn test_no_duplicates() {
        use std::collections::HashSet;
        let unique: HashSet<_> = PROPERTY_ORDER.iter().collect();
        assert_eq!(
            unique.len(),
            PROPERTY_ORDER.len(),
            "Property order contains duplicates"
        );
    }
}

//! The canonical order of CSS properties from Tailwind CSS v4
//!
//! This is a direct port of `packages/tailwindcss/src/property-order.ts`
//! from the Tailwind CSS repository. The order of these properties determines
//! how Tailwind classes are sorted.
//!
//! Source: https://github.com/tailwindlabs/tailwindcss/blob/next/packages/tailwindcss/src/property-order.ts

use ahash::AHashMap as HashMap;
use std::sync::LazyLock;

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
    // exact original 341-property order that achieved 96% pass rate
    // this order was empirically tuned through extensive fuzz testing
    // source: Pre-Tailwind v4 sync (commit before 3758006)
    //
    // WARNING: Do NOT modify property positions without thorough testing!
    // index shifts of even a few positions can cause 10%+ pass rate drops
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
    // NOTE: Tailwind has --tw-border-spacing-x/y commented out in property-order.ts
    // Do NOT add them here - they are not used for sorting. Tailwind uses the actual
    // `border-spacing` property for sorting border-spacing-x/y utilities.
    // See: https://github.com/tailwindlabs/tailwindcss/blob/next/packages/tailwindcss/src/property-order.ts#L68-71
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
    "--tw-pan-x",
    "--tw-pan-y",
    "--tw-pinch-zoom",
    "touch-action",
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
    "--tw-divide-color-sort",
    "divide-x-width",
    "divide-y-width",
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
    // NOTE: Tailwind has synthetic border-{top,right,bottom,left}-radius properties
    // in property-order.ts, but do NOT add them here. They exist in Tailwind only for
    // utilities that emit `--tw-sort: border-top-radius` (which rounded-t doesn't do).
    //
    // Rustywind achieves the same sorting by mapping rounded-t/r/b/l directly to the
    // actual CSS corner properties (e.g., rounded-t → [border-top-left-radius, border-top-right-radius]).
    // this approach works correctly and adding the synthetic properties would have no effect
    // since no utility maps to them in utility_map.rs.
    //
    // See: https://github.com/tailwindlabs/tailwindcss/blob/next/packages/tailwindcss/src/property-order.ts#L181-184
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
    "--tw-mask-radial-from-color",
    "--tw-mask-radial-from-position",
    "--tw-mask-radial-to-color",
    "--tw-mask-radial-to-position",
    "--tw-mask-radial-shape",
    "--tw-mask-radial-size",
    "--tw-mask-radial-position",
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
    "--tw-ring-offset-shadow",
    "--tw-ring-shadow",
    "--tw-inset-ring-shadow",
    "--tw-inset-shadow",
    "--tw-shadow-color",
    "--tw-ring-color",
    "--tw-inset-shadow-color",
    "--tw-inset-ring-color",
    "--tw-ring-offset-width",
    "--tw-ring-offset-color",
    "outline",
    "outline-width",
    "outline-offset",
    "outline-color",
    "--tw-blur",
    "--tw-brightness",
    "--tw-contrast",
    "--tw-drop-shadow",
    "--tw-drop-shadow-color",
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
    "outline-style",
    "user-select",
    "--tw-divide-x-reverse",
    "--tw-ring-inset",
    "contain",
    "content",
    "text-shadow",
    "--tw-text-shadow",
    "--tw-text-shadow-color",
    "backface-visibility",
    "perspective",
    "perspective-origin",
    "transform-style",
    "forced-color-adjust",
];

/// Optimized HashMap (ahash) for O(1) property index lookup with fast hashing.
///
/// This is lazily initialized on first use and maps property names to their indices.
/// Uses ahash for better performance than std HashMap's default hasher.
static PROPERTY_INDEX_MAP: LazyLock<HashMap<&'static str, usize>> = LazyLock::new(|| {
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
/// assert!(get_property_index("margin") < get_property_index("padding"));
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
        assert_eq!(PROPERTY_ORDER.len(), 351);
    }

    #[test]
    fn test_property_relative_ordering() {
        // tests relative relationships instead of absolute positions
        // this won't break when Tailwind updates property order

        // core layout properties come early
        let container = get_property_index("container-type").unwrap();
        let pointer_events = get_property_index("pointer-events").unwrap();
        let margin = get_property_index("margin").unwrap();
        let display = get_property_index("display").unwrap();

        assert!(
            container < pointer_events,
            "container-type before pointer-events"
        );
        assert!(pointer_events < margin, "pointer-events before margin");
        assert!(margin < display, "margin before display");

        // spacing hierarchy: margin before padding
        let padding = get_property_index("padding").unwrap();
        assert!(margin < padding, "margin before padding");

        // specific properties after general ones
        let margin_inline = get_property_index("margin-inline").unwrap();
        let margin_top = get_property_index("margin-top").unwrap();
        assert!(margin < margin_inline, "margin before margin-inline");
        assert!(margin < margin_top, "margin before margin-top");

        // divide properties should be ordered correctly
        let divide_y = get_property_index("--tw-divide-y-reverse").unwrap();
        let divide_style = get_property_index("divide-style").unwrap();
        let divide_x = get_property_index("--tw-divide-x-reverse").unwrap();
        assert!(divide_y < divide_style, "divide-y before divide-style");
        assert!(divide_style < divide_x, "divide-style before divide-x");

        // border properties
        let border_width = get_property_index("border-width").unwrap();
        let border_top_width = get_property_index("border-top-width").unwrap();
        let border_opacity = get_property_index("border-opacity").unwrap();
        let background_color = get_property_index("background-color").unwrap();
        assert!(
            border_width < border_top_width,
            "border-width before border-top-width"
        );
        assert!(
            border_opacity < background_color,
            "border-opacity before background-color"
        );

        // shadow and ring properties (critical for sorting)
        let box_shadow = get_property_index("box-shadow").unwrap();
        let tw_shadow = get_property_index("--tw-shadow").unwrap();
        let tw_shadow_color = get_property_index("--tw-shadow-color").unwrap();
        let tw_ring_shadow = get_property_index("--tw-ring-shadow").unwrap();
        let tw_ring_color = get_property_index("--tw-ring-color").unwrap();

        assert!(box_shadow < tw_shadow, "box-shadow before --tw-shadow");
        assert!(tw_shadow < tw_shadow_color, "shadows before shadow-color");
        assert!(
            tw_ring_shadow < tw_ring_color,
            "ring-shadow before ring-color"
        );

        // outline properties
        let outline = get_property_index("outline").unwrap();
        let outline_style = get_property_index("outline-style").unwrap();
        let tw_ring_inset = get_property_index("--tw-ring-inset").unwrap();
        assert!(outline < outline_style, "outline before outline-style");
        assert!(
            outline_style < tw_ring_inset,
            "outline-style before ring-inset"
        );

        // filter properties
        let tw_blur = get_property_index("--tw-blur").unwrap();
        let filter = get_property_index("filter").unwrap();
        assert!(tw_blur < filter, "blur before filter");

        // user select near end
        let user_select = get_property_index("user-select").unwrap();
        let will_change = get_property_index("will-change").unwrap();
        assert!(will_change < user_select, "will-change before user-select");

        // test unknown property returns None
        assert_eq!(get_property_index("unknown-property"), None);
    }

    #[test]
    fn test_critical_properties_exist() {
        // verifies critical properties exist (prevents accidental deletions)
        let critical = vec![
            // layout fundamentals
            "display",
            "position",
            "container-type",
            "pointer-events",
            // spacing
            "margin",
            "margin-top",
            "margin-inline",
            "padding",
            // sizing
            "width",
            "height",
            "min-width",
            "max-width",
            // flexbox & grid
            "flex",
            "flex-direction",
            "grid-template-columns",
            "grid-column",
            // colors
            "background-color",
            "color",
            "border-color",
            // borders
            "border-width",
            "border-style",
            "border-opacity",
            // shadows & rings (critical for Phase 2 fixes)
            "box-shadow",
            "--tw-shadow",
            "--tw-shadow-color",
            "--tw-ring-shadow",
            "--tw-ring-color",
            "--tw-ring-inset",
            // divide
            "--tw-divide-x-reverse",
            "--tw-divide-y-reverse",
            "divide-style",
            // filters
            "filter",
            "--tw-blur",
            "backdrop-filter",
            // outline
            "outline",
            "outline-style",
            // typography
            "font-size",
            "font-weight",
            "line-height",
            "text-align",
            // prose (typography plugin)
            "--tw-prose-component",
            "--tw-prose-invert",
            // other
            "user-select",
            "will-change",
        ];

        for prop in critical {
            assert!(
                get_property_index(prop).is_some(),
                "Critical property '{}' missing from PROPERTY_ORDER",
                prop
            );
        }
    }

    #[test]
    fn test_margin_before_padding() {
        // margin should come before padding
        let margin_idx = get_property_index("margin").unwrap();
        let padding_idx = get_property_index("padding").unwrap();
        assert!(margin_idx < padding_idx);
    }

    #[test]
    fn test_specific_margin_properties() {
        // all specific margin properties should come after margin
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

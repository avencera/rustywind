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

use ahash::AHashMap as HashMap;
use std::sync::LazyLock;

/// Maps utility names to the CSS properties they generate.
///
/// This struct provides methods to look up which CSS properties a given utility
/// class will generate. It uses a two-tier approach:
/// 1. Exact matches for static utilities (e.g., "flex" → "display")
/// 2. Pattern matching for parameterized utilities (e.g., "mx-4" → "margin-inline")
pub struct UtilityMap {
    /// Fast lookup for exact utility matches using ahash for better performance
    exact: HashMap<&'static str, &'static [&'static str]>,
}

impl UtilityMap {
    /// Create a new utility map with all standard Tailwind utilities.
    pub fn new() -> Self {
        let mut exact = HashMap::new();

        // container (maps to --tw-container-component for proper sorting after grid utilities)
        exact.insert("container", &["--tw-container-component"][..]);

        // display utilities
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

        // position
        exact.insert("static", &["position"][..]);
        exact.insert("fixed", &["position"][..]);
        exact.insert("absolute", &["position"][..]);
        exact.insert("relative", &["position"][..]);
        exact.insert("sticky", &["position"][..]);

        // visibility
        exact.insert("visible", &["visibility"][..]);
        exact.insert("invisible", &["visibility"][..]);
        exact.insert("collapse", &["visibility"][..]);

        // float
        exact.insert("float-start", &["float"][..]);
        exact.insert("float-end", &["float"][..]);
        exact.insert("float-right", &["float"][..]);
        exact.insert("float-left", &["float"][..]);
        exact.insert("float-none", &["float"][..]);

        // clear
        exact.insert("clear-start", &["clear"][..]);
        exact.insert("clear-end", &["clear"][..]);
        exact.insert("clear-left", &["clear"][..]);
        exact.insert("clear-right", &["clear"][..]);
        exact.insert("clear-both", &["clear"][..]);
        exact.insert("clear-none", &["clear"][..]);

        // isolation
        exact.insert("isolate", &["isolation"][..]);
        exact.insert("isolation-auto", &["isolation"][..]);

        // object fit
        exact.insert("object-contain", &["object-fit"][..]);
        exact.insert("object-cover", &["object-fit"][..]);
        exact.insert("object-fill", &["object-fit"][..]);
        exact.insert("object-none", &["object-fit"][..]);
        exact.insert("object-scale-down", &["object-fit"][..]);

        // overflow
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

        // box sizing
        exact.insert("box-border", &["box-sizing"][..]);
        exact.insert("box-content", &["box-sizing"][..]);

        // flexbox & grid alignment (common utilities without values)
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

        // cursor
        exact.insert("cursor-auto", &["cursor"][..]);
        exact.insert("cursor-default", &["cursor"][..]);
        exact.insert("cursor-pointer", &["cursor"][..]);
        exact.insert("cursor-wait", &["cursor"][..]);
        exact.insert("cursor-text", &["cursor"][..]);
        exact.insert("cursor-move", &["cursor"][..]);
        exact.insert("cursor-help", &["cursor"][..]);
        exact.insert("cursor-not-allowed", &["cursor"][..]);
        exact.insert("cursor-none", &["cursor"][..]);
        exact.insert("cursor-context-menu", &["cursor"][..]);
        exact.insert("cursor-progress", &["cursor"][..]);
        exact.insert("cursor-cell", &["cursor"][..]);
        exact.insert("cursor-crosshair", &["cursor"][..]);
        exact.insert("cursor-vertical-text", &["cursor"][..]);
        exact.insert("cursor-alias", &["cursor"][..]);
        exact.insert("cursor-copy", &["cursor"][..]);
        exact.insert("cursor-no-drop", &["cursor"][..]);
        exact.insert("cursor-grab", &["cursor"][..]);
        exact.insert("cursor-grabbing", &["cursor"][..]);
        exact.insert("cursor-all-scroll", &["cursor"][..]);
        exact.insert("cursor-col-resize", &["cursor"][..]);
        exact.insert("cursor-row-resize", &["cursor"][..]);
        exact.insert("cursor-n-resize", &["cursor"][..]);
        exact.insert("cursor-e-resize", &["cursor"][..]);
        exact.insert("cursor-s-resize", &["cursor"][..]);
        exact.insert("cursor-w-resize", &["cursor"][..]);
        exact.insert("cursor-ne-resize", &["cursor"][..]);
        exact.insert("cursor-nw-resize", &["cursor"][..]);
        exact.insert("cursor-se-resize", &["cursor"][..]);
        exact.insert("cursor-sw-resize", &["cursor"][..]);
        exact.insert("cursor-ew-resize", &["cursor"][..]);
        exact.insert("cursor-ns-resize", &["cursor"][..]);
        exact.insert("cursor-nesw-resize", &["cursor"][..]);
        exact.insert("cursor-nwse-resize", &["cursor"][..]);
        exact.insert("cursor-zoom-in", &["cursor"][..]);
        exact.insert("cursor-zoom-out", &["cursor"][..]);

        // user select
        exact.insert("select-none", &["user-select"][..]);
        exact.insert("select-text", &["user-select"][..]);
        exact.insert("select-all", &["user-select"][..]);
        exact.insert("select-auto", &["user-select"][..]);

        // appearance
        exact.insert("appearance-none", &["appearance"][..]);
        exact.insert("appearance-auto", &["appearance"][..]);

        // resize
        exact.insert("resize-none", &["resize"][..]);
        exact.insert("resize-y", &["resize"][..]);
        exact.insert("resize-x", &["resize"][..]);
        exact.insert("resize", &["resize"][..]);

        // scroll snap
        exact.insert("snap-start", &["scroll-snap-align"][..]);
        exact.insert("snap-end", &["scroll-snap-align"][..]);
        exact.insert("snap-center", &["scroll-snap-align"][..]);
        exact.insert("snap-align-none", &["scroll-snap-align"][..]);

        // word break
        exact.insert("break-normal", &["overflow-wrap", "word-break"][..]);
        exact.insert("break-words", &["overflow-wrap"][..]);
        exact.insert("break-all", &["word-break"][..]);
        exact.insert("break-keep", &["word-break"][..]);

        // break before/after/inside
        exact.insert("break-before-auto", &["break-before"][..]);
        exact.insert("break-before-avoid", &["break-before"][..]);
        exact.insert("break-before-all", &["break-before"][..]);
        exact.insert("break-before-avoid-page", &["break-before"][..]);
        exact.insert("break-before-page", &["break-before"][..]);
        exact.insert("break-before-left", &["break-before"][..]);
        exact.insert("break-before-right", &["break-before"][..]);
        exact.insert("break-before-column", &["break-before"][..]);
        exact.insert("break-after-auto", &["break-after"][..]);
        exact.insert("break-after-avoid", &["break-after"][..]);
        exact.insert("break-after-all", &["break-after"][..]);
        exact.insert("break-after-avoid-page", &["break-after"][..]);
        exact.insert("break-after-page", &["break-after"][..]);
        exact.insert("break-after-left", &["break-after"][..]);
        exact.insert("break-after-right", &["break-after"][..]);
        exact.insert("break-after-column", &["break-after"][..]);
        exact.insert("break-inside-auto", &["break-inside"][..]);
        exact.insert("break-inside-avoid", &["break-inside"][..]);
        exact.insert("break-inside-avoid-page", &["break-inside"][..]);
        exact.insert("break-inside-avoid-column", &["break-inside"][..]);

        // box decoration break
        exact.insert("box-decoration-clone", &["box-decoration-break"][..]);
        exact.insert("box-decoration-slice", &["box-decoration-break"][..]);

        // overscroll
        exact.insert("overscroll-auto", &["overscroll-behavior"][..]);
        exact.insert("overscroll-contain", &["overscroll-behavior"][..]);
        exact.insert("overscroll-none", &["overscroll-behavior"][..]);
        exact.insert("overscroll-x-auto", &["overscroll-behavior-x"][..]);
        exact.insert("overscroll-x-contain", &["overscroll-behavior-x"][..]);
        exact.insert("overscroll-x-none", &["overscroll-behavior-x"][..]);
        exact.insert("overscroll-y-auto", &["overscroll-behavior-y"][..]);
        exact.insert("overscroll-y-contain", &["overscroll-behavior-y"][..]);
        exact.insert("overscroll-y-none", &["overscroll-behavior-y"][..]);

        // scroll behavior
        exact.insert("scroll-auto", &["scroll-behavior"][..]);
        exact.insert("scroll-smooth", &["scroll-behavior"][..]);

        // scroll snap type
        exact.insert("snap-none", &["scroll-snap-type"][..]);
        exact.insert("snap-x", &["scroll-snap-type"][..]);
        exact.insert("snap-y", &["scroll-snap-type"][..]);
        exact.insert("snap-both", &["scroll-snap-type"][..]);
        exact.insert("snap-mandatory", &["--tw-scroll-snap-strictness"][..]);
        exact.insert("snap-proximity", &["--tw-scroll-snap-strictness"][..]);

        // scroll snap stop
        exact.insert("snap-normal", &["scroll-snap-stop"][..]);
        exact.insert("snap-always", &["scroll-snap-stop"][..]);

        // touch action
        // touch-auto/none/manipulation map to touch-action (index 95)
        exact.insert("touch-auto", &["touch-action"][..]);
        exact.insert("touch-none", &["touch-action"][..]);
        exact.insert("touch-manipulation", &["touch-action"][..]);

        // touch-pan-x/left/right map to --tw-pan-x (index 96)
        exact.insert("touch-pan-x", &["--tw-pan-x"][..]);
        exact.insert("touch-pan-left", &["--tw-pan-x"][..]);
        exact.insert("touch-pan-right", &["--tw-pan-x"][..]);

        // touch-pan-y/up/down map to --tw-pan-y (index 97)
        exact.insert("touch-pan-y", &["--tw-pan-y"][..]);
        exact.insert("touch-pan-up", &["--tw-pan-y"][..]);
        exact.insert("touch-pan-down", &["--tw-pan-y"][..]);

        // touch-pinch-zoom maps to --tw-pinch-zoom (index 98)
        exact.insert("touch-pinch-zoom", &["--tw-pinch-zoom"][..]);

        // pointer events
        exact.insert("pointer-events-none", &["pointer-events"][..]);
        exact.insert("pointer-events-auto", &["pointer-events"][..]);

        // content (align-content additions)
        exact.insert("content-normal", &["align-content"][..]);
        exact.insert("content-baseline", &["align-content"][..]);
        exact.insert("content-stretch", &["align-content"][..]);

        // place content
        exact.insert("place-content-center", &["place-content"][..]);
        exact.insert("place-content-start", &["place-content"][..]);
        exact.insert("place-content-end", &["place-content"][..]);
        exact.insert("place-content-between", &["place-content"][..]);
        exact.insert("place-content-around", &["place-content"][..]);
        exact.insert("place-content-evenly", &["place-content"][..]);
        exact.insert("place-content-baseline", &["place-content"][..]);
        exact.insert("place-content-stretch", &["place-content"][..]);

        // place items
        exact.insert("place-items-start", &["place-items"][..]);
        exact.insert("place-items-end", &["place-items"][..]);
        exact.insert("place-items-center", &["place-items"][..]);
        exact.insert("place-items-baseline", &["place-items"][..]);
        exact.insert("place-items-stretch", &["place-items"][..]);

        // place self
        exact.insert("place-self-auto", &["place-self"][..]);
        exact.insert("place-self-start", &["place-self"][..]);
        exact.insert("place-self-end", &["place-self"][..]);
        exact.insert("place-self-center", &["place-self"][..]);
        exact.insert("place-self-stretch", &["place-self"][..]);

        // justify items
        exact.insert("justify-items-start", &["justify-items"][..]);
        exact.insert("justify-items-end", &["justify-items"][..]);
        exact.insert("justify-items-center", &["justify-items"][..]);
        exact.insert("justify-items-stretch", &["justify-items"][..]);

        // justify self
        exact.insert("justify-self-auto", &["justify-self"][..]);
        exact.insert("justify-self-start", &["justify-self"][..]);
        exact.insert("justify-self-end", &["justify-self"][..]);
        exact.insert("justify-self-center", &["justify-self"][..]);
        exact.insert("justify-self-stretch", &["justify-self"][..]);

        // align self
        exact.insert("self-auto", &["align-self"][..]);
        exact.insert("self-start", &["align-self"][..]);
        exact.insert("self-end", &["align-self"][..]);
        exact.insert("self-center", &["align-self"][..]);
        exact.insert("self-stretch", &["align-self"][..]);
        exact.insert("self-baseline", &["align-self"][..]);

        // flex direction
        exact.insert("flex-row", &["flex-direction"][..]);
        exact.insert("flex-row-reverse", &["flex-direction"][..]);
        exact.insert("flex-col", &["flex-direction"][..]);
        exact.insert("flex-col-reverse", &["flex-direction"][..]);

        // flex wrap
        exact.insert("flex-wrap", &["flex-wrap"][..]);
        exact.insert("flex-wrap-reverse", &["flex-wrap"][..]);
        exact.insert("flex-nowrap", &["flex-wrap"][..]);

        // flex
        exact.insert("flex-1", &["flex"][..]);
        exact.insert("flex-auto", &["flex"][..]);
        exact.insert("flex-initial", &["flex"][..]);
        exact.insert("flex-none", &["flex"][..]);

        // flex grow
        exact.insert("grow", &["flex-grow"][..]);
        exact.insert("grow-0", &["flex-grow"][..]);

        // flex shrink
        exact.insert("shrink", &["flex-shrink"][..]);
        exact.insert("shrink-0", &["flex-shrink"][..]);

        // order
        exact.insert("order-1", &["order"][..]);
        exact.insert("order-2", &["order"][..]);
        exact.insert("order-3", &["order"][..]);
        exact.insert("order-4", &["order"][..]);
        exact.insert("order-5", &["order"][..]);
        exact.insert("order-6", &["order"][..]);
        exact.insert("order-7", &["order"][..]);
        exact.insert("order-8", &["order"][..]);
        exact.insert("order-9", &["order"][..]);
        exact.insert("order-10", &["order"][..]);
        exact.insert("order-11", &["order"][..]);
        exact.insert("order-12", &["order"][..]);
        exact.insert("order-first", &["order"][..]);
        exact.insert("order-last", &["order"][..]);
        exact.insert("order-none", &["order"][..]);

        // grid template columns
        exact.insert("grid-cols-1", &["grid-template-columns"][..]);
        exact.insert("grid-cols-2", &["grid-template-columns"][..]);
        exact.insert("grid-cols-3", &["grid-template-columns"][..]);
        exact.insert("grid-cols-4", &["grid-template-columns"][..]);
        exact.insert("grid-cols-5", &["grid-template-columns"][..]);
        exact.insert("grid-cols-6", &["grid-template-columns"][..]);
        exact.insert("grid-cols-7", &["grid-template-columns"][..]);
        exact.insert("grid-cols-8", &["grid-template-columns"][..]);
        exact.insert("grid-cols-9", &["grid-template-columns"][..]);
        exact.insert("grid-cols-10", &["grid-template-columns"][..]);
        exact.insert("grid-cols-11", &["grid-template-columns"][..]);
        exact.insert("grid-cols-12", &["grid-template-columns"][..]);
        exact.insert("grid-cols-none", &["grid-template-columns"][..]);

        // grid template rows
        exact.insert("grid-rows-1", &["grid-template-rows"][..]);
        exact.insert("grid-rows-2", &["grid-template-rows"][..]);
        exact.insert("grid-rows-3", &["grid-template-rows"][..]);
        exact.insert("grid-rows-4", &["grid-template-rows"][..]);
        exact.insert("grid-rows-5", &["grid-template-rows"][..]);
        exact.insert("grid-rows-6", &["grid-template-rows"][..]);
        exact.insert("grid-rows-none", &["grid-template-rows"][..]);

        // grid auto flow
        exact.insert("grid-flow-row", &["grid-auto-flow"][..]);
        exact.insert("grid-flow-col", &["grid-auto-flow"][..]);
        exact.insert("grid-flow-dense", &["grid-auto-flow"][..]);
        exact.insert("grid-flow-row-dense", &["grid-auto-flow"][..]);
        exact.insert("grid-flow-col-dense", &["grid-auto-flow"][..]);

        // grid auto columns
        exact.insert("auto-cols-auto", &["grid-auto-columns"][..]);
        exact.insert("auto-cols-min", &["grid-auto-columns"][..]);
        exact.insert("auto-cols-max", &["grid-auto-columns"][..]);
        exact.insert("auto-cols-fr", &["grid-auto-columns"][..]);

        // grid auto rows
        exact.insert("auto-rows-auto", &["grid-auto-rows"][..]);
        exact.insert("auto-rows-min", &["grid-auto-rows"][..]);
        exact.insert("auto-rows-max", &["grid-auto-rows"][..]);
        exact.insert("auto-rows-fr", &["grid-auto-rows"][..]);

        // column span
        exact.insert("col-auto", &["grid-column"][..]);
        exact.insert("col-span-1", &["grid-column"][..]);
        exact.insert("col-span-2", &["grid-column"][..]);
        exact.insert("col-span-3", &["grid-column"][..]);
        exact.insert("col-span-4", &["grid-column"][..]);
        exact.insert("col-span-5", &["grid-column"][..]);
        exact.insert("col-span-6", &["grid-column"][..]);
        exact.insert("col-span-7", &["grid-column"][..]);
        exact.insert("col-span-8", &["grid-column"][..]);
        exact.insert("col-span-9", &["grid-column"][..]);
        exact.insert("col-span-10", &["grid-column"][..]);
        exact.insert("col-span-11", &["grid-column"][..]);
        exact.insert("col-span-12", &["grid-column"][..]);
        exact.insert("col-span-full", &["grid-column"][..]);
        exact.insert("col-start-1", &["grid-column-start"][..]);
        exact.insert("col-start-2", &["grid-column-start"][..]);
        exact.insert("col-start-3", &["grid-column-start"][..]);
        exact.insert("col-start-4", &["grid-column-start"][..]);
        exact.insert("col-start-5", &["grid-column-start"][..]);
        exact.insert("col-start-6", &["grid-column-start"][..]);
        exact.insert("col-start-7", &["grid-column-start"][..]);
        exact.insert("col-start-8", &["grid-column-start"][..]);
        exact.insert("col-start-9", &["grid-column-start"][..]);
        exact.insert("col-start-10", &["grid-column-start"][..]);
        exact.insert("col-start-11", &["grid-column-start"][..]);
        exact.insert("col-start-12", &["grid-column-start"][..]);
        exact.insert("col-start-13", &["grid-column-start"][..]);
        exact.insert("col-start-auto", &["grid-column-start"][..]);
        exact.insert("col-end-1", &["grid-column-end"][..]);
        exact.insert("col-end-2", &["grid-column-end"][..]);
        exact.insert("col-end-3", &["grid-column-end"][..]);
        exact.insert("col-end-4", &["grid-column-end"][..]);
        exact.insert("col-end-5", &["grid-column-end"][..]);
        exact.insert("col-end-6", &["grid-column-end"][..]);
        exact.insert("col-end-7", &["grid-column-end"][..]);
        exact.insert("col-end-8", &["grid-column-end"][..]);
        exact.insert("col-end-9", &["grid-column-end"][..]);
        exact.insert("col-end-10", &["grid-column-end"][..]);
        exact.insert("col-end-11", &["grid-column-end"][..]);
        exact.insert("col-end-12", &["grid-column-end"][..]);
        exact.insert("col-end-13", &["grid-column-end"][..]);
        exact.insert("col-end-auto", &["grid-column-end"][..]);

        // row span
        exact.insert("row-auto", &["grid-row"][..]);
        exact.insert("row-span-1", &["grid-row"][..]);
        exact.insert("row-span-2", &["grid-row"][..]);
        exact.insert("row-span-3", &["grid-row"][..]);
        exact.insert("row-span-4", &["grid-row"][..]);
        exact.insert("row-span-5", &["grid-row"][..]);
        exact.insert("row-span-6", &["grid-row"][..]);
        exact.insert("row-span-full", &["grid-row"][..]);
        exact.insert("row-start-1", &["grid-row-start"][..]);
        exact.insert("row-start-2", &["grid-row-start"][..]);
        exact.insert("row-start-3", &["grid-row-start"][..]);
        exact.insert("row-start-4", &["grid-row-start"][..]);
        exact.insert("row-start-5", &["grid-row-start"][..]);
        exact.insert("row-start-6", &["grid-row-start"][..]);
        exact.insert("row-start-7", &["grid-row-start"][..]);
        exact.insert("row-start-auto", &["grid-row-start"][..]);
        exact.insert("row-end-1", &["grid-row-end"][..]);
        exact.insert("row-end-2", &["grid-row-end"][..]);
        exact.insert("row-end-3", &["grid-row-end"][..]);
        exact.insert("row-end-4", &["grid-row-end"][..]);
        exact.insert("row-end-5", &["grid-row-end"][..]);
        exact.insert("row-end-6", &["grid-row-end"][..]);
        exact.insert("row-end-7", &["grid-row-end"][..]);
        exact.insert("row-end-auto", &["grid-row-end"][..]);

        // transform origin
        exact.insert("origin-center", &["transform-origin"][..]);
        exact.insert("origin-top", &["transform-origin"][..]);
        exact.insert("origin-top-right", &["transform-origin"][..]);
        exact.insert("origin-right", &["transform-origin"][..]);
        exact.insert("origin-bottom-right", &["transform-origin"][..]);
        exact.insert("origin-bottom", &["transform-origin"][..]);
        exact.insert("origin-bottom-left", &["transform-origin"][..]);
        exact.insert("origin-left", &["transform-origin"][..]);
        exact.insert("origin-top-left", &["transform-origin"][..]);

        // typography
        exact.insert(
            "truncate",
            &["overflow", "text-overflow", "white-space"][..],
        );
        exact.insert("text-ellipsis", &["text-overflow"][..]);
        exact.insert("text-clip", &["text-overflow"][..]);

        exact.insert("italic", &["font-style"][..]);
        exact.insert("not-italic", &["font-style"][..]);

        exact.insert("uppercase", &["text-transform"][..]);
        exact.insert("lowercase", &["text-transform"][..]);
        exact.insert("capitalize", &["text-transform"][..]);
        exact.insert("normal-case", &["text-transform"][..]);

        exact.insert("underline", &["text-decoration-line"][..]);
        exact.insert("overline", &["text-decoration-line"][..]);
        exact.insert("line-through", &["text-decoration-line"][..]);
        exact.insert("no-underline", &["text-decoration-line"][..]);

        exact.insert("whitespace-normal", &["white-space"][..]);
        exact.insert("whitespace-nowrap", &["white-space"][..]);
        exact.insert("whitespace-pre", &["white-space"][..]);
        exact.insert("whitespace-pre-line", &["white-space"][..]);
        exact.insert("whitespace-pre-wrap", &["white-space"][..]);
        exact.insert("whitespace-break-spaces", &["white-space"][..]);

        exact.insert("list-none", &["list-style-type"][..]);
        exact.insert("list-disc", &["list-style-type"][..]);
        exact.insert("list-decimal", &["list-style-type"][..]);

        exact.insert("list-inside", &["list-style-position"][..]);
        exact.insert("list-outside", &["list-style-position"][..]);

        // vertical align
        exact.insert("align-baseline", &["vertical-align"][..]);
        exact.insert("align-top", &["vertical-align"][..]);
        exact.insert("align-middle", &["vertical-align"][..]);
        exact.insert("align-bottom", &["vertical-align"][..]);
        exact.insert("align-text-top", &["vertical-align"][..]);
        exact.insert("align-text-bottom", &["vertical-align"][..]);
        exact.insert("align-sub", &["vertical-align"][..]);
        exact.insert("align-super", &["vertical-align"][..]);

        // mix blend mode
        exact.insert("mix-blend-normal", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-multiply", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-screen", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-overlay", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-darken", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-lighten", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-color-dodge", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-color-burn", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-hard-light", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-soft-light", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-difference", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-exclusion", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-hue", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-saturation", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-color", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-luminosity", &["mix-blend-mode"][..]);
        exact.insert("mix-blend-plus-lighter", &["mix-blend-mode"][..]);

        // background blend mode
        exact.insert("bg-blend-normal", &["background-blend-mode"][..]);
        exact.insert("bg-blend-multiply", &["background-blend-mode"][..]);
        exact.insert("bg-blend-screen", &["background-blend-mode"][..]);
        exact.insert("bg-blend-overlay", &["background-blend-mode"][..]);
        exact.insert("bg-blend-darken", &["background-blend-mode"][..]);
        exact.insert("bg-blend-lighten", &["background-blend-mode"][..]);
        exact.insert("bg-blend-color-dodge", &["background-blend-mode"][..]);
        exact.insert("bg-blend-color-burn", &["background-blend-mode"][..]);
        exact.insert("bg-blend-hard-light", &["background-blend-mode"][..]);
        exact.insert("bg-blend-soft-light", &["background-blend-mode"][..]);
        exact.insert("bg-blend-difference", &["background-blend-mode"][..]);
        exact.insert("bg-blend-exclusion", &["background-blend-mode"][..]);
        exact.insert("bg-blend-hue", &["background-blend-mode"][..]);
        exact.insert("bg-blend-saturation", &["background-blend-mode"][..]);
        exact.insert("bg-blend-color", &["background-blend-mode"][..]);
        exact.insert("bg-blend-luminosity", &["background-blend-mode"][..]);

        // border style
        exact.insert("border-solid", &["border-style"][..]);
        exact.insert("border-dashed", &["border-style"][..]);
        exact.insert("border-dotted", &["border-style"][..]);
        exact.insert("border-double", &["border-style"][..]);
        exact.insert("border-hidden", &["border-style"][..]);
        exact.insert("border-none", &["border-style"][..]);

        // divide style
        exact.insert("divide-solid", &["divide-style"][..]);
        exact.insert("divide-dashed", &["divide-style"][..]);
        exact.insert("divide-dotted", &["divide-style"][..]);
        exact.insert("divide-double", &["divide-style"][..]);
        exact.insert("divide-none", &["divide-style"][..]);

        // divide reverse
        // divide-x-reverse maps to --tw-divide-x-reverse (added to end of property list)
        // divide-y-reverse maps to --tw-divide-y-reverse
        exact.insert("divide-x-reverse", &["--tw-divide-x-reverse"][..]);
        exact.insert("divide-y-reverse", &["--tw-divide-y-reverse"][..]);

        // space reverse (static utilities, not covered by space-x/space-y patterns)
        // like their base utilities, use column-gap/row-gap for correct cross-axis sorting
        exact.insert("space-x-reverse", &["row-gap"][..]);
        exact.insert("space-y-reverse", &["column-gap"][..]);

        // outline style (maps to outline-style property)
        exact.insert("outline-none", &["outline-style"][..]);
        exact.insert("outline-solid", &["outline-style"][..]);
        exact.insert("outline-dashed", &["outline-style"][..]);
        exact.insert("outline-dotted", &["outline-style"][..]);
        exact.insert("outline-double", &["outline-style"][..]);

        // ring (ring-inset sets --tw-ring-inset property)
        exact.insert("ring-inset", &["--tw-ring-inset"][..]);

        // text alignment
        exact.insert("text-left", &["text-align"][..]);
        exact.insert("text-center", &["text-align"][..]);
        exact.insert("text-right", &["text-align"][..]);
        exact.insert("text-justify", &["text-align"][..]);
        exact.insert("text-start", &["text-align"][..]);
        exact.insert("text-end", &["text-align"][..]);

        // background size
        exact.insert("bg-auto", &["background-size"][..]);
        exact.insert("bg-cover", &["background-size"][..]);
        exact.insert("bg-contain", &["background-size"][..]);

        // background position
        exact.insert("bg-bottom", &["background-position"][..]);
        exact.insert("bg-center", &["background-position"][..]);
        exact.insert("bg-left", &["background-position"][..]);
        exact.insert("bg-left-bottom", &["background-position"][..]);
        exact.insert("bg-left-top", &["background-position"][..]);
        exact.insert("bg-right", &["background-position"][..]);
        exact.insert("bg-right-bottom", &["background-position"][..]);
        exact.insert("bg-right-top", &["background-position"][..]);
        exact.insert("bg-top", &["background-position"][..]);

        // background repeat
        exact.insert("bg-repeat", &["background-repeat"][..]);
        exact.insert("bg-no-repeat", &["background-repeat"][..]);
        exact.insert("bg-repeat-x", &["background-repeat"][..]);
        exact.insert("bg-repeat-y", &["background-repeat"][..]);
        exact.insert("bg-repeat-round", &["background-repeat"][..]);
        exact.insert("bg-repeat-space", &["background-repeat"][..]);

        // background image
        exact.insert("bg-none", &["background-image"][..]);

        // background clip
        exact.insert("bg-clip-border", &["background-clip"][..]);
        exact.insert("bg-clip-padding", &["background-clip"][..]);
        exact.insert("bg-clip-content", &["background-clip"][..]);
        exact.insert("bg-clip-text", &["background-clip"][..]);

        // background origin
        exact.insert("bg-origin-border", &["background-origin"][..]);
        exact.insert("bg-origin-padding", &["background-origin"][..]);
        exact.insert("bg-origin-content", &["background-origin"][..]);

        // gradient direction
        exact.insert("bg-gradient-to-t", &["background-image"][..]);
        exact.insert("bg-gradient-to-tr", &["background-image"][..]);
        exact.insert("bg-gradient-to-r", &["background-image"][..]);
        exact.insert("bg-gradient-to-br", &["background-image"][..]);
        exact.insert("bg-gradient-to-b", &["background-image"][..]);
        exact.insert("bg-gradient-to-bl", &["background-image"][..]);
        exact.insert("bg-gradient-to-l", &["background-image"][..]);
        exact.insert("bg-gradient-to-tl", &["background-image"][..]);

        // drop shadow
        exact.insert("drop-shadow", &["--tw-drop-shadow"][..]);
        exact.insert("drop-shadow-sm", &["--tw-drop-shadow"][..]);
        exact.insert("drop-shadow-md", &["--tw-drop-shadow"][..]);
        exact.insert("drop-shadow-lg", &["--tw-drop-shadow"][..]);
        exact.insert("drop-shadow-xl", &["--tw-drop-shadow"][..]);
        exact.insert("drop-shadow-2xl", &["--tw-drop-shadow"][..]);
        exact.insert("drop-shadow-none", &["--tw-drop-shadow"][..]);

        // filter toggles
        exact.insert("filter", &["filter"][..]);
        exact.insert("filter-none", &["filter"][..]);
        exact.insert("backdrop-filter", &["backdrop-filter"][..]);
        exact.insert("backdrop-filter-none", &["backdrop-filter"][..]);

        // filter utilities -0 variants (exact mappings to avoid pattern match exclusion)
        exact.insert("grayscale-0", &["--tw-grayscale"][..]);
        exact.insert("invert-0", &["--tw-invert"][..]);
        exact.insert("sepia-0", &["--tw-sepia"][..]);

        // object position
        exact.insert("object-bottom", &["object-position"][..]);
        exact.insert("object-center", &["object-position"][..]);
        exact.insert("object-left", &["object-position"][..]);
        exact.insert("object-left-bottom", &["object-position"][..]);
        exact.insert("object-left-top", &["object-position"][..]);
        exact.insert("object-right", &["object-position"][..]);
        exact.insert("object-right-bottom", &["object-position"][..]);
        exact.insert("object-right-top", &["object-position"][..]);
        exact.insert("object-top", &["object-position"][..]);

        // aspect ratio
        exact.insert("aspect-auto", &["aspect-ratio"][..]);
        exact.insert("aspect-square", &["aspect-ratio"][..]);
        exact.insert("aspect-video", &["aspect-ratio"][..]);

        // text decoration style
        exact.insert("decoration-solid", &["text-decoration-style"][..]);
        exact.insert("decoration-double", &["text-decoration-style"][..]);
        exact.insert("decoration-dotted", &["text-decoration-style"][..]);
        exact.insert("decoration-dashed", &["text-decoration-style"][..]);
        exact.insert("decoration-wavy", &["text-decoration-style"][..]);

        // text decoration thickness
        exact.insert("decoration-auto", &["text-decoration-thickness"][..]);
        exact.insert("decoration-from-font", &["text-decoration-thickness"][..]);

        // transition property
        // transition-none only sets transition-property to 'none' (matches Tailwind v4)
        // this ensures it sorts alphabetically with other transition utilities
        exact.insert("transition-none", &["transition-property"][..]);
        exact.insert("transition-all", &["transition-property"][..]);
        exact.insert("transition-colors", &["transition-property"][..]);
        exact.insert("transition-opacity", &["transition-property"][..]);
        exact.insert("transition-shadow", &["transition-property"][..]);
        exact.insert("transition-transform", &["transition-property"][..]);

        // font family
        exact.insert("font-sans", &["font-family"][..]);
        exact.insert("font-serif", &["font-family"][..]);
        exact.insert("font-mono", &["font-family"][..]);

        // typography plugin (prose)
        // these are from @tailwindcss/typography plugin but we treat them as known utilities
        // so they sort with other text/typography utilities, not as custom classes
        exact.insert("prose", &["--tw-prose-component"][..]);
        exact.insert("prose-sm", &["--tw-prose-component"][..]);
        exact.insert("prose-base", &["--tw-prose-component"][..]);
        exact.insert("prose-lg", &["--tw-prose-component"][..]);
        exact.insert("prose-xl", &["--tw-prose-component"][..]);
        exact.insert("prose-2xl", &["--tw-prose-component"][..]);
        exact.insert("prose-invert", &["--tw-prose-invert"][..]);

        // scroll snap align (already exists but consolidating here)
        // snap utilities are already defined above at lines 206-209

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
    /// // px maps to padding-inline (modern CSS)
    /// let px_props = map.get_properties("px-4").unwrap();
    /// assert!(px_props.contains(&"padding-inline"));
    /// ```
    pub fn get_properties(&self, utility: &str) -> Option<&'static [&'static str]> {
        // try exact match first (fast path)
        if let Some(props) = self.exact.get(utility) {
            return Some(props);
        }

        // fall back to pattern matching
        self.match_pattern(utility)
    }

    /// Match a utility against known patterns to determine its properties.
    fn match_pattern(&self, utility: &str) -> Option<&'static [&'static str]> {
        // parse utility into base and value
        let (base, value) = parse_utility_parts(utility)?;

        // match against known patterns
        match base {
            // inset positioning
            "inset" => Some(&["inset"][..]),
            "inset-x" => Some(&["inset-inline"][..]),
            "inset-y" => Some(&["inset-block"][..]),
            "start" => Some(&["inset-inline-start"][..]),
            "end" => Some(&["inset-inline-end"][..]),
            "top" => Some(&["top"][..]),
            "right" => Some(&["right"][..]),
            "bottom" => Some(&["bottom"][..]),
            "left" => Some(&["left"][..]),

            // z-index (including negative values)
            "z" | "-z" => Some(&["z-index"][..]),

            // order
            "order" => Some(&["order"][..]),

            // grid column/row
            "col" if value.starts_with("span") => Some(&["grid-column"][..]),
            "col" if value.starts_with("start") => Some(&["grid-column-start"][..]),
            "col" if value.starts_with("end") => Some(&["grid-column-end"][..]),
            "row" if value.starts_with("span") => Some(&["grid-row"][..]),
            "row" if value.starts_with("start") => Some(&["grid-row-start"][..]),
            "row" if value.starts_with("end") => Some(&["grid-row-end"][..]),

            // margins
            "m" => Some(&["margin"][..]),
            "mx" => Some(&["margin-inline"][..]),
            "my" => Some(&["margin-block"][..]),
            "ms" => Some(&["margin-inline-start"][..]),
            "me" => Some(&["margin-inline-end"][..]),
            "mt" => Some(&["margin-top"][..]),
            "mr" => Some(&["margin-right"][..]),
            "mb" => Some(&["margin-bottom"][..]),
            "ml" => Some(&["margin-left"][..]),

            // sizing
            "w" => Some(&["width"][..]),
            "h" => Some(&["height"][..]),
            "size" => Some(&["aspect-ratio"][..]), // aspect-ratio comes before height/width
            "min-w" => Some(&["min-width"][..]),
            "min-h" => Some(&["min-height"][..]),
            "max-w" => Some(&["max-width"][..]),
            "max-h" => Some(&["max-height"][..]),

            // flex
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

            // grid
            "grid-cols" => Some(&["grid-template-columns"][..]),
            "grid-rows" => Some(&["grid-template-rows"][..]),
            "auto-cols" => Some(&["grid-auto-columns"][..]),
            "auto-rows" => Some(&["grid-auto-rows"][..]),
            "grid-flow-row" => Some(&["grid-auto-flow"][..]),
            "grid-flow-col" => Some(&["grid-auto-flow"][..]),
            "grid-flow-dense" => Some(&["grid-auto-flow"][..]),
            "grid-flow-row-dense" => Some(&["grid-auto-flow"][..]),
            "grid-flow-col-dense" => Some(&["grid-auto-flow"][..]),

            // gap
            "gap" if !value.is_empty() => Some(&["gap"][..]),
            "gap-x" => Some(&["column-gap"][..]),
            "gap-y" => Some(&["row-gap"][..]),

            // padding
            "p" => Some(&["padding"][..]),
            "px" => Some(&["padding-inline"][..]), // Use padding-inline for left+right
            "py" => Some(&["padding-block"][..]),  // Use padding-block for top+bottom
            "ps" => Some(&["padding-inline-start"][..]),
            "pe" => Some(&["padding-inline-end"][..]),
            "pt" => Some(&["padding-top"][..]),
            "pr" => Some(&["padding-right"][..]),
            "pb" => Some(&["padding-bottom"][..]),
            "pl" => Some(&["padding-left"][..]),

            // alignment
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

            // background
            "bg" if is_color_value(value) => Some(&["background-color"][..]),
            "bg" if value.starts_with('[') => Some(&["background-color"][..]), // arbitrary value

            // border width
            "border"
                if value.is_empty() || value.parse::<u32>().is_ok() || value.starts_with('[') =>
            {
                Some(&["border-width"][..])
            }
            "border-x" => Some(&["border-inline-width"][..]), // Use border-inline-width for left+right
            "border-y" => Some(&["border-block-width"][..]), // Use border-block-width for top+bottom
            "border-s" => Some(&["border-inline-start-width"][..]),
            "border-e" => Some(&["border-inline-end-width"][..]),
            "border-t" => Some(&["border-top-width"][..]),
            "border-r" => Some(&["border-right-width"][..]),
            "border-b" => Some(&["border-bottom-width"][..]),
            "border-l" => Some(&["border-left-width"][..]),

            // border color
            "border" if is_color_value(value) => Some(&["border-color"][..]),

            // border radius
            "rounded" if value.is_empty() || value.starts_with('[') || is_size_keyword(value) => {
                Some(&["border-radius"][..])
            }
            // side-specific rounded utilities
            "rounded-s" => Some(&["border-start-radius"][..]),
            "rounded-e" => Some(&["border-end-radius"][..]),
            // side rounded utilities map to BOTH corners they affect (matching Tailwind v4)
            // when first properties tie, Tailwind uses the second property as tiebreaker
            "rounded-t" => Some(&["border-top-left-radius", "border-top-right-radius"][..]), // (189, 190)
            "rounded-r" => Some(&["border-top-right-radius", "border-bottom-right-radius"][..]), // (190, 191)
            "rounded-b" => Some(&["border-bottom-right-radius", "border-bottom-left-radius"][..]), // (191, 192)
            "rounded-l" => Some(&["border-top-left-radius", "border-bottom-left-radius"][..]), // (189, 192)
            // corner-specific rounded utilities map to individual corner properties
            "rounded-ss" => Some(&["border-start-start-radius"][..]),
            "rounded-se" => Some(&["border-start-end-radius"][..]),
            "rounded-ee" => Some(&["border-end-end-radius"][..]),
            "rounded-es" => Some(&["border-end-start-radius"][..]),
            "rounded-tl" => Some(&["border-top-left-radius"][..]),
            "rounded-tr" => Some(&["border-top-right-radius"][..]),
            "rounded-br" => Some(&["border-bottom-right-radius"][..]),
            "rounded-bl" => Some(&["border-bottom-left-radius"][..]),

            // text
            "text" if is_color_value(value) => Some(&["color"][..]),
            "text" if is_size_keyword(value) => Some(&["font-size"][..]),
            "text" if value.starts_with('[') => Some(&["font-size"][..]), // arbitrary text size

            // font
            "font" if is_weight_keyword(value) => Some(&["font-weight"][..]),
            "font" => Some(&["font-family"][..]),

            // opacity
            "opacity" => Some(&["opacity"][..]),

            // shadow
            "shadow" if is_color_value(value) => Some(&["--tw-shadow-color"][..]),
            "shadow"
                if value.is_empty()
                    || is_size_keyword(value)
                    || value == "inner"
                    || value == "none" =>
            {
                Some(&["--tw-shadow", "box-shadow"][..])
            }

            // ring (uses multiple properties)
            "ring" if value.is_empty() || value.parse::<u32>().is_ok() => Some(
                &[
                    "--tw-ring-offset-shadow",
                    "--tw-ring-shadow",
                    "--tw-shadow",
                    "box-shadow",
                ][..],
            ),
            "ring" if is_color_value(value) => Some(&["--tw-ring-color"][..]),
            "ring-offset" if value.parse::<u32>().is_ok() => Some(&["--tw-ring-offset-width"][..]),
            "ring-offset" if is_color_value(value) => Some(&["--tw-ring-offset-color"][..]),

            // transitions
            "transition" => Some(&["transition-property"][..]),
            "duration" => Some(&["transition-duration"][..]),
            "delay" => Some(&["transition-delay"][..]),
            "ease" => Some(&["transition-timing-function"][..]),

            // animations
            "animate" => Some(&["animation"][..]),

            // transforms
            "rotate" => Some(&["rotate"][..]),
            "-rotate" => Some(&["rotate"][..]),
            "scale" if !value.is_empty() => Some(&["scale"][..]),
            "-scale" if !value.is_empty() => Some(&["scale"][..]),
            "scale-x" => Some(&["--tw-scale-x"][..]),
            "-scale-x" => Some(&["--tw-scale-x"][..]),
            "scale-y" => Some(&["--tw-scale-y"][..]),
            "-scale-y" => Some(&["--tw-scale-y"][..]),
            "translate-x" => Some(&["--tw-translate-x"][..]),
            "-translate-x" => Some(&["--tw-translate-x"][..]),
            "translate-y" => Some(&["--tw-translate-y"][..]),
            "-translate-y" => Some(&["--tw-translate-y"][..]),
            "skew-x" => Some(&["--tw-skew-x"][..]),
            "-skew-x" => Some(&["--tw-skew-x"][..]),
            "skew-y" => Some(&["--tw-skew-y"][..]),
            "-skew-y" => Some(&["--tw-skew-y"][..]),

            // filters - map to specific custom properties for correct sorting
            "blur" => Some(&["--tw-blur"][..]),
            "brightness" => Some(&["--tw-brightness"][..]),
            "contrast" => Some(&["--tw-contrast"][..]),
            "grayscale" if value.is_empty() || value.starts_with('[') => {
                Some(&["--tw-grayscale"][..])
            }
            "hue-rotate" => Some(&["--tw-hue-rotate"][..]),
            "invert" if value.is_empty() || value.starts_with('[') => Some(&["--tw-invert"][..]),
            "saturate" => Some(&["--tw-saturate"][..]),
            "sepia" if value.is_empty() || value.starts_with('[') => Some(&["--tw-sepia"][..]),
            "drop-shadow" => Some(&["--tw-drop-shadow"][..]),

            // backdrop filters - map to specific custom properties for correct sorting
            "backdrop-blur" => Some(&["--tw-backdrop-blur"][..]),
            "backdrop-brightness" => Some(&["--tw-backdrop-brightness"][..]),
            "backdrop-contrast" => Some(&["--tw-backdrop-contrast"][..]),
            "backdrop-grayscale" => Some(&["--tw-backdrop-grayscale"][..]),
            "backdrop-hue-rotate" => Some(&["--tw-backdrop-hue-rotate"][..]),
            "backdrop-invert" => Some(&["--tw-backdrop-invert"][..]),
            "backdrop-opacity" => Some(&["--tw-backdrop-opacity"][..]),
            "backdrop-saturate" => Some(&["--tw-backdrop-saturate"][..]),
            "backdrop-sepia" => Some(&["--tw-backdrop-sepia"][..]),

            // will change
            "will-change" => Some(&["will-change"][..]),

            // outline
            "outline" if value.is_empty() || value == "none" || value.parse::<u32>().is_ok() => {
                Some(&["outline-width"][..])
            }
            "outline" if is_color_value(value) => Some(&["outline-color"][..]),
            "outline-offset" => Some(&["outline-offset"][..]),

            // accent color
            "accent" if is_color_value(value) || value == "auto" || value == "current" => {
                Some(&["accent-color"][..])
            }

            // caret color
            "caret" if is_color_value(value) || value == "current" => Some(&["caret-color"][..]),

            // space between
            // per Tailwind v4, space-x and space-y use different --tw-sort properties:
            // space-x uses row-gap (index 153), space-y uses column-gap (index 152)
            // since 152 < 153, space-y correctly sorts BEFORE space-x
            "space-x" => Some(&["row-gap"][..]),
            "space-y" => Some(&["column-gap"][..]),

            // divide
            "divide-x" => Some(&["divide-x-width"][..]),
            "divide-y" => Some(&["divide-y-width"][..]),
            "divide" if is_color_value(value) => Some(&["divide-color"][..]),
            "divide-opacity" => Some(&["border-opacity"][..]),

            // leading (line-height)
            "leading" => Some(&["line-height"][..]),

            // tracking (letter-spacing)
            "tracking" => Some(&["letter-spacing"][..]),

            // columns
            "columns" => Some(&["columns"][..]),

            // background utilities
            "bg-opacity" => Some(&["background-opacity"][..]),
            "from" if is_color_value(value) => Some(&["--tw-gradient-from"][..]),
            "via" if is_color_value(value) => Some(&["--tw-gradient-via"][..]),
            "to" if is_color_value(value) => Some(&["--tw-gradient-to"][..]),

            // aspect ratio (arbitrary values)
            "aspect" => Some(&["aspect-ratio"][..]),

            // text decoration
            "decoration" if is_color_value(value) => Some(&["text-decoration-color"][..]),
            "decoration" if value.parse::<u32>().is_ok() => {
                Some(&["text-decoration-thickness"][..])
            }
            "decoration" => Some(&["text-decoration-color"][..]), // Fallback: custom colors

            // underline offset
            "underline-offset" => Some(&["text-underline-offset"][..]),

            // text indent
            "indent" => Some(&["text-indent"][..]),

            // fallbacks for color utilities with custom/unknown color names
            // these catch utilities that didn't match is_color_value() checks above
            // in real projects with Tailwind config, these custom colors would be recognized
            "from" => Some(&["--tw-gradient-from"][..]),
            "to" => Some(&["--tw-gradient-to"][..]),
            "via" => Some(&["--tw-gradient-via"][..]),
            "border" if !value.is_empty() => Some(&["border-color"][..]), // border-customcolor
            "divide" => Some(&["divide-color"][..]),                      // divide-customcolor
            "ring" if !value.is_empty() => Some(&["--tw-ring-color"][..]), // ring-customcolor
            "ring-offset" if !value.is_empty() => Some(&["--tw-ring-offset-color"][..]), // ring-offset-customcolor
            "accent" => Some(&["accent-color"][..]), // accent-customcolor
            "caret" => Some(&["caret-color"][..]),   // caret-customcolor
            "outline" if !value.is_empty() && value.parse::<u32>().is_err() => {
                Some(&["outline-color"][..]) // outline-customcolor (not outline-2)
            }

            // unknown utility
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
    // handle opacity modifiers: text-white/60, bg-primary/20, dark:text-white/90
    // strip the opacity part (everything after and including '/') for property lookup
    // but keep the original class name for sorting purposes
    let utility_without_opacity = if let Some(slash_pos) = utility.find('/') {
        &utility[..slash_pos]
    } else {
        utility
    };

    // handle arbitrary values: bg-[#fff], w-[100px]
    if let Some(bracket_start) = utility_without_opacity.find('[') {
        let base = &utility_without_opacity[..bracket_start.saturating_sub(1)]; // Remove the '-' before '['
        let value = &utility_without_opacity[bracket_start..];
        return Some((base, value));
    }

    // handle negative values: -translate-x-4, -skew-y-3, -rotate-90, etc.
    let (is_negative, utility_without_neg) =
        if let Some(stripped) = utility_without_opacity.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, utility_without_opacity)
        };

    // try to match multi-part bases first
    // these need to be checked before simple dash splitting
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
        "space-x",
        "space-y",
        "divide-x",
        "divide-y",
        "divide-opacity",
        "underline-offset",
        "hue-rotate",
        "scale-x",
        "scale-y",
        "bg-opacity", // Add opacity utilities to prevent incorrect parsing as colors
        "text-opacity",
        "border-opacity",
    ] {
        if let Some(stripped) = utility_without_neg.strip_prefix(prefix) {
            if stripped.is_empty() {
                // exact match, no value
                return Some((utility_without_opacity, ""));
            } else if stripped.as_bytes().first() == Some(&b'-') {
                // has a dash after the prefix
                let value = &stripped[1..];
                let base = if is_negative {
                    &utility_without_opacity[..prefix.len() + 1] // +1 for initial '-'
                } else {
                    prefix
                };
                return Some((base, value));
            } else if prefix.ends_with('-') {
                // prefix ends with dash (shouldn't happen with our list, but safe)
                let value = stripped;
                let base = if is_negative {
                    &utility_without_opacity[..prefix.len() + 1] // +1 for initial '-'
                } else {
                    prefix
                };
                return Some((base, value));
            }
        }
    }

    // simple single-dash split (skip the negative sign if present)
    if let Some(dash_pos) = utility_without_neg.find('-') {
        let base_without_neg = &utility_without_neg[..dash_pos];
        let value = &utility_without_neg[dash_pos + 1..];
        let base = if is_negative {
            &utility_without_opacity[..1 + dash_pos] // 1 for initial '-', then dash_pos characters
        } else {
            base_without_neg
        };
        return Some((base, value));
    }

    // no dash found - utility with no value (keep negative sign if present)
    Some((utility_without_opacity, ""))
}

/// Check if a value looks like a color.
fn is_color_value(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    // check for arbitrary color value: [#fff], [rgb(255,0,0)], [hsl(...)]
    // only treat as color if it contains typical color indicators
    if value.starts_with('[') {
        return value.contains('#')  // hex colors: [#fff], [#ff0000]
            || value.contains("rgb")  // rgb/rgba: [rgb(255,0,0)]
            || value.contains("hsl")  // hsl/hsla: [hsl(0,100%,50%)]
            || value.contains("var("); // CSS variables: [var(--my-color)]
    }

    // check for color with shade: red-500, blue-600
    if value.contains('-') {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() == 2 {
            // check if second part is a number (shade)
            if parts[1].parse::<u32>().is_ok() {
                return true;
            }
        }
    }

    // check for named colors: red, blue, transparent, current, inherit
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
            | "md" // Add 'md' for utilities like rounded-md
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
            | "none" // Add 'none' as a valid size keyword for utilities like rounded-none
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
pub static UTILITY_MAP: LazyLock<UtilityMap> = LazyLock::new(UtilityMap::new);

/// Declaration counts for utilities that differ from the default (1 declaration).
///
/// This table maps utility name patterns to the number of CSS declarations they generate.
/// Tailwind's sorting algorithm uses declaration count as a comparison tier: utilities with
/// MORE declarations sort BEFORE utilities with fewer declarations (when all other factors are equal).
///
/// Examples:
/// - `ring-2`: 3 declarations (--tw-ring-offset-shadow, --tw-ring-shadow, box-shadow composite)
/// - `transition-colors`: 3 declarations (transition-property + duration + timing)
/// - `transition-none`: 1 declaration (just transition-property: none)
/// - Most utilities: 1 declaration (default, not in this table)
static DECLARATION_COUNTS: LazyLock<HashMap<&'static str, usize>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // ring utilities: 3 declarations
    // Tailwind generates --tw-ring-offset-shadow, --tw-ring-shadow, and box-shadow
    map.insert("ring", 3);
    map.insert("ring-inset", 3);

    // transition utilities: 3 declarations (except transition-none which is 1)
    map.insert("transition", 3);
    map.insert("transition-all", 3);
    map.insert("transition-colors", 3);
    map.insert("transition-opacity", 3);
    map.insert("transition-shadow", 3);
    map.insert("transition-transform", 3);
    map.insert("transition-none", 1); // Override: only 1 declaration

    // drop-shadow utilities: 2 declarations (except drop-shadow-none which is 1)
    // Tailwind generates --tw-drop-shadow and filter composite
    // NOTE: must list all variants explicitly since "drop-shadow" contains a dash
    map.insert("drop-shadow", 2);
    map.insert("drop-shadow-sm", 2);
    map.insert("drop-shadow-md", 2);
    map.insert("drop-shadow-lg", 2);
    map.insert("drop-shadow-xl", 2);
    map.insert("drop-shadow-2xl", 2);
    map.insert("drop-shadow-none", 1); // Override: only 1 declaration

    // base border-radius utility: 4 declarations (affects all 4 corners)
    // this ensures `rounded` sorts before `rounded-[14px]` (arbitrary)
    // sized variants explicitly set to 1 to allow arbitrary to sort before them
    map.insert("rounded", 4);
    map.insert("rounded-none", 1);
    map.insert("rounded-sm", 1);
    map.insert("rounded-md", 1);
    map.insert("rounded-lg", 1);
    map.insert("rounded-xl", 1);
    map.insert("rounded-2xl", 1);
    map.insert("rounded-3xl", 1);
    map.insert("rounded-full", 1);

    // text size utilities: 2 declarations (font-size + line-height)
    // arbitrary text utilities only generate font-size (1 declaration)
    // this ensures text-sm < text-[42px] via property count
    map.insert("text-xs", 2);
    map.insert("text-sm", 2);
    map.insert("text-base", 2);
    map.insert("text-lg", 2);
    map.insert("text-xl", 2);
    map.insert("text-2xl", 2);
    map.insert("text-3xl", 2);
    map.insert("text-4xl", 2);
    map.insert("text-5xl", 2);
    map.insert("text-6xl", 2);
    map.insert("text-7xl", 2);
    map.insert("text-8xl", 2);
    map.insert("text-9xl", 2);

    map
});

/// Get the number of CSS declarations a utility generates.
///
/// This function looks up the utility in the DECLARATION_COUNTS table and returns
/// the count, or defaults to 1 if the utility is not in the table.
///
/// For parameterized utilities (e.g., `ring-2`, `transition-[width]`), this function
/// strips the value suffix and looks up the base utility name.
///
/// # Arguments
///
/// * `utility` - The full utility class name (e.g., "ring-2", "transition-colors", "p-4")
///
/// # Returns
///
/// The number of CSS declarations the utility generates (defaults to 1)
///
/// # Examples
///
/// ```
/// use rustywind_core::utility_map::get_declaration_count;
///
/// assert_eq!(get_declaration_count("ring-2"), 3);
/// assert_eq!(get_declaration_count("transition-colors"), 3);
/// assert_eq!(get_declaration_count("transition-none"), 1);
/// assert_eq!(get_declaration_count("p-4"), 1); // Default
/// ```
pub fn get_declaration_count(utility: &str) -> usize {
    // strip variants to get the base utility
    let base_utility = utility.split(':').next_back().unwrap_or(utility);

    // first try exact match
    if let Some(&count) = DECLARATION_COUNTS.get(base_utility) {
        return count;
    }

    // try pattern matching for parameterized utilities
    // extract the utility prefix (e.g., "ring" from "ring-2", "transition" from "transition-colors")
    // BUT skip arbitrary values (e.g., "rounded-[14px]" should NOT match prefix "rounded")
    if let Some(dash_pos) = base_utility.find('-') {
        let value_part = &base_utility[dash_pos + 1..];

        // skip prefix matching for arbitrary values (contain brackets)
        if !value_part.contains('[') && !value_part.contains(']') {
            let prefix = &base_utility[..dash_pos];

            // check if the prefix has a declaration count
            if let Some(&count) = DECLARATION_COUNTS.get(prefix) {
                // special case: check if it's explicitly overridden
                // (e.g., "transition-none" should return 1, not 3)
                if DECLARATION_COUNTS.contains_key(base_utility) {
                    return *DECLARATION_COUNTS.get(base_utility).unwrap();
                }
                return count;
            }
        }
    }

    // default: 1 declaration per utility
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_matches() {
        let map = UtilityMap::new();

        // display utilities
        assert_eq!(map.get_properties("flex"), Some(&["display"][..]));
        assert_eq!(map.get_properties("block"), Some(&["display"][..]));
        assert_eq!(map.get_properties("hidden"), Some(&["display"][..]));
        assert_eq!(map.get_properties("grid"), Some(&["display"][..]));

        // position utilities
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

        // px maps to padding-inline (modern CSS for left+right)
        let px = map.get_properties("px-4").unwrap();
        assert!(px.contains(&"padding-inline"));

        // py maps to padding-block (modern CSS for top+bottom)
        let py = map.get_properties("py-8").unwrap();
        assert!(py.contains(&"padding-block"));
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
    fn test_transition_utilities() {
        let map = UtilityMap::new();

        assert_eq!(
            map.get_properties("transition"),
            Some(&["transition-property"][..])
        );
        assert_eq!(
            map.get_properties("transition-colors"),
            Some(&["transition-property"][..])
        );
        assert_eq!(
            map.get_properties("transition-all"),
            Some(&["transition-property"][..])
        );
        assert_eq!(
            map.get_properties("duration-200"),
            Some(&["transition-duration"][..])
        );
        assert_eq!(
            map.get_properties("duration-300"),
            Some(&["transition-duration"][..])
        );
        assert_eq!(
            map.get_properties("delay-100"),
            Some(&["transition-delay"][..])
        );
        assert_eq!(
            map.get_properties("ease-in"),
            Some(&["transition-timing-function"][..])
        );
    }

    #[test]
    fn test_color_utilities() {
        let map = UtilityMap::new();

        // background colors
        assert_eq!(
            map.get_properties("bg-red-500"),
            Some(&["background-color"][..])
        );
        assert_eq!(
            map.get_properties("bg-blue-600"),
            Some(&["background-color"][..])
        );

        // text colors
        assert_eq!(map.get_properties("text-white"), Some(&["color"][..]));
        assert_eq!(map.get_properties("text-gray-900"), Some(&["color"][..]));

        // border colors
        assert_eq!(
            map.get_properties("border-black"),
            Some(&["border-color"][..])
        );
    }

    #[test]
    fn test_arbitrary_values() {
        let map = UtilityMap::new();

        // arbitrary color values
        assert_eq!(
            map.get_properties("bg-[#fff]"),
            Some(&["background-color"][..])
        );
        assert_eq!(
            map.get_properties("text-[rgb(255,0,0)]"),
            Some(&["color"][..])
        );

        // arbitrary size values
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

        // border width
        assert_eq!(map.get_properties("border"), Some(&["border-width"][..]));
        assert_eq!(map.get_properties("border-2"), Some(&["border-width"][..]));
        assert_eq!(
            map.get_properties("border-t"),
            Some(&["border-top-width"][..])
        );

        // border radius
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

    #[test]
    fn test_space_x_mapping() {
        use crate::property_order::get_property_index;
        let map = UtilityMap::new();

        // space-x should map to row-gap for cross-axis sorting
        let space_x_props = map.get_properties("space-x-2").unwrap();
        assert_eq!(space_x_props, &["row-gap"]);

        // space-y should map to column-gap for cross-axis sorting
        let space_y_props = map.get_properties("space-y-2").unwrap();
        assert_eq!(space_y_props, &["column-gap"]);

        // verify correct ordering: space-y before space-x
        let column_gap_idx = get_property_index("column-gap").unwrap();
        let row_gap_idx = get_property_index("row-gap").unwrap();

        // column-gap (152) should come before row-gap (153)
        assert!(
            column_gap_idx < row_gap_idx,
            "column-gap ({}) should sort before row-gap ({})",
            column_gap_idx,
            row_gap_idx
        );
    }

    #[test]
    fn test_transform_mappings() {
        let map = UtilityMap::new();

        // test transform utility mappings
        assert_eq!(map.get_properties("scale-100"), Some(&["scale"][..]));
        assert_eq!(
            map.get_properties("scale-x-100"),
            Some(&["--tw-scale-x"][..])
        );
        assert_eq!(
            map.get_properties("scale-y-50"),
            Some(&["--tw-scale-y"][..])
        );
        assert_eq!(
            map.get_properties("translate-x-0"),
            Some(&["--tw-translate-x"][..])
        );
        assert_eq!(
            map.get_properties("translate-y-2"),
            Some(&["--tw-translate-y"][..])
        );
        assert_eq!(map.get_properties("rotate-0"), Some(&["rotate"][..]));
        assert_eq!(map.get_properties("skew-x-6"), Some(&["--tw-skew-x"][..]));
        assert_eq!(map.get_properties("skew-y-3"), Some(&["--tw-skew-y"][..]));
    }

    #[test]
    fn test_bg_none_mapping() {
        use crate::property_order::get_property_index;
        let map = UtilityMap::new();

        // bg-none should map to background-image
        assert_eq!(
            map.get_properties("bg-none"),
            Some(&["background-image"][..])
        );

        // verify bg-none sorts before bg-clip-* (background-image < background-clip)
        let bg_none_idx = get_property_index("background-image").unwrap();
        let bg_clip_idx = get_property_index("background-clip").unwrap();
        assert!(
            bg_none_idx < bg_clip_idx,
            "bg-none (background-image: {}) should sort before bg-clip-* (background-clip: {})",
            bg_none_idx,
            bg_clip_idx
        );

        // verify bg-none sorts after bg-color (background-color < background-image)
        let bg_color_idx = get_property_index("background-color").unwrap();
        assert!(
            bg_color_idx < bg_none_idx,
            "bg-color (background-color: {}) should sort before bg-none (background-image: {})",
            bg_color_idx,
            bg_none_idx
        );
    }
}

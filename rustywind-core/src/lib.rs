//! Sort Tailwind CSS classes in language-aware source documents
//!
//! Use [`RustyWind::sort_document`] with a [`SourceDocument`] so RustyWind can
//! distinguish static class text from embedded template code
pub mod app;
mod attribute_parser;
mod class_name;
pub mod class_wrapping;
pub mod consts;
pub mod defaults;
pub mod parser;
pub mod sorter;
mod source;
pub mod tailwind_prefix;
mod template_parser;

// Pattern-based sorting modules
pub mod class_parser;
pub mod hybrid_sorter;
mod jsx_parser;
mod markup_parser;
pub mod pattern_sorter;
pub mod property_order;
pub mod utility_map;
pub mod variant_order;

pub use app::{InvalidClassList, PlainClassList, RustyWind, SortReport};
pub use source::{SourceDocument, SourceLanguage};

//! The functionality you need is in the [`sorter`] module.
//! Call [`sorter::sort_file_contents`] with the file contents and the sorter options.
//!
//! The [`parser`] module contains the functions to parse the classes from a file.
//! The [`parser::parse_classes_from_file`] function will return a `HashMap<String, usize>` with the classes and their order.
//!
//! You can use this to create a custom sorter. Using this customer sorter you can call [`sorter::sort_file_contents`].
pub(crate) mod app;
pub mod bump_ext;
pub mod class_wrapping;
pub mod consts;
pub mod defaults;
pub mod parser;
pub mod sorter;

pub type RustyWind = app::RustyWind;

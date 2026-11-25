//! The module that sorts the classes in the file contents.
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Deref;

use ahash::AHashMap as HashMap;

use regex::{Captures, Match, Regex};

use crate::css_class_order::class_names_in_order;
use crate::defaults::RE;
use eyre::Result;

/// A custom class extractor with one class-list capture
#[derive(Debug, Clone)]
pub struct CustomClassExtractor(Regex, ClassCapture);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassCapture {
    Named,
    First,
}

impl CustomClassExtractor {
    /// Creates an extractor with a named `classes` capture or a first positional capture
    pub fn new(regex: Regex) -> std::result::Result<Self, InvalidCustomClassExtractor> {
        let capture = if regex
            .capture_names()
            .flatten()
            .any(|name| name == "classes")
        {
            ClassCapture::Named
        } else if regex.captures_len() > 1 {
            ClassCapture::First
        } else {
            return Err(InvalidCustomClassExtractor);
        };

        Ok(Self(regex, capture))
    }

    /// Returns the configured extraction regex
    pub fn regex(&self) -> &Regex {
        &self.0
    }

    pub(crate) fn classes<'a>(&self, captures: &'a Captures<'a>) -> Option<Match<'a>> {
        match self.1 {
            ClassCapture::Named => captures.name("classes"),
            ClassCapture::First => captures.get(1),
        }
    }
}

impl Deref for CustomClassExtractor {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        self.regex()
    }
}

/// Error returned when a custom extractor has no class-list capture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidCustomClassExtractor;

impl fmt::Display for InvalidCustomClassExtractor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("custom extractor requires at least one capture group")
    }
}

impl std::error::Error for InvalidCustomClassExtractor {}

/// Selects the default source extractor or an explicit custom extractor
#[derive(Debug, Clone)]
pub enum FinderRegex {
    /// Uses RustyWind's language-aware default extraction
    DefaultRegex,
    /// Uses an explicit regex whose `classes` or first capture contains the class list
    CustomRegex(CustomClassExtractor),
}

impl Deref for FinderRegex {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        match &self {
            Self::DefaultRegex => &RE,
            Self::CustomRegex(extractor) => extractor.regex(),
        }
    }
}

/// Uses either pattern-based sorting or a sorter from a class list or a CSS file
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Sorter {
    /// Pattern-based sorting matching Tailwind CSS v4's canonical algorithm
    PatternSorter,
    /// Custom sorter from a list of classes, such as the `sortOrder` of a config file. A class
    /// that is not in the list goes after the classes of the list.
    CustomSorter(HashMap<String, usize>),
    /// Sorter loaded from a CSS file that Tailwind generated.
    ///
    /// A class that the file does not name is unknown. Unknown classes go before the classes of
    /// the file, in input order, as in prettier-plugin-tailwindcss. The placeholders `...` and `…`
    /// go last.
    ///
    /// The sorter looks up the whole class first. A class that the file names itself, such as
    /// `hover:bg-w` in Tailwind CSS output, keeps its own position.
    ///
    /// A class with a built-in variant prefix, such as `hover:` or `md:`, that the file does not
    /// name itself is not unknown when the file names the part after the prefix. `hover:flex` is
    /// then known when the file names `flex` but not `hover:flex`. It goes after the classes of
    /// the file, with the variants. Only the first prefix counts:
    /// `md:hover:flex` is known when the file names `hover:flex`. A class with another prefix,
    /// such as `max-md:flex` or `peer-hover:flex`, is unknown unless the file names the whole
    /// class.
    CssSorter(HashMap<String, usize>),
}

impl Deref for Sorter {
    type Target = HashMap<String, usize>;

    fn deref(&self) -> &Self::Target {
        match &self {
            Self::PatternSorter => {
                panic!("PatternSorter should not be used with HashMap-based sorting")
            }
            Self::CustomSorter(sorter) | Self::CssSorter(sorter) => sorter,
        }
    }
}

impl Sorter {
    /// Creates a [`Sorter::CustomSorter`] from class positions
    ///
    /// A class that the map lacks goes after the classes of the map. To put such classes first,
    /// wrap the map in [`Sorter::CssSorter`] instead.
    pub fn new(sorter: HashMap<String, usize>) -> Self {
        Self::CustomSorter(sorter)
    }

    /// Creates a [`Sorter::CssSorter`] from a [File]
    ///
    /// See [`Sorter::new_from_reader`] for the order of the classes, and [`Sorter::CssSorter`]
    /// for the order of unknown classes.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Sorter::new_from_reader`]. The file is open already, so this
    /// method has no error of its own.
    pub fn new_from_file(css_file: File) -> Result<Self> {
        let css_reader = BufReader::new(css_file);
        Self::new_from_reader(css_reader)
    }

    /// Creates a [`Sorter::CssSorter`] from any [BufReader]
    ///
    /// The order is the order of the first rule that names a class. Each selector gives one class:
    /// the longest one with a `:` in its name, else the first class of its last compound selector
    /// that holds a class. A prelude with `&` and an at-rule prelude give none.
    ///
    /// See [`Sorter::CssSorter`] for the order of unknown classes.
    ///
    /// # Errors
    ///
    /// Returns an error when reading fails, or when the input is not valid UTF-8
    /// ([`Read::read_to_string`]).
    pub fn new_from_reader<T: Read>(mut css_file: BufReader<T>) -> Result<Self> {
        let mut css = String::new();
        css_file.read_to_string(&mut css)?;

        let mut classes: HashMap<String, usize> = HashMap::new();
        for class in class_names_in_order(&css) {
            let next = classes.len();
            classes.entry(class).or_insert(next);
        }

        Ok(Self::CssSorter(classes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::io::BufReader;

    #[test]
    fn custom_extractor_accepts_named_and_legacy_positional_captures() {
        let named = Regex::new(r#"class=["'](?P<classes>[^"']*)["']"#).unwrap();
        let unnamed = Regex::new(r#"class=["']([^"']*)["']"#).unwrap();
        let missing = Regex::new(r#"class=["'][^"']*["']"#).unwrap();

        assert!(CustomClassExtractor::new(named).is_ok());
        assert!(CustomClassExtractor::new(unnamed).is_ok());
        assert_eq!(
            CustomClassExtractor::new(missing).unwrap_err(),
            InvalidCustomClassExtractor
        );
    }

    #[test]
    fn extracts_all_classes() {
        let css_file = std::fs::File::open("tests/fixtures/tailwind.css").unwrap();
        let classes = Sorter::new_from_file(css_file).unwrap();

        assert_eq!(classes.get("container"), Some(&0));
        // A class with a pseudo-class, such as `.hover\:bg-blue-500:hover`, is one name. The
        // fixture holds 304 unique utility names.
        assert!(classes.contains_key("hover:bg-blue-500"));
        assert!(classes.contains_key("placeholder-gray-400"));
        assert_eq!(classes.len(), 304);
    }

    #[test]
    fn extracts_real_tailwind_classes_with_escaped_chars() {
        let css_file = std::fs::File::open("tests/fixtures/tailwind.css").unwrap();
        let classes = Sorter::new_from_file(css_file).unwrap();

        // verify that classes with escaped characters are properly extracted
        // these classes exist in the tailwind.css fixture file
        assert!(
            classes.contains_key("mr-0.5"),
            "Should extract mr-0.5 (from .mr-0\\.5)"
        );
        assert!(
            classes.contains_key("-ml-0.5"),
            "Should extract -ml-0.5 (from .-ml-0\\.5)"
        );

        // verify order is preserved (container should be first)
        assert_eq!(
            classes.get("container"),
            Some(&0),
            "container should be at index 0"
        );

        // verify mr-0.5 comes after container
        let mr_index = classes.get("mr-0.5").expect("mr-0.5 should exist");
        assert!(*mr_index > 0, "mr-0.5 should have index > 0");
    }

    #[test]
    fn extracts_classes_with_leading_whitespace() {
        let css_content = r#".no-whitespace {
            color: red;
        }
  .with-spaces {
            color: blue;
        }
	.with-tab {
            color: green;
        }
    .multiple-spaces {
            color: yellow;
        }"#;

        let reader = BufReader::new(css_content.as_bytes());
        let classes = Sorter::new_from_reader(reader).unwrap();

        // all classes should be extracted regardless of leading whitespace
        assert_eq!(classes.get("no-whitespace"), Some(&0));
        assert_eq!(classes.get("with-spaces"), Some(&1));
        assert_eq!(classes.get("with-tab"), Some(&2));
        assert_eq!(classes.get("multiple-spaces"), Some(&3));
        assert_eq!(classes.len(), 4);
    }

    #[test]
    fn extracts_classes_with_escaped_characters() {
        let css_content = r#".mr-0\.5 {
            margin-right: 0.125rem;
        }
        .-ml-0\.5 {
            margin-left: -0.125rem;
        }
        .w-1\/2 {
            width: 50%;
        }"#;

        let reader = BufReader::new(css_content.as_bytes());
        let classes = Sorter::new_from_reader(reader).unwrap();

        // backslashes should be removed from escaped characters
        assert_eq!(classes.get("mr-0.5"), Some(&0));
        assert_eq!(classes.get("-ml-0.5"), Some(&1));
        assert_eq!(classes.get("w-1/2"), Some(&2));
        assert_eq!(classes.len(), 3);
    }

    #[test]
    fn preserves_order_of_classes() {
        let css_content = r#".first {
            color: red;
        }
        .second {
            color: blue;
        }
        .third {
            color: green;
        }
        .fourth {
            color: yellow;
        }"#;

        let reader = BufReader::new(css_content.as_bytes());
        let classes = Sorter::new_from_reader(reader).unwrap();

        // classes should maintain their order from the CSS file
        assert_eq!(classes.get("first"), Some(&0));
        assert_eq!(classes.get("second"), Some(&1));
        assert_eq!(classes.get("third"), Some(&2));
        assert_eq!(classes.get("fourth"), Some(&3));
    }

    #[test]
    fn handles_duplicate_classes() {
        let css_content = r#".duplicate {
            color: red;
        }
        .unique {
            color: blue;
        }
        .duplicate {
            color: green;
        }"#;

        let reader = BufReader::new(css_content.as_bytes());
        let classes = Sorter::new_from_reader(reader).unwrap();

        // first occurrence should be preserved
        assert_eq!(classes.get("duplicate"), Some(&0));
        assert_eq!(classes.get("unique"), Some(&1));
        // only 2 unique classes should be in the map
        assert_eq!(classes.len(), 2);
    }

    #[test]
    fn handles_empty_css() {
        let css_content = "";

        let reader = BufReader::new(css_content.as_bytes());
        let classes = Sorter::new_from_reader(reader).unwrap();

        assert_eq!(classes.len(), 0);
    }

    #[test]
    fn rejects_css_that_is_not_valid_utf8() {
        let reader = BufReader::new(&[0xff_u8][..]);

        assert!(Sorter::new_from_reader(reader).is_err());
    }

    #[test]
    fn extracts_classes_without_space_before_brace() {
        let css_content = r#".with-space {
            color: red;
        }
        .no-space{
            color: blue;
        }"#;

        let reader = BufReader::new(css_content.as_bytes());
        let classes = Sorter::new_from_reader(reader).unwrap();

        assert_eq!(classes.get("with-space"), Some(&0));
        assert_eq!(classes.get("no-space"), Some(&1));
        assert_eq!(classes.len(), 2);
    }

    #[test]
    fn extracts_classes_with_complex_names() {
        let css_content = r#".hover\:bg-blue-500 {
            background-color: #3b82f6;
        }
        .sm\:text-lg {
            font-size: 1.125rem;
        }
        .dark\:md\:hover\:text-white {
            color: white;
        }"#;

        let reader = BufReader::new(css_content.as_bytes());
        let classes = Sorter::new_from_reader(reader).unwrap();

        // escaped colons should have backslashes removed
        assert_eq!(classes.get("hover:bg-blue-500"), Some(&0));
        assert_eq!(classes.get("sm:text-lg"), Some(&1));
        assert_eq!(classes.get("dark:md:hover:text-white"), Some(&2));
        assert_eq!(classes.len(), 3);
    }

    #[test]
    fn extracts_arbitrary_value_classes() {
        let css_content = r#".w-\[500px\] {
            width: 500px;
        }
        .bg-\[\#1da1f2\] {
            background-color: #1da1f2;
        }"#;

        let reader = BufReader::new(css_content.as_bytes());
        let classes = Sorter::new_from_reader(reader).unwrap();

        // arbitrary values should have backslashes removed
        assert_eq!(classes.get("w-[500px]"), Some(&0));
        assert_eq!(classes.get("bg-[#1da1f2]"), Some(&1));
        assert_eq!(classes.len(), 2);
    }

    #[test]
    fn extracts_all_classes_from_tailwind_v4() {
        let css_file = std::fs::File::open("tests/fixtures/tailwind-v4.css").unwrap();
        let classes = Sorter::new_from_file(css_file).unwrap();

        // debug: print all classes containing "2xl" or "32xl"
        println!("\nClasses containing '2xl' or '32xl':");
        for key in classes.keys() {
            if key.contains("2xl") || key.contains("32xl") {
                println!("  - {}", key);
            }
        }

        // verify that all classes are extracted from Tailwind v4 CSS
        // test core utility classes
        assert!(
            classes.contains_key("container"),
            "Should extract container"
        );
        assert!(classes.contains_key("flex"), "Should extract flex");
        assert!(classes.contains_key("grid"), "Should extract grid");
        assert!(classes.contains_key("hidden"), "Should extract hidden");

        // test responsive variants
        assert!(classes.contains_key("sm:block"), "Should extract sm:block");
        assert!(
            classes.contains_key("md:grid-cols-2"),
            "Should extract md:grid-cols-2"
        );
        assert!(
            classes.contains_key("lg:grid-cols-3"),
            "Should extract lg:grid-cols-3"
        );
        assert!(
            classes.contains_key("xl:hidden"),
            "Should extract xl:hidden"
        );

        // the CSS escape `\32` is the digit '2', so `.\32xl\:block` is the class `2xl:block`
        assert!(
            classes.contains_key("2xl:block"),
            "Should extract 2xl:block (CSS escape \\32xl is 2xl)"
        );

        // test state variants
        assert!(
            classes.contains_key("hover:bg-blue-700"),
            "Should extract hover:bg-blue-700"
        );
        assert!(
            classes.contains_key("focus:ring-2"),
            "Should extract focus:ring-2"
        );
        assert!(
            classes.contains_key("active:bg-blue-800"),
            "Should extract active:bg-blue-800"
        );
        assert!(
            classes.contains_key("disabled:opacity-50"),
            "Should extract disabled:opacity-50"
        );
        assert!(
            classes.contains_key("checked:bg-blue-600"),
            "Should extract checked:bg-blue-600"
        );

        // test group variants
        assert!(
            classes.contains_key("group-hover:bg-gray-100"),
            "Should extract group-hover:bg-gray-100"
        );
        assert!(
            classes.contains_key("group-hover:text-gray-900"),
            "Should extract group-hover:text-gray-900"
        );

        // test dark mode
        assert!(
            classes.contains_key("dark:text-white"),
            "Should extract dark:text-white"
        );
        assert!(
            classes.contains_key("dark:bg-gray-800"),
            "Should extract dark:bg-gray-800"
        );

        // test complex responsive + state variants
        assert!(
            classes.contains_key("md:hover:text-white"),
            "Should extract md:hover:text-white"
        );

        // test arbitrary values (Tailwind v4 feature)
        assert!(
            classes.contains_key("w-[500px]"),
            "Should extract w-[500px]"
        );
        assert!(
            classes.contains_key("h-[200px]"),
            "Should extract h-[200px]"
        );
        assert!(
            classes.contains_key("bg-[#1da1f2]"),
            "Should extract bg-[#1da1f2]"
        );
        assert!(
            classes.contains_key("rounded-[32px]"),
            "Should extract rounded-[32px]"
        );
        assert!(classes.contains_key("p-[24px]"), "Should extract p-[24px]");
        assert!(
            classes.contains_key("text-[18px]"),
            "Should extract text-[18px]"
        );
        assert!(
            classes.contains_key("leading-[1.5]"),
            "Should extract leading-[1.5]"
        );

        // test fractional widths
        assert!(classes.contains_key("w-1/2"), "Should extract w-1/2");
        assert!(classes.contains_key("w-1/3"), "Should extract w-1/3");
        assert!(classes.contains_key("w-1/4"), "Should extract w-1/4");

        // test negative values
        assert!(classes.contains_key("-mt-4"), "Should extract -mt-4");
        assert!(classes.contains_key("-ml-2"), "Should extract -ml-2");

        // verify order preservation (container should be first)
        assert_eq!(
            classes.get("container"),
            Some(&0),
            "container should be at index 0"
        );

        // verify exact number of classes extracted from our comprehensive v4 fixture
        // this fixture contains 759 lines with 152 unique utility classes covering:
        // - Responsive breakpoints (sm, md, lg, xl, 2xl)
        // - State variants (hover, focus, active, disabled, checked)
        // - Dark mode, group variants, and arbitrary values
        println!(
            "Total classes extracted from Tailwind v4: {}",
            classes.len()
        );
        assert_eq!(
            classes.len(),
            152,
            "Should extract exactly 152 classes from Tailwind v4 fixture"
        );
    }
}

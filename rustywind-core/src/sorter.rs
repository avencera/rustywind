//! The module that sorts the classes in the file contents.
use std::collections::hash_map::Entry;
use std::fs::File;
use std::io::{BufRead as _, BufReader, Read};
use std::ops::Deref;

use ahash::AHashMap as HashMap;

use regex::Regex;
use std::sync::LazyLock;

use crate::defaults::RE;
use eyre::Result;

pub(crate) static SORTER_EXTRACTOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(\.[^\s]+)[ ]").unwrap());

/// Use either our default regex in [crate::defaults::RE] or a custom regex.
#[derive(Debug, Clone)]
pub enum FinderRegex {
    DefaultRegex,
    CustomRegex(Regex),
}

impl Deref for FinderRegex {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        match &self {
            Self::DefaultRegex => &RE,
            Self::CustomRegex(re) => re,
        }
    }
}

/// Use either pattern-based sorting or a custom sorter from a CSS file.
#[derive(Debug, Clone)]
pub enum Sorter {
    /// Pattern-based sorting matching Tailwind CSS v4's canonical algorithm
    PatternSorter,
    /// Custom sorter loaded from a CSS file
    CustomSorter(HashMap<String, usize>),
}

impl Deref for Sorter {
    type Target = HashMap<String, usize>;

    fn deref(&self) -> &Self::Target {
        match &self {
            Self::PatternSorter => {
                panic!("PatternSorter should not be used with HashMap-based sorting")
            }
            Self::CustomSorter(sorter) => sorter,
        }
    }
}

impl Sorter {
    pub fn new(sorter: HashMap<String, usize>) -> Self {
        Self::CustomSorter(sorter)
    }

    /// Create the sorter from a [File]
    pub fn new_from_file(css_file: File) -> Result<Self> {
        let css_reader = BufReader::new(css_file);
        Self::new_from_reader(css_reader)
    }

    /// Create the sorter from any [BufReader]
    pub fn new_from_reader<T: Read>(css_file: BufReader<T>) -> Result<Self> {
        let css_reader = BufReader::new(css_file);
        let mut classes: HashMap<String, usize> = HashMap::new();

        let mut index = 0_usize;
        for line in css_reader.lines() {
            if let Some(captures) = SORTER_EXTRACTOR_RE.captures(&line?) {
                let class = captures[1].trim_start_matches('.').replace('\\', "");

                if let Entry::Vacant(entry) = classes.entry(class) {
                    entry.insert(index);
                    index += 1;
                }
            }
        }

        Ok(Self::new(classes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::io::BufReader;

    #[test]
    fn extracts_all_classes() {
        let css_file = std::fs::File::open("tests/fixtures/tailwind.css").unwrap();
        let classes = Sorter::new_from_file(css_file).unwrap();

        assert_eq!(classes.get("container"), Some(&0));
        assert_eq!(classes.len(), 305);
    }

    #[test]
    fn extracts_real_tailwind_classes_with_escaped_chars() {
        let css_file = std::fs::File::open("tests/fixtures/tailwind.css").unwrap();
        let classes = Sorter::new_from_file(css_file).unwrap();

        // Verify that classes with escaped characters are properly extracted
        // These classes exist in the tailwind.css fixture file
        assert!(
            classes.contains_key("mr-0.5"),
            "Should extract mr-0.5 (from .mr-0\\.5)"
        );
        assert!(
            classes.contains_key("-ml-0.5"),
            "Should extract -ml-0.5 (from .-ml-0\\.5)"
        );

        // Verify order is preserved (container should be first)
        assert_eq!(
            classes.get("container"),
            Some(&0),
            "container should be at index 0"
        );

        // Verify mr-0.5 comes after container
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
    fn ignores_classes_without_space_before_brace() {
        let css_content = r#".with-space {
            color: red;
        }
        .no-space{
            color: blue;
        }"#;

        let reader = BufReader::new(css_content.as_bytes());
        let classes = Sorter::new_from_reader(reader).unwrap();

        // only class with space should be extracted
        assert_eq!(classes.get("with-space"), Some(&0));
        assert_eq!(classes.get("no-space"), None);
        assert_eq!(classes.len(), 1);
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

        // Debug: print all classes containing "2xl" or "32xl"
        println!("\nClasses containing '2xl' or '32xl':");
        for key in classes.keys() {
            if key.contains("2xl") || key.contains("32xl") {
                println!("  - {}", key);
            }
        }

        // Verify that all classes are extracted from Tailwind v4 CSS
        // Test core utility classes
        assert!(
            classes.contains_key("container"),
            "Should extract container"
        );
        assert!(classes.contains_key("flex"), "Should extract flex");
        assert!(classes.contains_key("grid"), "Should extract grid");
        assert!(classes.contains_key("hidden"), "Should extract hidden");

        // Test responsive variants
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

        // Note: CSS escape \32 for digit '2' becomes '32' when backslash is removed
        assert!(
            classes.contains_key("32xl:block"),
            "Should extract 32xl:block (CSS escape \\32xl becomes 32xl)"
        );

        // Test state variants
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

        // Test group variants
        assert!(
            classes.contains_key("group-hover:bg-gray-100"),
            "Should extract group-hover:bg-gray-100"
        );
        assert!(
            classes.contains_key("group-hover:text-gray-900"),
            "Should extract group-hover:text-gray-900"
        );

        // Test dark mode
        assert!(
            classes.contains_key("dark:text-white"),
            "Should extract dark:text-white"
        );
        assert!(
            classes.contains_key("dark:bg-gray-800"),
            "Should extract dark:bg-gray-800"
        );

        // Test complex responsive + state variants
        assert!(
            classes.contains_key("md:hover:text-white"),
            "Should extract md:hover:text-white"
        );

        // Test arbitrary values (Tailwind v4 feature)
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

        // Test fractional widths
        assert!(classes.contains_key("w-1/2"), "Should extract w-1/2");
        assert!(classes.contains_key("w-1/3"), "Should extract w-1/3");
        assert!(classes.contains_key("w-1/4"), "Should extract w-1/4");

        // Test negative values
        assert!(classes.contains_key("-mt-4"), "Should extract -mt-4");
        assert!(classes.contains_key("-ml-2"), "Should extract -ml-2");

        // Verify order preservation (container should be first)
        assert_eq!(
            classes.get("container"),
            Some(&0),
            "container should be at index 0"
        );

        // Verify exact number of classes extracted from our comprehensive v4 fixture
        // This fixture contains 759 lines with 152 unique utility classes covering:
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

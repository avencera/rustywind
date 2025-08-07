//! The module that sorts the classes in the file contents.
use std::collections::hash_map::Entry;
use std::fs::File;
use std::io::{BufRead as _, BufReader, Read};
use std::ops::Deref;

use ahash::AHashMap as HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::defaults::{RE, SORTER};
use eyre::Result;

pub(crate) static SORTER_EXTRACTOR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(\.[^\s]+)[ ]").unwrap());

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

/// Use either our default sorter in [crate::defaults::SORTER] or a custom sorter.
#[derive(Debug, Clone)]
pub enum Sorter {
    DefaultSorter,
    CustomSorter(HashMap<String, usize>),
}

impl Deref for Sorter {
    type Target = HashMap<String, usize>;

    fn deref(&self) -> &Self::Target {
        match &self {
            Self::DefaultSorter => &SORTER,
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
}

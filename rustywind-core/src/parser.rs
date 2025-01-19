//! Create a sorter from a CSS file to sort classes in the order that they appear in the file
pub mod regex;

use std::{
    collections::hash_map::Entry,
    fs::File,
    io::{BufRead, BufReader, Read},
};

use eyre::Result;

use ahash::AHashMap as HashMap;
use regex::PARSER_RE;

/// Create the sorter from a [File]
pub fn parse_classes_from_file(css_file: File) -> Result<HashMap<String, usize>> {
    let css_reader = BufReader::new(css_file);
    parse_classes(css_reader)
}

/// Create the sorter from any [BufReader]
pub fn parse_classes<T: Read>(css_file: BufReader<T>) -> Result<HashMap<String, usize>> {
    let css_reader = BufReader::new(css_file);
    let mut classes: HashMap<String, usize> = HashMap::new();

    let mut index = 0_usize;
    for line in css_reader.lines() {
        if let Some(captures) = PARSER_RE.captures(&line?) {
            let class = captures[1].trim_start_matches('.').replace('\\', "");

            if let Entry::Vacant(entry) = classes.entry(class) {
                entry.insert(index);
                index += 1;
            }
        }
    }

    Ok(classes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn extracts_all_classes() {
        let css_file = std::fs::File::open("tests/fixtures/tailwind.css").unwrap();
        let classes = parse_classes_from_file(css_file).unwrap();

        assert_eq!(classes.get("container"), Some(&0));
        assert_eq!(classes.len(), 221);
    }
}

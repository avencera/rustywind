use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{BufRead, BufReader},
};

use once_cell::sync::Lazy;
use regex::Regex;

static PARSER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(\.[^\s]+)[ ]"#).unwrap());

pub fn parse_classes(css_file: File) -> eyre::Result<HashMap<String, usize>> {
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
        let classes = parse_classes(css_file).unwrap();

        assert_eq!(classes.get("container"), Some(&0));
        assert_eq!(classes.len(), 221);
    }
}

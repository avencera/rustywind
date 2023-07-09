use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

pub static PARSER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(\.[^\s]+)[ ]"#).unwrap());

pub fn parse_classes(css: &str) -> HashMap<String, usize> {
    let mut classes: HashMap<String, usize> = HashMap::new();

    let mut index = 0_usize;
    for line in css.lines() {
        if let Some(captures) = PARSER_RE.captures(line) {
            let class = captures[1].trim_start_matches('.').replace("\\", "");

            if !classes.contains_key(&class) {
                classes.insert(class, index);
                index += 1;
            }
        }
    }

    classes
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn extracts_all_classes() {
        let css = std::fs::read_to_string("tests/fixtures/tailwind.css").unwrap();
        let classes = parse_classes(&css);

        assert_eq!(classes.get("container"), Some(&0));
        assert_eq!(classes.len(), 221);
    }
}

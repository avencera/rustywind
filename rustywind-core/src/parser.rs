//! Create a sorter from a CSS file to sort classes in the order that they appear in the file
pub mod css;
pub mod regex;

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

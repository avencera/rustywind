use eyre::Result;

fn parse_apply_classes(_css_file: &str) -> Result<Vec<&str>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    #[test_case(
        "@apply w-full opacity-0;",
        vec!["w-full", "opacity-0"] ; 
        "basic apply directive"
    )]
    #[test_case(
        "@apply flex items-center;\nheight: 60px;\n@apply bg-gray-100;",
        vec!["flex", "items-center", "bg-gray-100"] ;
        "mixed content with multiple applies"
    )]
    #[test_case(
        "/* Header styles */\n@apply text-lg font-bold;",
        vec!["text-lg", "font-bold"] ;
        "with comments"
    )]
    #[test_case( "@apply;", vec![] ; "empty apply")]
    fn test_extract_apply_classes(input: &str, output: Vec<&str>) {
        let classes = parse_apply_classes(input);
        assert!(classes.is_ok());
        assert_eq!(classes.unwrap(), output);
    }
}

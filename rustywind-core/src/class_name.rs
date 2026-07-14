#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClassSegments<'a> {
    parts: Vec<&'a str>,
    utility_start: usize,
}

impl<'a> ClassSegments<'a> {
    pub(crate) fn parse(class: &'a str) -> Option<Self> {
        if class.is_empty() {
            return None;
        }

        let mut parts = Vec::new();
        let mut start = 0;
        let mut delimiters = Vec::new();
        let mut quote = None;
        let mut escaped = false;

        for (index, character) in class.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }

            if character == '\\' {
                escaped = true;
                continue;
            }

            if let Some(active_quote) = quote {
                if character == active_quote {
                    quote = None;
                }
                continue;
            }

            if !delimiters.is_empty() && matches!(character, '\'' | '"' | '`') {
                quote = Some(character);
                continue;
            }

            match character {
                '[' => delimiters.push(']'),
                '(' => delimiters.push(')'),
                ']' | ')' if delimiters.pop() != Some(character) => return None,
                ':' if delimiters.is_empty() => {
                    if start == index {
                        return None;
                    }
                    parts.push(&class[start..index]);
                    start = index + character.len_utf8();
                }
                _ => {}
            }
        }

        if escaped || quote.is_some() || !delimiters.is_empty() || start == class.len() {
            return None;
        }

        let utility_start = start;
        parts.push(&class[start..]);

        Some(Self {
            parts,
            utility_start,
        })
    }

    pub(crate) fn variants(&self) -> &[&'a str] {
        &self.parts[..self.parts.len() - 1]
    }

    pub(crate) fn utility(&self) -> &'a str {
        self.parts[self.parts.len() - 1]
    }

    pub(crate) fn utility_start(&self) -> usize {
        self.utility_start
    }
}

#[cfg(test)]
mod tests {
    use super::ClassSegments;

    #[test]
    fn splits_only_top_level_variant_separators() {
        let segments =
            ClassSegments::parse("supports-[selector(:has(*))]:hover:ring-(length:--ring-width)")
                .unwrap();

        assert_eq!(
            segments.variants(),
            &["supports-[selector(:has(*))]", "hover"]
        );
        assert_eq!(segments.utility(), "ring-(length:--ring-width)");
    }

    #[test]
    fn ignores_delimiters_inside_quoted_arbitrary_values() {
        let segments = ClassSegments::parse("before:content-[':)']").unwrap();

        assert_eq!(segments.variants(), &["before"]);
        assert_eq!(segments.utility(), "content-[':)']");
    }

    #[test]
    fn rejects_unbalanced_or_empty_segments() {
        assert!(ClassSegments::parse("hover::flex").is_none());
        assert!(ClassSegments::parse("hover:ring-(length:--width").is_none());
        assert!(ClassSegments::parse("hover:ring-length:--width)").is_none());
    }
}

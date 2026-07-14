use winnow::{
    Parser,
    ascii::multispace0,
    stream::{LocatingSlice, Location, Stream},
    token::{any, literal, take_till, take_while},
};

use crate::{
    attribute_parser::ClassAttribute,
    template_parser::{
        consume_block_comment, consume_escape, consume_line_comment, consume_quoted, consume_regex,
        expression_keyword_allows_regex,
    },
};

type Input<'a> = LocatingSlice<&'a str>;

pub(crate) fn class_attributes(source: &str) -> Option<Vec<ClassAttribute>> {
    JsxParser::new(source).parse()
}

struct JsxParser<'a> {
    source: &'a str,
    attributes: Vec<ClassAttribute>,
}

impl<'a> JsxParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            attributes: Vec::new(),
        }
    }

    fn parse(mut self) -> Option<Vec<ClassAttribute>> {
        let mut input = Input::new(self.source);
        self.parse_javascript(&mut input, None)?;
        Some(self.attributes)
    }

    fn parse_javascript(&mut self, input: &mut Input<'a>, closing: Option<char>) -> Option<()> {
        let mut can_start_expression = true;

        loop {
            let source = remaining(input);
            let Some(character) = source.chars().next() else {
                return closing.is_none().then_some(());
            };

            if closing == Some(character) {
                consume_character(input, character)?;
                return Some(());
            }
            if matches!(character, ')' | ']' | '}') {
                return None;
            }

            if source.starts_with("//") {
                consume_line_comment(input, "//")?;
                continue;
            }
            if source.starts_with("/*") {
                consume_block_comment(input)?;
                continue;
            }

            if matches!(character, '\'' | '"') {
                consume_quoted(input, character, None)?;
                can_start_expression = false;
                continue;
            }
            if character == '`' {
                self.parse_template_literal(input)?;
                can_start_expression = false;
                continue;
            }

            if character == '/' {
                if can_start_expression {
                    consume_regex(input)?;
                    can_start_expression = false;
                } else {
                    consume_character(input, '/')?;
                    can_start_expression = true;
                }
                continue;
            }

            if character == '<' && can_start_expression && is_jsx_opening_start(source.as_bytes()) {
                if self.try_parse_jsx(input) {
                    can_start_expression = false;
                    continue;
                }
                consume_character(input, '<')?;
                can_start_expression = false;
                continue;
            }

            if let Some(group_closing) = group_closing(character) {
                consume_character(input, character)?;
                self.parse_javascript(input, Some(group_closing))?;
                can_start_expression = false;
                continue;
            }

            if is_javascript_identifier_start(character) {
                let identifier: &str = take_while::<_, _, winnow::error::ContextError>(
                    1..,
                    is_javascript_identifier_continue,
                )
                .parse_next(input)
                .ok()?;
                can_start_expression = expression_keyword_allows_regex(identifier);
                continue;
            }

            if character.is_ascii_digit() {
                take_while::<_, _, winnow::error::ContextError>(1.., |character: char| {
                    character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '+' | '-')
                })
                .void()
                .parse_next(input)
                .ok()?;
                can_start_expression = false;
                continue;
            }

            any::<_, winnow::error::ContextError>
                .void()
                .parse_next(input)
                .ok()?;
            if !character.is_whitespace() {
                can_start_expression = true;
            }
        }
    }

    fn parse_template_literal(&mut self, input: &mut Input<'a>) -> Option<()> {
        consume_character(input, '`')?;
        loop {
            let source = remaining(input);
            match source.chars().next()? {
                '\\' => consume_escape(input)?,
                '`' => {
                    consume_character(input, '`')?;
                    return Some(());
                }
                '$' if source.starts_with("${") => {
                    consume_character(input, '$')?;
                    consume_character(input, '{')?;
                    self.parse_javascript(input, Some('}'))?;
                }
                _ => {
                    any::<_, winnow::error::ContextError>
                        .void()
                        .parse_next(input)
                        .ok()?;
                }
            }
        }
    }

    fn try_parse_jsx(&mut self, input: &mut Input<'a>) -> bool {
        let checkpoint = input.checkpoint();
        let attribute_count = self.attributes.len();
        if self.parse_jsx(input).is_some() {
            return true;
        }

        self.attributes.truncate(attribute_count);
        input.reset(&checkpoint);
        false
    }

    fn parse_jsx(&mut self, input: &mut Input<'a>) -> Option<()> {
        consume_literal(input, "<")?;
        if remaining(input).starts_with('>') {
            consume_literal(input, ">")?;
            return self.parse_jsx_children(input, None);
        }

        let name = self.parse_jsx_name(input)?;
        if self.parse_jsx_attributes(input)? {
            return Some(());
        }
        self.parse_jsx_children(input, Some(name))
    }

    fn parse_jsx_attributes(&mut self, input: &mut Input<'a>) -> Option<bool> {
        loop {
            let whitespace: &str = multispace0::<_, winnow::error::ContextError>
                .parse_next(input)
                .ok()?;
            if remaining(input).starts_with("/>") {
                consume_literal(input, "/>")?;
                return Some(true);
            }
            if remaining(input).starts_with('>') {
                consume_literal(input, ">")?;
                return Some(false);
            }
            if whitespace.is_empty() {
                return None;
            }

            if remaining(input).starts_with("//") {
                consume_line_comment(input, "//")?;
                continue;
            }
            if remaining(input).starts_with("/*") {
                consume_block_comment(input)?;
                continue;
            }

            if remaining(input).starts_with('{') {
                consume_character(input, '{')?;
                self.parse_javascript(input, Some('}'))?;
                continue;
            }

            let name = self.parse_jsx_attribute_name(input)?;
            let after_name = input.checkpoint();
            multispace0::<_, winnow::error::ContextError>
                .void()
                .parse_next(input)
                .ok()?;
            if !remaining(input).starts_with('=') {
                input.reset(&after_name);
                continue;
            }

            consume_literal(input, "=")?;
            multispace0::<_, winnow::error::ContextError>
                .void()
                .parse_next(input)
                .ok()?;
            let value = match remaining(input).chars().next()? {
                quote @ ('\'' | '"') => Some(self.parse_quoted_attribute(input, quote)?),
                '{' => {
                    consume_character(input, '{')?;
                    self.parse_javascript(input, Some('}'))?;
                    None
                }
                '<' if is_jsx_opening_start(remaining(input).as_bytes()) => {
                    self.parse_jsx(input)?;
                    None
                }
                _ => return None,
            };

            if matches!(name, "class" | "className")
                && let Some(value) = value
            {
                self.attributes.push(ClassAttribute::new(value));
            }
        }
    }

    fn parse_quoted_attribute(
        &self,
        input: &mut Input<'a>,
        quote: char,
    ) -> Option<std::ops::Range<usize>> {
        consume_character(input, quote)?;
        let start = input.current_token_start();
        take_till::<_, _, winnow::error::ContextError>(0.., |character| character == quote)
            .void()
            .parse_next(input)
            .ok()?;
        let end = input.current_token_start();
        consume_character(input, quote)?;
        Some(start..end)
    }

    fn parse_jsx_children(
        &mut self,
        input: &mut Input<'a>,
        expected_name: Option<&'a str>,
    ) -> Option<()> {
        loop {
            let source = remaining(input);
            if source.starts_with("</") {
                consume_literal(input, "</")?;
                match expected_name {
                    Some(expected_name) if self.parse_jsx_name(input)? == expected_name => {}
                    None if remaining(input).starts_with('>') => {}
                    Some(_) | None => return None,
                }
                multispace0::<_, winnow::error::ContextError>
                    .void()
                    .parse_next(input)
                    .ok()?;
                consume_literal(input, ">")?;
                return Some(());
            }
            if source.starts_with('<') {
                self.parse_jsx(input)?;
                continue;
            }
            if source.starts_with('{') {
                consume_character(input, '{')?;
                self.parse_javascript(input, Some('}'))?;
                continue;
            }

            take_till::<_, _, winnow::error::ContextError>(1.., ('<', '{'))
                .void()
                .parse_next(input)
                .ok()?;
        }
    }

    fn parse_jsx_name(&self, input: &mut Input<'a>) -> Option<&'a str> {
        let start = input.current_token_start();
        self.parse_jsx_name_segment(input)?;
        while matches!(remaining(input).chars().next(), Some('.' | ':')) {
            any::<_, winnow::error::ContextError>
                .void()
                .parse_next(input)
                .ok()?;
            self.parse_jsx_name_segment(input)?;
        }
        Some(&self.source[start..input.current_token_start()])
    }

    fn parse_jsx_name_segment(&self, input: &mut Input<'a>) -> Option<()> {
        let first = remaining(input).chars().next()?;
        if !is_jsx_name_start(first) {
            return None;
        }
        take_while::<_, _, winnow::error::ContextError>(1.., is_jsx_name_continue)
            .void()
            .parse_next(input)
            .ok()
    }

    fn parse_jsx_attribute_name(&self, input: &mut Input<'a>) -> Option<&'a str> {
        let start = input.current_token_start();
        self.parse_jsx_name_segment(input)?;
        if remaining(input).starts_with(':') {
            consume_literal(input, ":")?;
            self.parse_jsx_name_segment(input)?;
        }
        Some(&self.source[start..input.current_token_start()])
    }
}

fn group_closing(opening: char) -> Option<char> {
    match opening {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        _ => None,
    }
}

fn is_jsx_opening_start(source: &[u8]) -> bool {
    source
        .get(1)
        .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$' | b'>'))
}

fn is_jsx_name_start(character: char) -> bool {
    character.is_alphabetic() || matches!(character, '_' | '$')
}

fn is_jsx_name_continue(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '$' | '-')
}

fn is_javascript_identifier_start(character: char) -> bool {
    character.is_alphabetic() || matches!(character, '_' | '$')
}

fn is_javascript_identifier_continue(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '$')
}

fn remaining<'a>(input: &Input<'a>) -> &'a str {
    input.as_ref()
}

fn consume_literal(input: &mut Input<'_>, value: &str) -> Option<()> {
    literal::<_, _, winnow::error::ContextError>(value)
        .void()
        .parse_next(input)
        .ok()
}

fn consume_character(input: &mut Input<'_>, expected: char) -> Option<()> {
    let character = any::<_, winnow::error::ContextError>
        .parse_next(input)
        .ok()?;
    (character == expected).then_some(())
}

#[cfg(test)]
mod tests {
    use super::class_attributes;

    fn values(source: &str) -> Option<Vec<&str>> {
        class_attributes(source).map(|attributes| {
            attributes
                .into_iter()
                .map(|attribute| &source[attribute.value_range()])
                .collect()
        })
    }

    #[test]
    fn extracts_nested_quoted_attributes() {
        let source = r#"const view = <><div className="p-4 flex"><Icon class='size-4 block' /></div><svg:path class="stroke-2" /></>;"#;

        assert_eq!(
            values(source),
            Some(vec!["p-4 flex", "size-4 block", "stroke-2"])
        );
    }

    #[test]
    fn ignores_program_text_and_dynamic_attributes() {
        let source = r#"
            const string = '<div className="fake string" />';
            const template = `<div class="fake template">${value}</div>`;
            const pattern = /<div className="fake regex">/;
            // <div className="fake line comment" />
            /* <div class="fake block comment" /> */
            const view = <div data-class="ignored" className={value} {...props} class="real p-4" />;
        "#;

        assert_eq!(values(source), Some(vec!["real p-4"]));
    }

    #[test]
    fn parses_jsx_inside_javascript_expressions() {
        let source = r#"const view = condition ? <Panel render={<span className="p-4 flex" />} /> : <div class='m-2 grid' />;"#;

        assert_eq!(values(source), Some(vec!["p-4 flex", "m-2 grid"]));
    }

    #[test]
    fn boolean_attributes_do_not_consume_the_next_separator() {
        let source = r#"const view = <AreaChart accessibilityLayer data={chartData} className="p-4 flex" />;"#;

        assert_eq!(values(source), Some(vec!["p-4 flex"]));
    }

    #[test]
    fn comments_can_separate_attributes() {
        let source = r#"
            const view = (
                <Button
                    onClick={() => submit()}
                    // keep this prop documented
                    className="p-4 flex"
                    /* and this one */ disabled
                />
            );
        "#;

        assert_eq!(values(source), Some(vec!["p-4 flex"]));
    }

    #[test]
    fn generic_and_relational_syntax_is_not_jsx() {
        let source = r#"
            const identity = <T,>(value: T): T => value;
            const result = left < right && right > left;
            const view = <div className="p-4 flex" />;
        "#;

        assert_eq!(values(source), Some(vec!["p-4 flex"]));
    }

    #[test]
    fn incomplete_lexical_constructs_fail_closed() {
        for source in [
            r#"const value = "unterminated <div className='fake'>"#,
            "/* unterminated <div className=\"fake\">",
            "const value = `unterminated <div className=\"fake\">",
        ] {
            assert_eq!(values(source), None, "{source}");
        }

        assert_eq!(
            values("const view = <div className=\"fake\">"),
            Some(Vec::new())
        );
    }
}

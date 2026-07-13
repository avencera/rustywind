use std::ops::Range;

use winnow::{
    Parser,
    ascii::multispace0,
    combinator::opt,
    stream::{LocatingSlice, Location},
    token::{any, literal, take_till, take_while},
};

type Input<'a> = LocatingSlice<&'a str>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpressionSyntax {
    JavaScript,
    Php,
    Ruby,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClassValueValidation {
    Direct,
    Unspecified,
}

pub(crate) fn delimited_islands(
    value: &str,
    delimiters: &[(&str, &str)],
) -> Option<Vec<Range<usize>>> {
    let mut islands = Vec::new();
    let mut cursor = 0;

    while let Some((start, opener, closer)) = next_delimiter(value, cursor, delimiters) {
        let end = delimited_end(value, start + opener.len(), closer)?;
        islands.push(start..end);
        cursor = end;
    }

    Some(islands)
}

pub(crate) fn delimited_island_end_at(
    value: &str,
    cursor: usize,
    delimiters: &[(&str, &str)],
) -> Option<Option<usize>> {
    delimiters.iter().find_map(|&(opener, closer)| {
        value[cursor..]
            .starts_with(opener)
            .then(|| delimited_end(value, cursor + opener.len(), closer))
    })
}

fn delimited_end(value: &str, cursor: usize, closer: &str) -> Option<usize> {
    let mut input = Input::new(&value[cursor..]);
    loop {
        if remaining(&input).starts_with(closer) {
            literal::<_, _, winnow::error::ContextError>(closer)
                .void()
                .parse_next(&mut input)
                .ok()?;
            return Some(cursor + input.current_token_start());
        }

        match remaining(&input).chars().next()? {
            quote @ ('\'' | '"' | '`') => consume_quoted(&mut input, quote, None)?,
            _ => {
                any::<_, winnow::error::ContextError>
                    .void()
                    .parse_next(&mut input)
                    .ok()?;
            }
        }
    }
}

fn next_delimiter<'a>(
    value: &str,
    cursor: usize,
    delimiters: &'a [(&'a str, &'a str)],
) -> Option<(usize, &'a str, &'a str)> {
    delimiters
        .iter()
        .filter_map(|&(opener, closer)| {
            value[cursor..]
                .find(opener)
                .map(|relative_start| (cursor + relative_start, opener, closer))
        })
        .min_by_key(|(start, opener, _)| (*start, std::cmp::Reverse(opener.len())))
}

pub(crate) fn balanced_islands(
    value: &str,
    opener: &str,
    syntax: ExpressionSyntax,
) -> Option<Vec<Range<usize>>> {
    let mut islands = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = value[cursor..].find(opener) {
        let start = cursor + relative_start;
        let brace_start = start + opener.len() - 1;
        let end = balanced_end(value, brace_start, '{', '}', syntax)?;
        islands.push(start..end);
        cursor = end;
    }

    Some(islands)
}

pub(crate) fn balanced_end(
    value: &str,
    opening_index: usize,
    opening: char,
    closing: char,
    syntax: ExpressionSyntax,
) -> Option<usize> {
    let mut input = Input::new(&value[opening_index..]);
    ExpressionParser::new(syntax).parse_group(&mut input, opening, closing)?;
    Some(opening_index + input.current_token_start())
}

pub(crate) struct JavaScriptExpression {
    pub(crate) end: usize,
    pub(crate) markup_starts: Vec<usize>,
}

pub(crate) fn javascript_expression(
    value: &str,
    opening_index: usize,
) -> Option<JavaScriptExpression> {
    let mut input = Input::new(&value[opening_index..]);
    let mut parser = ExpressionParser::collecting_markup(opening_index);
    parser.parse_group(&mut input, '{', '}')?;
    Some(JavaScriptExpression {
        end: opening_index + input.current_token_start(),
        markup_starts: parser.markup_starts,
    })
}

struct ExpressionParser {
    syntax: ExpressionSyntax,
    can_start_regex: bool,
    base_offset: usize,
    markup_starts: Vec<usize>,
    collect_markup: bool,
}

impl ExpressionParser {
    const fn new(syntax: ExpressionSyntax) -> Self {
        Self {
            syntax,
            can_start_regex: true,
            base_offset: 0,
            markup_starts: Vec::new(),
            collect_markup: false,
        }
    }

    const fn collecting_markup(base_offset: usize) -> Self {
        Self {
            syntax: ExpressionSyntax::JavaScript,
            can_start_regex: true,
            base_offset,
            markup_starts: Vec::new(),
            collect_markup: true,
        }
    }

    fn parse_group(&mut self, input: &mut Input<'_>, opening: char, closing: char) -> Option<()> {
        consume_character(input, opening)?;
        self.can_start_regex = true;

        loop {
            let source = remaining(input);
            let character = source.chars().next()?;

            if self.collect_markup && source.starts_with("<!--") {
                consume_html_comment(input)?;
                continue;
            }

            if matches!(
                self.syntax,
                ExpressionSyntax::JavaScript | ExpressionSyntax::Php
            ) && source.starts_with("//")
            {
                consume_line_comment(input, "//")?;
                continue;
            }
            if matches!(
                self.syntax,
                ExpressionSyntax::JavaScript | ExpressionSyntax::Php
            ) && source.starts_with("/*")
            {
                consume_block_comment(input)?;
                continue;
            }
            if matches!(self.syntax, ExpressionSyntax::Ruby | ExpressionSyntax::Php)
                && character == '#'
            {
                consume_line_comment(input, "#")?;
                continue;
            }

            if character == '`' {
                match self.syntax {
                    ExpressionSyntax::JavaScript => self.consume_template_literal(input)?,
                    ExpressionSyntax::Php => consume_quoted(input, '`', None)?,
                    ExpressionSyntax::Ruby => return None,
                }
                self.can_start_regex = false;
                continue;
            }
            if matches!(character, '\'' | '"') {
                let rejected = matches!((self.syntax, character), (ExpressionSyntax::Ruby, '"'))
                    .then_some("#{");
                consume_quoted(input, character, rejected)?;
                self.can_start_regex = false;
                continue;
            }

            if matches!(self.syntax, ExpressionSyntax::Ruby)
                && character == '%'
                && source
                    .as_bytes()
                    .get(1)
                    .is_some_and(|kind| matches!(kind, b'q' | b'Q' | b'r' | b'w' | b'W' | b'x'))
            {
                return None;
            }

            if character == '/'
                && matches!(
                    self.syntax,
                    ExpressionSyntax::JavaScript | ExpressionSyntax::Ruby
                )
            {
                if self.can_start_regex {
                    consume_regex(input)?;
                    self.can_start_regex = false;
                } else {
                    consume_character(input, '/')?;
                    self.can_start_regex = true;
                }
                continue;
            }

            if self.collect_markup && character == '<' && is_markup_tag_start(source.as_bytes()) {
                self.markup_starts
                    .push(self.base_offset + input.current_token_start());
                consume_character(input, '<')?;
                self.can_start_regex = false;
                continue;
            }

            if character == opening {
                self.parse_group(input, opening, closing)?;
                self.can_start_regex = false;
                continue;
            }
            if character == closing {
                consume_character(input, closing)?;
                self.can_start_regex = false;
                return Some(());
            }

            if character.is_ascii_alphabetic() || matches!(character, '_' | '$') {
                let identifier: &str =
                    take_while::<_, _, winnow::error::ContextError>(1.., |character: char| {
                        character.is_ascii_alphanumeric() || matches!(character, '_' | '$')
                    })
                    .parse_next(input)
                    .ok()?;
                self.can_start_regex = expression_keyword_allows_regex(identifier);
                continue;
            }

            any::<_, winnow::error::ContextError>
                .void()
                .parse_next(input)
                .ok()?;
            if character.is_ascii_digit() || matches!(character, ')' | ']') {
                self.can_start_regex = false;
            } else if !character.is_ascii_whitespace() {
                self.can_start_regex = true;
            }
        }
    }

    fn consume_template_literal(&mut self, input: &mut Input<'_>) -> Option<()> {
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
                    self.parse_group(input, '{', '}')?;
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
}

fn consume_line_comment(input: &mut Input<'_>, opener: &str) -> Option<()> {
    literal::<_, _, winnow::error::ContextError>(opener)
        .void()
        .parse_next(input)
        .ok()?;
    take_till::<_, _, winnow::error::ContextError>(0.., ('\n', '\r'))
        .void()
        .parse_next(input)
        .ok()?;
    opt(any::<_, winnow::error::ContextError>)
        .void()
        .parse_next(input)
        .ok()
}

fn consume_block_comment(input: &mut Input<'_>) -> Option<()> {
    (
        literal::<_, _, winnow::error::ContextError>("/*"),
        winnow::token::take_until::<_, _, winnow::error::ContextError>(0.., "*/"),
        literal::<_, _, winnow::error::ContextError>("*/"),
    )
        .void()
        .parse_next(input)
        .ok()
}

fn consume_html_comment(input: &mut Input<'_>) -> Option<()> {
    (
        literal::<_, _, winnow::error::ContextError>("<!--"),
        winnow::token::take_until::<_, _, winnow::error::ContextError>(0.., "-->"),
        literal::<_, _, winnow::error::ContextError>("-->"),
    )
        .void()
        .parse_next(input)
        .ok()
}

fn consume_quoted(input: &mut Input<'_>, quote: char, rejected: Option<&str>) -> Option<()> {
    consume_character(input, quote)?;
    loop {
        let source = remaining(input);
        let character = source.chars().next()?;
        if character == '\\' {
            consume_escape(input)?;
        } else if rejected.is_some_and(|opener| source.starts_with(opener)) {
            return None;
        } else if character == quote {
            consume_character(input, quote)?;
            return Some(());
        } else {
            any::<_, winnow::error::ContextError>
                .void()
                .parse_next(input)
                .ok()?;
        }
    }
}

fn consume_escape(input: &mut Input<'_>) -> Option<()> {
    consume_character(input, '\\')?;
    any::<_, winnow::error::ContextError>
        .void()
        .parse_next(input)
        .ok()
}

fn consume_regex(input: &mut Input<'_>) -> Option<()> {
    consume_character(input, '/')?;
    let mut in_character_class = false;
    loop {
        match remaining(input).chars().next()? {
            '\\' => consume_escape(input)?,
            '[' => {
                consume_character(input, '[')?;
                in_character_class = true;
            }
            ']' => {
                consume_character(input, ']')?;
                in_character_class = false;
            }
            '/' if !in_character_class => {
                consume_character(input, '/')?;
                take_while::<_, _, winnow::error::ContextError>(0.., |character: char| {
                    character.is_ascii_alphabetic()
                })
                .void()
                .parse_next(input)
                .ok()?;
                return Some(());
            }
            '\n' | '\r' => return None,
            _ => {
                any::<_, winnow::error::ContextError>
                    .void()
                    .parse_next(input)
                    .ok()?;
            }
        }
    }
}

fn consume_character(input: &mut Input<'_>, expected: char) -> Option<()> {
    let character = any::<_, winnow::error::ContextError>
        .parse_next(input)
        .ok()?;
    (character == expected).then_some(())
}

fn expression_keyword_allows_regex(identifier: &str) -> bool {
    matches!(
        identifier,
        "await"
            | "and"
            | "case"
            | "delete"
            | "do"
            | "else"
            | "if"
            | "in"
            | "instanceof"
            | "new"
            | "not"
            | "of"
            | "or"
            | "return"
            | "then"
            | "throw"
            | "typeof"
            | "unless"
            | "until"
            | "void"
            | "when"
            | "while"
            | "yield"
    )
}

pub(crate) fn blade_islands(value: &str) -> Option<Vec<Range<usize>>> {
    const MUSTACHES: &[(&str, &str)] = &[("{{--", "--}}"), ("{!!", "!!}"), ("{{", "}}")];

    let mut islands = Vec::new();
    let mut cursor = 0;
    while let Some(island) = next_blade_island(value, cursor, MUSTACHES) {
        let (start, end) = match island {
            BladeIsland::Delimited {
                start,
                opener,
                closer,
            } => (start, delimited_end(value, start + opener.len(), closer)?),
            BladeIsland::Directive {
                start,
                identifier_end,
                parenthesis,
            } => (
                start,
                parenthesis.map_or(Some(identifier_end), |parenthesis| {
                    balanced_end(value, parenthesis, '(', ')', ExpressionSyntax::Php)
                })?,
            ),
        };
        islands.push(start..end);
        cursor = end;
    }
    Some(islands)
}

pub(crate) fn blade_island_end_at(value: &str, cursor: usize) -> Option<Option<usize>> {
    let mustache = delimited_island_end_at(
        value,
        cursor,
        &[("{{--", "--}}"), ("{!!", "!!}"), ("{{", "}}")],
    );
    if mustache.is_some() {
        return mustache;
    }

    let (start, identifier_end, parenthesis) = next_blade_directive(value, cursor)?;
    (start == cursor).then(|| {
        parenthesis.map_or(Some(identifier_end), |parenthesis| {
            balanced_end(value, parenthesis, '(', ')', ExpressionSyntax::Php)
        })
    })
}

enum BladeIsland<'a> {
    Delimited {
        start: usize,
        opener: &'a str,
        closer: &'a str,
    },
    Directive {
        start: usize,
        identifier_end: usize,
        parenthesis: Option<usize>,
    },
}

fn next_blade_island<'a>(
    value: &str,
    cursor: usize,
    mustaches: &'a [(&'a str, &'a str)],
) -> Option<BladeIsland<'a>> {
    let mustache = next_delimiter(value, cursor, mustaches);
    let directive = next_blade_directive(value, cursor);

    match (mustache, directive) {
        (None, None) => None,
        (Some((start, opener, closer)), None) => Some(BladeIsland::Delimited {
            start,
            opener,
            closer,
        }),
        (None, Some((start, identifier_end, parenthesis))) => Some(BladeIsland::Directive {
            start,
            identifier_end,
            parenthesis,
        }),
        (Some((start, opener, closer)), Some((directive_start, _, _)))
            if start <= directive_start =>
        {
            Some(BladeIsland::Delimited {
                start,
                opener,
                closer,
            })
        }
        (_, Some((start, identifier_end, parenthesis))) => Some(BladeIsland::Directive {
            start,
            identifier_end,
            parenthesis,
        }),
    }
}

fn next_blade_directive(value: &str, cursor: usize) -> Option<(usize, usize, Option<usize>)> {
    let relative_start = value[cursor..].find('@')?;
    let start = cursor + relative_start;
    let mut input = Input::new(&value[start..]);
    literal::<_, _, winnow::error::ContextError>("@")
        .void()
        .parse_next(&mut input)
        .ok()?;
    let identifier: &str =
        take_while::<_, _, winnow::error::ContextError>(1.., |character: char| {
            character.is_ascii_alphanumeric() || character == '_'
        })
        .parse_next(&mut input)
        .ok()?;
    if !identifier
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
    {
        return next_blade_directive(value, start + 1);
    }
    let identifier_end = start + input.current_token_start();
    multispace0::<_, winnow::error::ContextError>
        .void()
        .parse_next(&mut input)
        .ok()?;
    let parenthesis = remaining(&input)
        .starts_with('(')
        .then_some(start + input.current_token_start());
    Some((start, identifier_end, parenthesis))
}

pub(crate) fn validate_class_value(value: &str, validation: ClassValueValidation) -> bool {
    if value.trim().is_empty() {
        return false;
    }

    let mut input = Input::new(value);
    let mut bracket_depth = 0_u32;
    let mut bracket_quote = None;
    while let Some(character) = remaining(&input).chars().next() {
        if character == '\\' {
            if consume_escape(&mut input).is_none() {
                return false;
            }
            continue;
        }
        if let Some(quote) = bracket_quote {
            if consume_character(&mut input, character).is_none() {
                return false;
            }
            if character == quote {
                bracket_quote = None;
            }
            continue;
        }
        if bracket_depth > 0 && matches!(character, '\'' | '"' | '`') {
            bracket_quote = Some(character);
            if consume_character(&mut input, character).is_none() {
                return false;
            }
            continue;
        }

        let cursor = input.current_token_start();
        let valid = match character {
            '[' => {
                bracket_depth += 1;
                true
            }
            ']' if bracket_depth == 0 => false,
            ']' => {
                bracket_depth -= 1;
                true
            }
            '{' | '}' if bracket_depth == 0 => false,
            '<' if bracket_depth == 0 => {
                !(matches!(validation, ClassValueValidation::Direct)
                    || value[cursor..].starts_with("<%")
                    || value[cursor..].starts_with("<?"))
            }
            '\'' | '"' | '`' if bracket_depth == 0 => false,
            '>' | '$'
                if bracket_depth == 0 && matches!(validation, ClassValueValidation::Direct) =>
            {
                false
            }
            '%' if bracket_depth == 0 && value[cursor..].starts_with("%>") => false,
            '?' if bracket_depth == 0 && value[cursor..].starts_with("?>") => false,
            '@' if bracket_depth == 0 && is_blade_directive_at(value, cursor) => false,
            character if character.is_control() && !character.is_ascii_whitespace() => false,
            _ => true,
        };
        if !valid || consume_character(&mut input, character).is_none() {
            return false;
        }
    }

    bracket_depth == 0 && bracket_quote.is_none()
}

fn is_blade_directive_at(value: &str, start: usize) -> bool {
    let mut input = Input::new(&value[start + 1..]);
    let Ok(identifier) = take_while::<_, _, winnow::error::ContextError>(0.., |character: char| {
        character.is_ascii_alphanumeric() || character == '_'
    })
    .parse_next(&mut input) else {
        return false;
    };

    matches!(
        identifier,
        "auth"
            | "break"
            | "case"
            | "class"
            | "continue"
            | "default"
            | "else"
            | "elseif"
            | "empty"
            | "endauth"
            | "endempty"
            | "endenvironment"
            | "enderror"
            | "endfor"
            | "endforelse"
            | "endforeach"
            | "endguest"
            | "endif"
            | "endisset"
            | "endproduction"
            | "endswitch"
            | "endunless"
            | "endwhile"
            | "env"
            | "error"
            | "for"
            | "forelse"
            | "foreach"
            | "guest"
            | "if"
            | "isset"
            | "production"
            | "switch"
            | "unless"
            | "while"
    )
}

fn is_markup_tag_start(source: &[u8]) -> bool {
    match source.get(1).copied() {
        Some(first) if first.is_ascii_alphabetic() => true,
        Some(b'/') => source
            .get(2)
            .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'>'),
        Some(b'>') => true,
        _ => false,
    }
}

fn remaining<'a>(input: &Input<'a>) -> &'a str {
    input.as_ref()
}

#[cfg(test)]
mod tests {
    use super::{ExpressionSyntax, balanced_end};

    #[test]
    fn javascript_template_interpolation_is_balanced() {
        let source = r#"{`value ${nested({ value: "}" })}`} rest"#;
        assert_eq!(
            balanced_end(source, 0, '{', '}', ExpressionSyntax::JavaScript),
            Some(source.find(" rest").unwrap())
        );
    }
}

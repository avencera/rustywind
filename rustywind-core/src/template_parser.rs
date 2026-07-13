use std::ops::Range;

use winnow::{
    Parser,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DelimitedTemplateProfile<'a> {
    delimiters: &'a [(&'a str, &'a str)],
    closing_policy: DelimitedClosingPolicy,
}

impl<'a> DelimitedTemplateProfile<'a> {
    pub(crate) const fn template_tokens(delimiters: &'a [(&'a str, &'a str)]) -> Self {
        Self {
            delimiters,
            closing_policy: DelimitedClosingPolicy::QuotedFirstCloser,
        }
    }

    pub(crate) const fn php(delimiters: &'a [(&'a str, &'a str)]) -> Self {
        Self {
            delimiters,
            closing_policy: DelimitedClosingPolicy::PhpBlockComments,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DelimitedClosingPolicy {
    QuotedFirstCloser,
    PhpBlockComments,
}

pub(crate) fn delimited_islands(
    value: &str,
    profile: DelimitedTemplateProfile<'_>,
) -> Option<Vec<Range<usize>>> {
    let mut islands = Vec::new();
    let mut cursor = 0;

    while let Some((start, opener, closer)) = next_delimiter(value, cursor, profile.delimiters) {
        let end = delimited_end(value, start + opener.len(), closer, profile.closing_policy)?;
        islands.push(start..end);
        cursor = end;
    }

    Some(islands)
}

pub(crate) fn delimited_island_end_at(
    value: &str,
    cursor: usize,
    profile: DelimitedTemplateProfile<'_>,
) -> Option<Option<usize>> {
    profile.delimiters.iter().find_map(|&(opener, closer)| {
        value[cursor..]
            .starts_with(opener)
            .then(|| delimited_end(value, cursor + opener.len(), closer, profile.closing_policy))
    })
}

fn delimited_end(
    value: &str,
    cursor: usize,
    closer: &str,
    closing_policy: DelimitedClosingPolicy,
) -> Option<usize> {
    let mut input = Input::new(&value[cursor..]);
    loop {
        if remaining(&input).starts_with(closer) {
            literal::<_, _, winnow::error::ContextError>(closer)
                .void()
                .parse_next(&mut input)
                .ok()?;
            return Some(cursor + input.current_token_start());
        }

        if matches!(closing_policy, DelimitedClosingPolicy::PhpBlockComments)
            && remaining(&input).starts_with("/*")
        {
            consume_block_comment(&mut input)?;
            continue;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SvelteBraceContext {
    Fragment,
    StartTag,
    AttributeValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SvelteBlockKind {
    If,
    Each,
    Await,
    Key,
    Snippet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SvelteBranchKind {
    Else,
    ElseIf,
    Then,
    Catch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SvelteSpecialKind {
    Html,
    Debug,
    Const,
    Render,
    Attach,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SvelteBraceEvent {
    Expression { end: usize },
    Open { kind: SvelteBlockKind, end: usize },
    Branch { kind: SvelteBranchKind, end: usize },
    Close { kind: SvelteBlockKind, end: usize },
    Special { kind: SvelteSpecialKind, end: usize },
}

impl SvelteBraceEvent {
    pub(crate) const fn end(self) -> usize {
        match self {
            Self::Expression { end }
            | Self::Open { end, .. }
            | Self::Branch { end, .. }
            | Self::Close { end, .. }
            | Self::Special { end, .. } => end,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SvelteBraceScan {
    NotAnOpener,
    Closed(SvelteBraceEvent),
    Malformed,
}

pub(crate) fn scan_svelte_brace(
    source: &str,
    cursor: usize,
    context: SvelteBraceContext,
) -> SvelteBraceScan {
    let Some(tail) = source.get(cursor..) else {
        return SvelteBraceScan::NotAnOpener;
    };
    if !tail.starts_with('{') {
        return SvelteBraceScan::NotAnOpener;
    }

    let content_start = skip_whitespace(source, cursor + 1);
    let Some(content) = source.get(content_start..) else {
        return SvelteBraceScan::Malformed;
    };
    let Some(first) = content.chars().next() else {
        return SvelteBraceScan::Malformed;
    };

    match first {
        '#' => scan_svelte_open(source, content_start, context),
        ':' => scan_svelte_branch(source, content_start, context),
        '/' if !content.starts_with("/*") && !content.starts_with("//") => {
            scan_svelte_close(source, content_start, context)
        }
        '@' => scan_svelte_special(source, content_start, context),
        '}' => SvelteBraceScan::Malformed,
        _ => scan_svelte_expression(source, cursor + 1),
    }
}

fn scan_svelte_open(
    source: &str,
    sigil_start: usize,
    context: SvelteBraceContext,
) -> SvelteBraceScan {
    if !matches!(context, SvelteBraceContext::Fragment) {
        return SvelteBraceScan::Malformed;
    }

    let name_start = sigil_start + 1;
    let name_end = svelte_identifier_end(source, name_start);
    let Some(name) = source.get(name_start..name_end) else {
        return SvelteBraceScan::Malformed;
    };
    let kind = match name {
        "if" => SvelteBlockKind::If,
        "each" => SvelteBlockKind::Each,
        "await" => SvelteBlockKind::Await,
        "key" => SvelteBlockKind::Key,
        "snippet" => SvelteBlockKind::Snippet,
        _ => return SvelteBraceScan::Malformed,
    };
    if !svelte_body_is_present(source, name_end) {
        return SvelteBraceScan::Malformed;
    }

    scan_svelte_body(source, name_end, |end| SvelteBraceEvent::Open { kind, end })
}

fn scan_svelte_branch(
    source: &str,
    sigil_start: usize,
    context: SvelteBraceContext,
) -> SvelteBraceScan {
    if !matches!(context, SvelteBraceContext::Fragment) {
        return SvelteBraceScan::Malformed;
    }

    let name_start = sigil_start + 1;
    let name_end = svelte_identifier_end(source, name_start);
    let Some(name) = source.get(name_start..name_end) else {
        return SvelteBraceScan::Malformed;
    };
    match name {
        "then" => scan_svelte_body(source, name_end, |end| SvelteBraceEvent::Branch {
            kind: SvelteBranchKind::Then,
            end,
        }),
        "catch" => scan_svelte_body(source, name_end, |end| SvelteBraceEvent::Branch {
            kind: SvelteBranchKind::Catch,
            end,
        }),
        "else" => scan_svelte_else(source, name_end),
        _ => SvelteBraceScan::Malformed,
    }
}

fn scan_svelte_else(source: &str, name_end: usize) -> SvelteBraceScan {
    let next = skip_whitespace(source, name_end);
    if source.get(next..).is_some_and(|tail| tail.starts_with('}')) {
        return SvelteBraceScan::Closed(SvelteBraceEvent::Branch {
            kind: SvelteBranchKind::Else,
            end: next + 1,
        });
    }

    let if_end = svelte_identifier_end(source, next);
    if source.get(next..if_end) != Some("if") || !svelte_body_is_present(source, if_end) {
        return SvelteBraceScan::Malformed;
    }
    scan_svelte_body(source, if_end, |end| SvelteBraceEvent::Branch {
        kind: SvelteBranchKind::ElseIf,
        end,
    })
}

fn scan_svelte_close(
    source: &str,
    sigil_start: usize,
    context: SvelteBraceContext,
) -> SvelteBraceScan {
    if !matches!(context, SvelteBraceContext::Fragment) {
        return SvelteBraceScan::Malformed;
    }

    let name_start = sigil_start + 1;
    let name_end = svelte_identifier_end(source, name_start);
    let Some(name) = source.get(name_start..name_end) else {
        return SvelteBraceScan::Malformed;
    };
    let kind = match name {
        "if" => SvelteBlockKind::If,
        "each" => SvelteBlockKind::Each,
        "await" => SvelteBlockKind::Await,
        "key" => SvelteBlockKind::Key,
        "snippet" => SvelteBlockKind::Snippet,
        _ => return SvelteBraceScan::Malformed,
    };
    let closing = skip_whitespace(source, name_end);
    if !source
        .get(closing..)
        .is_some_and(|tail| tail.starts_with('}'))
    {
        return SvelteBraceScan::Malformed;
    }

    SvelteBraceScan::Closed(SvelteBraceEvent::Close {
        kind,
        end: closing + 1,
    })
}

fn scan_svelte_special(
    source: &str,
    sigil_start: usize,
    context: SvelteBraceContext,
) -> SvelteBraceScan {
    let name_start = sigil_start + 1;
    let name_end = svelte_identifier_end(source, name_start);
    let Some(name) = source.get(name_start..name_end) else {
        return SvelteBraceScan::Malformed;
    };
    let kind = match (context, name) {
        (SvelteBraceContext::Fragment, "html") => SvelteSpecialKind::Html,
        (SvelteBraceContext::Fragment, "debug") => SvelteSpecialKind::Debug,
        (SvelteBraceContext::Fragment, "const") => SvelteSpecialKind::Const,
        (SvelteBraceContext::Fragment, "render") => SvelteSpecialKind::Render,
        (SvelteBraceContext::StartTag, "attach") => SvelteSpecialKind::Attach,
        _ => return SvelteBraceScan::Malformed,
    };
    if !matches!(kind, SvelteSpecialKind::Debug) && !svelte_body_is_present(source, name_end) {
        return SvelteBraceScan::Malformed;
    }

    scan_svelte_body(source, name_end, |end| SvelteBraceEvent::Special {
        kind,
        end,
    })
}

fn scan_svelte_expression(source: &str, body_start: usize) -> SvelteBraceScan {
    if !svelte_body_is_present(source, body_start) {
        return SvelteBraceScan::Malformed;
    }
    scan_svelte_body(source, body_start, |end| SvelteBraceEvent::Expression {
        end,
    })
}

fn scan_svelte_body(
    source: &str,
    body_start: usize,
    event: impl FnOnce(usize) -> SvelteBraceEvent,
) -> SvelteBraceScan {
    let Some(body) = source.get(body_start..) else {
        return SvelteBraceScan::Malformed;
    };
    let mut input = Input::new(body);
    let mut parser = ExpressionParser::new(ExpressionSyntax::JavaScript);
    parser.can_start_regex = true;
    if parser.parse_group_body(&mut input, '{', '}').is_none() {
        return SvelteBraceScan::Malformed;
    }

    SvelteBraceScan::Closed(event(body_start + input.current_token_start()))
}

fn svelte_body_is_present(source: &str, body_start: usize) -> bool {
    let body_start = skip_whitespace(source, body_start);
    source
        .get(body_start..)
        .and_then(|tail| tail.chars().next())
        .is_some_and(|character| character != '}')
}

fn svelte_identifier_end(source: &str, start: usize) -> usize {
    let Some(tail) = source.get(start..) else {
        return start;
    };
    let mut bytes = tail.bytes();
    let Some(first) = bytes.next() else {
        return start;
    };
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return start;
    }

    start
        + 1
        + bytes
            .position(|byte| !(byte.is_ascii_alphanumeric() || byte == b'_'))
            .unwrap_or(tail.len() - 1)
}

fn skip_whitespace(source: &str, start: usize) -> usize {
    let Some(tail) = source.get(start..) else {
        return start;
    };
    tail.char_indices()
        .find(|(_, character)| !character.is_whitespace())
        .map_or(source.len(), |(relative, _)| start + relative)
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

        self.parse_group_body(input, opening, closing)
    }

    fn parse_group_body(
        &mut self,
        input: &mut Input<'_>,
        opening: char,
        closing: char,
    ) -> Option<()> {
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
    const PROFILE: DelimitedTemplateProfile<'_> =
        DelimitedTemplateProfile::template_tokens(MUSTACHES);

    let mut islands = Vec::new();
    let mut cursor = 0;
    while let Some(island) = next_blade_island(value, cursor, PROFILE.delimiters) {
        let (start, end) = match island {
            BladeIsland::Delimited {
                start,
                opener,
                closer,
            } => (
                start,
                delimited_end(value, start + opener.len(), closer, PROFILE.closing_policy)?,
            ),
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
    const PROFILE: DelimitedTemplateProfile<'_> = DelimitedTemplateProfile::template_tokens(&[
        ("{{--", "--}}"),
        ("{!!", "!!}"),
        ("{{", "}}"),
    ]);
    let mustache = delimited_island_end_at(value, cursor, PROFILE);
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
    let mut search_from = cursor;
    while let Some(relative_start) = value[search_from..].find('@') {
        let found = search_from + relative_start;
        let escape_run = blade_at_run(value, found);
        search_from = escape_run.end;
        if escape_run.len().is_multiple_of(2) {
            continue;
        }
        let start = escape_run.end - 1;

        let Some(first) = value.as_bytes().get(start + 1) else {
            continue;
        };
        if !(first.is_ascii_alphabetic() || *first == b'_') {
            continue;
        }

        let identifier_end = value.as_bytes()[start + 2..]
            .iter()
            .position(|byte| !(byte.is_ascii_alphanumeric() || *byte == b'_'))
            .map_or(value.len(), |relative_end| start + 2 + relative_end);
        let parenthesis = value[identifier_end..]
            .char_indices()
            .find(|(_, character)| !character.is_ascii_whitespace())
            .and_then(|(relative_start, character)| {
                (character == '(').then_some(identifier_end + relative_start)
            });
        return Some((start, identifier_end, parenthesis));
    }
    None
}

fn blade_at_is_escaped(value: &str, start: usize) -> bool {
    !(start - blade_at_run(value, start).start).is_multiple_of(2)
}

fn blade_at_run(value: &str, start: usize) -> Range<usize> {
    let run_start = value.as_bytes()[..start]
        .iter()
        .rposition(|byte| *byte != b'@')
        .map_or(0, |index| index + 1);
    let run_end = value.as_bytes()[start..]
        .iter()
        .position(|byte| *byte != b'@')
        .map_or(value.len(), |relative_end| start + relative_end);
    run_start..run_end
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
    if blade_at_is_escaped(value, start) {
        return false;
    }

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
    use super::{
        DelimitedTemplateProfile, ExpressionSyntax, SvelteBlockKind, SvelteBraceContext,
        SvelteBraceEvent, SvelteBraceScan, SvelteBranchKind, SvelteSpecialKind, balanced_end,
        blade_island_end_at, blade_islands, delimited_island_end_at, delimited_islands,
        next_blade_directive, scan_svelte_brace,
    };

    fn svelte_event(source: &str, context: SvelteBraceContext) -> SvelteBraceEvent {
        let SvelteBraceScan::Closed(event) = scan_svelte_brace(source, 0, context) else {
            panic!("expected a closed Svelte brace tag: {source}");
        };
        assert_eq!(event.end(), source.find(" tail").unwrap_or(source.len()));
        event
    }

    #[test]
    fn svelte_fragment_recognizes_every_block_form() {
        let openings = [
            ("{#if ready}", SvelteBlockKind::If),
            ("{#each items as item}", SvelteBlockKind::Each),
            ("{#await promise}", SvelteBlockKind::Await),
            ("{#key selected}", SvelteBlockKind::Key),
            ("{#snippet row(item)}", SvelteBlockKind::Snippet),
        ];
        for (source, expected) in openings {
            assert_eq!(
                svelte_event(source, SvelteBraceContext::Fragment),
                SvelteBraceEvent::Open {
                    kind: expected,
                    end: source.len(),
                }
            );
        }

        let closings = [
            ("{/if}", SvelteBlockKind::If),
            ("{/each}", SvelteBlockKind::Each),
            ("{/await}", SvelteBlockKind::Await),
            ("{/key}", SvelteBlockKind::Key),
            ("{/snippet}", SvelteBlockKind::Snippet),
        ];
        for (source, expected) in closings {
            assert_eq!(
                svelte_event(source, SvelteBraceContext::Fragment),
                SvelteBraceEvent::Close {
                    kind: expected,
                    end: source.len(),
                }
            );
        }
    }

    #[test]
    fn svelte_fragment_recognizes_every_branch_and_special() {
        let branches = [
            ("{:else}", SvelteBranchKind::Else),
            ("{:else if ready}", SvelteBranchKind::ElseIf),
            ("{:then value}", SvelteBranchKind::Then),
            ("{:catch error}", SvelteBranchKind::Catch),
        ];
        for (source, expected) in branches {
            assert_eq!(
                svelte_event(source, SvelteBraceContext::Fragment),
                SvelteBraceEvent::Branch {
                    kind: expected,
                    end: source.len(),
                }
            );
        }

        let specials = [
            ("{@html content}", SvelteSpecialKind::Html),
            ("{@debug value}", SvelteSpecialKind::Debug),
            ("{@const doubled = value * 2}", SvelteSpecialKind::Const),
            ("{@render children()}", SvelteSpecialKind::Render),
        ];
        for (source, expected) in specials {
            assert_eq!(
                svelte_event(source, SvelteBraceContext::Fragment),
                SvelteBraceEvent::Special {
                    kind: expected,
                    end: source.len(),
                }
            );
        }
    }

    #[test]
    fn svelte_contexts_only_accept_their_legal_forms() {
        for context in [
            SvelteBraceContext::Fragment,
            SvelteBraceContext::StartTag,
            SvelteBraceContext::AttributeValue,
        ] {
            assert_eq!(
                svelte_event("{value}", context),
                SvelteBraceEvent::Expression { end: 7 }
            );
        }

        assert_eq!(
            svelte_event("{...props}", SvelteBraceContext::StartTag),
            SvelteBraceEvent::Expression { end: 10 }
        );
        assert_eq!(
            svelte_event("{@attach action}", SvelteBraceContext::StartTag),
            SvelteBraceEvent::Special {
                kind: SvelteSpecialKind::Attach,
                end: 16,
            }
        );

        for (source, context) in [
            ("{#if ready}", SvelteBraceContext::StartTag),
            ("{:else}", SvelteBraceContext::StartTag),
            ("{/if}", SvelteBraceContext::StartTag),
            ("{@html value}", SvelteBraceContext::StartTag),
            ("{@attach action}", SvelteBraceContext::Fragment),
            ("{#if ready}", SvelteBraceContext::AttributeValue),
            ("{@attach action}", SvelteBraceContext::AttributeValue),
        ] {
            assert_eq!(
                scan_svelte_brace(source, 0, context),
                SvelteBraceScan::Malformed
            );
        }
    }

    #[test]
    fn svelte_tags_allow_leading_whitespace_but_not_split_sigils_and_names() {
        let source = "{ \n\t#await promise} tail";
        assert_eq!(
            svelte_event(source, SvelteBraceContext::Fragment),
            SvelteBraceEvent::Open {
                kind: SvelteBlockKind::Await,
                end: source.find(" tail").unwrap(),
            }
        );
        let close = "{  /snippet \t} tail";
        assert_eq!(
            svelte_event(close, SvelteBraceContext::Fragment),
            SvelteBraceEvent::Close {
                kind: SvelteBlockKind::Snippet,
                end: close.find(" tail").unwrap(),
            }
        );

        for source in ["{# if ready}", "{: else}", "{/ if}", "{@ html value}"] {
            assert_eq!(
                scan_svelte_brace(source, 0, SvelteBraceContext::Fragment),
                SvelteBraceScan::Malformed
            );
        }
    }

    #[test]
    fn svelte_bodies_keep_nested_javascript_lexically_balanced() {
        let source = r#"{#if /}/.test(`value ${nested({ value: "}" })}`)} tail"#;
        assert_eq!(
            svelte_event(source, SvelteBraceContext::Fragment),
            SvelteBraceEvent::Open {
                kind: SvelteBlockKind::If,
                end: source.find(" tail").unwrap(),
            }
        );

        for source in [
            "{value / divisor} tail",
            r#"{(/}/).test(value)} tail"#,
            "{/* } */ value} tail",
            "{// }\nvalue} tail",
        ] {
            assert_eq!(
                svelte_event(source, SvelteBraceContext::AttributeValue),
                SvelteBraceEvent::Expression {
                    end: source.find(" tail").unwrap(),
                }
            );
        }
    }

    #[test]
    fn svelte_slash_disambiguation_reserves_uncommented_slash_for_closers() {
        for source in [r#"{/}/.test(value)}"#, r#"{/unknown}"#, r#"{/if extra}"#] {
            assert_eq!(
                scan_svelte_brace(source, 0, SvelteBraceContext::Fragment),
                SvelteBraceScan::Malformed
            );
        }
        assert_eq!(
            scan_svelte_brace(
                r#"{/}/.test(value)}"#,
                0,
                SvelteBraceContext::AttributeValue
            ),
            SvelteBraceScan::Malformed
        );
    }

    #[test]
    fn malformed_svelte_tags_never_panic_or_consume_partial_input() {
        let embedded = "prefix {value} tail";
        let cursor = embedded.find('{').unwrap();
        assert_eq!(
            scan_svelte_brace(embedded, cursor, SvelteBraceContext::Fragment),
            SvelteBraceScan::Closed(SvelteBraceEvent::Expression {
                end: embedded.find(" tail").unwrap(),
            })
        );
        assert_eq!(
            scan_svelte_brace("plain", 0, SvelteBraceContext::Fragment),
            SvelteBraceScan::NotAnOpener
        );
        assert_eq!(
            scan_svelte_brace("é{value}", 1, SvelteBraceContext::Fragment),
            SvelteBraceScan::NotAnOpener
        );
        for source in [
            "{",
            "{}",
            "{value",
            "{#unknown value}",
            "{:unknown}",
            "{@unknown value}",
            "{#if}",
            "{:else if}",
            "{@html}",
        ] {
            assert_eq!(
                scan_svelte_brace(source, 0, SvelteBraceContext::Fragment),
                SvelteBraceScan::Malformed,
                "{source}"
            );
        }
    }

    #[test]
    fn javascript_template_interpolation_is_balanced() {
        let source = r#"{`value ${nested({ value: "}" })}`} rest"#;
        assert_eq!(
            balanced_end(source, 0, '{', '}', ExpressionSyntax::JavaScript),
            Some(source.find(" rest").unwrap())
        );
    }

    #[test]
    fn default_delimited_policy_shields_quotes_but_uses_the_first_other_closer() {
        const PROFILE: DelimitedTemplateProfile<'_> =
            DelimitedTemplateProfile::template_tokens(&[("<%", "%>")]);
        let quoted = r#"<% "not %> yet" %> tail"#;
        let block_comment = "<% /* %> still static";

        assert_eq!(
            delimited_islands(quoted, PROFILE),
            Some(std::iter::once(0..quoted.find(" tail").unwrap()).collect())
        );
        assert_eq!(
            delimited_island_end_at(block_comment, 0, PROFILE),
            Some(Some(block_comment.find(" still").unwrap()))
        );

        let blade_quoted = r#"{{ "not }} yet" }} tail"#;
        let blade_comment = "{{ /* }} tail";
        assert_eq!(
            blade_island_end_at(blade_quoted, 0),
            Some(Some(blade_quoted.find(" tail").unwrap()))
        );
        assert_eq!(
            blade_island_end_at(blade_comment, 0),
            Some(Some(blade_comment.find(" tail").unwrap()))
        );
    }

    #[test]
    fn php_delimited_policy_only_shields_block_comments() {
        const PROFILE: DelimitedTemplateProfile<'_> =
            DelimitedTemplateProfile::php(&[("<?", "?>")]);
        let block_comment = "<?php /* ?> */ echo ?> tail";

        assert_eq!(
            delimited_island_end_at(block_comment, 0, PROFILE),
            Some(Some(block_comment.find(" tail").unwrap()))
        );
        assert_eq!(
            delimited_island_end_at("<?php /* ?>", 0, PROFILE),
            Some(None)
        );

        for line_comment in ["<?php // ?> later ?>", "<?php # ?> later ?>"] {
            assert_eq!(
                delimited_island_end_at(line_comment, 0, PROFILE),
                Some(Some(line_comment.find(" later").unwrap()))
            );
        }
    }

    #[test]
    fn blade_directive_scanner_skips_escapes_and_invalid_candidates() {
        let value = "@@literal @ @1 @- tail@ @if($ready) @custom_name trailing@";
        let if_start = value.find("@if").unwrap();
        let custom_start = value.find("@custom_name").unwrap();

        assert_eq!(
            next_blade_directive(value, 0),
            Some((if_start, if_start + "@if".len(), Some(if_start + 3)))
        );
        assert_eq!(blade_island_end_at(value, 0), None);
        assert_eq!(blade_island_end_at(value, 1), None);
        for invalid in ["@ @1", "@1", "@-", "tail@"] {
            assert_eq!(
                blade_island_end_at(value, value.find(invalid).unwrap()),
                None
            );
        }
        assert_eq!(
            next_blade_directive(value, if_start + "@if($ready)".len()),
            Some((custom_start, custom_start + "@custom_name".len(), None))
        );
        assert_eq!(
            blade_island_end_at(value, custom_start),
            Some(Some(custom_start + "@custom_name".len()))
        );
    }

    #[test]
    fn blade_escape_runs_have_the_same_point_and_whole_value_meaning() {
        let value = "@@if @@@unless($hidden)";
        let directive_start = value.find("@@@unless").unwrap() + 2;
        let islands = blade_islands(value).unwrap();

        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0], directive_start..value.len());
        assert_eq!(blade_island_end_at(value, 0), None);
        assert_eq!(blade_island_end_at(value, 1), None);
        assert_eq!(
            blade_island_end_at(value, directive_start),
            Some(Some(value.len()))
        );
    }

    #[test]
    fn blade_directive_scanner_handles_long_invalid_sequences_iteratively() {
        let invalid = "@1".repeat(50_000);
        let value = format!("{invalid}@later");

        assert_eq!(
            next_blade_directive(&value, 0),
            Some((invalid.len(), invalid.len() + "@later".len(), None))
        );

        let escaped = "@".repeat(50_000);
        let value = format!("{escaped}@later");
        assert_eq!(
            next_blade_directive(&value, 0),
            Some((escaped.len(), escaped.len() + "@later".len(), None))
        );
    }
}

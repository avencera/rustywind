use std::ops::Range;

use winnow::{
    Parser,
    ascii::multispace0,
    stream::{LocatingSlice, Location, Stream},
    token::{any, literal, take_till, take_until, take_while},
};

use crate::source::{
    MarkupDialect, MarkupProfile, SourceDocument, StatelessTemplateSyntax, TemplateIslandEnd,
    TemplateIslandSyntax, template_island_end_at,
};
use crate::template_parser::{
    ExpressionSyntax, SvelteBlockKind, SvelteBraceContext, SvelteBraceEvent, SvelteBraceScan,
    SvelteBranchKind, balanced_end, javascript_expression, scan_svelte_brace,
};

type Input<'a> = LocatingSlice<&'a str>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClassAttribute(Range<usize>);

impl ClassAttribute {
    pub(crate) fn value_range(&self) -> Range<usize> {
        self.0.clone()
    }
}

pub(crate) fn class_attributes(document: SourceDocument<'_>) -> Option<Vec<ClassAttribute>> {
    let profile = document.language().markup_profile()?;
    MarkupParser::new(document.text(), profile).parse()
}

struct MarkupParser<'a> {
    source: &'a str,
    dialect: MarkupDialect,
    template: MarkupTemplateState,
    attributes: Vec<ClassAttribute>,
}

impl<'a> MarkupParser<'a> {
    fn new(source: &'a str, profile: MarkupProfile) -> Self {
        Self {
            source,
            dialect: profile.dialect,
            template: MarkupTemplateState::new(profile.islands),
            attributes: Vec::new(),
        }
    }

    fn parse(mut self) -> Option<Vec<ClassAttribute>> {
        let mut input = Input::new(self.source);
        if matches!(self.dialect, MarkupDialect::Astro) {
            self.parse_astro_frontmatter(&mut input)?;
        }

        while !remaining(&input).is_empty() {
            if matches!(self.dialect, MarkupDialect::Astro) {
                take_till::<_, _, winnow::error::ContextError>(0.., ('<', '{'))
                    .void()
                    .parse_next(&mut input)
                    .ok()?;
                if remaining(&input).starts_with('{') {
                    self.parse_astro_expression(&mut input)?;
                    continue;
                }
                if remaining(&input).is_empty() {
                    break;
                }
            } else {
                take_till::<_, _, winnow::error::ContextError>(0.., ('<', '{', '$', '#', '@'))
                    .void()
                    .parse_next(&mut input)
                    .ok()?;
                if remaining(&input).is_empty() {
                    break;
                }
                if self.consume_template_island(&mut input, SvelteBraceContext::Fragment)? {
                    continue;
                }
                if !remaining(&input).starts_with('<') {
                    any::<_, winnow::error::ContextError>
                        .void()
                        .parse_next(&mut input)
                        .ok()?;
                    continue;
                }
            }

            if remaining(&input).starts_with("<!--") {
                self.parse_comment(&mut input)?;
                continue;
            }

            if !is_markup_tag_start(remaining(&input).as_bytes()) {
                any::<_, winnow::error::ContextError>
                    .void()
                    .parse_next(&mut input)
                    .ok()?;
                continue;
            }

            let checkpoint = input.checkpoint();
            let attribute_count = self.attributes.len();
            let Some(tag) = self.parse_tag(&mut input) else {
                if self.template.is_malformed() {
                    return None;
                }
                self.attributes.truncate(attribute_count);
                input.reset(&checkpoint);
                consume_slice(&mut input, "<")?;
                continue;
            };
            if let Some(raw_text) = tag.raw_text {
                self.skip_raw_text(&mut input, raw_text)?;
            }
        }

        self.template.finish()?;
        Some(self.attributes)
    }

    fn parse_astro_frontmatter(&self, input: &mut Input<'a>) -> Option<()> {
        if remaining(input).starts_with('\u{feff}') {
            consume_slice(input, "\u{feff}")?;
        }
        let source = remaining(input);
        if !(source.starts_with("---\n") || source.starts_with("---\r\n")) {
            return Some(());
        }

        let end = frontmatter_end(source)?;
        consume_slice(input, &source[..end])
    }

    fn parse_comment(&self, input: &mut Input<'a>) -> Option<()> {
        (
            literal::<_, _, winnow::error::ContextError>("<!--"),
            take_until::<_, _, winnow::error::ContextError>(0.., "-->"),
            literal::<_, _, winnow::error::ContextError>("-->"),
        )
            .void()
            .parse_next(input)
            .ok()
    }

    fn parse_tag(&mut self, input: &mut Input<'a>) -> Option<ParsedTag<'a>> {
        literal::<_, _, winnow::error::ContextError>("<")
            .void()
            .parse_next(input)
            .ok()?;

        if remaining(input).starts_with('!') || remaining(input).starts_with('?') {
            return self.parse_declaration(input);
        }

        let closing = if remaining(input).starts_with('/') {
            literal::<_, _, winnow::error::ContextError>("/")
                .void()
                .parse_next(input)
                .ok()?;
            true
        } else {
            false
        };

        if matches!(self.dialect, MarkupDialect::Astro) && remaining(input).starts_with('>') {
            literal::<_, _, winnow::error::ContextError>(">")
                .void()
                .parse_next(input)
                .ok()?;
            return Some(ParsedTag::default());
        }

        let name: &str =
            take_while::<_, _, winnow::error::ContextError>(1.., is_tag_name_character)
                .parse_next(input)
                .ok()?;
        if closing {
            self.skip_tag_remainder(input)?;
            return Some(ParsedTag::default());
        }

        let mut is_raw = false;
        loop {
            multispace0::<_, winnow::error::ContextError>
                .void()
                .parse_next(input)
                .ok()?;

            if remaining(input).starts_with("/>") {
                literal::<_, _, winnow::error::ContextError>("/>")
                    .void()
                    .parse_next(input)
                    .ok()?;
                return Some(ParsedTag::default());
            }
            if remaining(input).starts_with('>') {
                literal::<_, _, winnow::error::ContextError>(">")
                    .void()
                    .parse_next(input)
                    .ok()?;
                let raw_text = if is_raw {
                    Some(RawTextElement::new(name, self.name_matching()))
                } else {
                    raw_text_element(name, self.name_matching())
                };
                return Some(ParsedTag { raw_text });
            }

            if self.consume_template_island(input, SvelteBraceContext::StartTag)? {
                continue;
            }

            let attribute = self.parse_attribute(input)??;
            is_raw |= matches!(self.dialect, MarkupDialect::Astro) && attribute.name == "is:raw";
            if (self.name_matching().matches(attribute.name, "class")
                || attribute.name == "className")
                && let Some(value) = attribute.quoted_value
            {
                self.attributes.push(ClassAttribute(value));
            }
        }
    }

    fn parse_declaration(&self, input: &mut Input<'a>) -> Option<ParsedTag<'a>> {
        self.skip_tag_remainder(input)?;
        Some(ParsedTag::default())
    }

    fn skip_tag_remainder(&self, input: &mut Input<'a>) -> Option<()> {
        take_until::<_, _, winnow::error::ContextError>(0.., ">")
            .void()
            .parse_next(input)
            .ok()?;
        literal::<_, _, winnow::error::ContextError>(">")
            .void()
            .parse_next(input)
            .ok()
    }

    fn parse_attribute(&mut self, input: &mut Input<'a>) -> Option<Option<ParsedAttribute<'a>>> {
        let Some(first) = remaining(input).chars().next() else {
            return Some(None);
        };
        if !is_attribute_name_character(first) {
            return Some(None);
        }

        let name: &'a str =
            take_while::<_, _, winnow::error::ContextError>(1.., is_attribute_name_character)
                .parse_next(input)
                .ok()?;
        multispace0::<_, winnow::error::ContextError>
            .void()
            .parse_next(input)
            .ok()?;
        if !remaining(input).starts_with('=') {
            return Some(Some(ParsedAttribute {
                name,
                quoted_value: None,
            }));
        }

        literal::<_, _, winnow::error::ContextError>("=")
            .void()
            .parse_next(input)
            .ok()?;
        multispace0::<_, winnow::error::ContextError>
            .void()
            .parse_next(input)
            .ok()?;

        let quote = remaining(input)
            .as_bytes()
            .first()
            .copied()
            .filter(|byte| matches!(byte, b'\'' | b'"'));
        let Some(quote) = quote else {
            self.parse_unquoted_value(input)?;
            return Some(Some(ParsedAttribute {
                name,
                quoted_value: None,
            }));
        };

        any::<_, winnow::error::ContextError>
            .void()
            .parse_next(input)
            .ok()?;
        let value_start = input.current_token_start();
        loop {
            let source = remaining(input);
            let byte = source.as_bytes().first().copied()?;
            if byte == quote {
                let value_end = input.current_token_start();
                any::<_, winnow::error::ContextError>
                    .void()
                    .parse_next(input)
                    .ok()?;
                return Some(Some(ParsedAttribute {
                    name,
                    quoted_value: Some(value_start..value_end),
                }));
            }
            if !matches!(self.dialect, MarkupDialect::Astro)
                && self.consume_template_island(input, SvelteBraceContext::AttributeValue)?
            {
                continue;
            }
            any::<_, winnow::error::ContextError>
                .void()
                .parse_next(input)
                .ok()?;
        }
    }

    fn parse_unquoted_value(&mut self, input: &mut Input<'a>) -> Option<()> {
        if self.consume_template_island(input, SvelteBraceContext::AttributeValue)? {
            return Some(());
        }

        take_while::<_, _, winnow::error::ContextError>(1.., |character: char| {
            !character.is_ascii_whitespace() && character != '>'
        })
        .void()
        .parse_next(input)
        .ok()
    }

    fn consume_template_island(
        &mut self,
        input: &mut Input<'a>,
        context: SvelteBraceContext,
    ) -> Option<bool> {
        let cursor = input.current_token_start();
        if matches!(self.dialect, MarkupDialect::Astro) && remaining(input).starts_with('{') {
            let end = balanced_end(self.source, cursor, '{', '}', ExpressionSyntax::JavaScript)?;
            consume_slice(input, &self.source[cursor..end])?;
            return Some(true);
        }

        let island = match &mut self.template {
            MarkupTemplateState::Svelte(state) => {
                match scan_svelte_brace(self.source, cursor, context) {
                    SvelteBraceScan::NotAnOpener => TemplateIslandEnd::NotAnOpener,
                    SvelteBraceScan::Closed(event) => state
                        .apply(event)
                        .map_or(TemplateIslandEnd::Malformed, TemplateIslandEnd::Closed),
                    SvelteBraceScan::Malformed => TemplateIslandEnd::Malformed,
                }
            }
            MarkupTemplateState::Stateless(syntax) => {
                template_island_end_at(self.source, cursor, *syntax)
            }
            MarkupTemplateState::Malformed => return None,
        };
        let end = match island {
            TemplateIslandEnd::NotAnOpener => return Some(false),
            TemplateIslandEnd::Closed(end) => end,
            TemplateIslandEnd::Malformed => {
                self.template = MarkupTemplateState::Malformed;
                return None;
            }
        };

        consume_slice(input, &self.source[cursor..end])?;
        Some(true)
    }

    fn parse_astro_expression(&mut self, input: &mut Input<'a>) -> Option<()> {
        let cursor = input.current_token_start();
        let expression = javascript_expression(self.source, cursor)?;
        let mut raw_until = 0;
        let mut tag_input = *input;
        for start in expression.markup_starts {
            let tag_cursor = tag_input.current_token_start();
            if start < raw_until || start < tag_cursor {
                continue;
            }

            consume_slice(&mut tag_input, &self.source[tag_cursor..start])?;
            let checkpoint = tag_input.checkpoint();
            let attribute_count = self.attributes.len();
            let Some(tag) = self.parse_tag(&mut tag_input) else {
                if self.template.is_malformed() {
                    return None;
                }
                self.attributes.truncate(attribute_count);
                tag_input.reset(&checkpoint);
                continue;
            };
            if let Some(raw_text) = tag.raw_text {
                self.skip_raw_text(&mut tag_input, raw_text)?;
                raw_until = tag_input.current_token_start();
            }
        }

        consume_slice(input, &self.source[cursor..expression.end])
    }

    fn skip_raw_text(&self, input: &mut Input<'a>, element: RawTextElement<'a>) -> Option<()> {
        let source = remaining(input);
        let closing = find_closing_tag(source, element)?;
        consume_slice(input, &source[..closing])
    }

    fn name_matching(&self) -> NameMatching {
        match self.dialect {
            MarkupDialect::Html => NameMatching::AsciiInsensitive,
            MarkupDialect::Svelte | MarkupDialect::Astro => NameMatching::Exact,
        }
    }
}

enum MarkupTemplateState {
    Stateless(StatelessTemplateSyntax),
    Svelte(SvelteMarkupState),
    Malformed,
}

impl MarkupTemplateState {
    const fn new(syntax: TemplateIslandSyntax) -> Self {
        match syntax {
            TemplateIslandSyntax::Stateless(syntax) => Self::Stateless(syntax),
            TemplateIslandSyntax::Svelte => Self::Svelte(SvelteMarkupState::new()),
        }
    }

    fn finish(&self) -> Option<()> {
        match self {
            Self::Svelte(state) if !state.blocks.is_empty() => None,
            Self::Malformed => None,
            _ => Some(()),
        }
    }

    const fn is_malformed(&self) -> bool {
        matches!(self, Self::Malformed)
    }
}

struct SvelteMarkupState {
    blocks: Vec<SvelteBlockKind>,
}

impl SvelteMarkupState {
    const fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    fn apply(&mut self, event: SvelteBraceEvent) -> Option<usize> {
        let end = event.end();
        match event {
            SvelteBraceEvent::Expression { .. } | SvelteBraceEvent::Special { .. } => {}
            SvelteBraceEvent::Open { kind, .. } => self.blocks.push(kind),
            SvelteBraceEvent::Branch { kind, .. } => {
                let block = self.blocks.last().copied()?;
                let valid = match kind {
                    SvelteBranchKind::Else => {
                        matches!(block, SvelteBlockKind::If | SvelteBlockKind::Each)
                    }
                    SvelteBranchKind::ElseIf => {
                        matches!(block, SvelteBlockKind::If)
                    }
                    SvelteBranchKind::Then | SvelteBranchKind::Catch => {
                        matches!(block, SvelteBlockKind::Await)
                    }
                };
                if !valid {
                    return None;
                }
            }
            SvelteBraceEvent::Close { kind, .. } => {
                if self.blocks.pop()? != kind {
                    return None;
                }
            }
        }
        Some(end)
    }
}

#[derive(Default)]
struct ParsedTag<'a> {
    raw_text: Option<RawTextElement<'a>>,
}

struct ParsedAttribute<'a> {
    name: &'a str,
    quoted_value: Option<Range<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NameMatching {
    Exact,
    AsciiInsensitive,
}

impl NameMatching {
    fn matches(self, left: &str, right: &str) -> bool {
        match self {
            Self::Exact => left == right,
            Self::AsciiInsensitive => left.eq_ignore_ascii_case(right),
        }
    }
}

#[derive(Clone, Copy)]
struct RawTextElement<'a> {
    name: &'a str,
    matching: NameMatching,
}

impl<'a> RawTextElement<'a> {
    const fn new(name: &'a str, matching: NameMatching) -> Self {
        Self { name, matching }
    }
}

fn raw_text_element(name: &str, matching: NameMatching) -> Option<RawTextElement<'_>> {
    [
        "script", "style", "textarea", "title", "xmp", "iframe", "noembed", "noframes",
    ]
    .into_iter()
    .any(|raw_name| matching.matches(name, raw_name))
    .then_some(RawTextElement::new(name, matching))
}

fn find_closing_tag(source: &str, element: RawTextElement<'_>) -> Option<usize> {
    source.match_indices("</").find_map(|(start, _)| {
        let name_start = start + 2;
        let name_end = name_start + element.name.len();
        let candidate = source.get(name_start..name_end)?;
        let boundary = source.as_bytes().get(name_end);
        (element.matching.matches(candidate, element.name)
            && boundary.is_none_or(|byte| byte.is_ascii_whitespace() || *byte == b'>'))
        .then_some(start)
    })
}

fn frontmatter_end(source: &str) -> Option<usize> {
    let mut offset = source.find('\n')? + 1;
    while offset <= source.len() {
        let line_end = source[offset..]
            .find('\n')
            .map_or(source.len(), |relative| offset + relative + 1);
        if source[offset..line_end].trim_end_matches(['\r', '\n']) == "---" {
            return Some(line_end);
        }
        if line_end == source.len() {
            break;
        }
        offset = line_end;
    }
    None
}

fn is_markup_tag_start(source: &[u8]) -> bool {
    match source.get(1).copied() {
        Some(first) if first.is_ascii_alphabetic() || matches!(first, b'!' | b'?') => true,
        Some(b'/') => source
            .get(2)
            .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'>'),
        Some(b'>') => true,
        _ => false,
    }
}

fn is_tag_name_character(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '-' | '_' | ':' | '.')
}

pub(crate) fn is_attribute_name_character(character: char) -> bool {
    !character.is_ascii_whitespace()
        && !character.is_control()
        && !matches!(
            character,
            '=' | '/' | '>' | '<' | '&' | '\'' | '"' | '`' | '{' | '}'
        )
}

fn remaining<'a>(input: &Input<'a>) -> &'a str {
    input.as_ref()
}

fn consume_slice(input: &mut Input<'_>, slice: &str) -> Option<()> {
    if slice.is_empty() {
        return Some(());
    }
    literal::<_, _, winnow::error::ContextError>(slice)
        .void()
        .parse_next(input)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::class_attributes;
    use crate::source::{SourceDocument, SourceLanguage};

    fn values(source: &str, language: SourceLanguage) -> Option<Vec<&str>> {
        class_attributes(SourceDocument::new(source, language)).map(|attributes| {
            attributes
                .into_iter()
                .map(|attribute| &source[attribute.value_range()])
                .collect()
        })
    }

    #[test]
    fn extracts_only_real_quoted_class_attributes() {
        let source =
            r#"<div :class="ignored" data-code={' class="also ignored"'} class="p-4 m-4"></div>"#;
        assert_eq!(
            values(source, SourceLanguage::Svelte),
            Some(vec!["p-4 m-4"])
        );
    }

    #[test]
    fn html_class_names_are_ascii_case_insensitive_but_class_name_is_exact() {
        let source = r#"<div CLASS="one" Class="two" className="three" CLASSNAME="ignored"></div>"#;

        assert_eq!(
            values(source, SourceLanguage::Html),
            Some(vec!["one", "two", "three"])
        );
    }

    #[test]
    fn component_dialects_match_class_attribute_names_exactly() {
        let source =
            r#"<div CLASS="ignored" class="one" className="two" ClassName="ignored"></div>"#;

        assert_eq!(
            values(source, SourceLanguage::Svelte),
            Some(vec!["one", "two"])
        );
        assert_eq!(
            values(source, SourceLanguage::Astro),
            Some(vec!["one", "two"])
        );
    }

    #[test]
    fn extracts_classes_across_nested_svelte_blocks() {
        let source = r#"<div class="before"></div>{#if visible}<div class="inside-if"></div>{#each items as item}<div class="inside-each"></div>{:else}<div class="inside-else"></div>{/each}{/if}<div class="after"></div>"#;

        assert_eq!(
            values(source, SourceLanguage::Svelte),
            Some(vec![
                "before",
                "inside-if",
                "inside-each",
                "inside-else",
                "after"
            ])
        );
    }

    #[test]
    fn svelte_structure_controls_whether_class_extraction_is_safe() {
        let attached = r#"<div {@attach setup} class="sortable"></div>"#;
        assert_eq!(
            values(attached, SourceLanguage::Svelte),
            Some(vec!["sortable"])
        );

        for malformed in [
            r#"{#if visible}<div class="unsafe"></div>{/each}"#,
            r#"{#if visible}<div class="unsafe"></div>"#,
            r#"<div class="{#if visible} unsafe {/if}"></div>"#,
        ] {
            assert_eq!(values(malformed, SourceLanguage::Svelte), None);
        }
    }

    #[test]
    fn astro_frontmatter_is_opaque() {
        let source =
            "---\nconst markup = '<div class=\"ignored\">';\n---\n<div class=\"p-4 m-4\"></div>";
        assert_eq!(values(source, SourceLanguage::Astro), Some(vec!["p-4 m-4"]));
    }
}

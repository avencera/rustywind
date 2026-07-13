use std::{ops::Range, path::Path};

/// The template or markup language used by a source document
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLanguage {
    /// Plain HTML
    Html,
    /// Svelte markup
    Svelte,
    /// Django templates
    Django,
    /// Jinja templates
    Jinja,
    /// Twig templates
    Twig,
    /// Liquid templates
    Liquid,
    /// Handlebars templates
    Handlebars,
    /// Embedded Ruby templates
    Erb,
    /// Embedded JavaScript templates
    Ejs,
    /// PHP templates
    Php,
    /// Blade templates
    Blade,
    /// Lit templates
    Lit,
    /// Ruby source
    Ruby,
    /// A source language that could not be inferred
    Unknown,
}

impl SourceLanguage {
    /// Infers a source language from a path's conventional extension
    #[must_use]
    pub fn from_path(path: &Path) -> Self {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            return Self::Unknown;
        };
        let file_name = file_name.to_ascii_lowercase();

        if file_name.ends_with(".blade.php") {
            return Self::Blade;
        }
        if file_name.ends_with(".html.erb") || file_name.ends_with(".htm.erb") {
            return Self::Erb;
        }
        if file_name.ends_with(".lit.html") {
            return Self::Lit;
        }
        if file_name.ends_with(".django.html") {
            return Self::Django;
        }
        if file_name.ends_with(".jinja.html") || file_name.ends_with(".jinja2.html") {
            return Self::Jinja;
        }

        match Path::new(&file_name)
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some("html" | "htm") => Self::Html,
            Some("svelte") => Self::Svelte,
            Some("django" | "djhtml") => Self::Django,
            Some("jinja" | "jinja2" | "j2") => Self::Jinja,
            Some("twig") => Self::Twig,
            Some("liquid") => Self::Liquid,
            Some("hbs" | "handlebars") => Self::Handlebars,
            Some("erb") => Self::Erb,
            Some("ejs") => Self::Ejs,
            Some("php" | "phtml") => Self::Php,
            Some("blade") => Self::Blade,
            Some("lit") => Self::Lit,
            Some("rb") => Self::Ruby,
            _ => Self::Unknown,
        }
    }
}

/// Source text paired with the language profile used to interpret it
#[derive(Debug, Clone, Copy)]
pub struct SourceDocument<'a>(&'a str, SourceLanguage);

impl<'a> SourceDocument<'a> {
    /// Creates a source document from borrowed text and its language
    #[must_use]
    pub const fn new(text: &'a str, language: SourceLanguage) -> Self {
        Self(text, language)
    }

    /// Returns the source text
    #[must_use]
    pub const fn text(self) -> &'a str {
        self.0
    }

    /// Returns the source language
    #[must_use]
    pub const fn language(self) -> SourceLanguage {
        self.1
    }
}

pub(crate) fn sortable_spans(value: &str, language: SourceLanguage) -> Option<Vec<Range<usize>>> {
    if value.is_empty() {
        return Some(Vec::new());
    }

    let islands = match language {
        SourceLanguage::Html | SourceLanguage::Unknown => Some(Vec::new()),
        SourceLanguage::Svelte => balanced_brace_islands(value, "{", ExpressionSyntax::JavaScript),
        SourceLanguage::Django | SourceLanguage::Jinja | SourceLanguage::Twig => {
            delimited_islands(value, &[("{{", "}}"), ("{%", "%}"), ("{#", "#}")])
        }
        SourceLanguage::Liquid => delimited_islands(value, &[("{{", "}}"), ("{%", "%}")]),
        SourceLanguage::Handlebars => delimited_islands(value, &[("{{{", "}}}"), ("{{", "}}")]),
        SourceLanguage::Erb | SourceLanguage::Ejs => delimited_islands(value, &[("<%", "%>")]),
        SourceLanguage::Php => delimited_islands(value, &[("<?", "?>")]),
        SourceLanguage::Blade => blade_islands(value),
        SourceLanguage::Lit => balanced_brace_islands(value, "${", ExpressionSyntax::JavaScript),
        SourceLanguage::Ruby => balanced_brace_islands(value, "#{", ExpressionSyntax::Ruby),
    }?;

    Some(static_spans(value, islands))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TemplateIslandEnd {
    NotAnOpener,
    Closed(usize),
    Malformed,
}

pub(crate) fn template_island_end_at(
    value: &str,
    cursor: usize,
    language: SourceLanguage,
) -> TemplateIslandEnd {
    if !value
        .as_bytes()
        .get(cursor)
        .is_some_and(|byte| matches!(byte, b'{' | b'<' | b'$' | b'#' | b'@'))
    {
        return TemplateIslandEnd::NotAnOpener;
    }

    let remaining = &value[cursor..];
    let balanced = |opener: &str, syntax| {
        if !remaining.starts_with(opener) {
            return TemplateIslandEnd::NotAnOpener;
        }
        balanced_end(value, cursor + opener.len() - 1, b'{', b'}', syntax)
            .map_or(TemplateIslandEnd::Malformed, TemplateIslandEnd::Closed)
    };

    match language {
        SourceLanguage::Html | SourceLanguage::Unknown => TemplateIslandEnd::NotAnOpener,
        SourceLanguage::Svelte => balanced("{", ExpressionSyntax::JavaScript),
        SourceLanguage::Django | SourceLanguage::Jinja | SourceLanguage::Twig => {
            delimited_island_end_at(value, cursor, &[("{{", "}}"), ("{%", "%}"), ("{#", "#}")])
        }
        SourceLanguage::Liquid => {
            delimited_island_end_at(value, cursor, &[("{{", "}}"), ("{%", "%}")])
        }
        SourceLanguage::Handlebars => {
            delimited_island_end_at(value, cursor, &[("{{{", "}}}"), ("{{", "}}")])
        }
        SourceLanguage::Erb | SourceLanguage::Ejs => {
            delimited_island_end_at(value, cursor, &[("<%", "%>")])
        }
        SourceLanguage::Php => delimited_island_end_at(value, cursor, &[("<?", "?>")]),
        SourceLanguage::Blade => {
            let mustache = delimited_island_end_at(
                value,
                cursor,
                &[("{{--", "--}}"), ("{!!", "!!}"), ("{{", "}}")],
            );
            if !matches!(mustache, TemplateIslandEnd::NotAnOpener) {
                return mustache;
            }

            match next_blade_directive(value, cursor) {
                Some((start, identifier_end, parenthesis)) if start == cursor => parenthesis
                    .map(|parenthesis| {
                        balanced_end(value, parenthesis, b'(', b')', ExpressionSyntax::Php)
                            .map_or(TemplateIslandEnd::Malformed, TemplateIslandEnd::Closed)
                    })
                    .unwrap_or(TemplateIslandEnd::Closed(identifier_end)),
                _ => TemplateIslandEnd::NotAnOpener,
            }
        }
        SourceLanguage::Lit => balanced("${", ExpressionSyntax::JavaScript),
        SourceLanguage::Ruby => balanced("#{", ExpressionSyntax::Ruby),
    }
}

fn delimited_island_end_at(
    value: &str,
    cursor: usize,
    delimiters: &[(&str, &str)],
) -> TemplateIslandEnd {
    for &(opener, closer) in delimiters {
        if value[cursor..].starts_with(opener) {
            return delimited_end(value, cursor + opener.len(), closer)
                .map_or(TemplateIslandEnd::Malformed, TemplateIslandEnd::Closed);
        }
    }

    TemplateIslandEnd::NotAnOpener
}

fn delimited_islands(value: &str, delimiters: &[(&str, &str)]) -> Option<Vec<Range<usize>>> {
    let mut islands = Vec::new();
    let mut cursor = 0;

    while cursor < value.len() {
        let Some((start, opener, closer)) = next_delimiter(value, cursor, delimiters) else {
            break;
        };
        let content_start = start + opener.len();
        let end = delimited_end(value, content_start, closer)?;
        islands.push(start..end);
        cursor = end;
    }

    Some(islands)
}

fn delimited_end(value: &str, mut cursor: usize, closer: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let closer = closer.as_bytes();
    let mut quote = None;
    let mut escaped = false;

    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == active_quote {
                quote = None;
            }
        } else if matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
        } else if bytes[cursor..].starts_with(closer) {
            return Some(cursor + closer.len());
        }
        cursor += 1;
    }

    None
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpressionSyntax {
    JavaScript,
    Php,
    Ruby,
}

fn balanced_brace_islands(
    value: &str,
    opener: &str,
    syntax: ExpressionSyntax,
) -> Option<Vec<Range<usize>>> {
    let mut islands = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = value[cursor..].find(opener) {
        let start = cursor + relative_start;
        let brace_start = start + opener.len() - 1;
        let end = balanced_end(value, brace_start, b'{', b'}', syntax)?;
        islands.push(start..end);
        cursor = end;
    }

    Some(islands)
}

fn balanced_end(
    value: &str,
    opening_index: usize,
    opening: u8,
    closing: u8,
    syntax: ExpressionSyntax,
) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut cursor = opening_index;
    let mut depth = 0;
    let mut can_start_regex = true;

    while cursor < bytes.len() {
        let byte = bytes[cursor];

        if matches!(syntax, ExpressionSyntax::JavaScript | ExpressionSyntax::Php)
            && bytes[cursor..].starts_with(b"//")
        {
            cursor = line_end(bytes, cursor + 2);
            continue;
        }
        if matches!(syntax, ExpressionSyntax::JavaScript | ExpressionSyntax::Php)
            && bytes[cursor..].starts_with(b"/*")
        {
            cursor = block_comment_end(bytes, cursor + 2)?;
            continue;
        }
        if matches!(syntax, ExpressionSyntax::Ruby | ExpressionSyntax::Php) && byte == b'#' {
            cursor = line_end(bytes, cursor + 1);
            continue;
        }

        if matches!(syntax, ExpressionSyntax::Ruby) && byte == b'`' {
            return None;
        }

        if matches!(byte, b'\'' | b'"')
            || byte == b'`'
                && matches!(syntax, ExpressionSyntax::JavaScript | ExpressionSyntax::Php)
        {
            let rejected_opener = match (syntax, byte) {
                (ExpressionSyntax::JavaScript, b'`') => Some(b"${".as_slice()),
                (ExpressionSyntax::Ruby, b'"') => Some(b"#{".as_slice()),
                _ => None,
            };
            cursor = quoted_end(bytes, cursor, byte, rejected_opener)?;
            can_start_regex = false;
            continue;
        }

        if matches!(syntax, ExpressionSyntax::Ruby)
            && byte == b'%'
            && bytes
                .get(cursor + 1)
                .is_some_and(|kind| matches!(kind, b'q' | b'Q' | b'r' | b'w' | b'W' | b'x'))
        {
            return None;
        }

        if byte == b'/'
            && matches!(
                syntax,
                ExpressionSyntax::JavaScript | ExpressionSyntax::Ruby
            )
        {
            if can_start_regex {
                cursor = regex_end(bytes, cursor)?;
                can_start_regex = false;
            } else {
                cursor += 1;
                can_start_regex = true;
            }
            continue;
        }

        if byte == opening {
            depth += 1;
            can_start_regex = true;
        } else if byte == closing {
            depth -= 1;
            if depth == 0 {
                return Some(cursor + 1);
            }
            can_start_regex = false;
        } else if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') {
            let start = cursor;
            cursor += 1;
            while cursor < bytes.len()
                && (bytes[cursor].is_ascii_alphanumeric() || matches!(bytes[cursor], b'_' | b'$'))
            {
                cursor += 1;
            }
            can_start_regex = expression_keyword_allows_regex(&value[start..cursor]);
            continue;
        } else if byte.is_ascii_digit() || matches!(byte, b')' | b']') {
            can_start_regex = false;
        } else if !byte.is_ascii_whitespace() {
            can_start_regex = true;
        }
        cursor += 1;
    }

    None
}

fn line_end(bytes: &[u8], cursor: usize) -> usize {
    bytes[cursor..]
        .iter()
        .position(|byte| matches!(byte, b'\n' | b'\r'))
        .map_or(bytes.len(), |offset| cursor + offset + 1)
}

fn block_comment_end(bytes: &[u8], cursor: usize) -> Option<usize> {
    bytes[cursor..]
        .windows(2)
        .position(|window| window == b"*/")
        .map(|offset| cursor + offset + 2)
}

fn quoted_end(
    bytes: &[u8],
    start: usize,
    quote: u8,
    rejected_opener: Option<&[u8]>,
) -> Option<usize> {
    let mut cursor = start + 1;
    let mut escaped = false;

    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if rejected_opener.is_some_and(|opener| bytes[cursor..].starts_with(opener)) {
            return None;
        } else if byte == quote {
            return Some(cursor + 1);
        }
        cursor += 1;
    }

    None
}

fn regex_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start + 1;
    let mut escaped = false;
    let mut character_class = false;

    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'[' {
            character_class = true;
        } else if byte == b']' {
            character_class = false;
        } else if byte == b'/' && !character_class {
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_alphabetic() {
                cursor += 1;
            }
            return Some(cursor);
        } else if matches!(byte, b'\n' | b'\r') {
            return None;
        }
        cursor += 1;
    }

    None
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

fn blade_islands(value: &str) -> Option<Vec<Range<usize>>> {
    const MUSTACHES: &[(&str, &str)] = &[("{{--", "--}}"), ("{!!", "!!}"), ("{{", "}}")];

    let mut islands = Vec::new();
    let mut cursor = 0;

    while let Some(island) = next_blade_island(value, cursor, MUSTACHES) {
        match island {
            BladeIsland::Delimited {
                start,
                opener,
                closer,
            } => {
                let content_start = start + opener.len();
                let end = delimited_end(value, content_start, closer)?;
                islands.push(start..end);
                cursor = end;
            }
            BladeIsland::Directive {
                start,
                identifier_end,
                parenthesis,
            } => {
                let end = if let Some(parenthesis) = parenthesis {
                    balanced_end(value, parenthesis, b'(', b')', ExpressionSyntax::Php)?
                } else {
                    identifier_end
                };
                islands.push(start..end);
                cursor = end;
            }
        }
    }

    Some(islands)
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
    let bytes = value.as_bytes();
    let mut search = cursor;

    while let Some(relative_start) = value[search..].find('@') {
        let start = search + relative_start;
        let mut index = start + 1;
        if index == bytes.len() || !(bytes[index].is_ascii_alphabetic() || bytes[index] == b'_') {
            search = index;
            continue;
        }
        index += 1;
        while index < bytes.len() && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
        {
            index += 1;
        }
        let identifier_end = index;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        return Some((
            start,
            identifier_end,
            (index < bytes.len() && bytes[index] == b'(').then_some(index),
        ));
    }

    None
}

fn static_spans(value: &str, islands: Vec<Range<usize>>) -> Vec<Range<usize>> {
    if islands.is_empty() {
        return std::iter::once(0..value.len()).collect();
    }

    let mut opaque = islands
        .into_iter()
        .map(|island| expand_to_token(value, island))
        .collect::<Vec<_>>();
    opaque.sort_unstable_by_key(|span| span.start);

    let mut merged: Vec<Range<usize>> = Vec::with_capacity(opaque.len());
    for span in opaque {
        if let Some(previous) = merged.last_mut()
            && span.start <= previous.end
        {
            previous.end = previous.end.max(span.end);
        } else {
            merged.push(span);
        }
    }

    let mut spans = Vec::new();
    let mut cursor = 0;
    for island in merged {
        push_trimmed_span(value, cursor..island.start, &mut spans);
        cursor = island.end;
    }
    push_trimmed_span(value, cursor..value.len(), &mut spans);
    spans
}

fn expand_to_token(value: &str, mut island: Range<usize>) -> Range<usize> {
    while let Some((index, character)) = value[..island.start].char_indices().next_back() {
        if character.is_ascii_whitespace() {
            break;
        }
        island.start = index;
    }
    while let Some(character) = value[island.end..].chars().next() {
        if character.is_ascii_whitespace() {
            break;
        }
        island.end += character.len_utf8();
    }
    island
}

fn push_trimmed_span(value: &str, mut span: Range<usize>, spans: &mut Vec<Range<usize>>) {
    let bytes = value.as_bytes();
    while span.start < span.end && bytes[span.start].is_ascii_whitespace() {
        span.start += 1;
    }
    while span.end > span.start && bytes[span.end - 1].is_ascii_whitespace() {
        span.end -= 1;
    }
    if !span.is_empty() {
        spans.push(span);
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceDocument, SourceLanguage, sortable_spans};
    use std::{ops::Range, path::Path};

    fn span_texts<'a>(value: &'a str, spans: &[Range<usize>]) -> Vec<&'a str> {
        spans.iter().map(|span| &value[span.clone()]).collect()
    }

    #[test]
    fn infers_source_language_from_simple_and_compound_extensions() {
        let cases = [
            ("index.HTML", SourceLanguage::Html),
            ("component.svelte", SourceLanguage::Svelte),
            ("page.django.html", SourceLanguage::Django),
            ("page.jinja2", SourceLanguage::Jinja),
            ("page.twig", SourceLanguage::Twig),
            ("page.liquid", SourceLanguage::Liquid),
            ("page.hbs", SourceLanguage::Handlebars),
            ("page.html.erb", SourceLanguage::Erb),
            ("page.ejs", SourceLanguage::Ejs),
            ("page.php", SourceLanguage::Php),
            ("page.blade.php", SourceLanguage::Blade),
            ("component.lit.html", SourceLanguage::Lit),
            ("helper.rb", SourceLanguage::Ruby),
            ("README.md", SourceLanguage::Unknown),
        ];

        for (path, expected) in cases {
            assert_eq!(SourceLanguage::from_path(Path::new(path)), expected);
        }
    }

    #[test]
    fn source_document_exposes_its_text_and_language() {
        let document = SourceDocument::new("text-sm", SourceLanguage::Html);

        assert_eq!(document.text(), "text-sm");
        assert_eq!(document.language(), SourceLanguage::Html);
    }

    #[test]
    fn keeps_svelte_interpolation_from_issue_142_opaque() {
        let value = "flex {condition ? 'items-center' : 'items-start'} gap-2";
        let spans = sortable_spans(value, SourceLanguage::Svelte).unwrap();

        assert_eq!(span_texts(value, &spans), ["flex", "gap-2"]);
    }

    #[test]
    fn balances_nested_svelte_braces_and_quoted_braces() {
        let value = "p-2 {thing({ nested: `}` }, \"}\")} text-sm";
        let spans = sortable_spans(value, SourceLanguage::Svelte).unwrap();

        assert_eq!(span_texts(value, &spans), ["p-2", "text-sm"]);
    }

    #[test]
    fn attached_interpolations_make_the_entire_token_opaque() {
        let value = "p-2 btn-{{kind}} flex{active} text-sm";
        let spans = sortable_spans(value, SourceLanguage::Svelte).unwrap();

        assert_eq!(span_texts(value, &spans), ["p-2", "text-sm"]);

        let jinja = sortable_spans(value, SourceLanguage::Jinja).unwrap();
        assert_eq!(span_texts(value, &jinja), ["p-2", "flex{active} text-sm"]);
    }

    #[test]
    fn handles_multiple_islands_without_consuming_separators() {
        let value = "  p-2 gap-1  {first} \t text-sm font-bold {second}  block  ";
        let spans = sortable_spans(value, SourceLanguage::Svelte).unwrap();

        assert_eq!(
            span_texts(value, &spans),
            ["p-2 gap-1", "text-sm font-bold", "block"]
        );
        assert_eq!(&value[spans[0].end..spans[1].start], "  {first} \t ");
    }

    #[test]
    fn malformed_template_openers_reject_the_whole_value() {
        assert_eq!(sortable_spans("p-2 {missing", SourceLanguage::Svelte), None);
        assert_eq!(
            sortable_spans("p-2 {{ missing", SourceLanguage::Jinja),
            None
        );
        assert_eq!(sortable_spans("p-2 <% missing", SourceLanguage::Erb), None);
        assert_eq!(sortable_spans("p-2 ${missing", SourceLanguage::Lit), None);
    }

    #[test]
    fn identifies_jinja_static_runs() {
        let value = "px-2 {{ user.class_name }} py-1 {% if wide %} w-full {% endif %}";
        let spans = sortable_spans(value, SourceLanguage::Jinja).unwrap();

        assert_eq!(span_texts(value, &spans), ["px-2", "py-1", "w-full"]);
    }

    #[test]
    fn identifies_erb_static_runs() {
        let value = "px-2 <%= size_class %> py-1 <% if wide %> w-full <% end %>";
        let spans = sortable_spans(value, SourceLanguage::Erb).unwrap();

        assert_eq!(span_texts(value, &spans), ["px-2", "py-1", "w-full"]);
    }

    #[test]
    fn ignores_delimiters_inside_template_strings() {
        let value = r#"px-2 {{ value == "}}" ? "a" : "b" }} py-1"#;
        let spans = sortable_spans(value, SourceLanguage::Jinja).unwrap();

        assert_eq!(span_texts(value, &spans), ["px-2", "py-1"]);
    }

    #[test]
    fn ignores_braces_inside_expression_comments_and_regex_literals() {
        let block_comment = "p-4 { /* } */ foo foo ? /}/.test(value) : 'block' } m-4";
        let line_comment = "p-4 { value // }\n ? 'flex' : 'block' } m-4";

        for value in [block_comment, line_comment] {
            let spans = sortable_spans(value, SourceLanguage::Svelte).unwrap();
            assert_eq!(span_texts(value, &spans), ["p-4", "m-4"]);
        }
    }

    #[test]
    fn keeps_bare_blade_directives_and_unicode_tokens_opaque() {
        let value = "p-2 café-{{ $kind }} @error('name') border-red @enderror text-sm";
        let spans = sortable_spans(value, SourceLanguage::Blade).unwrap();

        assert_eq!(span_texts(value, &spans), ["p-2", "border-red", "text-sm"]);
    }

    #[test]
    fn balances_lit_and_ruby_interpolations() {
        let lit = "p-2 ${map({ key: `}` })} text-sm";
        let ruby = "p-2 #{call({ key: \"}\" })} text-sm";

        assert_eq!(
            span_texts(lit, &sortable_spans(lit, SourceLanguage::Lit).unwrap()),
            ["p-2", "text-sm"]
        );
        assert_eq!(
            span_texts(ruby, &sortable_spans(ruby, SourceLanguage::Ruby).unwrap()),
            ["p-2", "text-sm"]
        );
    }

    #[test]
    fn treats_plain_html_as_one_sortable_span() {
        let value = "  text-sm  font-bold\tblock  ";

        assert_eq!(
            sortable_spans(value, SourceLanguage::Html),
            Some(std::iter::once(0..value.len()).collect())
        );
    }
}

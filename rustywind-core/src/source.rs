use std::{ops::Range, path::Path};

use crate::template_parser::{
    ClassValueValidation, DelimitedTemplateProfile, ExpressionSyntax, balanced_end,
    balanced_islands as balanced_brace_islands, blade_island_end_at, blade_islands,
    delimited_island_end_at, delimited_islands, validate_class_value,
};

const PHP_DELIMITERS: &[(&str, &str)] = &[("<?", "?>")];
const DJANGO_DELIMITERS: &[(&str, &str)] = &[("{{", "}}"), ("{%", "%}"), ("{#", "#}")];
const LIQUID_DELIMITERS: &[(&str, &str)] = &[("{{", "}}"), ("{%", "%}")];
const HANDLEBARS_DELIMITERS: &[(&str, &str)] = &[("{{{", "}}}"), ("{{", "}}")];
const ERB_DELIMITERS: &[(&str, &str)] = &[("<%", "%>")];

const PHP_PROFILE: DelimitedTemplateProfile<'_> = DelimitedTemplateProfile::php(PHP_DELIMITERS);
const DJANGO_PROFILE: DelimitedTemplateProfile<'_> =
    DelimitedTemplateProfile::template_tokens(DJANGO_DELIMITERS);

const LIQUID_PROFILE: DelimitedTemplateProfile<'_> =
    DelimitedTemplateProfile::template_tokens(LIQUID_DELIMITERS);

const HANDLEBARS_PROFILE: DelimitedTemplateProfile<'_> =
    DelimitedTemplateProfile::template_tokens(HANDLEBARS_DELIMITERS);

const ERB_PROFILE: DelimitedTemplateProfile<'_> =
    DelimitedTemplateProfile::template_tokens(ERB_DELIMITERS);

/// The template or markup language used by a source document
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLanguage {
    /// Plain HTML
    Html,
    /// Svelte markup
    Svelte,
    /// Astro components
    Astro,
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
            Some("astro") => Self::Astro,
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

    pub(crate) const fn markup_dialect(self) -> Option<MarkupDialect> {
        SourceProfile::new(self).markup
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceProfile {
    class_values: ClassValueSyntax,
    markup: Option<MarkupDialect>,
}

impl SourceProfile {
    const fn new(language: SourceLanguage) -> Self {
        let markup = match language {
            SourceLanguage::Unknown => None,
            SourceLanguage::Svelte => Some(MarkupDialect::Svelte),
            SourceLanguage::Astro => Some(MarkupDialect::Astro),
            _ => Some(MarkupDialect::Html),
        };
        let class_values = match language {
            SourceLanguage::Html | SourceLanguage::Astro | SourceLanguage::Unknown => {
                ClassValueSyntax::Unspecified
            }
            SourceLanguage::Svelte => ClassValueSyntax::Balanced {
                opener: "{",
                expression: ExpressionSyntax::JavaScript,
            },
            SourceLanguage::Django | SourceLanguage::Jinja | SourceLanguage::Twig => {
                ClassValueSyntax::Delimited(DJANGO_PROFILE)
            }
            SourceLanguage::Liquid => ClassValueSyntax::Delimited(LIQUID_PROFILE),
            SourceLanguage::Handlebars => ClassValueSyntax::Delimited(HANDLEBARS_PROFILE),
            SourceLanguage::Erb | SourceLanguage::Ejs => ClassValueSyntax::Delimited(ERB_PROFILE),
            SourceLanguage::Php => ClassValueSyntax::Delimited(PHP_PROFILE),
            SourceLanguage::Blade => ClassValueSyntax::Blade,
            SourceLanguage::Lit => ClassValueSyntax::Balanced {
                opener: "${",
                expression: ExpressionSyntax::JavaScript,
            },
            SourceLanguage::Ruby => ClassValueSyntax::Balanced {
                opener: "#{",
                expression: ExpressionSyntax::Ruby,
            },
        };

        Self {
            class_values,
            markup,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassValueSyntax {
    Unspecified,
    Balanced {
        opener: &'static str,
        expression: ExpressionSyntax,
    },
    Delimited(DelimitedTemplateProfile<'static>),
    Blade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarkupDialect {
    Html,
    Svelte,
    Astro,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ClassValueAnalysis {
    Sortable(StaticRuns),
    Opaque,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaticRuns(Vec<Range<usize>>);

impl StaticRuns {
    fn new(value: &str, spans: Vec<Range<usize>>) -> Option<Self> {
        debug_assert!(spans.iter().all(|span| {
            span.start < span.end
                && value.is_char_boundary(span.start)
                && value.is_char_boundary(span.end)
        }));
        debug_assert!(spans.windows(2).all(|pair| pair[0].end <= pair[1].start));

        (!spans.is_empty()).then_some(Self(spans))
    }

    fn whole(value: &str) -> Option<Self> {
        (!value.trim().is_empty()).then(|| Self(std::iter::once(0..value.len()).collect()))
    }

    pub(crate) fn iter(&self) -> impl DoubleEndedIterator<Item = &Range<usize>> {
        self.0.iter()
    }
}

pub(crate) fn analyze_class_value(value: &str, language: SourceLanguage) -> ClassValueAnalysis {
    let syntax = SourceProfile::new(language).class_values;
    if matches!(syntax, ClassValueSyntax::Unspecified) {
        return if is_static_unspecified_class_value(value) {
            StaticRuns::whole(value)
                .map_or(ClassValueAnalysis::Opaque, ClassValueAnalysis::Sortable)
        } else {
            ClassValueAnalysis::Opaque
        };
    }

    let islands = match syntax {
        ClassValueSyntax::Unspecified => unreachable!("handled above"),
        ClassValueSyntax::Balanced { opener, expression } => {
            balanced_brace_islands(value, opener, expression)
        }
        ClassValueSyntax::Delimited(profile) => delimited_islands(value, profile),
        ClassValueSyntax::Blade => blade_islands(value),
    };

    islands
        .and_then(|islands| StaticRuns::new(value, static_spans(value, islands)))
        .map_or(ClassValueAnalysis::Opaque, ClassValueAnalysis::Sortable)
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
        balanced_end(value, cursor + opener.len() - 1, '{', '}', syntax)
            .map_or(TemplateIslandEnd::Malformed, TemplateIslandEnd::Closed)
    };

    match SourceProfile::new(language).class_values {
        ClassValueSyntax::Unspecified => TemplateIslandEnd::NotAnOpener,
        ClassValueSyntax::Balanced { opener, expression } => balanced(opener, expression),
        ClassValueSyntax::Delimited(profile) => {
            match delimited_island_end_at(value, cursor, profile) {
                None => TemplateIslandEnd::NotAnOpener,
                Some(Some(end)) => TemplateIslandEnd::Closed(end),
                Some(None) => TemplateIslandEnd::Malformed,
            }
        }
        ClassValueSyntax::Blade => match blade_island_end_at(value, cursor) {
            None => TemplateIslandEnd::NotAnOpener,
            Some(Some(end)) => TemplateIslandEnd::Closed(end),
            Some(None) => TemplateIslandEnd::Malformed,
        },
    }
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

pub(crate) fn is_plain_class_list(value: &str) -> bool {
    validate_class_value(value, ClassValueValidation::Direct)
}

fn is_static_unspecified_class_value(value: &str) -> bool {
    validate_class_value(value, ClassValueValidation::Unspecified)
}

#[cfg(test)]
mod tests {
    use super::{
        ClassValueAnalysis, SourceDocument, SourceLanguage, TemplateIslandEnd, analyze_class_value,
        template_island_end_at,
    };
    use std::{ops::Range, path::Path};

    fn sortable_spans(value: &str, language: SourceLanguage) -> Option<Vec<Range<usize>>> {
        match analyze_class_value(value, language) {
            ClassValueAnalysis::Sortable(runs) => Some(runs.0),
            ClassValueAnalysis::Opaque => None,
        }
    }

    fn span_texts<'a>(value: &'a str, spans: &[Range<usize>]) -> Vec<&'a str> {
        spans.iter().map(|span| &value[span.clone()]).collect()
    }

    #[test]
    fn infers_source_language_from_simple_and_compound_extensions() {
        let cases = [
            ("index.HTML", SourceLanguage::Html),
            ("component.svelte", SourceLanguage::Svelte),
            ("component.astro", SourceLanguage::Astro),
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
    fn php_block_comment_closers_stay_inside_the_template_island() {
        for value in [
            "px-2 <?php /* ?> */ echo $class; ?> py-1",
            "px-2 <?= /* ?> */ $class ?> py-1",
        ] {
            let spans = sortable_spans(value, SourceLanguage::Php).unwrap();

            assert_eq!(span_texts(value, &spans), ["px-2", "py-1"]);
        }
        assert_eq!(
            sortable_spans("px-2 <?php /* ?>", SourceLanguage::Php),
            None
        );
    }

    #[test]
    fn php_line_comment_closers_keep_first_closer_behavior() {
        for value in ["<?php // ?> later ?>", "<?php # ?> later ?>"] {
            assert_eq!(
                template_island_end_at(value, 0, SourceLanguage::Php),
                TemplateIslandEnd::Closed(value.find(" later").unwrap())
            );
        }
    }

    #[test]
    fn default_delimited_languages_keep_quote_shielding_and_first_closer_behavior() {
        let quoted = r#"<%= "not %> yet" %> tail"#;
        let unshielded_comment = "<% /* %> tail";

        assert_eq!(
            template_island_end_at(quoted, 0, SourceLanguage::Erb),
            TemplateIslandEnd::Closed(quoted.find(" tail").unwrap())
        );
        assert_eq!(
            template_island_end_at(unshielded_comment, 0, SourceLanguage::Ejs),
            TemplateIslandEnd::Closed(unshielded_comment.find(" tail").unwrap())
        );
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
    fn escaped_blade_literal_before_directives_keeps_independent_static_runs() {
        let value = "@@literal p-2 @if($visible) flex @endif text-sm";
        let spans = sortable_spans(value, SourceLanguage::Blade).unwrap();

        assert_eq!(
            span_texts(value, &spans),
            ["@@literal p-2", "flex", "text-sm"]
        );
    }

    #[test]
    fn invalid_blade_at_signs_do_not_hide_later_directives() {
        let value = "p-2 @ @1 @- text-sm @if($visible) flex @endif block @";
        let spans = sortable_spans(value, SourceLanguage::Blade).unwrap();

        assert_eq!(
            span_texts(value, &spans),
            ["p-2 @ @1 @- text-sm", "flex", "block @"]
        );
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

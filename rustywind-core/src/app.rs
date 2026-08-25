use std::{borrow::Cow, fmt, ops::Range};

use crate::{
    attribute_parser::{ClassAttribute, class_attributes},
    class_wrapping::ClassWrapping,
    consts::{VARIANT_SEARCHER, VARIANTS},
    hybrid_sorter::HybridSorter,
    markup_parser::is_attribute_name_character,
    sorter::{FinderRegex, Sorter},
    source::{
        ClassValueAnalysis, SourceDocument, SourceLanguage, StaticRuns, analyze_class_value,
        is_plain_class_list,
    },
    tailwind_prefix::{normalize_tailwind_prefix, normalize_tailwind_prefix_value},
};
use ahash::{AHashMap as HashMap, AHashSet as HashSet};
use aho_corasick::{Anchored, Input};
use regex::{Captures, Match, Regex};
use std::sync::{Arc, LazyLock, RwLock};

/// Global instance of the HybridSorter for pattern-based sorting
static PATTERN_SORTER: LazyLock<HybridSorter> = LazyLock::new(HybridSorter::new);
static PREFIXED_PATTERN_SORTERS: LazyLock<RwLock<HashMap<String, Arc<HybridSorter>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

struct SortCandidate<'a> {
    original: &'a str,
    lookup: Cow<'a, str>,
}

struct SortableCapture<'a> {
    full_match: Match<'a>,
    classes_match: Match<'a>,
    value: SortableClassValue<'a>,
}

enum SortableClassValue<'a> {
    Static(StaticRuns),
    Wrapped(WrappedClassList<'a>),
}

/// Result of sorting a source document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortReport<'a> {
    /// Sorted source contents
    pub contents: Cow<'a, str>,
    /// Number of class groups whose final text changed
    pub changed_class_groups: usize,
}

/// A validated comma-separated list of quoted static class tokens
#[derive(Debug, Clone, PartialEq, Eq)]
struct WrappedClassList<'a> {
    quote: WrappedQuote,
    classes: Vec<&'a str>,
}

impl<'a> WrappedClassList<'a> {
    fn parse(value: &'a str, wrapping: ClassWrapping) -> Option<Self> {
        let quote = WrappedQuote::from_wrapping(wrapping)?;
        let mut cursor = skip_ascii_whitespace(value, 0);
        let mut classes = Vec::new();

        loop {
            let (class, next) = parse_wrapped_class(value, cursor, quote.as_char())?;
            classes.push(class);
            cursor = skip_ascii_whitespace(value, next);

            if cursor == value.len() {
                return Some(Self { quote, classes });
            }
            if value.as_bytes().get(cursor) != Some(&b',') {
                return None;
            }

            cursor = skip_ascii_whitespace(value, cursor + 1);
            if cursor == value.len() {
                return None;
            }
        }
    }

    fn iter(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.classes.iter().copied()
    }

    fn render(&self, classes: &[&str]) -> String {
        let quote = self.quote.as_char();
        classes
            .iter()
            .map(|class| format!("{quote}{class}{quote}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WrappedQuote {
    Single,
    Double,
}

impl WrappedQuote {
    fn from_wrapping(wrapping: ClassWrapping) -> Option<Self> {
        match wrapping {
            ClassWrapping::NoWrapping => None,
            ClassWrapping::CommaSingleQuotes => Some(Self::Single),
            ClassWrapping::CommaDoubleQuotes => Some(Self::Double),
        }
    }

    const fn as_char(self) -> char {
        match self {
            Self::Single => '\'',
            Self::Double => '"',
        }
    }
}

/// A validated whitespace-separated list containing only static class tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlainClassList<'a>(&'a str);

impl<'a> PlainClassList<'a> {
    /// Parses a static whitespace-separated class list
    ///
    /// Template delimiters outside Tailwind arbitrary-value brackets are
    /// rejected so program source cannot enter the direct class-list sorter
    pub fn parse(value: &'a str) -> Result<Self, InvalidClassList> {
        if is_plain_class_list(value) {
            Ok(Self(value))
        } else {
            Err(InvalidClassList)
        }
    }

    /// Returns the validated class-list source
    pub fn as_str(self) -> &'a str {
        self.0
    }
}

/// Error returned when a direct class list contains source-language syntax
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidClassList;

impl fmt::Display for InvalidClassList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("class list contains template syntax or unbalanced brackets")
    }
}

impl std::error::Error for InvalidClassList {}

/// The options to pass to the sorter
#[derive(Debug, Clone)]
pub struct RustyWind {
    /// Attribute extractor used to find candidate class values
    pub regex: FinderRegex,
    /// Class ordering strategy
    pub sorter: Sorter,
    /// Whether repeated classes are retained within a static run
    pub allow_duplicates: bool,
    /// Encoding used by custom extracted class lists
    pub class_wrapping: ClassWrapping,
    /// Tailwind prefix normalized while computing sort order
    pub tailwind_prefix: Option<String>,
    /// Preserve the original whitespace around classes when sorting
    pub preserve_whitespace: bool,
}

impl Default for RustyWind {
    fn default() -> Self {
        Self {
            regex: FinderRegex::DefaultRegex,
            sorter: Sorter::PatternSorter,
            allow_duplicates: false,
            class_wrapping: ClassWrapping::NoWrapping,
            tailwind_prefix: None,
            preserve_whitespace: false,
        }
    }
}

impl RustyWind {
    /// Creates a sorter without a Tailwind prefix
    pub fn new(
        regex: FinderRegex,
        sorter: Sorter,
        allow_duplicates: bool,
        class_wrapping: ClassWrapping,
    ) -> Self {
        Self::new_with_tailwind_prefix(regex, sorter, allow_duplicates, class_wrapping, None)
    }

    /// Creates a sorter with an optional Tailwind prefix
    pub fn new_with_tailwind_prefix(
        regex: FinderRegex,
        sorter: Sorter,
        allow_duplicates: bool,
        class_wrapping: ClassWrapping,
        tailwind_prefix: Option<String>,
    ) -> Self {
        Self {
            regex,
            sorter,
            allow_duplicates,
            class_wrapping,
            tailwind_prefix,
            preserve_whitespace: false,
        }
    }

    /// Checks whether a source document contains a sortable static class run
    pub fn has_classes(&self, document: SourceDocument<'_>) -> bool {
        if matches!(self.regex, FinderRegex::DefaultRegex)
            && document.language().attribute_parser_profile().is_some()
        {
            return class_attributes(document).is_some_and(|attributes| {
                attributes
                    .iter()
                    .any(|attribute| self.sortable_attribute(attribute, document).is_some())
            });
        }

        self.extraction_regex()
            .captures_iter(document.text())
            .any(|captures| self.sortable_capture(&captures, document).is_some())
    }

    /// Sorts proven-static class runs in a language-aware source document
    ///
    /// Embedded expressions are preserved byte-for-byte, and sorting or
    /// deduplication never crosses an expression boundary
    pub fn sort_document<'a>(&self, document: SourceDocument<'a>) -> Cow<'a, str> {
        self.sort_document_with_report(document).contents
    }

    /// Sorts a language-aware source document and reports changed class groups
    ///
    /// A class group is one structured attribute value or one regular-expression
    /// capture, regardless of how many static runs it contains
    pub fn sort_document_with_report<'a>(&self, document: SourceDocument<'a>) -> SortReport<'a> {
        if matches!(self.regex, FinderRegex::DefaultRegex)
            && document.language().attribute_parser_profile().is_some()
        {
            return self.sort_structured_document(document);
        }

        let mut changed_class_groups = 0;
        let contents =
            self.extraction_regex()
                .replace_all(document.text(), |captures: &Captures| {
                    let Some(capture) = self.sortable_capture(captures, document) else {
                        return captures[0].to_string();
                    };

                    let sorted_classes =
                        self.sort_source_value(capture.classes_match.as_str(), &capture.value);
                    if sorted_classes != capture.classes_match.as_str() {
                        changed_class_groups += 1;
                    }
                    splice_capture(
                        capture.full_match.as_str(),
                        &capture.full_match,
                        &capture.classes_match,
                        &sorted_classes,
                    )
                });

        SortReport {
            contents,
            changed_class_groups,
        }
    }

    /// Sorts a validated static class list and normalizes its whitespace
    pub fn sort_class_list(&self, class_list: PlainClassList<'_>) -> String {
        self.sort_class_run(class_list.as_str())
    }

    /// Sorts a bare class attribute value with template awareness
    ///
    /// Handles one value like [`RustyWind::sort_document`] handles the
    /// attribute values it discovers itself, for tools that locate class
    /// attributes with their own parser
    pub fn sort_class_value<'a>(&self, value: &'a str, language: SourceLanguage) -> Cow<'a, str> {
        match self.sortable_class_value(value, language) {
            Some(sortable) => self.sort_source_value(value, &sortable),
            None => Cow::Borrowed(value),
        }
    }

    fn extraction_regex(&self) -> &Regex {
        &self.regex
    }

    fn sortable_class_value<'a>(
        &self,
        value: &'a str,
        language: SourceLanguage,
    ) -> Option<SortableClassValue<'a>> {
        if matches!(self.class_wrapping, ClassWrapping::NoWrapping) {
            match analyze_class_value(value, language) {
                ClassValueAnalysis::Sortable(runs) => Some(SortableClassValue::Static(runs)),
                ClassValueAnalysis::Opaque => None,
            }
        } else {
            WrappedClassList::parse(value, self.class_wrapping).map(SortableClassValue::Wrapped)
        }
    }

    fn sortable_capture<'a>(
        &self,
        captures: &'a Captures<'a>,
        document: SourceDocument<'a>,
    ) -> Option<SortableCapture<'a>> {
        let full_match = captures.get(0)?;

        if matches!(self.regex, FinderRegex::DefaultRegex)
            && has_attribute_name_prefix(document.text(), full_match.start())
        {
            return None;
        }

        let classes_match = match &self.regex {
            FinderRegex::DefaultRegex => captures.get(1).or_else(|| captures.get(2))?,
            FinderRegex::CustomRegex(extractor) => extractor.classes(captures)?,
        };

        let value = self.sortable_class_value(classes_match.as_str(), document.language())?;

        Some(SortableCapture {
            full_match,
            classes_match,
            value,
        })
    }

    fn sortable_attribute<'a>(
        &self,
        attribute: &ClassAttribute,
        document: SourceDocument<'a>,
    ) -> Option<(Range<usize>, SortableClassValue<'a>)> {
        let range = attribute.value_range();
        let value =
            self.sortable_class_value(&document.text()[range.clone()], document.language())?;

        Some((range, value))
    }

    fn sort_structured_document<'a>(&self, document: SourceDocument<'a>) -> SortReport<'a> {
        let Some(attributes) = class_attributes(document) else {
            return SortReport {
                contents: Cow::Borrowed(document.text()),
                changed_class_groups: 0,
            };
        };
        let sortable = attributes
            .iter()
            .filter_map(|attribute| self.sortable_attribute(attribute, document))
            .collect::<Vec<_>>();
        if sortable.is_empty() {
            return SortReport {
                contents: Cow::Borrowed(document.text()),
                changed_class_groups: 0,
            };
        }

        let mut output = document.text().to_string();
        let mut changed_class_groups = 0;
        for (range, value) in sortable.into_iter().rev() {
            let original = &document.text()[range.clone()];
            let sorted = self.sort_source_value(original, &value);
            if sorted != original {
                output.replace_range(range, &sorted);
                changed_class_groups += 1;
            }
        }

        let contents = if changed_class_groups > 0 {
            Cow::Owned(output)
        } else {
            Cow::Borrowed(document.text())
        };

        SortReport {
            contents,
            changed_class_groups,
        }
    }

    fn sort_source_value<'a>(
        &self,
        value: &'a str,
        class_value: &SortableClassValue<'a>,
    ) -> Cow<'a, str> {
        let runs = match class_value {
            SortableClassValue::Static(runs) => runs,
            SortableClassValue::Wrapped(classes) => {
                return Cow::Owned(self.sort_wrapped_classes(classes));
            }
        };

        let mut sorted_value = value.to_string();
        let mut changed = false;

        for span in runs.iter().rev() {
            let original_run = &value[span.clone()];
            let sorted_run = self.sort_class_run(original_run);
            if sorted_run != original_run {
                sorted_value.replace_range(span.clone(), &sorted_run);
                changed = true;
            }
        }

        if changed {
            Cow::Owned(sorted_value)
        } else {
            Cow::Borrowed(value)
        }
    }

    fn sort_class_run(&self, class_string: &str) -> String {
        let tokens = split_class_tokens(class_string);
        let mut sorted = self.sort_classes_vec(tokens.iter().copied());

        if !self.allow_duplicates {
            deduplicate_classes(&mut sorted);
        }

        if self.preserve_whitespace {
            interleave_separators(class_string, &tokens, &sorted)
        } else {
            sorted.join(" ")
        }
    }

    fn sort_wrapped_classes(&self, class_list: &WrappedClassList<'_>) -> String {
        let mut sorted = self.sort_classes_vec(class_list.iter());
        if !self.allow_duplicates {
            deduplicate_classes(&mut sorted);
        }

        class_list.render(&sorted)
    }

    fn sort_classes_vec<'a>(&self, classes: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
        // use pattern-based sorting if PatternSorter is selected
        if matches!(self.sorter, Sorter::PatternSorter) {
            let classes_vec: Vec<&str> = classes.collect();
            if let Some(tailwind_prefix) = self
                .tailwind_prefix
                .as_deref()
                .and_then(normalize_tailwind_prefix_value)
            {
                return prefixed_pattern_sorter(tailwind_prefix).sort_classes(&classes_vec);
            }
            return PATTERN_SORTER.sort_classes(&classes_vec);
        }

        // otherwise, use the old HashMap-based approach
        let candidates = classes.map(|class| SortCandidate {
            original: class,
            lookup: normalize_tailwind_prefix(class, self.tailwind_prefix.as_deref()),
        });

        let mut tailwind_classes: Vec<(&str, &usize)> = vec![];
        let mut custom_classes: Vec<&str> = vec![];
        let mut variants: HashMap<&str, Vec<SortCandidate>> = HashMap::new();

        for candidate in candidates {
            match self
                .sorter
                .get(candidate.original)
                .or_else(|| self.sorter.get(candidate.lookup.as_ref()))
            {
                Some(size) => tailwind_classes.push((candidate.original, size)),
                None => {
                    let lookup = candidate.lookup.as_ref();
                    let input = Input::new(lookup).anchored(Anchored::Yes);
                    match VARIANT_SEARCHER.find(input).filter(|prefix_match| {
                        lookup.as_bytes().get(prefix_match.end()) == Some(&b':')
                    }) {
                        Some(prefix_match) => {
                            let prefix = VARIANTS[prefix_match.pattern()];
                            variants.entry(prefix).or_default().push(candidate)
                        }
                        None => custom_classes.push(candidate.original),
                    }
                }
            }
        }

        tailwind_classes.sort_by_key(|&(_class, class_placement)| class_placement);

        let sorted_tailwind_classes: Vec<&str> = tailwind_classes
            .iter()
            .map(|(class, _index)| *class)
            .collect();

        let mut sorted_variant_classes = vec![];

        for key in VARIANTS.iter() {
            let (mut sorted_classes, new_custom_classes) = self.sort_variant_classes(
                variants.remove(key).unwrap_or_default(),
                custom_classes,
                key.len() + 1,
            );

            sorted_variant_classes.append(&mut sorted_classes);
            custom_classes = new_custom_classes
        }

        [
            &sorted_tailwind_classes[..],
            &sorted_variant_classes[..],
            &custom_classes[..],
        ]
        .concat()
    }

    fn sort_variant_classes<'a>(
        &self,
        classes: Vec<SortCandidate<'a>>,
        mut custom_classes: Vec<&'a str>,
        class_after: usize,
    ) -> (Vec<&'a str>, Vec<&'a str>) {
        let mut tailwind_classes = Vec::with_capacity(classes.len());

        for candidate in classes {
            let normalized_remainder = candidate.lookup.get(class_after..);
            let v4_original_remainder = self
                .tailwind_prefix
                .as_deref()
                .and_then(normalize_tailwind_prefix_value)
                .and_then(|prefix| {
                    candidate
                        .original
                        .strip_prefix(prefix)
                        .and_then(|rest| rest.strip_prefix(':'))?;
                    normalized_remainder
                        .map(|normalized_remainder| format!("{prefix}:{normalized_remainder}"))
                });

            match candidate
                .original
                .get(class_after..)
                .and_then(|class| self.sorter.get(class))
                .or_else(|| {
                    v4_original_remainder
                        .as_deref()
                        .and_then(|class| self.sorter.get(class))
                })
                .or_else(|| normalized_remainder.and_then(|class| self.sorter.get(class)))
            {
                Some(class_placement) => {
                    tailwind_classes.push((candidate.original, class_placement))
                }
                None => custom_classes.push(candidate.original),
            }
        }

        tailwind_classes.sort_by_key(|&(_class, class_placement)| class_placement);

        let sorted_classes = tailwind_classes
            .iter()
            .map(|(class, _index)| *class)
            .collect();

        (sorted_classes, custom_classes)
    }
}

fn prefixed_pattern_sorter(tailwind_prefix: &str) -> Arc<HybridSorter> {
    if let Some(sorter) = PREFIXED_PATTERN_SORTERS
        .read()
        .expect("prefixed pattern sorter cache should not be poisoned")
        .get(tailwind_prefix)
    {
        return Arc::clone(sorter);
    }

    let mut sorters = PREFIXED_PATTERN_SORTERS
        .write()
        .expect("prefixed pattern sorter cache should not be poisoned");

    Arc::clone(
        sorters
            .entry(tailwind_prefix.to_string())
            .or_insert_with(|| {
                Arc::new(HybridSorter::new_with_tailwind_prefix(Some(
                    tailwind_prefix,
                )))
            }),
    )
}

fn has_attribute_name_prefix(source: &str, match_start: usize) -> bool {
    source[..match_start]
        .chars()
        .next_back()
        .is_some_and(is_attribute_name_character)
}

fn parse_wrapped_class(value: &str, start: usize, quote: char) -> Option<(&str, usize)> {
    if value[start..].chars().next()? != quote {
        return None;
    }

    let class_start = start + quote.len_utf8();
    let mut escaped = false;
    for (relative, character) in value[class_start..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character != quote {
            continue;
        }

        let class_end = class_start + relative;
        let class = &value[class_start..class_end];
        if !is_plain_class_list(class) || split_class_tokens(class).as_slice() != [class] {
            return None;
        }
        return Some((class, class_end + quote.len_utf8()));
    }

    None
}

fn skip_ascii_whitespace(value: &str, mut cursor: usize) -> usize {
    while value
        .as_bytes()
        .get(cursor)
        .is_some_and(u8::is_ascii_whitespace)
    {
        cursor += 1;
    }
    cursor
}

fn splice_capture(
    full_source: &str,
    full_match: &regex::Match<'_>,
    classes_match: &regex::Match<'_>,
    replacement: &str,
) -> String {
    let class_start = classes_match.start() - full_match.start();
    let class_end = classes_match.end() - full_match.start();
    let mut output =
        String::with_capacity(full_source.len() - classes_match.as_str().len() + replacement.len());
    output.push_str(&full_source[..class_start]);
    output.push_str(replacement);
    output.push_str(&full_source[class_end..]);
    output
}

fn split_class_tokens(class_string: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut bracket_depth: u32 = 0;
    let mut bracket_quote = None;
    let mut escaped = false;

    for (index, character) in class_string.char_indices() {
        if bracket_depth > 0 {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if let Some(active_quote) = bracket_quote {
                if character == active_quote {
                    bracket_quote = None;
                }
            } else if matches!(character, '\'' | '"' | '`') {
                bracket_quote = Some(character);
            } else if character == '[' {
                bracket_depth += 1;
            } else if character == ']' {
                bracket_depth -= 1;
            }
        } else if character == '[' {
            bracket_depth = 1;
        }

        if character.is_ascii_whitespace() && bracket_depth == 0 {
            if let Some(token_start) = start.take() {
                tokens.push(&class_string[token_start..index]);
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }

    if let Some(token_start) = start {
        tokens.push(&class_string[token_start..]);
    }

    tokens
}

/// Join the sorted classes of a run, reusing the original whitespace instead of single spaces.
///
/// Whitespace is reused positionally: the Nth sorted class is preceded by the whitespace that preceded
/// the Nth original class, and the trailing whitespace is kept as-is. A class list spread over several
/// indented lines thus keeps its exact line structure — only the class names move between the slots.
///
/// When deduplication dropped classes, `sorted` is shorter than `tokens` and the surplus slots are
/// dropped along with their separators.
fn interleave_separators(original: &str, tokens: &[&str], sorted: &[&str]) -> String {
    // `tokens` are subslices of `original`, so an offset marks the end of the preceding whitespace.
    let offset = |token: &str| token.as_ptr() as usize - original.as_ptr() as usize;
    debug_assert!(sorted.len() <= tokens.len());

    // The output is a subset of the original separators and tokens, so this capacity never reallocates.
    let mut out = String::with_capacity(original.len());
    let mut pos = 0;
    let mut replacements = sorted.iter();
    for token in tokens {
        if let Some(replacement) = replacements.next() {
            out.push_str(&original[pos..offset(token)]);
            out.push_str(replacement);
        }
        pos = offset(token) + token.len();
    }
    out.push_str(&original[pos..]);
    out
}

fn deduplicate_classes(classes: &mut Vec<&str>) {
    let mut seen = HashSet::new();
    classes.retain(|class| is_ellipsis_placeholder(class) || seen.insert(*class));
}

fn is_ellipsis_placeholder(class: &str) -> bool {
    class == "..." || class == "…"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceLanguage;
    use pretty_assertions::assert_eq;
    use regex::Regex;
    use test_case::test_case;
    const RUSTYWIND_DEFAULT: RustyWind = RustyWind {
        regex: FinderRegex::DefaultRegex,
        sorter: Sorter::PatternSorter,
        allow_duplicates: false,
        class_wrapping: ClassWrapping::NoWrapping,
        tailwind_prefix: None,
        preserve_whitespace: false,
    };

    trait TestRustyWindExt {
        fn sort_file_contents<'a>(&self, input: &'a str) -> Cow<'a, str>;
        fn sort_classes(&self, input: &str) -> String;
    }

    impl TestRustyWindExt for RustyWind {
        fn sort_file_contents<'a>(&self, input: &'a str) -> Cow<'a, str> {
            self.sort_document(SourceDocument::new(input, SourceLanguage::Html))
        }

        fn sort_classes(&self, input: &str) -> String {
            self.sort_class_list(PlainClassList::parse(input).unwrap())
        }
    }

    // HAS_CLASSES --------------------------------------------------------------------------------
    #[test_case( r#"<div class="flex-col inline flex"></div>"#, true ; "div tag with class")]
    #[test_case( r#"<body class="unknown-class"></body>"#, true ; "body tag with unknown class")]
    #[test_case( r#"<p className="unknown-class"></p>"#, true ; "p tag with unknown class")]
    #[test_case( r#"<p>not a class</p>"#, false ; "p tag with no class")]
    #[test_case( r#"<div><p></p><p></p></div>"#, false ; "nested tags, no class")]
    #[test_case( r#"<div><p><span className="inline"></span></p><p></p></div>"#, true ; "nested tags, class in child")]
    fn test_has_classes(input: &str, output: bool) {
        assert_eq!(
            RUSTYWIND_DEFAULT.has_classes(SourceDocument::new(input, SourceLanguage::Html)),
            output
        );
    }

    #[test]
    fn sorts_every_attribute_in_a_document_with_many_tags() {
        let source = (0..2_000)
            .map(|_| r#"<div class="p-4 m-4 flex"></div>"#)
            .collect::<String>();
        let result = RUSTYWIND_DEFAULT
            .sort_document(SourceDocument::new(&source, SourceLanguage::Html))
            .into_owned();

        assert_eq!(result.matches(r#"class="m-4 flex p-4""#).count(), 2_000);
    }

    #[test]
    fn legacy_positional_custom_regex_still_sorts_classes() {
        let extractor =
            crate::sorter::CustomClassExtractor::new(Regex::new(r#"class="([^"]+)""#).unwrap())
                .unwrap();
        let app = RustyWind {
            regex: FinderRegex::CustomRegex(extractor),
            ..RUSTYWIND_DEFAULT
        };
        let source = r#"<div class="p-4 m-4 flex"></div>"#;

        assert_eq!(
            app.sort_document(SourceDocument::new(source, SourceLanguage::Unknown)),
            r#"<div class="m-4 flex p-4"></div>"#
        );
    }

    #[test]
    fn report_keeps_unchanged_content_borrowed() {
        let source = r#"<div class="m-4 flex p-4"></div>"#;
        let report = RUSTYWIND_DEFAULT
            .sort_document_with_report(SourceDocument::new(source, SourceLanguage::Html));

        assert!(matches!(report.contents, Cow::Borrowed(_)));
        assert_eq!(report.contents, source);
        assert_eq!(report.changed_class_groups, 0);
    }

    #[test]
    fn report_counts_one_changed_structured_attribute() {
        let source = r#"<div class="p-4 m-4 flex"></div>"#;
        let report = RUSTYWIND_DEFAULT
            .sort_document_with_report(SourceDocument::new(source, SourceLanguage::Html));

        assert_eq!(report.contents, r#"<div class="m-4 flex p-4"></div>"#);
        assert_eq!(report.changed_class_groups, 1);
    }

    #[test]
    fn report_counts_multiple_changed_structured_attributes() {
        let source = r#"<div class="p-4 m-4"></div><span className="p-2 m-2"></span>"#;
        let report = RUSTYWIND_DEFAULT
            .sort_document_with_report(SourceDocument::new(source, SourceLanguage::Html));

        assert_eq!(report.changed_class_groups, 2);
    }

    #[test]
    fn report_counts_duplicate_removal_without_reordering() {
        let source = r#"<div class="flex flex"></div>"#;
        let report = RUSTYWIND_DEFAULT
            .sort_document_with_report(SourceDocument::new(source, SourceLanguage::Html));

        assert_eq!(report.contents, r#"<div class="flex"></div>"#);
        assert_eq!(report.changed_class_groups, 1);
    }

    #[test]
    fn report_counts_whitespace_normalization() {
        let source = r#"<div class="  flex   p-4  "></div>"#;
        let report = RUSTYWIND_DEFAULT
            .sort_document_with_report(SourceDocument::new(source, SourceLanguage::Html));

        assert_eq!(report.contents, r#"<div class="flex p-4"></div>"#);
        assert_eq!(report.changed_class_groups, 1);
    }

    #[test]
    fn report_counts_changed_template_static_runs_as_one_group() {
        let source = r#"<div class="p-4 m-4 {active} p-2 m-2"></div>"#;
        let report = RUSTYWIND_DEFAULT
            .sort_document_with_report(SourceDocument::new(source, SourceLanguage::Svelte));

        assert_eq!(
            report.contents,
            r#"<div class="m-4 p-4 {active} m-2 p-2"></div>"#
        );
        assert_eq!(report.changed_class_groups, 1);
    }

    #[test]
    fn report_does_not_count_opaque_values() {
        let source = r#"<div class="p-4 m-4 {missing"></div>"#;
        let report = RUSTYWIND_DEFAULT
            .sort_document_with_report(SourceDocument::new(source, SourceLanguage::Svelte));

        assert_eq!(report.contents, source);
        assert_eq!(report.changed_class_groups, 0);
    }

    #[test]
    fn report_uses_structured_counting_for_jsx() {
        let source = r#"<div className="p-4 m-4" />"#;
        let report = RUSTYWIND_DEFAULT
            .sort_document_with_report(SourceDocument::new(source, SourceLanguage::Jsx));

        assert_eq!(report.contents, r#"<div className="m-4 p-4" />"#);
        assert_eq!(report.changed_class_groups, 1);
    }

    #[test]
    fn report_uses_regex_counting_for_profileless_languages() {
        let source = r#"<div class="p-4 m-4"></div>"#;
        let report = RUSTYWIND_DEFAULT
            .sort_document_with_report(SourceDocument::new(source, SourceLanguage::Unknown));

        assert_eq!(report.contents, r#"<div class="m-4 p-4"></div>"#);
        assert_eq!(report.changed_class_groups, 1);
    }

    #[test]
    fn report_uses_regex_counting_for_wrapped_custom_captures() {
        let extractor = crate::sorter::CustomClassExtractor::new(
            Regex::new(r#"classes\((?P<classes>[^)]*)\)"#).unwrap(),
        )
        .unwrap();
        let app = RustyWind {
            regex: FinderRegex::CustomRegex(extractor),
            class_wrapping: ClassWrapping::CommaSingleQuotes,
            ..RUSTYWIND_DEFAULT
        };
        let source = "classes('p-4', 'm-4')";
        let report =
            app.sort_document_with_report(SourceDocument::new(source, SourceLanguage::Unknown));

        assert_eq!(report.contents, "classes('m-4', 'p-4')");
        assert_eq!(report.changed_class_groups, 1);
    }

    #[test_case("data-class" ; "data attribute")]
    #[test_case("x-class" ; "x attribute")]
    #[test_case(":class" ; "colon directive")]
    #[test_case(".class" ; "dot directive")]
    #[test_case("@class" ; "event directive")]
    #[test_case("[class]" ; "bracket binding")]
    fn default_regex_rejects_class_suffixes_in_attribute_names(attribute_name: &str) {
        let source = format!(r#"<div {attribute_name}="p-4 m-4"></div>"#);
        let document = SourceDocument::new(&source, SourceLanguage::Unknown);

        assert!(!RUSTYWIND_DEFAULT.has_classes(document));
        assert_eq!(RUSTYWIND_DEFAULT.sort_document(document), source);
    }

    #[test]
    fn default_regex_accepts_a_standalone_class_attribute() {
        let source = r#"<div class="p-4 m-4">"#;
        let document = SourceDocument::new(source, SourceLanguage::Unknown);

        assert!(RUSTYWIND_DEFAULT.has_classes(document));
        assert_eq!(
            RUSTYWIND_DEFAULT.sort_document(document),
            r#"<div class="m-4 p-4">"#
        );
    }

    // SORT_CLASSES_VEC ---------------------------------------------------------------------------
    // Note: Removed old static-list ordering tests. Pattern-based sorting follows
    // Tailwind v4's canonical property order, tested in integration_tests.rs

    // SORT_FILE_CONTENTS -------------------------------------------------------------------------
    // test behavioral properties, not exact ordering (which is tested in integration_tests.rs)

    #[test]
    fn test_deduplicates_classes() {
        let input =
            r#"<p className="py-2 py-2 random-class underline underline underline">text</p>"#;
        let result = RUSTYWIND_DEFAULT.sort_file_contents(input);

        // should have only one py-2 and one underline
        assert_eq!(result.matches("py-2").count(), 1);
        assert_eq!(result.matches("underline").count(), 1);
    }

    #[test]
    fn test_keeps_duplicates_when_configured() {
        let app = RustyWind {
            allow_duplicates: true,
            ..RUSTYWIND_DEFAULT
        };
        let input =
            r#"<section className="inline py-2 py-2 random-class italic italic italic"></section>"#;
        let result = app.sort_file_contents(input);

        // should have two py-2 and three italic
        assert_eq!(result.matches("py-2").count(), 2);
        assert_eq!(result.matches("italic").count(), 3);
    }

    #[test]
    fn test_pattern_sorter_removes_duplicates_by_default() {
        // test that PatternSorter (default) removes duplicates when allow_duplicates=false
        // this ensures the fast path doesn't bypass deduplication logic
        let app = RustyWind {
            sorter: Sorter::PatternSorter,
            allow_duplicates: false,
            ..RUSTYWIND_DEFAULT
        };

        // test case from the issue description
        let input = r#"<div class="flex flex"></div>"#;
        let result = app.sort_file_contents(input);

        // should collapse to single flex
        assert_eq!(
            result.matches("flex").count(),
            1,
            "Duplicates should be removed with PatternSorter"
        );
        assert_eq!(result, r#"<div class="flex"></div>"#);

        // test with more duplicates
        let input2 = r#"<div class="m-4 p-4 m-4 flex p-4 flex m-4"></div>"#;
        let result2 = app.sort_file_contents(input2);
        assert_eq!(
            result2.matches("m-4").count(),
            1,
            "All m-4 duplicates should be removed"
        );
        assert_eq!(
            result2.matches("p-4").count(),
            1,
            "All p-4 duplicates should be removed"
        );
        assert_eq!(
            result2.matches("flex").count(),
            1,
            "All flex duplicates should be removed"
        );
    }

    #[test]
    fn test_keeps_duplicate_ellipsis_placeholders() {
        let input = r#"<div className="transition ... ... flex"></div>"#;
        let result = RUSTYWIND_DEFAULT.sort_file_contents(input);

        assert_eq!(result.matches("...").count(), 2);
    }

    #[test]
    fn test_pattern_sorter_keeps_duplicates_when_configured() {
        // test that allow_duplicates=true works with PatternSorter
        let app = RustyWind {
            sorter: Sorter::PatternSorter,
            allow_duplicates: true,
            regex: FinderRegex::DefaultRegex,
            class_wrapping: ClassWrapping::NoWrapping,
            tailwind_prefix: None,
            preserve_whitespace: false,
        };

        let input = r#"<div class="flex flex m-4 m-4"></div>"#;
        let result = app.sort_file_contents(input);

        // should keep all duplicates
        assert_eq!(
            result.matches("flex").count(),
            2,
            "Duplicates should be kept when allow_duplicates=true"
        );
        assert_eq!(
            result.matches("m-4").count(),
            2,
            "Duplicates should be kept when allow_duplicates=true"
        );
    }

    #[test]
    fn test_base_classes_before_variants() {
        let input = r#"<div class='hover:flex focus:flex flex'></div>"#;
        let result = RUSTYWIND_DEFAULT.sort_file_contents(input);

        // extract the class content
        let class_content = result
            .split("class='")
            .nth(1)
            .unwrap()
            .split('\'')
            .next()
            .unwrap();
        let classes: Vec<&str> = class_content.split_whitespace().collect();

        // flex (base) should come before all variants
        let flex_idx = classes.iter().position(|&c| c == "flex").unwrap();
        let hover_idx = classes.iter().position(|&c| c == "hover:flex").unwrap();
        let focus_idx = classes.iter().position(|&c| c == "focus:flex").unwrap();

        assert!(
            flex_idx < hover_idx,
            "Base 'flex' should come before 'hover:flex'"
        );
        assert!(
            flex_idx < focus_idx,
            "Base 'flex' should come before 'focus:flex'"
        );
    }

    #[test]
    fn test_multiline_gets_flattened() {
        let input = r#"
            <div
              class="
                flex
                p-4
                m-4
              "
            >
            </div>
        "#;
        let result = RUSTYWIND_DEFAULT.sort_file_contents(input);

        // should be on one line
        let class_content = result
            .split("class=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert!(!class_content.contains('\n'));
    }

    #[test_case(
        &RUSTYWIND_DEFAULT,
        r#"This is to represent any other normal file."#,
        r#"This is to represent any other normal file."#,
        SourceLanguage::Html
        ; "makes no change to files without class string"
    )]
    #[test_case(
        &RUSTYWIND_DEFAULT,
        r#"<div><p><img height="100" width="250" /></p><p></p></div>"#,
        r#"<div><p><img height="100" width="250" /></p><p></p></div>"#,
        SourceLanguage::Html
        ; "makes no change to elements without class string"
    )]
    #[test_case(
        &RUSTYWIND_DEFAULT,
        r#"<div class="<%= layout == :cards ? 'flex flex-col gap-3 sm:flex-row sm:justify-between sm:items-center' : 'sm:flex sm:items-center' %>">"#,
        r#"<div class="<%= layout == :cards ? 'flex flex-col gap-3 sm:flex-row sm:justify-between sm:items-center' : 'sm:flex sm:items-center' %>">"#,
        SourceLanguage::Erb
        ; "makes no change to class string that is a single erb ternary"
    )]
    #[test_case(
        &RUSTYWIND_DEFAULT,
        r#"<span class="inline-flex items-center <%= data["company"].present? ? 'h-auto py-2 px-3 gap-x-1.5' : 'h-8 py-1 px-2 gap-x-0.5' %> rounded-md">"#,
        r#"<span class="inline-flex items-center <%= data["company"].present? ? 'h-auto py-2 px-3 gap-x-1.5' : 'h-8 py-1 px-2 gap-x-0.5' %> rounded-md">"#,
        SourceLanguage::Erb
        ; "makes no change to class string with erb tag between static classes"
    )]
    #[test_case(
        &RUSTYWIND_DEFAULT,
        r#"<span class="flex h-8 w-8 items-center justify-center rounded-full <%= record.active? ? 'bg-brand-yellow text-gray-950' : 'border-2 border-gray-300 bg-white' %>">"#,
        r#"<span class="flex h-8 w-8 items-center justify-center rounded-full <%= record.active? ? 'bg-brand-yellow text-gray-950' : 'border-2 border-gray-300 bg-white' %>">"#,
        SourceLanguage::Erb
        ; "makes no change to class string with static classes before an erb ternary"
    )]
    #[test_case(
        &RUSTYWIND_DEFAULT,
        r##"<div class="box f-col #{reply.reply_id ? "ok" : ""}">"##,
        r##"<div class="box f-col #{reply.reply_id ? "ok" : ""}">"##,
        SourceLanguage::Ruby
        ; "makes no change to class string with ruby string interpolation"
    )]
    #[test_case(
        &RUSTYWIND_DEFAULT,
        r#"<div class="p-4 {{ active ? 'flex flex-col' : 'hidden' }}">"#,
        r#"<div class="p-4 {{ active ? 'flex flex-col' : 'hidden' }}">"#,
        SourceLanguage::Handlebars
        ; "makes no change to class string with mustache style interpolation"
    )]
    #[test_case(
        &RUSTYWIND_DEFAULT,
        r#"html`<div class="p-4 m-4 ${active ? 'flex flex-col' : 'hidden'}">`"#,
        r#"html`<div class="m-4 p-4 ${active ? 'flex flex-col' : 'hidden'}">`"#,
        SourceLanguage::Lit
        ; "makes no change to class string with js template literal interpolation"
    )]
    fn test_sort_file_contents(
        app: &RustyWind,
        input: &str,
        output: &str,
        language: SourceLanguage,
    ) {
        assert_eq!(
            app.sort_document(SourceDocument::new(input, language)),
            output
        );
    }
    // CLASS WRAPPING
    #[test_case(
        r#"'flex-col', 'inline', 'flex'"#,
        ClassWrapping::CommaSingleQuotes,
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#]
        ; "comma single quotes"
    )]
    #[test_case(
        r#""flex-col", "inline", "flex""#,
        ClassWrapping::CommaDoubleQuotes,
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#]
        ; "comma double quotes"
    )]
    #[test_case(
        r#"  'grid-cols-[1fr,2fr]' , 'before:content-[\'foo\']'  "#,
        ClassWrapping::CommaSingleQuotes,
        vec![r#"grid-cols-[1fr,2fr]"#, r#"before:content-[\'foo\']"#]
        ; "payload commas and escaped quotes"
    )]
    fn parses_valid_wrapped_class_lists(input: &str, wrapping: ClassWrapping, expected: Vec<&str>) {
        let parsed = WrappedClassList::parse(input, wrapping).unwrap();

        assert_eq!(parsed.iter().collect::<Vec<_>>(), expected);
    }

    #[test_case("", ClassWrapping::CommaSingleQuotes ; "empty list")]
    #[test_case("   ", ClassWrapping::CommaSingleQuotes ; "whitespace only")]
    #[test_case("flex", ClassWrapping::CommaSingleQuotes ; "unquoted entry")]
    #[test_case(r#"'flex', "p-4""#, ClassWrapping::CommaSingleQuotes ; "mixed quote modes")]
    #[test_case("'flex", ClassWrapping::CommaSingleQuotes ; "unterminated entry")]
    #[test_case(r#"'flex\'"#, ClassWrapping::CommaSingleQuotes ; "escaped closing quote")]
    #[test_case("''", ClassWrapping::CommaSingleQuotes ; "empty entry")]
    #[test_case("'flex p-4'", ClassWrapping::CommaSingleQuotes ; "multiple class tokens")]
    #[test_case("' flex'", ClassWrapping::CommaSingleQuotes ; "leading payload whitespace")]
    #[test_case("'flex',", ClassWrapping::CommaSingleQuotes ; "trailing empty entry")]
    #[test_case("'flex', , 'p-4'", ClassWrapping::CommaSingleQuotes ; "middle empty entry")]
    #[test_case("'flex' + active", ClassWrapping::CommaSingleQuotes ; "expression syntax")]
    #[test_case("'{{ flex }}'", ClassWrapping::CommaSingleQuotes ; "template syntax")]
    #[test_case(r#"'flex'"#, ClassWrapping::CommaDoubleQuotes ; "wrong configured quote")]
    fn rejects_invalid_wrapped_class_lists(input: &str, wrapping: ClassWrapping) {
        assert_eq!(WrappedClassList::parse(input, wrapping), None);
    }

    #[test_case(SourceLanguage::Html ; "structured markup parser")]
    #[test_case(SourceLanguage::Unknown ; "regex capture parser")]
    fn invalid_wrapped_values_fail_closed(language: SourceLanguage) {
        let app = RustyWind {
            class_wrapping: ClassWrapping::CommaSingleQuotes,
            ..RUSTYWIND_DEFAULT
        };
        let source = r#"<div class="'p-4', active, 'm-4'"></div>"#;
        let document = SourceDocument::new(source, language);

        assert!(!app.has_classes(document));
        assert_eq!(app.sort_document(document), source);
    }

    #[test]
    fn valid_wrapped_values_sort_and_canonicalize_separators() {
        let app = RustyWind {
            sorter: Sorter::new(HashMap::from([
                ("grid-cols-[1fr,2fr]".to_string(), 1),
                ("flex".to_string(), 0),
            ])),
            class_wrapping: ClassWrapping::CommaSingleQuotes,
            ..RUSTYWIND_DEFAULT
        };
        let source = r#"<div class="  'grid-cols-[1fr,2fr]' , 'flex'  "></div>"#;

        assert_eq!(
            app.sort_document(SourceDocument::new(source, SourceLanguage::Html)),
            r#"<div class="'flex', 'grid-cols-[1fr,2fr]'"></div>"#
        );
    }

    #[test]
    fn test_arbitrary_value_with_whitespace_stays_intact() {
        let classes = "my-6 max-w-[min(100%, 500px)]";
        let sorted = RUSTYWIND_DEFAULT.sort_classes(classes);

        assert_eq!(sorted, "max-w-[min(100%, 500px)] my-6");
    }

    #[test]
    fn test_pattern_sorter_integration() {
        // test that PatternSorter can be used in RustyWind
        let app = RustyWind {
            sorter: Sorter::PatternSorter,
            ..RUSTYWIND_DEFAULT
        };

        let classes = "p-4 m-4 flex hover:p-1";
        let sorted = app.sort_classes(classes);

        // pattern-based sorting: margin(25) < display(35) < padding(252) < variants
        assert_eq!(sorted, "m-4 flex p-4 hover:p-1");
    }

    #[test]
    fn test_pattern_sorter_with_file_contents() {
        let app = RustyWind {
            sorter: Sorter::PatternSorter,
            ..RUSTYWIND_DEFAULT
        };

        let input = r#"<div class="p-4 m-4 flex"></div>"#;
        let output = app.sort_file_contents(input);

        // pattern-based sorting: margin(25) < display(35) < padding(252)
        assert_eq!(output, r#"<div class="m-4 flex p-4"></div>"#);
    }

    /// Test that arbitrary variant classes are matched by the regex (Issue #115)
    #[test]
    fn test_regex_matches_arbitrary_variants() {
        let app = RUSTYWIND_DEFAULT;

        // test element state selectors
        let input = r#"<div class="[&.htmx-request]:h-0 flex p-4"></div>"#;
        assert!(
            app.has_classes(SourceDocument::new(input, SourceLanguage::Html)),
            "Should match [&.class] syntax"
        );

        let sorted = app.sort_file_contents(input);
        assert!(
            sorted.contains("[&.htmx-request]:h-0"),
            "Arbitrary variant should be preserved in output"
        );

        // test child/sibling selectors
        let input2 = r#"<div class="[&>*]:p-4 [&+*]:mt-4 block"></div>"#;
        assert!(
            app.has_classes(SourceDocument::new(input2, SourceLanguage::Html)),
            "Should match combinator syntax"
        );

        // test attribute selectors
        let input3 = r#"<div class="[&[data-state=open]]:bg-gray-100 flex"></div>"#;
        assert!(
            app.has_classes(SourceDocument::new(input3, SourceLanguage::Html)),
            "Should match attribute selector syntax"
        );

        // test at-rule variants
        let input4 = r#"<div class="[@supports(display:grid)]:grid flex"></div>"#;
        assert!(
            app.has_classes(SourceDocument::new(input4, SourceLanguage::Html)),
            "Should match @-rule syntax"
        );

        // test calc with percentage
        let input5 = r#"<div class="w-[calc(100%+20px)] flex"></div>"#;
        assert!(
            app.has_classes(SourceDocument::new(input5, SourceLanguage::Html)),
            "Should match calc with percentage"
        );
    }

    #[test_case(
        None,
        ClassWrapping::NoWrapping,
        r#"<div class="flex-col inline flex"></div>"#,
        r#"<div class="flex inline flex-col"></div>"#
        ; "normal HTML use case"
    )]
    #[test_case(
        Some(r#"(?:\[)(?P<classes>[_a-zA-Z0-9\.,\-'"\s]+)(?:\])"#),
        ClassWrapping::CommaSingleQuotes,
        r#"classes = ['flex-col', 'inline', 'flex']"#,
        r#"classes = ['flex', 'inline', 'flex-col']"#
        ; "array with single quotes"
    )]
    #[test_case(
        Some(r#"(?:\[)(?P<classes>[_a-zA-Z0-9\.,\-'"\s]+)(?:\])"#),
        ClassWrapping::CommaDoubleQuotes,
        r#"classes = ["flex-col", "inline", "flex"]"#,
        r#"classes = ["flex", "inline", "flex-col"]"#
        ; "array with double quotes"
    )]
    fn test_unusual_use_cases(
        regex_overwrite: Option<&str>,
        class_wrapping: ClassWrapping,
        input: &str,
        output: &str,
    ) {
        let regex = match regex_overwrite {
            Some(re) => FinderRegex::CustomRegex(
                crate::sorter::CustomClassExtractor::new(Regex::new(re).unwrap()).unwrap(),
            ),
            None => FinderRegex::DefaultRegex,
        };

        let app = RustyWind {
            regex,
            sorter: Sorter::PatternSorter,
            allow_duplicates: false,
            class_wrapping,
            tailwind_prefix: None,
            preserve_whitespace: false,
        };

        assert_eq!(app.sort_file_contents(input), output);
    }

    #[test]
    fn custom_sorter_only_recognizes_variants_followed_by_a_colon() {
        let app = RustyWind {
            regex: FinderRegex::DefaultRegex,
            sorter: Sorter::new(HashMap::from([("flex".to_string(), 0)])),
            allow_duplicates: false,
            class_wrapping: ClassWrapping::NoWrapping,
            tailwind_prefix: None,
            preserve_whitespace: false,
        };
        let input = "even-columns empty-state hovercraft event.status status_color even:flex";

        assert_eq!(
            app.sort_classes(input),
            "even:flex even-columns empty-state hovercraft event.status status_color"
        );
    }

    #[test]
    /// The one-class-per-line layout from avencera/rustywind#25, which default
    /// sorting flattens to a single line.
    fn preserve_whitespace_keeps_one_class_per_line() {
        let app = RustyWind {
            preserve_whitespace: true,
            ..RustyWind::default()
        };
        let input = r#"<div
  class="
    grid
    border
    fixed
    top-0
    right-0
    z-20
    grid-flow-col
    gap-2
    justify-start
    my-12
    mx-8
    text-red-800
    bg-red-50
    rounded
    border-red-100
    shadow-2xl
  "
>"#;
        let expected = r#"<div
  class="
    fixed
    top-0
    right-0
    z-20
    mx-8
    my-12
    grid
    grid-flow-col
    justify-start
    gap-2
    rounded
    border
    border-red-100
    bg-red-50
    text-red-800
    shadow-2xl
  "
>"#;
        assert_eq!(app.sort_file_contents(input), expected);
    }

    #[test]
    fn preserve_whitespace_drops_surplus_separators_on_dedup() {
        let app = RustyWind {
            preserve_whitespace: true,
            ..RustyWind::default()
        };
        let input = "<div class=\"flex\n    flex p-4\"></div>";
        assert_eq!(
            app.sort_file_contents(input),
            "<div class=\"flex\n    p-4\"></div>"
        );
    }
}

use std::borrow::Cow;

use crate::{
    class_wrapping::ClassWrapping,
    consts::{VARIANT_SEARCHER, VARIANTS},
    hybrid_sorter::HybridSorter,
    sorter::{FinderRegex, Sorter},
    tailwind_prefix::{normalize_tailwind_prefix, normalize_tailwind_prefix_value},
};
use ahash::{AHashMap as HashMap, AHashSet as HashSet};
use aho_corasick::{Anchored, Input};
use regex::Captures;
use std::sync::{Arc, LazyLock, RwLock};

/// Global instance of the HybridSorter for pattern-based sorting.
static PATTERN_SORTER: LazyLock<HybridSorter> = LazyLock::new(HybridSorter::new);
static PREFIXED_PATTERN_SORTERS: LazyLock<RwLock<HashMap<String, Arc<HybridSorter>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

struct SortCandidate<'a> {
    original: &'a str,
    lookup: Cow<'a, str>,
}

/// The options to pass to the sorter.
#[derive(Debug, Clone)]
pub struct RustyWind {
    pub regex: FinderRegex,
    pub sorter: Sorter,
    pub allow_duplicates: bool,
    pub class_wrapping: ClassWrapping,
    pub tailwind_prefix: Option<String>,
}

impl Default for RustyWind {
    fn default() -> Self {
        Self {
            regex: FinderRegex::DefaultRegex,
            sorter: Sorter::PatternSorter,
            allow_duplicates: false,
            class_wrapping: ClassWrapping::NoWrapping,
            tailwind_prefix: None,
        }
    }
}

impl RustyWind {
    pub fn new(
        regex: FinderRegex,
        sorter: Sorter,
        allow_duplicates: bool,
        class_wrapping: ClassWrapping,
    ) -> Self {
        Self::new_with_tailwind_prefix(regex, sorter, allow_duplicates, class_wrapping, None)
    }

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
        }
    }

    /// Checks if the file contents have any classes.
    pub fn has_classes(&self, file_contents: &str) -> bool {
        self.regex.is_match(file_contents)
    }

    /// Sorts the classes in the file contents.
    pub fn sort_file_contents<'a>(&self, file_contents: &'a str) -> Cow<'a, str> {
        self.regex.replace_all(file_contents, |caps: &Captures| {
            let classes = caps
                .get(1)
                .or_else(|| caps.get(2))
                .expect("class extractor regex must include a capture group")
                .as_str();
            let sorted_classes = self.sort_classes(classes);
            caps[0].replace(classes, &sorted_classes)
        })
    }

    /// Given a [&str] of whitespace-separated classes, returns a [String] of sorted classes.
    /// Does not preserve whitespace.
    pub fn sort_classes(&self, class_string: &str) -> String {
        let extracted_classes = self.unwrap_wrapped_classes(class_string);

        let mut sorted = self.sort_classes_vec(extracted_classes.into_iter());

        if !self.allow_duplicates {
            deduplicate_classes(&mut sorted);
        }

        self.rewrap_wrapped_classes(sorted)
    }

    fn unwrap_wrapped_classes<'a>(&self, class_string: &'a str) -> Vec<&'a str> {
        match self.class_wrapping {
            ClassWrapping::NoWrapping => split_class_tokens(class_string),
            ClassWrapping::CommaSingleQuotes => class_string
                .split(',')
                .flat_map(split_class_tokens)
                .map(|class| class.trim_matches('\''))
                .collect(),
            ClassWrapping::CommaDoubleQuotes => class_string
                .split(',')
                .flat_map(split_class_tokens)
                .map(|class| class.trim_matches('"'))
                .collect(),
        }
    }

    fn rewrap_wrapped_classes(&self, classes: Vec<&str>) -> String {
        match self.class_wrapping {
            ClassWrapping::NoWrapping => classes.join(" "),
            ClassWrapping::CommaSingleQuotes => classes
                .iter()
                .map(|class| format!("'{}'", class))
                .collect::<Vec<String>>()
                .join(", "),
            ClassWrapping::CommaDoubleQuotes => classes
                .iter()
                .map(|class| format!("\"{}\"", class))
                .collect::<Vec<String>>()
                .join(", "),
        }
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
                    let input = Input::new(candidate.lookup.as_ref()).anchored(Anchored::Yes);
                    match VARIANT_SEARCHER.find(input) {
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

fn split_class_tokens(class_string: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut bracket_depth: u32 = 0;

    for (index, character) in class_string.char_indices() {
        match character {
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
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
    use pretty_assertions::assert_eq;
    use regex::Regex;
    use test_case::test_case;
    const RUSTYWIND_DEFAULT: RustyWind = RustyWind {
        regex: FinderRegex::DefaultRegex,
        sorter: Sorter::PatternSorter,
        allow_duplicates: false,
        class_wrapping: ClassWrapping::NoWrapping,
        tailwind_prefix: None,
    };

    // HAS_CLASSES --------------------------------------------------------------------------------
    #[test_case( r#"<div class="flex-col inline flex"></div>"#, true ; "div tag with class")]
    #[test_case( r#"<body class="unknown-class"></body>"#, true ; "body tag with unknown class")]
    #[test_case( r#"<p className="unknown-class"></p>"#, true ; "p tag with unknown class")]
    #[test_case( r#"<p>not a class</p>"#, false ; "p tag with no class")]
    #[test_case( r#"<div><p></p><p></p></div>"#, false ; "nested tags, no class")]
    #[test_case( r#"<div><p><span className="inline"></span></p><p></p></div>"#, true ; "nested tags, class in child")]
    fn test_has_classes(input: &str, output: bool) {
        assert_eq!(RUSTYWIND_DEFAULT.has_classes(input), output);
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
        r#"This is to represent any other normal file."#
        ; "makes no change to files without class string"
    )]
    #[test_case(
        &RUSTYWIND_DEFAULT,
        r#"<div><p><img height="100" width="250" /></p><p></p></div>"#,
        r#"<div><p><img height="100" width="250" /></p><p></p></div>"#
        ; "makes no change to elements without class string"
    )]
    fn test_sort_file_contents(app: &RustyWind, input: &str, output: &str) {
        assert_eq!(app.sort_file_contents(input), output);
    }
    // CLASS WRAPPING
    #[test_case(
        r#"flex-col inline flex"#,
        ClassWrapping::NoWrapping,
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#]
        ; "no wrapping"
    )]
    #[test_case(
        r#"max-w-[min(100%, 500px)] my-6"#,
        ClassWrapping::NoWrapping,
        vec![r#"max-w-[min(100%, 500px)]"#, r#"my-6"#]
        ; "arbitrary value with whitespace"
    )]
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
    fn test_unwrap_wrapped_classes(input: &str, wrapping: ClassWrapping, output: Vec<&str>) {
        let app = RustyWind {
            class_wrapping: wrapping,
            ..RUSTYWIND_DEFAULT
        };

        assert_eq!(app.unwrap_wrapped_classes(input), output)
    }

    #[test_case(
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#],
        ClassWrapping::NoWrapping,
        r#"flex-col inline flex"#
        ; "no wrapping"
    )]
    #[test_case(
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#],
        ClassWrapping::CommaSingleQuotes,
        r#"'flex-col', 'inline', 'flex'"#
        ; "comma single quotes"
    )]
    #[test_case(
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#],
        ClassWrapping::CommaDoubleQuotes,
        r#""flex-col", "inline", "flex""#
        ; "comma double quotes"
    )]
    fn test_rewrap_wrapped_classes(input: Vec<&str>, wrapping: ClassWrapping, output: &str) {
        let app = RustyWind {
            class_wrapping: wrapping,
            ..RUSTYWIND_DEFAULT
        };

        assert_eq!(app.rewrap_wrapped_classes(input), output)
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
        assert!(app.has_classes(input), "Should match [&.class] syntax");

        let sorted = app.sort_file_contents(input);
        assert!(
            sorted.contains("[&.htmx-request]:h-0"),
            "Arbitrary variant should be preserved in output"
        );

        // test child/sibling selectors
        let input2 = r#"<div class="[&>*]:p-4 [&+*]:mt-4 block"></div>"#;
        assert!(app.has_classes(input2), "Should match combinator syntax");

        // test attribute selectors
        let input3 = r#"<div class="[&[data-state=open]]:bg-gray-100 flex"></div>"#;
        assert!(
            app.has_classes(input3),
            "Should match attribute selector syntax"
        );

        // test at-rule variants
        let input4 = r#"<div class="[@supports(display:grid)]:grid flex"></div>"#;
        assert!(app.has_classes(input4), "Should match @-rule syntax");

        // test calc with percentage
        let input5 = r#"<div class="w-[calc(100%+20px)] flex"></div>"#;
        assert!(app.has_classes(input5), "Should match calc with percentage");
    }

    #[test_case(
        None,
        ClassWrapping::NoWrapping,
        r#"<div class="flex-col inline flex"></div>"#,
        r#"<div class="flex inline flex-col"></div>"#
        ; "normal HTML use case"
    )]
    #[test_case(
        Some(r#"(?:\[)([_a-zA-Z0-9\.,\-'"\s]+)(?:\])"#),
        ClassWrapping::CommaSingleQuotes,
        r#"classes = ['flex-col', 'inline', 'flex']"#,
        r#"classes = ['flex', 'inline', 'flex-col']"#
        ; "array with single quotes"
    )]
    #[test_case(
        Some(r#"(?:\[)([_a-zA-Z0-9\.,\-'"\s]+)(?:\])"#),
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
            Some(re) => FinderRegex::CustomRegex(Regex::new(re).unwrap()),
            None => FinderRegex::DefaultRegex,
        };

        let app = RustyWind {
            regex,
            sorter: Sorter::PatternSorter,
            allow_duplicates: false,
            class_wrapping,
            tailwind_prefix: None,
        };

        assert_eq!(app.sort_file_contents(input), output);
    }
}

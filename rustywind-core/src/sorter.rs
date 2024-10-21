//! The module that sorts the classes in the file contents.
use std::borrow::Cow;
use std::ops::Deref;

use ahash::AHashMap as HashMap;

use aho_corasick::{Anchored, Input};
use itertools::Itertools;
use regex::{Captures, Regex};

use crate::consts::{VARIANTS, VARIANT_SEARCHER};
use crate::defaults::{RE, SORTER};

/// Use either our default regex in [crate::defaults::RE] or a custom regex.
#[derive(Debug)]
pub enum FinderRegex {
    DefaultRegex,
    CustomRegex(Regex),
}

impl Deref for FinderRegex {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        match &self {
            Self::DefaultRegex => &RE,
            Self::CustomRegex(re) => re,
        }
    }
}

/// How individual classes are wrapped.
#[derive(Debug, Clone, Copy)]
pub enum HowClassesAreWrapped {
    NoWrapping,
    CommaSingleQuotes,
    CommaDoubleQuotes,
}

impl Default for HowClassesAreWrapped {
    fn default() -> Self {
        Self::NoWrapping
    }
}

impl HowClassesAreWrapped {
    pub fn as_str(&self) -> &'static str {
        match self {
            HowClassesAreWrapped::NoWrapping => "no-wrapping",
            HowClassesAreWrapped::CommaSingleQuotes => "comma-single-quotes",
            HowClassesAreWrapped::CommaDoubleQuotes => "comma-double-quotes",
        }
    }
}

impl<T: AsRef<str>> From<T> for HowClassesAreWrapped {
    fn from(s: T) -> Self {
        match s.as_ref() {
            "no-wrapping" => Self::NoWrapping,
            "comma-single-quotes" => Self::CommaSingleQuotes,
            "comma-double-quotes" => Self::CommaDoubleQuotes,
            _ => Self::NoWrapping,
        }
    }
}

/// Use either our default sorter in [crate::defaults::SORTER] or a custom sorter.
#[derive(Debug)]
pub enum Sorter {
    DefaultSorter,
    CustomSorter(HashMap<String, usize>),
}

impl Deref for Sorter {
    type Target = HashMap<String, usize>;

    fn deref(&self) -> &Self::Target {
        match &self {
            Self::DefaultSorter => &SORTER,
            Self::CustomSorter(sorter) => sorter,
        }
    }
}

/// The options to pass to the sorter.
#[derive(Debug)]
pub struct Options {
    pub regex: FinderRegex,
    pub sorter: Sorter,
    pub allow_duplicates: bool,
    pub class_wrapping: HowClassesAreWrapped,
}

/// Checks if the file contents have any classes.
pub fn has_classes(file_contents: &str, options: &Options) -> bool {
    options.regex.is_match(file_contents)
}

/// Sorts the classes in the file contents.
pub fn sort_file_contents<'a>(file_contents: &'a str, options: &Options) -> Cow<'a, str> {
    options.regex.replace_all(file_contents, |caps: &Captures| {
        let classes = &caps[1];
        let sorted_classes = sort_classes(classes, options);
        caps[0].replace(classes, &sorted_classes)
    })
}

/// Given a [&str] of whitespace-separated classes, returns a [String] of sorted classes.
/// Does not preserve whitespace.
pub fn sort_classes(class_string: &str, options: &Options) -> String {
    let sorter = &options.sorter;

    let extracted_classes = unwrap_wrapped_classes(class_string, options.class_wrapping);

    let sorted = if options.allow_duplicates {
        sort_classes_vec(extracted_classes.into_iter(), sorter)
    } else {
        sort_classes_vec(extracted_classes.into_iter().unique(), sorter)
    };

    rewrap_wrapped_classes(sorted, options.class_wrapping)
}

fn unwrap_wrapped_classes<'a>(
    class_string: &'a str,
    wrapping: HowClassesAreWrapped,
) -> Vec<&'a str> {
    match wrapping {
        HowClassesAreWrapped::NoWrapping => class_string.split_ascii_whitespace().collect(),
        HowClassesAreWrapped::CommaSingleQuotes => class_string
            .split(',')
            .flat_map(|class| class.split_ascii_whitespace())
            .map(|class| class.trim_matches('\''))
            .collect(),
        HowClassesAreWrapped::CommaDoubleQuotes => class_string
            .split(',')
            .flat_map(|class| class.split_ascii_whitespace())
            .map(|class| class.trim_matches('"'))
            .collect(),
    }
}

fn rewrap_wrapped_classes<'a>(classes: Vec<&'a str>, wrapping: HowClassesAreWrapped) -> String {
    match wrapping {
        HowClassesAreWrapped::NoWrapping => classes.join(" "),
        HowClassesAreWrapped::CommaSingleQuotes => classes
            .iter()
            .map(|class| format!("'{}'", class))
            .join(", "),
        HowClassesAreWrapped::CommaDoubleQuotes => classes
            .iter()
            .map(|class| format!("\"{}\"", class))
            .join(", "),
    }
}

fn sort_classes_vec<'a>(
    classes: impl Iterator<Item = &'a str>,
    sorter: &HashMap<String, usize>,
) -> Vec<&'a str> {
    let enumerated_classes = classes.map(|class| ((class), sorter.get(class)));

    let mut tailwind_classes: Vec<(&str, &usize)> = vec![];
    let mut custom_classes: Vec<&str> = vec![];
    let mut variants: HashMap<&str, Vec<&str>> = HashMap::new();

    for (class, maybe_size) in enumerated_classes {
        match maybe_size {
            Some(size) => tailwind_classes.push((class, size)),
            None => {
                let input = Input::new(class).anchored(Anchored::Yes);
                match VARIANT_SEARCHER.find(input) {
                    Some(prefix_match) => {
                        let prefix = VARIANTS[prefix_match.pattern()];
                        variants.entry(prefix).or_default().push(class)
                    }
                    None => custom_classes.push(class),
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
        let (mut sorted_classes, new_custom_classes) = sort_variant_classes(
            variants.remove(key).unwrap_or_default(),
            custom_classes,
            key.len() + 1,
            sorter,
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
    classes: Vec<&'a str>,
    mut custom_classes: Vec<&'a str>,
    class_after: usize,
    sorter: &HashMap<String, usize>,
) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut tailwind_classes = Vec::with_capacity(classes.len());

    for class in classes {
        match class.get(class_after..).and_then(|class| sorter.get(class)) {
            Some(class_placement) => tailwind_classes.push((class, class_placement)),
            None => custom_classes.push(class),
        }
    }

    tailwind_classes.sort_by_key(|&(_class, class_placement)| class_placement);

    let sorted_classes = tailwind_classes
        .iter()
        .map(|(class, _index)| *class)
        .collect();

    (sorted_classes, custom_classes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    const OPTS_DEFAULT: Options = Options {
        regex: FinderRegex::DefaultRegex,
        sorter: Sorter::DefaultSorter,
        allow_duplicates: false,
        class_wrapping: HowClassesAreWrapped::NoWrapping,
    };

    // HAS_CLASSES --------------------------------------------------------------------------------
    #[test_case( r#"<div class="flex-col inline flex"></div>"#, true ; "div tag with class")]
    #[test_case( r#"<body class="unknown-class"></body>"#, true ; "body tag with unknown class")]
    #[test_case( r#"<p className="unknown-class"></p>"#, true ; "p tag with unknown class")]
    #[test_case( r#"<p>not a class</p>"#, false ; "p tag with no class")]
    #[test_case( r#"<div><p></p><p></p></div>"#, false ; "nested tags, no class")]
    #[test_case( r#"<div><p><span className="inline"></span></p><p></p></div>"#, true ; "nested tags, class in child")]
    fn test_has_classes(input: &str, output: bool) {
        assert_eq!(has_classes(input, &OPTS_DEFAULT), output);
    }

    // SORT_CLASSES_VEC ---------------------------------------------------------------------------
    #[test_case(
        ["inline", "inline-block", "random-class", "shadow-sm", "py-2", "justify-end", "px-2", "flex"],
        vec!["inline-block", "inline", "flex", "justify-end", "py-2", "px-2", "shadow-sm", "random-class"]
        ; "classes inline inline-block random-class shadow-sm py-2 justify-end px-2 flex"
    )]
    #[test_case(
        ["bg-purple", "text-white", "unknown-class", "flex-col", "gap-4", "flex", "skew-y-0"],
        vec!["flex", "flex-col", "gap-4", "text-white", "skew-y-0", "bg-purple", "unknown-class"]
        ; "classes bg-purple text-white unknown-class flex-col gap-4 flex skew-y-0"
    )]
    #[test_case(
        ["translate-x-7", "bg-orange-200", "unknown-class", "static", "top-5", "flex", "items-center"],
        vec!["flex", "static", "top-5", "items-center", "bg-orange-200", "translate-x-7", "unknown-class"]
        ; "classes translate-x-7 bg-orange-200 unknown-class static top-5 flex items-center"
    )]
    fn test_sort_classes_vec<'a>(input: impl IntoIterator<Item = &'a str>, output: Vec<&str>) {
        assert_eq!(sort_classes_vec(input.into_iter(), &SORTER), output)
    }

    // SORT_FILE_CONTENTS -------------------------------------------------------------------------
    // BASIC, SINGLE ELEMENT TESTS
    #[test_case(
        &OPTS_DEFAULT,
        r#"<div class="py-2 inline random-class shadow-sm"></div>"#,
        r#"<div class="inline py-2 shadow-sm random-class"></div>"#
        ; "div tag using class"
    )]
    #[test_case(
        &OPTS_DEFAULT,
        r#"<section className="inline lg:inline-block abcd py-2"></section>"#,
        r#"<section className="inline py-2 lg:inline-block abcd"></section>"#
        ; "section tag using className"
    )]
    #[test_case(
        &OPTS_DEFAULT,
        r#"<p class="unknown-class bg-blue-300 py-2 object-top">content</p>"#,
        r#"<p class="object-top py-2 bg-blue-300 unknown-class">content</p>"#
        ; "p tag using class"
    )]
    #[test_case(
        &OPTS_DEFAULT,
        r#"<p className="py-2 py-2 random-class underline underline underline">text</p>"#,
        r#"<p className="py-2 underline random-class">text</p>"#
        ; "p tag remove duplicates"
    )]
    #[test_case(
        &Options { allow_duplicates: true, ..OPTS_DEFAULT},
        r#"<section className="inline py-2 py-2 random-class italic italic italic"></section>"#,
        r#"<section className="inline py-2 py-2 italic italic italic random-class"></section>"#
        ; "section tag keeps duplicates if bool set"
    )]
    // BASE
    #[test_case(
        &OPTS_DEFAULT,
        r#"
            <div>
                <div class='mt-4 mb-0.5 flex inline-block inline px-0.5 pt-10 random-class justify-items absolute relative another-random-class'>
                    <ul class='flex items-center md:pr-4 lg:pr-6'>
                    </ul>
                </div>
            </div>
        "#,
        r#"
            <div>
                <div class='inline-block inline flex absolute relative px-0.5 pt-10 mt-4 mb-0.5 random-class justify-items another-random-class'>
                    <ul class='flex items-center md:pr-4 lg:pr-6'>
                    </ul>
                </div>
            </div>
        "#
        ; "sorts classes"
    )]
    #[test_case(
        &OPTS_DEFAULT,
        r#"
            <div>
                <div class='4xl:inline-block absolute xl:relative relative flex inline-block xl:absolute sm:relative sm:flex inline random-class justify-items another-random-class
                sm:absolute 4xl:flex xl:random-class sm:inline-block'>
                    <ul class='flex items-center md:pr-4 lg:pr-6 xl:flex'>
                </div>
            </div>
        "#,
        r#"
            <div>
                <div class='inline-block inline flex absolute relative sm:inline-block sm:flex sm:absolute sm:relative xl:absolute xl:relative 4xl:inline-block 4xl:flex random-class justify-items another-random-class xl:random-class'>
                    <ul class='flex items-center md:pr-4 lg:pr-6 xl:flex'>
                </div>
            </div>
        "#
        ; "sorts responsive classes"
    )]
    #[test_case(
        &OPTS_DEFAULT,
        r#"
            <div>
                <div class='even:inline 4xl:inline-block focus-visible:flex absolute xl:relative relative focus:flex flex active:flex disabled:flex visited:flex inline-block dark:absolute sm:relative sm:flex inline random-class justify-items another-random-class 
                sm:absolute 4xl:flex xl:random-class sm:inline-block'>
                    <ul class='flex items-center md:pr-4 lg:pr-6 xl:flex'>
                </div>
            </div>
        "#,
        r#"
            <div>
                <div class='inline-block inline flex absolute relative sm:inline-block sm:flex sm:absolute sm:relative xl:relative 4xl:inline-block 4xl:flex dark:absolute even:inline visited:flex focus:flex focus-visible:flex active:flex disabled:flex random-class justify-items another-random-class xl:random-class'>
                    <ul class='flex items-center md:pr-4 lg:pr-6 xl:flex'>
                </div>
            </div>
        "#
        ; "sorts variant classes"
    )]
    // DUPLICATES
    #[test_case(
        &OPTS_DEFAULT,
        r#"
            <div>
                <div class='absolute relative flex flex flex flex inline-block inline random-class justify-items another-random-class'>
                    <ul class='flex items-center md:pr-4 lg:pr-6'>
                    </ul>
                </div>
            </div>
        "#,
        r#"
            <div>
                <div class='inline-block inline flex absolute relative random-class justify-items another-random-class'>
                    <ul class='flex items-center md:pr-4 lg:pr-6'>
                    </ul>
                </div>
            </div>
        "#
        ; "removes duplicates"
    )]
    #[test_case(
        &Options { allow_duplicates: true, ..OPTS_DEFAULT},
        r#"
            <div>
                <div class='absolute relative flex flex flex flex inline-block inline random-class justify-items another-random-class'>
                    <ul class='flex items-center md:pr-4 lg:pr-6'>
                    </ul>
                </div>
            </div>
        "#,
        r#"
            <div>
                <div class='inline-block inline flex flex flex flex absolute relative random-class justify-items another-random-class'>
                    <ul class='flex items-center md:pr-4 lg:pr-6'>
                    </ul>
                </div>
            </div>
        "#
        ; "keeps duplicates if bool set"
    )]
    // MULTI-LINE AND OTHER SPACING
    // Note the intentionally poor spacing. Rustywind isn't concerned so much about formatting, but
    // due to how whitespace is handled, it all ends up on one line as a side effect. This makes it
    // easier for formatters like Prettier to do their job.
    #[test_case(
        &OPTS_DEFAULT,
        r#"
            <div
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
            >
              <!-- ... -->
            </div>
        "#,
        r#"
            <div
              class="grid fixed top-0 right-0 z-20 grid-flow-col gap-2 justify-start my-12 mx-8 text-red-800 bg-red-50 rounded border border-red-100 shadow-2xl"
            >
              <!-- ... -->
            </div>
        "#
        ; "sorts and formats multiline class list"
    )]
    #[test_case(
        &OPTS_DEFAULT,
        r#"
            <div
              class="
                grid border fixed
                top-0
                right-0
                z-20
                grid-flow-col
                gap-2
                justify-start
                my-12 mx-8 text-red-800
                bg-red-50
                rounded
                border-red-100
                shadow-2xl
              "
            >
              <!-- ... -->
            </div>
        "#,
        r#"
            <div
              class="grid fixed top-0 right-0 z-20 grid-flow-col gap-2 justify-start my-12 mx-8 text-red-800 bg-red-50 rounded border border-red-100 shadow-2xl"
            >
              <!-- ... -->
            </div>
        "#
        ; "sorts and formats multiline and space separated class list"
    )]
    #[test_case(
        &OPTS_DEFAULT,
        r#"
            <div class="m-2 grid-cols-4
                    gap-1 foo
                border  theres-a-tab-here:	bar border-red-600
                    ">
            </div>
        "#,
        r#"
            <div class="grid-cols-4 gap-1 m-2 border border-red-600 foo theres-a-tab-here: bar">
            </div>
        "#
        ; "sorts and formats multiline and space separated class list, with custom classes"
    )]
    // NO CLASSES
    #[test_case(
        &OPTS_DEFAULT,
        r#"This is to represent any other normal file."#,
        r#"This is to represent any other normal file."#
        ; "makes no change to files without class string"
    )]
    #[test_case(
        &OPTS_DEFAULT,
        r#"<div><p><img height="100" width="250" /></p><p></p></div>"#,
        r#"<div><p><img height="100" width="250" /></p><p></p></div>"#
        ; "makes no change to elements without class string"
    )]
    fn test_sort_file_contents(opts: &Options, input: &str, output: &str) {
        assert_eq!(sort_file_contents(input, opts), output);
    }
    // CLASS WRAPPING
    #[test_case(
        r#"flex-col inline flex"#,
        HowClassesAreWrapped::NoWrapping,
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#]
        ; "no wrapping"
    )]
    #[test_case(
        r#"'flex-col', 'inline', 'flex'"#,
        HowClassesAreWrapped::CommaSingleQuotes,
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#]
        ; "comma single quotes"
    )]
    #[test_case(
        r#""flex-col", "inline", "flex""#,
        HowClassesAreWrapped::CommaDoubleQuotes,
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#]
        ; "comma double quotes"
    )]
    fn test_unwrap_wrapped_classes<'a>(
        input: &str,
        wrapping: HowClassesAreWrapped,
        output: Vec<&str>,
    ) {
        assert_eq!(unwrap_wrapped_classes(input, wrapping), output)
    }

    #[test_case(
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#],
        HowClassesAreWrapped::NoWrapping,
        r#"flex-col inline flex"#
        ; "no wrapping"
    )]
    #[test_case(
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#],
        HowClassesAreWrapped::CommaSingleQuotes,
        r#"'flex-col', 'inline', 'flex'"#
        ; "comma single quotes"
    )]
    #[test_case(
        vec![r#"flex-col"#, r#"inline"#, r#"flex"#],
        HowClassesAreWrapped::CommaDoubleQuotes,
        r#""flex-col", "inline", "flex""#
        ; "comma double quotes"
    )]
    fn test_rewrap_wrapped_classes<'a>(
        input: Vec<&'a str>,
        wrapping: HowClassesAreWrapped,
        output: &str,
    ) {
        assert_eq!(rewrap_wrapped_classes(input, wrapping), output)
    }

    #[test_case(
        None,
        HowClassesAreWrapped::NoWrapping,
        r#"<div class="flex-col inline flex"></div>"#,
        r#"<div class="inline flex flex-col"></div>"#
        ; "normal HTML use case"
    )]
    #[test_case(
        Some(r#"(?:\[)([_a-zA-Z0-9\.,\-'"\s]+)(?:\])"#),
        HowClassesAreWrapped::CommaSingleQuotes,
        r#"classes = ['flex-col', 'inline', 'flex']"#,
        r#"classes = ['inline', 'flex', 'flex-col']"#
        ; "array with single quotes"
    )]
    #[test_case(
        Some(r#"(?:\[)([_a-zA-Z0-9\.,\-'"\s]+)(?:\])"#),
        HowClassesAreWrapped::CommaDoubleQuotes,
        r#"classes = ["flex-col", "inline", "flex"]"#,
        r#"classes = ["inline", "flex", "flex-col"]"#
        ; "array with double quotes"
    )]
    fn test_unusual_use_cases(
        regex_overwrite: Option<&str>,
        class_wrapping: HowClassesAreWrapped,
        input: &str,
        output: &str,
    ) {
        let regex = match regex_overwrite {
            Some(re) => FinderRegex::CustomRegex(Regex::new(re).unwrap()),
            None => FinderRegex::DefaultRegex,
        };

        let opts = Options {
            regex,
            sorter: Sorter::DefaultSorter,
            allow_duplicates: false,
            class_wrapping,
        };

        assert_eq!(sort_file_contents(input, &opts), output);
    }
}

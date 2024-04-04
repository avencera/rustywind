use std::borrow::Cow;

use ahash::AHashMap as HashMap;

use aho_corasick::{Anchored, Input};
use itertools::Itertools;
use regex::{Captures, Regex};

use crate::consts::{VARIANTS, VARIANT_SEARCHER};
use crate::defaults::{RE, SORTER};

#[derive(Debug)]
pub enum FinderRegex {
    DefaultRegex,
    CustomRegex(Regex),
}

#[derive(Debug)]
pub enum Sorter {
    DefaultSorter,
    CustomSorter(HashMap<String, usize>),
}

#[derive(Debug)]
pub struct Options {
    pub regex: FinderRegex,
    pub sorter: Sorter,
    pub allow_duplicates: bool,
}

pub fn has_classes(file_contents: &str, options: &Options) -> bool {
    let regex = match &options.regex {
        FinderRegex::DefaultRegex => &RE,
        FinderRegex::CustomRegex(regex) => regex,
    };

    regex.is_match(file_contents)
}

pub fn sort_file_contents<'a>(file_contents: &'a str, options: &Options) -> Cow<'a, str> {
    let regex = match &options.regex {
        FinderRegex::DefaultRegex => &RE,
        FinderRegex::CustomRegex(regex) => regex,
    };

    regex.replace_all(file_contents, |caps: &Captures| {
        let classes = &caps[1];
        let sorted_classes = sort_classes(classes, options);

        caps[0].replace(classes, &sorted_classes)
    })
}

fn sort_classes(class_string: &str, options: &Options) -> String {
    let sorter: &HashMap<String, usize> = match &options.sorter {
        Sorter::DefaultSorter => &SORTER,
        Sorter::CustomSorter(custom_sorter) => custom_sorter,
    };

    let str_vec = if options.allow_duplicates {
        sort_classes_vec(class_string.split_ascii_whitespace(), sorter)
    } else {
        sort_classes_vec(class_string.split_ascii_whitespace().unique(), sorter)
    };

    let mut string = String::with_capacity(str_vec.len() * 2);

    for str in str_vec {
        string.push_str(str);
        string.push(' ')
    }

    string.pop();
    string
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
use pretty_assertions::assert_eq;

#[test]
fn test_sort_classes_vec() {
    assert_eq!(
        sort_classes_vec(
            vec![
                "inline",
                "inline-block",
                "random-class",
                "py-2",
                "justify-end",
                "px-2",
                "flex"
            ]
            .into_iter(),
            &SORTER
        ),
        vec![
            "inline-block",
            "inline",
            "flex",
            "justify-end",
            "py-2",
            "px-2",
            "random-class",
        ]
    )
}

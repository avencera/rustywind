use itertools::Itertools;
use regex::Captures;

pub mod defaults;
pub mod options;

use defaults::{RE, SORTER};
use options::Options;

pub fn has_classes(file_contents: &str) -> bool {
    RE.is_match(file_contents)
}

pub fn sort_file_contents(file_contents: String, options: &Options) -> String {
    RE.replace_all(&file_contents, |caps: &Captures| {
        // caps[1] is class' or className"
        // caps[2] is the class list as a string
        // caps[3] is the last ' or ""
        format!(
            "{}{}{}",
            &caps[1],
            sort_classes(&caps[2], options),
            &caps[3]
        )
    })
    .to_string()
}

fn sort_classes(class_string: &str, options: &Options) -> String {
    let classes_vec = collect_classes(class_string, options);
    let sorted_classes_vec = sort_classes_vec(classes_vec);

    sorted_classes_vec.join(" ")
}

fn collect_classes(class_string: &str, options: &Options) -> Vec<String> {
    let classes = class_string.split(' ').map(|string| string.to_string());

    if options.allow_duplicates {
        classes.collect()
    } else {
        classes.unique().collect()
    }
}

fn sort_classes_vec(classes: Vec<String>) -> Vec<String> {
    let enumerated_classes = classes
        .into_iter()
        .map(|class| (String::from(&class), SORTER.get(&class)));

    let mut tailwind_classes: Vec<(String, &usize)> = vec![];
    let mut custom_classes: Vec<String> = vec![];

    for (class, maybe_size) in enumerated_classes {
        match maybe_size {
            Some(size) => tailwind_classes.push((class, size)),
            None => custom_classes.push(class),
        }
    }

    tailwind_classes.sort_by(|(_c1, i1), (_c2, i2)| i1.partial_cmp(i2).unwrap());

    let sorted_tailwind_classes: Vec<String> = tailwind_classes
        .into_iter()
        .map(|(class, _index)| class)
        .collect();

    [&sorted_tailwind_classes[..], &custom_classes[..]].concat()
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
                "justify-end",
                "flex"
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect()
        ),
        vec![
            "inline-block",
            "inline",
            "flex",
            "justify-end",
            "random-class",
        ]
    )
}

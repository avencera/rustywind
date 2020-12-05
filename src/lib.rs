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
        let classes = &caps[1];
        let sorted_classes = &sort_classes(classes, options);

        format!("{}", &caps[0].replace(classes, sorted_classes))
    })
    .to_string()
}

fn sort_classes(class_string: &str, options: &Options) -> String {
    if options.allow_duplicates {
        sort_classes_vec(class_string.split(' ')).join(" ")
    } else {
        sort_classes_vec(class_string.split(' ').unique()).join(" ")
    }
}

fn sort_classes_vec<'a>(classes: impl Iterator<Item = &'a str>) -> Vec<String> {
    let enumerated_classes = classes.map(|class| ((class.to_string()), SORTER.get(class)));

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

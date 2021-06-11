use std::borrow::Cow;

use itertools::Itertools;
use regex::Captures;

pub mod defaults;
pub mod options;

use defaults::{RE, SORTER};
use options::Options;

pub fn has_classes(file_contents: &str) -> bool {
    RE.is_match(file_contents)
}

pub fn sort_file_contents<'a>(file_contents: &'a str, options: &Options) -> Cow<'a, str> {
    RE.replace_all(file_contents, |caps: &Captures| {
        let classes = &caps[1];
        let sorted_classes = sort_classes(classes, options);

        caps[0].replace(classes, &sorted_classes)
    })
}

fn sort_classes(class_string: &str, options: &Options) -> String {
    let str_vec = if options.allow_duplicates {
        sort_classes_vec(class_string.split_ascii_whitespace())
    } else {
        sort_classes_vec(class_string.split_ascii_whitespace().unique())
    };

    let mut string = String::with_capacity(str_vec.len() * 2);

    for str in str_vec {
        string.push_str(str);
        string.push(' ')
    }

    string.pop();
    string
}

fn sort_classes_vec<'a>(classes: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let enumerated_classes = classes.map(|class| ((class), SORTER.get(class)));

    let mut tailwind_classes: Vec<(&str, &usize)> = vec![];
    let mut custom_classes: Vec<&str> = vec![];
    let mut classes_sm = vec![];
    let mut classes_md = vec![];
    let mut classes_lg = vec![];
    let mut classes_xl = vec![];
    let mut classes_2xl = vec![];
    let mut classes_3xl = vec![];
    let mut classes_4xl = vec![];
    let mut classes_5xl = vec![];
    let mut classes_6xl = vec![];

    for (class, maybe_size) in enumerated_classes {
        match maybe_size {
            Some(size) => tailwind_classes.push((class, size)),
            None => match class.as_bytes() {
                [b's', b'm', b':', ..] => classes_sm.push(class),
                [b'm', b'd', b':', ..] => classes_md.push(class),
                [b'l', b'g', b':', ..] => classes_lg.push(class),
                [b'x', b'l', b':', ..] => classes_xl.push(class),
                [b'2', b'x', b'l', b':', ..] => classes_2xl.push(class),
                [b'3', b'x', b'l', b':', ..] => classes_3xl.push(class),
                [b'4', b'x', b'l', b':', ..] => classes_4xl.push(class),
                [b'5', b'x', b'l', b':', ..] => classes_5xl.push(class),
                [b'6', b'x', b'l', b':', ..] => classes_6xl.push(class),
                _ => custom_classes.push(class),
            },
        }
    }

    tailwind_classes.sort_by_key(|&(_class, class_placement)| class_placement);

    let sorted_tailwind_classes: Vec<&str> = tailwind_classes
        .into_iter()
        .map(|(class, _index)| class)
        .collect();

    let (sorted_sm_classes, custom_classes) =
        sort_responsive_classes(classes_sm, custom_classes, 3);

    let (sorted_md_classes, custom_classes) =
        sort_responsive_classes(classes_md, custom_classes, 3);

    let (sorted_lg_classes, custom_classes) =
        sort_responsive_classes(classes_lg, custom_classes, 3);

    let (sorted_xl_classes, custom_classes) =
        sort_responsive_classes(classes_xl, custom_classes, 3);

    let (sorted_2xl_classes, custom_classes) =
        sort_responsive_classes(classes_2xl, custom_classes, 4);

    let (sorted_3xl_classes, custom_classes) =
        sort_responsive_classes(classes_3xl, custom_classes, 4);

    let (sorted_4xl_classes, custom_classes) =
        sort_responsive_classes(classes_4xl, custom_classes, 4);

    let (sorted_5xl_classes, custom_classes) =
        sort_responsive_classes(classes_5xl, custom_classes, 4);

    let (sorted_6xl_classes, custom_classes) =
        sort_responsive_classes(classes_6xl, custom_classes, 4);

    [
        &sorted_tailwind_classes[..],
        &sorted_sm_classes[..],
        &sorted_md_classes[..],
        &sorted_lg_classes[..],
        &sorted_xl_classes[..],
        &sorted_2xl_classes[..],
        &sorted_3xl_classes[..],
        &sorted_4xl_classes[..],
        &sorted_5xl_classes[..],
        &sorted_6xl_classes[..],
        &custom_classes[..],
    ]
    .concat()
}

fn sort_responsive_classes<'a>(
    classes: Vec<&'a str>,
    mut custom_classes: Vec<&'a str>,
    class_after: usize,
) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut tailwind_classes = Vec::with_capacity(classes.len());

    for class in classes {
        match SORTER.get(&class[class_after..]) {
            Some(class_placement) => tailwind_classes.push((class, class_placement)),
            None => custom_classes.push(class),
        }
    }

    tailwind_classes.sort_by_key(|&(_class, class_placement)| class_placement);

    let sorted_classes = tailwind_classes
        .into_iter()
        .map(|(class, _index)| class)
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
            .into_iter()
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

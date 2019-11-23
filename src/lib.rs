use regex::Regex;
#[macro_use]
extern crate lazy_static;
use regex::Captures;
mod sorter;

lazy_static! {
    static ref RE: Regex =
        Regex::new(r#"\b(class(?:Name)*\s*=\s*["'])([_a-zA-Z0-9\s\-:/]+)(["'])"#).unwrap();
}

pub fn sort_file_contents(file_contents: String) -> String {
    RE.replace_all(&file_contents, |caps: &Captures| {
        format!("{}{}{}", &caps[1], sort_classes(&caps[2]), &caps[3])
    })
    .to_string()
}

fn sort_classes(class_string: &str) -> String {
    let classes_vec = collect_classes(class_string);
    let sorted_classes = sort_classes_vec(classes_vec);

    sorted_classes.join(" ")
}

fn collect_classes(class_string: &str) -> Vec<String> {
    class_string
        .split(" ")
        .map(|string| string.to_string())
        .collect()
}

fn sort_classes_vec(classes: Vec<String>) -> Vec<String> {
    let enumerated_classes = classes
        .into_iter()
        .map(|class| (String::from(&class), sorter::SORTER.get(&class)));

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
            "flex",
            "justify-end",
            "inline-block",
            "inline",
            "random-class",
        ]
    )
}

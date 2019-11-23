extern crate ignore;
use ignore::WalkBuilder;
use regex::Regex;
use std::fs;
#[macro_use]
extern crate lazy_static;
use std::path::PathBuf;

lazy_static! {
    static ref RE: Regex =
        Regex::new(r#"\b(class(?:Name)*\s*=)\s*["']([_a-zA-Z0-9\s\-:]+)["']"#).unwrap();
}

pub fn run(dir: PathBuf) {
    let walker = WalkBuilder::new(dir)
        .build()
        .filter_map(Result::ok)
        .filter_map(|f| if f.path().is_dir() { None } else { Some(f) });

    for file in walker {
        let file_name = file.path();

        let contents =
            fs::read_to_string(file_name).expect("Something went wrong reading the file");

        let classes = collect_classes(contents);

        println!("{:?}", classes)
    }
}

fn collect_classes(string: String) -> Vec<Vec<String>> {
    RE.captures_iter(&string)
        .filter_map(|cap| match cap.get(2) {
            Some(capture) => Some(
                capture
                    .as_str()
                    .split(" ")
                    .map(|string| string.to_string())
                    .collect(),
            ),
            None => None,
        })
        .collect()
}

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_collect_classes() {
    assert_eq!(
        collect_classes(r#"<ul class='flex items-center md:pr-4 lg:pr-6'>"#.to_string()),
        vec![vec!["flex", "items-center", "md:pr-4", "lg:pr-6"]]
    )
}

#[test]
fn test_collect_classes_on_multiple_elements() {
    assert_eq!(
        collect_classes(
            r#"
        <div>
            <div class='inline inline-block random-class justify-content'>
                <ul class='flex items-center md:pr-4 lg:pr-6'>
            </div>
        </div>
        "#
            .to_string()
        ),
        vec![
            vec!["inline", "inline-block", "random-class", "justify-content"],
            vec!["flex", "items-center", "md:pr-4", "lg:pr-6"]
        ]
    )
}

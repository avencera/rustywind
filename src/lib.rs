extern crate ignore;
use ignore::WalkBuilder;
use regex::Regex;
use std::fs;
#[macro_use]
extern crate lazy_static;
use std::path::PathBuf;

lazy_static! {
    static ref RE: Regex =
        Regex::new(r###"\bclass(?:Name)*\s*=\s*(["']([_a-zA-Z0-9\s\-:]+)["'])"###).unwrap();
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

        println!(
            "FILENAME:{}\nWith text:\n\n{}\n\n--------------------------------------\n\n",
            file_name.display(),
            contents
        );
    }
}

pub fn find_and_replace_classes(string: String) -> String {
    "".to_string()
}

#[test]
fn test_regex_matches() {
    assert!(RE.is_match("<ul class=\"flex items-center md:pr-4 lg:pr-6\">"));
}

#[test]
fn test_regex_doesnt_match_incorrect() {
    assert_eq!(
        RE.is_match("<ul clasSs=\"flex items-center md:pr-4 lg:pr-6\">"),
        false
    );
}

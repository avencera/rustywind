use pretty_assertions::assert_eq;
use rustywind;
use rustywind::options::Options;
use rustywind::options::{FinderRegex, Sorter, WriteMode};
use std::path::Path;

fn default_options_for_test() -> Options {
    Options {
        write_mode: WriteMode::ToConsole,
        regex: FinderRegex::DefaultRegex,
        sorter: Sorter::DefaultSorter,
        starting_path: Path::new(".").to_owned(),
        search_paths: vec![Path::new(".").to_owned()],
        allow_duplicates: false,
    }
}

#[test]
fn test_sort_file_contents() {
    let file_contents = r#"
    <div>
        <div class='absolute relative flex inline-block inline random-class justify-items another-random-class'>
            <ul class='flex items-center md:pr-4 lg:pr-6'>
        </div>
    </div>
    "#.to_string();

    let expected_outcome = r#"
    <div>
        <div class='inline-block inline flex absolute relative random-class justify-items another-random-class'>
            <ul class='flex items-center md:pr-4 lg:pr-6'>
        </div>
    </div>
    "#.to_string();

    assert_eq!(
        rustywind::sort_file_contents(file_contents, &default_options_for_test()),
        expected_outcome
    )
}

#[test]
fn test_sort_file_contents_with_duplicates() {
    let file_contents = r#"
    <div>
        <div class='absolute relative flex flex flex flex inline-block inline random-class justify-items another-random-class'>
            <ul class='flex items-center md:pr-4 lg:pr-6'>
        </div>
    </div>
    "#.to_string();

    let expected_outcome = r#"
    <div>
        <div class='inline-block inline flex absolute relative random-class justify-items another-random-class'>
            <ul class='flex items-center md:pr-4 lg:pr-6'>
        </div>
    </div>
    "#.to_string();

    assert_eq!(
        rustywind::sort_file_contents(file_contents, &default_options_for_test()),
        expected_outcome
    )
}

#[test]
fn test_does_not_remove_duplicates_if_bool_set() {
    let file_contents = r#"
    <div>
        <div class='absolute relative flex flex flex flex inline-block inline random-class justify-items another-random-class'>
            <ul class='flex items-center md:pr-4 lg:pr-6'>
        </div>
    </div>
    "#.to_string();

    let expected_outcome = r#"
    <div>
        <div class='inline-block inline flex flex flex flex absolute relative random-class justify-items another-random-class'>
            <ul class='flex items-center md:pr-4 lg:pr-6'>
        </div>
    </div>
    "#.to_string();

    assert_eq!(
        rustywind::sort_file_contents(
            file_contents,
            &Options {
                allow_duplicates: true,
                ..default_options_for_test()
            }
        ),
        expected_outcome
    )
}

#[test]
fn test_returns_files_without_class_strings_as_is() {
    let file_contents = r#"
        This is to a represent any other normal file.
    "#
    .to_string();

    let expected_outcome = r#"
        This is to a represent any other normal file.
    "#
    .to_string();

    assert_eq!(
        rustywind::sort_file_contents(file_contents, &default_options_for_test()),
        expected_outcome
    )
}

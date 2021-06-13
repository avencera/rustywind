use pretty_assertions::assert_eq;
use rustywind;
use rustywind::options::Options;
use rustywind::options::{FinderRegex, Sorter, WriteMode};
use std::path::Path;

fn default_options_for_test() -> Options {
    Options {
        stdin: None,
        write_mode: WriteMode::ToConsole,
        regex: FinderRegex::DefaultRegex,
        sorter: Sorter::DefaultSorter,
        starting_paths: vec![Path::new(".").to_owned()],
        search_paths: vec![Path::new(".").to_owned()],
        allow_duplicates: false,
    }
}

#[cfg(test)]
mod tests {
    use rustywind::options::Options;

    use crate::default_options_for_test;

    #[test]
    fn test_sort_file_contents() {
        let file_contents = r#"
    <div>
        <div class='mt-4 mb-0.5 flex inline-block inline px-0.5 pt-10 random-class justify-items absolute relative another-random-class'>
            <ul class='flex items-center md:pr-4 lg:pr-6'>
        </div>
    </div>
    "#;

        let expected_outcome = r#"
    <div>
        <div class='inline-block inline flex absolute relative px-0.5 pt-10 mt-4 mb-0.5 random-class justify-items another-random-class'>
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
    "#;

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
    "#;

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
    "#;

        let expected_outcome = r#"
        This is to a represent any other normal file.
    "#
        .to_string();

        assert_eq!(
            rustywind::sort_file_contents(file_contents, &default_options_for_test()),
            expected_outcome
        )
    }
}

#[test]
fn test_multi_line_class_list() {
    let file_contents = r#"
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
    "#;

    let expected_outcome = r#"
    <div
      class="grid fixed top-0 right-0 z-20 grid-flow-col gap-2 justify-start my-12 mx-8 text-red-800 bg-red-50 rounded border border-red-100 shadow-2xl"
    >
      <!-- ... -->
    </div>
    "#
    .to_string();

    assert_eq!(
        rustywind::sort_file_contents(file_contents, &default_options_for_test()),
        expected_outcome
    )
}

#[test]
fn test_sort_file_contents_with_space_and_newline_separated_class_lists() {
    let file_contents = r#"
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
    "#;

    let expected_outcome = r#"
    <div
      class="grid fixed top-0 right-0 z-20 grid-flow-col gap-2 justify-start my-12 mx-8 text-red-800 bg-red-50 rounded border border-red-100 shadow-2xl"
    >
      <!-- ... -->
    </div>
    "#
    .to_string();

    assert_eq!(
        rustywind::sort_file_contents(file_contents, &default_options_for_test()),
        expected_outcome
    )
}

#[test]
fn test_sort_file_contents_with_spaces_newlines_and_custom_classes() {
    // Note the intentionally poor spacing. Rustywind isn't concerned so much about formatting, but
    // due to how whitespace is handled, it all ends up on one line as a side effect. This makes it
    // easier for formatters like Prettier to do their job.
    let file_contents = r#"
    <div class="m-2 grid-cols-4
            gap-1 foo
        border  theres-a-tab-here:	bar border-red-600
            ">
    </div>
    "#;

    let expected_outcome = r#"
    <div class="grid-cols-4 gap-1 m-2 border border-red-600 foo theres-a-tab-here: bar">
    </div>
    "#
    .to_string();

    assert_eq!(
        rustywind::sort_file_contents(file_contents, &default_options_for_test()),
        expected_outcome
    )
}

#[test]
fn test_sort_file_contents_with_responsive_classes() {
    let file_contents = r#"
<div>
    <div class='4xl:inline-block absolute xl:relative relative flex inline-block xl:absolute sm:relative sm:flex inline random-class justify-items another-random-class
    sm:absolute 4xl:flex xl:random-class sm:inline-block'>
        <ul class='flex items-center md:pr-4 lg:pr-6 xl:flex'>
    </div>
</div>
"#;

    let expected_outcome = r#"
<div>
    <div class='inline-block inline flex absolute relative sm:inline-block sm:flex sm:absolute sm:relative xl:absolute xl:relative 4xl:inline-block 4xl:flex random-class justify-items another-random-class xl:random-class'>
        <ul class='flex items-center md:pr-4 lg:pr-6 xl:flex'>
    </div>
</div>
"#.to_string();

    assert_eq!(
        rustywind::sort_file_contents(file_contents, &default_options_for_test()),
        expected_outcome
    )
}

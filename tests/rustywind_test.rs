use pretty_assertions::assert_eq;
use rustywind;

#[test]
fn test_sort_file_contents() {
    let file_contents = r#"
    <div>
        <div class='inline inline-block random-class justify-items flex absolute relative another-random-class'>
            <ul class='flex items-center md:pr-4 lg:pr-6'>
        </div>
    </div>
    "#.to_string();

    let expected_outcome = r#"
    <div>
        <div class='absolute relative flex inline-block inline random-class justify-items another-random-class'>
            <ul class='flex items-center md:pr-4 lg:pr-6'>
        </div>
    </div>
    "#.to_string();

    assert_eq!(
        rustywind::sort_file_contents(file_contents),
        expected_outcome
    )
}

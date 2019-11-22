extern crate ignore;
use ignore::WalkBuilder;
use std::fs;

fn main() {
    let walker = WalkBuilder::new("./")
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

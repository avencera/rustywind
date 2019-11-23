use ignore::WalkBuilder;
use std::env;
use std::fs;

fn main() {
    let current_dir = env::current_dir().expect("Connot run in current directory");

    let walker = WalkBuilder::new(current_dir)
        .build()
        .filter_map(Result::ok)
        .filter_map(|f| if f.path().is_dir() { None } else { Some(f) });

    for file in walker {
        let file_name = file.path();

        let contents =
            fs::read_to_string(file_name).expect("Something went wrong reading the file");

        let sorted_content = rustywind::sort_file_contents(contents);

        println!("\n\n\nFILENAME: {}\n\n\n", file_name.display());
        println!("{:?}", sorted_content)
    }
}

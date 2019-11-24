use clap::{App, Arg};
use ignore::WalkBuilder;
use std::fs;
use std::path::Path;

fn main() {
    let matches = App::new("Rusty Wind")
        .version("0.1.0")
        .author("Praveen Perera <praveen@avencera.com>")
        .about("Organize all your tailwind classes")
        .arg(
            Arg::with_name("file_or_dir")
                .value_name("PATH")
                .help("A file or directory to run on")
                .index(1)
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let file_or_dir = Path::new(
        matches
            .value_of("file_or_dir")
            .expect("Invalid PATH provided"),
    );

    let walker = WalkBuilder::new(&file_or_dir)
        .build()
        .filter_map(Result::ok)
        .filter_map(|f| if f.path().is_dir() { None } else { Some(f) });

    for file in walker {
        let file_path = file.path();

        match fs::read_to_string(file_path) {
            Ok(contents) => {
                if rustywind::has_classes(&contents) {
                    let sorted_content = rustywind::sort_file_contents(contents);

                    match fs::write(file_path, sorted_content.as_bytes()) {
                        Ok(_) => println!(" * {}", get_file_name(file_path, file_or_dir)),
                        Err(err) => {
                            println!("\nError: {:?}", err);
                            println!(
                                "Unable to to save file: {}",
                                get_file_name(file_path, file_or_dir)
                            );
                        }
                    }
                }
            }
            Err(_error) => (),
        }
    }
}

fn get_file_name(file_path: &Path, dir: &Path) -> String {
    file_path
        .strip_prefix(dir)
        .unwrap_or(file_path)
        .display()
        .to_string()
}

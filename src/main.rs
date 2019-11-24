use ignore::WalkBuilder;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let current_dir = env::current_dir().expect("Connot run in current directory");

    let walker = WalkBuilder::new(&current_dir)
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
                        Ok(_) => println!(" * {}", get_file_name(file_path, &current_dir)),
                        Err(err) => {
                            println!("\nError: {:?}", err);
                            println!(
                                "Unable to to save file: {}",
                                get_file_name(file_path, &current_dir)
                            );
                        }
                    }
                }
            }
            Err(_error) => (),
        }
    }
}

fn get_file_name(file_path: &Path, dir: &PathBuf) -> String {
    file_path
        .strip_prefix(dir)
        .unwrap_or(file_path)
        .display()
        .to_string()
}

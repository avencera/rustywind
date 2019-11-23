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

        match fs::read_to_string(file_name) {
            Ok(contents) => {
                if rustywind::has_classes(&contents) {
                    let sorted_content = rustywind::sort_file_contents(contents);

                    println!("\n\n--------------------------------------");
                    println!("Classes detected!");
                    println!("filename: {}\n", file_name.display());
                    println!("saving file.....");

                    match fs::write(file_name, sorted_content.as_bytes()) {
                        Ok(_) => println!("file: ({}) has been saved!", file_name.display()),
                        Err(err) => {
                            println!("Error: {:?}", err);
                            println!("Unable to to save file: {}", file_name.display());
                        }
                    }
                    println!("--------------------------------------\n\n");
                } else {
                    println!("--------------------------------------");
                    println!("No classes found in:  {}!", file_name.display());
                    println!("--------------------------------------");
                }
            }
            Err(error) => {
                println!("--------------------------------------");
                println!("Unable to read file: {}", file_name.display());
                println!("error: {:?}", error);
                println!("--------------------------------------\n");
            }
        }
    }
}

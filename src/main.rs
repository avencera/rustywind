use clap::{App, AppSettings, Arg, ArgMatches};
use ignore::WalkBuilder;
use indoc::indoc;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let matches = App::new("Rusty Wind")
        .version(clap::crate_version!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .author("Praveen Perera <praveen@avencera.com>")
        .about("\nOrganize all your tailwind classes")
        .usage(indoc!("
        Run rustywind with a path to get a list of files that will be changed
              rustywind . --dry-run

            If you want to reorganize all classes in place, and change the files run with the `--write` flag
              rustywind --write .
                         
            rustywind [FLAGS] <PATH>"))
        .arg(
            Arg::with_name("file_or_dir")
                .value_name("PATH")
                .help("A file or directory to run on")
                .index(1)
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("write")
                .long("write")
                .help("Changes the files in place with the reorganized classes"),
        )
        .arg(
            Arg::with_name("dry_run")
                .long("dry-run")
                .help("Prints out the new file content with the sorted classes to the terminal"),
        )
        .get_matches();

    let file_or_dir = Path::new(
        matches
            .value_of("file_or_dir")
            .expect("Invalid PATH provided"),
    );

    match (matches.is_present("write"), matches.is_present("dry_run")) {
        (_, true) => println!(
            "\ndry run mode activated: here is a list of files that \
             would be changed when you run with the --write flag"
        ),

        (true, false) => println!("\nwrite mode is active the following files are being saved:"),

        _ => println!(
            "\nprinting file contents to console, run with --write to save changes to files:"
        ),
    }

    let mut file_paths: Vec<PathBuf> = vec![];
    WalkBuilder::new(&file_or_dir)
        .build()
        .filter_map(Result::ok)
        .filter(|f| f.path().is_file())
        .for_each(|file| file_paths.push(file.path().to_owned()));

    file_paths
        .par_iter()
        .for_each(|file_path| run_on_file_paths(&file_path, &matches, file_or_dir))
}

fn run_on_file_paths(file_path: &Path, matches: &ArgMatches, file_or_dir: &Path) {
    match fs::read_to_string(file_path) {
        Ok(contents) => {
            if rustywind::has_classes(&contents) {
                let sorted_content = rustywind::sort_file_contents(contents);

                match (matches.is_present("write"), matches.is_present("dry_run")) {
                    (_, true) => print_file_name(file_path, file_or_dir),
                    (true, false) => write_to_file(file_path, file_or_dir, &sorted_content),
                    _ => print_file_contents(&sorted_content),
                }
            }
        }
        Err(_error) => (),
    }
}

fn write_to_file(file_path: &Path, file_or_dir: &Path, sorted_contents: &str) {
    match fs::write(file_path, sorted_contents.as_bytes()) {
        Ok(_) => print_file_name(file_path, file_or_dir),
        Err(err) => {
            println!("\nError: {:?}", err);
            println!(
                "Unable to to save file: {}",
                get_file_name(file_path, file_or_dir)
            );
        }
    }
}

fn print_file_name(file_path: &Path, file_or_dir: &Path) {
    println!("  * {}", get_file_name(file_path, file_or_dir))
}

fn get_file_name(file_path: &Path, dir: &Path) -> String {
    file_path
        .strip_prefix(dir)
        .unwrap_or(file_path)
        .display()
        .to_string()
}

fn print_file_contents(file_contents: &str) {
    println!("\n\n{}\n\n", file_contents)
}

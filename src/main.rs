use clap::{App, AppSettings, Arg};
use indoc::indoc;
use rayon::prelude::*;
use rustywind::options::{Options, WriteMode};
use std::fs;
use std::path::Path;

fn main() {
    let matches = App::new("RustyWind")
        .version(clap::crate_version!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .author("Praveen Perera <praveen@avencera.com>")
        .about("\nOrganize all your tailwind classes")
        .usage(indoc!("
        Run rustywind with a path to get a list of files that will be changed
              rustywind . --dry-run

            If you want to reorganize all classes in place, and change the files run with the `--write` flag
              rustywind --write .

            If you want to run it on your STDIN, you can do:
              echo \"<FILE CONTENTS>\" | rustywind --stdin
                         
            rustywind [FLAGS] <PATH>"))
        .arg(
            Arg::with_name("file_or_dir")
                .value_name("PATH")
                .help("A file or directory to run on")
                .conflicts_with("stdin")
                .index(1)
                .required_unless("stdin")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("stdin")
                .long("stdin")
                .conflicts_with_all(&["write", "file_or_dir", "dry-run"])
                .required_unless("file_or_dir")
                .help("Uses stdin instead of a file or folder")
        )
        .arg(
            Arg::with_name("write")
                .long("write")
                .conflicts_with_all(&["stdin", "dry-run"])
                .help("Changes the files in place with the reorganized classes"),
        )
        .arg(
            Arg::with_name("dry_run")
                .long("dry-run")
                .conflicts_with_all(&["stdin", "write"])
                .help("Prints out the new file content with the sorted classes to the terminal"),
        )
        .arg(
            Arg::with_name("allow-duplicates")
                .long("allow-duplicates")
                .help("When set, rustywind will not delete duplicated classes"),
        )
        .get_matches();

    let options = Options::new_from_matches(&matches);

    match &options.write_mode {
        WriteMode::ToStdOut => (),
        WriteMode::DryRun => println!(
            "\ndry run mode activated: here is a list of files that \
             would be changed when you run with the --write flag"
        ),

        WriteMode::ToFile => {
            println!("\nwrite mode is active the following files are being saved:")
        }

        WriteMode::ToConsole => println!(
            "\nprinting file contents to console, run with --write to save changes to files:"
        ),
    }

    match &options.write_mode {
        WriteMode::ToStdOut => {
            let contents = options.stdin.clone().unwrap_or_else(|| "".to_string());

            if rustywind::has_classes(&contents) {
                let sorted_content = rustywind::sort_file_contents(contents, &options);
                print!("{}", sorted_content)
            } else {
                print!("{}", contents);
                eprint!("[ERROR] No classes were found in STDIN");
                std::process::exit(2)
            }
        }
        _ => options
            .search_paths
            .par_iter()
            .for_each(|file_path| run_on_file_paths(&file_path, &options)),
    }
}

fn run_on_file_paths(file_path: &Path, options: &Options) {
    match fs::read_to_string(file_path) {
        Ok(contents) => {
            if rustywind::has_classes(&contents) {
                let sorted_content = rustywind::sort_file_contents(contents, options);

                match &options.write_mode {
                    WriteMode::ToStdOut => (),
                    WriteMode::DryRun => print_file_name(file_path, options),
                    WriteMode::ToFile => write_to_file(file_path, &sorted_content, options),
                    WriteMode::ToConsole => print_file_contents(&sorted_content),
                }
            }
        }
        Err(_error) => (),
    }
}

fn write_to_file(file_path: &Path, sorted_contents: &str, options: &Options) {
    match fs::write(file_path, sorted_contents.as_bytes()) {
        Ok(_) => print_file_name(file_path, options),
        Err(err) => {
            eprintln!("\nError: {:?}", err);
            eprintln!(
                "Unable to to save file: {}",
                get_file_name(file_path, &options.starting_path)
            );
        }
    }
}

fn print_file_name(file_path: &Path, options: &Options) {
    println!("  * {}", get_file_name(file_path, &options.starting_path))
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

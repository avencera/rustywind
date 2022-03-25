use clap::{App, AppSettings, Arg};
use indoc::indoc;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use rustywind::options::{Options, WriteMode};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

static EXIT_ERROR: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

fn main() {
    let matches = App::new("RustyWind")
        .version(clap::crate_version!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .author("Praveen Perera <praveen@avencera.com>")
        .about("\nOrganize all your tailwind classes")
        .override_usage(indoc!("
        Run rustywind with a path to get a list of files that will be changed
              rustywind . --dry-run

            If you want to reorganize all classes in place, and change the files run with the `--write` flag
              rustywind --write .

            To print only the file names that would be changed run with the `--check-formatted` flag
              rustywind --check-formatted .

            If you want to run it on your STDIN, you can do:
              echo \"<FILE CONTENTS>\" | rustywind --stdin
                         
            rustywind [FLAGS] <PATH>"))
        .arg(
            Arg::new("file_or_dir")
                .value_name("PATH")
                .help("A file or directory to run on")
                .conflicts_with("stdin")
                .index(1)
                .required_unless_present("stdin")
                .multiple_occurrences(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("stdin")
                .long("stdin")
                .conflicts_with_all(&["write", "file_or_dir", "dry_run"])
                .required_unless_present("file_or_dir")
                .help("Uses stdin instead of a file or folder")
        )
        .arg(
            Arg::new("write")
                .long("write")
                .conflicts_with_all(&["stdin", "dry_run", "check_formatted"])
                .help("Changes the files in place with the reorganized classes"),
        )
        .arg(
            Arg::new("dry_run")
                .long("dry-run")
                .conflicts_with_all(&["stdin", "write", "check_formatted"])
                .help("Prints out the new file content with the sorted classes to the terminal"),
        )
        .arg(
            Arg::new("check_formatted")
                .long("check-formatted")
                .conflicts_with_all(&["stdin", "write", "dry_run"])
                .help("Prints out the new file content with the sorted classes to the terminal")
        )
        .arg(
            Arg::new("allow-duplicates")
                .long("allow-duplicates")
                .help("When set, rustywind will not delete duplicated classes"),
        )
        .arg(
            Arg::new("ignored_files")
                .long("ignored-files")
                .help("When set, rustywind will ignore this list of files")
                .takes_value(true),
        )
        .arg(
            Arg::new("custom-regex")
                .long("custom-regex")
                .help("Uses a custom regex instead of default one")
                .takes_value(true),
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
            println!("\nwrite mode is active the following files are being saved:");
        }

        WriteMode::ToConsole => println!(
            "\nprinting file contents to console, run with --write to save changes to files:"
        ),
        WriteMode::CheckFormatted => println!("\nonly printing changed files"),
    }

    if let WriteMode::ToStdOut = &options.write_mode {
        let contents = options.stdin.clone().unwrap_or_else(|| "".to_string());

        if rustywind::has_classes(&contents, &options) {
            let sorted_content = rustywind::sort_file_contents(&contents, &options);
            print!("{}", sorted_content);
        } else {
            print!("{}", contents);
            eprint!("[WARN] No classes were found in STDIN");
        }
    } else {
        options
            .search_paths
            .par_iter()
            .for_each(|file_path| run_on_file_paths(file_path, &options));

        if EXIT_ERROR.load(Ordering::Relaxed) {
            std::process::exit(1);
        }
    }
}

fn run_on_file_paths(file_path: &Path, options: &Options) {
    match fs::read_to_string(file_path) {
        Ok(contents) => {
            if rustywind::has_classes(&contents, options) {
                let sorted_content = rustywind::sort_file_contents(&contents, options);

                match &options.write_mode {
                    WriteMode::ToStdOut => (),
                    WriteMode::DryRun => print_file_name(file_path, options),
                    WriteMode::ToFile => write_to_file(file_path, &sorted_content, options),
                    WriteMode::ToConsole => print_file_contents(&sorted_content),
                    WriteMode::CheckFormatted => {
                        print_changed_files(file_path, &sorted_content, &contents, options);
                    }
                }
            }
        }
        Err(_error) => (),
    }
}

fn print_changed_files(
    file_path: &Path,
    sorted_content: &str,
    original_content: &str,
    options: &Options,
) {
    if sorted_content != original_content {
        if !EXIT_ERROR.load(Ordering::Relaxed) {
            EXIT_ERROR.store(true, Ordering::Relaxed);
        }

        if !should_ignore_current_file(&options.ignored_files, &current_file_path) {
            let file_name = get_file_name(file_path, &options.starting_paths);
            eprintln!("  * [UNFORMATTED FILE] {file_name}")
        }
    }
}

/// Return a boolean indicating whether the file should be ignored
fn should_ignore_current_file(ignored_files: &HashSet<String>, current_file: &str) -> bool {
    current_file
        .split('/')
        .last()
        .map(|file_name_clean| ignored_files.contains(file_name_clean))
        .unwrap_or(false)
}

fn write_to_file(file_path: &Path, sorted_contents: &str, options: &Options) {
    match fs::write(file_path, sorted_contents.as_bytes()) {
        Ok(_) => print_file_name(file_path, options),
        Err(err) => {
            eprintln!("\nError: {:?}", err);
            eprintln!(
                "Unable to to save file: {}",
                get_file_name(file_path, &options.starting_paths)
            );
        }
    }
}

fn print_file_name(file_path: &Path, options: &Options) {
    println!("  * {}", get_file_name(file_path, &options.starting_paths));
}

fn get_file_name(file_path: &Path, starting_paths: &[PathBuf]) -> String {
    for starting_path in starting_paths {
        if starting_path.is_dir() && file_path.starts_with(starting_path) {
            let dir = starting_path.parent().unwrap_or(starting_path);

            return file_path
                .strip_prefix(dir)
                .unwrap_or(file_path)
                .display()
                .to_string();
        }
    }

    file_path.display().to_string()
}

fn print_file_contents(file_contents: &str) {
    println!("\n\n{}\n\n", file_contents);
}

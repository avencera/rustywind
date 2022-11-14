pub mod consts;
pub mod defaults;
pub mod options;
pub mod utils;

use clap::Parser;
use eyre::Result;
use indoc::indoc;
use once_cell::sync::Lazy;
use options::{Options, WriteMode};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

static EXIT_ERROR: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

#[derive(Parser, Debug)]
#[clap(name = "RustyWind", author, version, about, long_about = None)]
#[clap(args_override_self = true, arg_required_else_help = true)]
#[clap(override_usage = indoc!("
Run rustywind with a path to get a list of files that will be changed
      rustywind . --dry-run

    If you want to reorganize all classes in place, and change the files run with the `--write` flag
      rustywind --write .

    To print only the file names that would be changed run with the `--check-formatted` flag
      rustywind --check-formatted .

    If you want to run it on your STDIN, you can do:
      echo \"<FILE CONTENTS>\" | rustywind --stdin
                 
    rustywind [FLAGS] <PATH>"))]
pub struct Cli {
    #[clap(
        name = "file-or-dir",
        help = "A file or directory to run on",
        value_name = "PATH",
        required_unless_present = "stdin"
    )]
    file_or_dir: Vec<String>,

    #[clap(
        long,
        help = "Uses stdin instead of a file or folder",
        conflicts_with_all = &["write", "file-or-dir", "dry-run"],
        required_unless_present = "file-or-dir",
    )]
    stdin: bool,

    #[clap(
        long,
        help = "Changes the files in place with the reorganized classes",
        conflicts_with_all = &["stdin", "dry-run", "check-formatted"],
    )]
    write: bool,

    #[clap(
        long,
        help = "Prints out the new file content with the sorted classes to the terminal",
        conflicts_with_all = &["stdin", "write", "check-formatted"]
    )]
    dry_run: bool,

    #[clap(
        long,
        help = "Checks if the files are already formatted, exits with 1 if not formatted",
        conflicts_with_all = &["stdin", "write", "dry-run"]

    )]
    check_formatted: bool,

    #[clap(long, help = "When set, RustyWind will not delete duplicated classes")]
    allow_duplicates: bool,

    #[clap(
        long,
        help = "When set, RustyWind will use the config file to derive configurations. \
        The config file current only supports json with one property sortOrder, \
        e.g. { \"sortOrder\": [\"class1\", ...] }"
    )]
    config_file: Option<String>,

    #[clap(long, help = "When set, RustyWind will ignore this list of files")]
    ignored_files: Option<Vec<String>>,

    #[clap(long, help = "Uses a custom regex instead of default one")]
    custom_regex: Option<String>,
}

fn main() -> Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let cli = Cli::parse();
    let options = Options::new_from_cli(cli)?;

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
        let contents = options.stdin.clone().unwrap_or_default();

        if utils::has_classes(&contents, &options) {
            let sorted_content = utils::sort_file_contents(&contents, &options);
            print!("{sorted_content}");
        } else {
            print!("{contents}");
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

    Ok(())
}

fn run_on_file_paths(file_path: &Path, options: &Options) {
    // if the file is in the ignored_files list return early
    if should_ignore_current_file(&options.ignored_files, file_path) {
        log::debug!("file path {file_path:#?} found in ignored_files, will not sort");
        return;
    }

    match fs::read_to_string(file_path) {
        Ok(contents) => {
            if utils::has_classes(&contents, options) {
                let sorted_content = utils::sort_file_contents(&contents, options);

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

        if !should_ignore_current_file(&options.ignored_files, file_path) {
            let file_name = get_file_name(file_path, &options.starting_paths);
            eprintln!("  * [UNFORMATTED FILE] {file_name}")
        }
    }
}

/// Return a boolean indicating whether the file should be ignored
fn should_ignore_current_file(ignored_files: &HashSet<PathBuf>, current_file: &Path) -> bool {
    if ignored_files.is_empty() {
        // if the ignored_files is empty no need to do any more work
        false
    } else {
        current_file
            .canonicalize()
            .map(|path| ignored_files.contains(&path))
            .unwrap_or(false)
    }
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

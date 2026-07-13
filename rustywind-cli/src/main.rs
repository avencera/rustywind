mod cli;
mod options;

use ahash::AHashSet as HashSet;
use clap::Parser;
use eyre::Result;
use indoc::indoc;
use once_cell::sync::Lazy;
use options::Options;
use options::WriteMode;
use rayon::ThreadPoolBuilder;
use rayon::iter::IntoParallelRefIterator as _;
use rayon::iter::ParallelIterator;
use rustywind_core::SourceDocument;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

static EXIT_ERROR: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
static GRAY: Lazy<colored::CustomColor> = Lazy::new(|| colored::CustomColor::new(120, 120, 120));

#[derive(Parser, Debug)]
#[clap(name = "RustyWind", author, version, about, long_about = None)]
#[command(styles=cli::get_styles())]
#[clap(args_override_self = true, arg_required_else_help = true)]
#[clap(override_usage = indoc!("
    rustywind [OPTIONS] [PATH]...

    Run rustywind with a path to get a list of files that will be changed
      rustywind . --dry-run

    If you want to reorganize all classes in place, and change the files run with the `--write` flag
      rustywind --write .

    To print only the file names that would be changed run with the `--check-formatted` flag
      rustywind --check-formatted .

    If you want to run it on your STDIN, you can do:
      echo \"<FILE CONTENTS>\" | rustywind --stdin"))]
pub struct Cli {
    /// A file or directory to run on.
    #[arg(value_name = "PATH", required_unless_present = "stdin")]
    file_or_dir: Vec<String>,
    /// Uses stdin instead of a file or folder.
    #[arg(
        long,
        conflicts_with_all = &["write", "file_or_dir", "dry_run"],
        required_unless_present = "file_or_dir",
    )]
    stdin: bool,
    /// Changes the files in place with the reorganized classes.
    #[arg(long, conflicts_with_all = &["stdin", "dry_run", "check_formatted"])]
    write: bool,
    /// Prints out the new file content with the sorted classes to the terminal.
    #[arg(long, conflicts_with_all = &["stdin", "write", "check_formatted"])]
    dry_run: bool,
    /// Checks if the files are already formatted, exits with 1 if not formatted.
    #[arg(long, conflicts_with_all = &["stdin", "write", "dry_run"])]
    check_formatted: bool,
    /// When set, RustyWind will not delete duplicated classes.
    #[arg(long)]
    allow_duplicates: bool,
    /// When set, RustyWind will use the config file to derive configurations. The config file
    /// current only supports json with one property sortOrder, e.g.
    /// { "sortOrder": ["class1", ...] }.
    #[arg(long, conflicts_with_all = &["output_css_file"])]
    config_file: Option<String>,
    /// When set RustyWind will determine the sort order by the order the class appear in the the given css file.
    #[arg(long, conflicts_with_all = &["config_file", "vite_css"])]
    output_css_file: Option<String>,
    /// When set RustyWind will determine the sort order by the order the class appear in the CSS file that vite generates.
    ///
    /// Please provide the full URL to the CSS file ex: `rustywind --vite-css "http://127.0.0.1:5173/src/assets/main.css" . --dry-run`
    ///
    /// Note: This option is experimental and may be removed in the future.
    #[arg(long, conflicts_with_all = &["config_file", "output_css_file"])]
    vite_css: Option<String>,

    /// When set, RustyWind will skip SSL verification for the vite_css option.
    #[arg(long, conflicts_with_all = &["config_file", "output_css_file"])]
    skip_ssl_verification: bool,

    /// When set, RustyWind will ignore this list of files
    #[arg(long)]
    ignored_files: Option<Vec<String>>,
    /// Uses a custom regex whose `classes` or first capture contains the class list
    #[arg(long)]
    custom_regex: Option<String>,
    /// Specifies how individual classes are wrapped
    #[arg(long)]
    class_wrapping: Option<options::CliClassWrapping>,
    /// Tailwind prefix used when sorting classes, e.g. tw for tw: or tw- classes
    #[arg(short = 'p', long)]
    tailwind_prefix: Option<String>,
    /// Source language to use for all inputs, overriding filename inference
    #[arg(short = 'l', long, value_name = "LANGUAGE")]
    language: Option<options::CliSourceLanguage>,
    /// Filename used to infer the source language of stdin
    #[arg(
        short = 'f',
        long,
        value_name = "PATH",
        requires = "stdin",
        conflicts_with = "file_or_dir"
    )]
    stdin_filename: Option<PathBuf>,
    /// Do not print log messages
    #[arg(long, default_value = "false", conflicts_with_all = &["dry_run"])]
    quiet: bool,
}

fn main() -> Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let cli = Cli::parse();

    let mut options = Options::new_from_cli(cli)?;

    let search_paths = std::mem::take(&mut options.search_paths);

    let options = Arc::new(options);
    let rustywind = &options.rustywind;

    match &options.write_mode {
        WriteMode::ToStdOut => (),
        WriteMode::DryRun => println!(
            "\ndry run mode activated: here is a list of files that \
             would be changed when you run with the --write flag"
        ),

        WriteMode::ToFile => {
            if !options.quiet {
                println!("\nwrite mode is active the following files are being saved:");
            }
        }

        WriteMode::ToConsole => println!(
            "\nprinting file contents to console, run with --write to save changes to files:"
        ),
        WriteMode::CheckFormatted => println!("\nonly printing changed files"),
    }

    if let WriteMode::ToStdOut = &options.write_mode {
        let contents = options.stdin.clone().unwrap_or_default();
        let language = options.stdin_source_language();

        if rustywind.has_classes(SourceDocument::new(&contents, language)) {
            let sorted_content = rustywind.sort_document(SourceDocument::new(&contents, language));
            print!("{sorted_content}");
        } else {
            print!("{contents}");
            eprint!("[WARN] No classes were found in STDIN");
        }
    } else {
        let available_parallelism = std::thread::available_parallelism()
            .map(|x| x.get())
            .unwrap_or(1);

        #[cfg(target_os = "macos")]
        let threads = available_parallelism.min(4);

        #[cfg(not(target_os = "macos"))]
        let threads = available_parallelism;

        ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .expect("failed to build thread pool");

        search_paths
            .par_iter()
            .for_each(|f| run_on_file_path(f, &options));

        // after running on all files, if there was an error, exit with 1
        if EXIT_ERROR.load(Ordering::Relaxed) {
            std::process::exit(1)
        }
    }

    Ok(())
}

pub fn run_on_file_path(file_path: &Path, options: &Options) {
    // if the file is in the ignored_files list return early
    if should_ignore_current_file(&options.ignored_files, file_path) {
        log::debug!("file path {file_path:#?} found in ignored_files, will not sort");
        return;
    }

    let rustywind = &options.rustywind;
    match std::fs::read_to_string(file_path) {
        Ok(contents) => {
            let language = options.source_language_for_path(file_path);

            if rustywind.has_classes(SourceDocument::new(&contents, language)) {
                let sorted_content =
                    rustywind.sort_document(SourceDocument::new(&contents, language));
                let contents_changed = sorted_content != contents;

                match (contents_changed, &options.write_mode) {
                    (_, WriteMode::ToStdOut) => (),
                    (_, WriteMode::DryRun) => print_file_name(file_path, contents_changed, options),

                    (true, WriteMode::ToFile) => write_to_file(file_path, &sorted_content, options),
                    (false, WriteMode::ToFile) => {
                        print_file_name(file_path, contents_changed, options)
                    }

                    // For now print the file contents to the console even if it hasn't changed to
                    // keep consistent with how rustywind has always worked. But in a later
                    // breaking release add a `--print-unchanged` flag to get the old behavior back
                    // but default to not printing unchanged files.
                    (true, WriteMode::ToConsole) => print_file_contents(&sorted_content),
                    (false, WriteMode::ToConsole) => print_file_contents(&sorted_content),

                    (contents_changed, WriteMode::CheckFormatted) => {
                        print_changed_files(file_path, contents_changed, options);
                    }
                }
            }
        }
        Err(_error) => (),
    }
}

fn print_changed_files(file_path: &Path, contents_changed: bool, options: &Options) {
    if contents_changed {
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
        Ok(_) => print_file_name(file_path, true, options),
        Err(err) => {
            eprintln!("\nError: {:?}", err);
            eprintln!(
                "Unable to to save file: {}",
                get_file_name(file_path, &options.starting_paths)
            );
        }
    }
}

fn print_file_name(file_path: &Path, contents_changed: bool, options: &Options) {
    use colored::*;

    if !options.quiet {
        let line = format!("  * {}", get_file_name(file_path, &options.starting_paths));

        if contents_changed {
            println!("{}", line);
        } else {
            eprintln!("{}", line.custom_color(*GRAY));
        }
    }
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::{CommandFactory, Parser};

    use super::{Cli, options::CliSourceLanguage};

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_explicit_source_language() {
        let cli = Cli::try_parse_from(["rustywind", "-l", "svelte", "index.html"])
            .expect("explicit language should parse");

        assert!(matches!(cli.language, Some(CliSourceLanguage::Svelte)));
    }

    #[test]
    fn parses_stdin_filename_for_language_inference() {
        let cli = Cli::try_parse_from(["rustywind", "--stdin", "-f", "components/card.blade.php"])
            .expect("stdin filename should parse with stdin mode");

        assert_eq!(
            cli.stdin_filename,
            Some(PathBuf::from("components/card.blade.php"))
        );
    }

    #[test]
    fn rejects_stdin_filename_without_stdin_mode() {
        assert!(
            Cli::try_parse_from(["rustywind", "--stdin-filename", "input.erb", "input.html"])
                .is_err()
        );
    }

    #[test]
    fn rejects_unknown_source_language() {
        assert!(Cli::try_parse_from(["rustywind", "--language", "unknown", "input.html"]).is_err());
    }

    #[test]
    fn parses_tailwind_prefix_short_option() {
        let cli = Cli::try_parse_from(["rustywind", "-p", "tw", "index.html"])
            .expect("tailwind prefix should parse");

        assert_eq!(cli.tailwind_prefix.as_deref(), Some("tw"));
    }
}

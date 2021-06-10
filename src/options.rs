use clap::ArgMatches;
use ignore::WalkBuilder;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum WriteMode {
    ToFile,
    DryRun,
    ToConsole,
    ToStdOut,
}

#[derive(Debug)]
pub enum FinderRegex {
    DefaultRegex,
}

#[derive(Debug)]
pub enum Sorter {
    DefaultSorter,
}

#[derive(Debug)]
pub struct Options {
    pub stdin: Option<String>,
    pub write_mode: WriteMode,
    pub regex: FinderRegex,
    pub sorter: Sorter,
    pub starting_paths: Vec<PathBuf>,
    pub allow_duplicates: bool,
    pub search_paths: Vec<PathBuf>,
}

impl Options {
    pub fn new_from_matches(matches: &ArgMatches) -> Options {
        match matches.is_present("stdin") {
            true => {
                let mut buffer = String::new();
                let mut stdin = std::io::stdin(); // We get `Stdin` here.
                stdin.read_to_string(&mut buffer).unwrap();

                Options {
                    stdin: Some(buffer.trim().to_string()),
                    write_mode: WriteMode::ToStdOut,
                    regex: FinderRegex::DefaultRegex,
                    sorter: Sorter::DefaultSorter,
                    starting_paths: vec![PathBuf::new()],
                    allow_duplicates: matches.is_present("allow-duplicates"),
                    search_paths: vec![],
                }
            }
            false => {
                let starting_paths = get_starting_path_from_matches(matches);
                let search_paths = get_search_paths_from_starting_paths(&starting_paths);

                Options {
                    stdin: None,
                    starting_paths,
                    search_paths,
                    write_mode: get_write_mode_from_matches(matches),
                    regex: FinderRegex::DefaultRegex,
                    sorter: Sorter::DefaultSorter,
                    allow_duplicates: matches.is_present("allow-duplicates"),
                }
            }
        }
    }
}

fn get_starting_path_from_matches(matches: &ArgMatches) -> Vec<PathBuf> {
    matches
        .values_of("file_or_dir")
        .expect("Invalid PATH provided")
        .map(|path| Path::new(path).to_owned())
        .collect()
}

fn get_write_mode_from_matches(matches: &ArgMatches) -> WriteMode {
    match (matches.is_present("write"), matches.is_present("dry_run")) {
        (_, true) => WriteMode::DryRun,
        (true, false) => WriteMode::ToFile,
        _ => WriteMode::ToConsole,
    }
}

fn get_search_paths_from_starting_paths(starting_paths: &[PathBuf]) -> Vec<PathBuf> {
    starting_paths
        .iter()
        .flat_map(|starting_path| {
            WalkBuilder::new(starting_path)
                .build()
                .filter_map(Result::ok)
                .filter(|f| f.path().is_file())
                .map(|file| file.path().to_owned())
        })
        .collect()
}

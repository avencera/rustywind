use clap::ArgMatches;
use ignore::WalkBuilder;
use itertools::Itertools;
use regex::Regex;
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum WriteMode {
    ToFile,
    DryRun,
    ToConsole,
    ToStdOut,
    CheckFormatted,
}

#[derive(Debug)]
pub enum FinderRegex {
    DefaultRegex,
    CustomRegex(Regex),
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
    pub ignored_files: HashSet<String>,
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
                    regex: get_custom_regex_from_matches(matches),
                    sorter: Sorter::DefaultSorter,
                    starting_paths: vec![PathBuf::new()],
                    allow_duplicates: matches.is_present("allow-duplicates"),
                    search_paths: vec![],
                    ignored_files: get_ignored_files_from_matches(matches),
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
                    regex: get_custom_regex_from_matches(matches),
                    sorter: Sorter::DefaultSorter,
                    allow_duplicates: matches.is_present("allow-duplicates"),
                    ignored_files: get_ignored_files_from_matches(matches),
                }
            }
        }
    }
}

fn get_custom_regex_from_matches(matches: &ArgMatches) -> FinderRegex {
    match matches.is_present("custom-regex") {
        true => {
            let string = matches
                .value_of("custom-regex")
                .expect("Invalid regex string provided");

            let regex = Regex::new(string).unwrap();

            FinderRegex::CustomRegex(regex)
        }

        false => FinderRegex::DefaultRegex,
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
    if matches.is_present("dry_run") {
        WriteMode::DryRun
    } else if matches.is_present("write") {
        WriteMode::ToFile
    } else if matches.is_present("check_formatted") {
        WriteMode::CheckFormatted
    } else {
        WriteMode::DryRun
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
        .unique()
        .collect()
}

fn get_ignored_files_from_matches(matches: &ArgMatches) -> HashSet<String> {
    match matches.values_of("ignored_files") {
        Some(values) => values.map(|s| s.to_string()).collect::<HashSet<String>>(),
        None => HashSet::new(),
    }
}

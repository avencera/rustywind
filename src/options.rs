use clap::ArgMatches;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum WriteMode {
    ToFile,
    DryRun,
    ToConsole,
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
    pub write_mode: WriteMode,
    pub regex: FinderRegex,
    pub sorter: Sorter,
    pub path: PathBuf,
    pub allow_duplicates: bool,
}

impl Options {
    pub fn new_from_matches(matches: &ArgMatches) -> Options {
        Options {
            write_mode: get_write_mode_from_matches(matches),
            regex: FinderRegex::DefaultRegex,
            sorter: Sorter::DefaultSorter,
            path: Path::new(
                matches
                    .value_of("file_or_dir")
                    .expect("Invalid PATH provided"),
            )
            .to_owned(),
            allow_duplicates: matches.is_present("allow-duplicates"),
        }
    }
}

fn get_write_mode_from_matches(matches: &ArgMatches) -> WriteMode {
    match (matches.is_present("write"), matches.is_present("dry_run")) {
        (_, true) => WriteMode::DryRun,
        (true, false) => WriteMode::ToFile,
        _ => WriteMode::ToConsole,
    }
}

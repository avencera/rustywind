use clap::ArgMatches;
use crossbeam_channel as channel;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum Message {
    FoundPath(Result<ignore::DirEntry, ignore::Error>),
    DoneScanning,
}

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
    pub starting_path: PathBuf,
    pub allow_duplicates: bool,
    pub search_paths: Vec<PathBuf>,
}

impl Options {
    pub fn new_from_matches(matches: &ArgMatches) -> Options {
        let starting_path = get_starting_path_from_matches(matches);

        Options {
            write_mode: get_write_mode_from_matches(matches),
            regex: FinderRegex::DefaultRegex,
            sorter: Sorter::DefaultSorter,
            starting_path: starting_path.to_owned(),
            allow_duplicates: matches.is_present("allow-duplicates"),
            search_paths: get_search_paths_from_starting_path(&starting_path),
        }
    }
}

fn get_starting_path_from_matches(matches: &ArgMatches) -> PathBuf {
    Path::new(
        matches
            .value_of("file_or_dir")
            .expect("Invalid PATH provided"),
    )
    .to_owned()
}

fn get_write_mode_from_matches(matches: &ArgMatches) -> WriteMode {
    match (matches.is_present("write"), matches.is_present("dry_run")) {
        (_, true) => WriteMode::DryRun,
        (true, false) => WriteMode::ToFile,
        _ => WriteMode::ToConsole,
    }
}

fn get_search_paths_from_starting_path(starting_path: &Path) -> Vec<PathBuf> {
    let (tx, rx) = channel::unbounded();
    let mut paths: Vec<ignore::DirEntry> = vec![];

    WalkBuilder::new(starting_path).build_parallel().run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            use ignore::WalkState::*;

            tx.send(Message::FoundPath(result)).unwrap();
            Continue
        })
    });

    tx.send(Message::DoneScanning).unwrap();

    loop {
        match rx.recv() {
            Ok(Message::DoneScanning) => break,
            Ok(Message::FoundPath(Ok(path))) => paths.push(path),
            _ => (),
        }
    }

    paths
        .iter()
        .filter(|f| f.path().is_file())
        .map(|file| file.path().to_owned())
        .collect()
}

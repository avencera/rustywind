use color_eyre::Help;
use eyre::{Context, Result};
use ignore::WalkBuilder;
use itertools::Itertools;
use regex::Regex;
use rustywind_core::sorter;
use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use ahash::AHashMap as HashMap;
use ahash::AHashSet as HashSet;

use crate::sorter::{FinderRegex, Sorter};
use crate::Cli;

#[derive(Debug)]
pub enum WriteMode {
    ToFile,
    DryRun,
    ToConsole,
    ToStdOut,
    CheckFormatted,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigFileContents {
    sort_order: Vec<String>,
}

#[derive(Debug)]
pub struct Options {
    pub stdin: Option<String>,
    pub sorter_options: sorter::Options,
    pub write_mode: WriteMode,
    pub starting_paths: Vec<PathBuf>,
    pub search_paths: Vec<PathBuf>,
    pub ignored_files: HashSet<PathBuf>,
    pub quiet: bool,
}

impl Options {
    pub fn new_from_cli(cli: Cli) -> Result<Options> {
        let stdin = if cli.stdin {
            let mut buffer = String::new();
            let mut stdin = std::io::stdin(); // We get `Stdin` here.
            stdin.read_to_string(&mut buffer).unwrap();
            Some(buffer.to_string())
        } else {
            None
        };

        let starting_paths = get_starting_path_from_cli(&cli);
        let search_paths = get_search_paths_from_starting_paths(&starting_paths);

        let sorter_options = sorter::Options {
            regex: get_custom_regex_from_cli(&cli)?,
            sorter: get_sorter_from_cli(&cli)?,
            allow_duplicates: cli.allow_duplicates,
        };

        Ok(Options {
            stdin,
            sorter_options,
            starting_paths,
            search_paths,
            write_mode: get_write_mode_from_cli(&cli),
            ignored_files: get_ignored_files_from_cli(&cli),
            quiet: cli.quiet,
        })
    }
}

fn get_sorter_from_cli(cli: &Cli) -> Result<Sorter> {
    if let Some(vite_css_url) = &cli.vite_css {
        return vite::create_sorter(vite_css_url, cli.skip_ssl_verification);
    }

    if let Some(css_file) = &cli.output_css_file {
        let css_file = std::fs::File::open(css_file)
            .wrap_err_with(|| format!("Error opening the css file {css_file}"))
            .with_suggestion(|| format!("Make sure the file {css_file} exists"))?;

        let sorter =
            parser::parse_classes_from_file(css_file).wrap_err("Error parsing the css file")?;

        return Ok(Sorter::CustomSorter(sorter));
    }

    if let Some(config_file) = &cli.config_file {
        let file_contents = fs::read_to_string(config_file)
            .wrap_err_with(|| format!("Error reading the config file {config_file}"))
            .with_suggestion(|| format!("Make sure the file {config_file} exists"));

        let config_file: ConfigFileContents = serde_json::from_str(&file_contents?)
            .wrap_err_with(|| format!("Error while parsing the config file {config_file}"))
            .with_suggestion(|| {
                format!("Make sure the {config_file} is valid json, with the expected format")
            })?;

        let sorter = parse_custom_sorter(config_file.sort_order);
        return Ok(Sorter::CustomSorter(sorter));
    }

    // if no other sorter is specified, use the default sorter
    Ok(Sorter::DefaultSorter)
}

fn get_custom_regex_from_cli(cli: &Cli) -> Result<FinderRegex> {
    match &cli.custom_regex {
        Some(regex_string) => {
            let regex = Regex::new(regex_string).wrap_err("Unable to parse custom regex")?;

            if regex.captures_len() < 2 {
                eyre::bail!("custom regex error, requires at-least 2 capture groups");
            }

            Ok(FinderRegex::CustomRegex(regex))
        }
        None => Ok(FinderRegex::DefaultRegex),
    }
}

fn get_starting_path_from_cli(cli: &Cli) -> Vec<PathBuf> {
    cli.file_or_dir
        .iter()
        .map(|path| Path::new(path).to_owned())
        .collect()
}

fn get_write_mode_from_cli(cli: &Cli) -> WriteMode {
    if cli.dry_run {
        WriteMode::DryRun
    } else if cli.write {
        WriteMode::ToFile
    } else if cli.check_formatted {
        WriteMode::CheckFormatted
    } else if cli.stdin {
        WriteMode::ToStdOut
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

fn get_ignored_files_from_cli(cli: &Cli) -> HashSet<PathBuf> {
    match &cli.ignored_files {
        Some(ignored_files) => ignored_files
            .iter()
            .map(|string| PathBuf::from_str(string))
            .filter_map(Result::ok)
            .map(std::fs::canonicalize)
            .filter_map(Result::ok)
            .collect(),
        None => HashSet::new(),
    }
}

fn parse_custom_sorter(contents: Vec<String>) -> HashMap<String, usize> {
    contents
        .into_iter()
        .enumerate()
        .map(|(index, class)| (class, index))
        .collect()
}

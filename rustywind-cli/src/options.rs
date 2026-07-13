use clap::ValueEnum;
use color_eyre::Help;
use eyre::{Context, Result};
use ignore::WalkBuilder;
use regex::Regex;
use rustywind_core::class_wrapping::ClassWrapping;
use rustywind_core::sorter::{CustomClassExtractor, FinderRegex, Sorter};
use rustywind_core::{RustyWind, SourceLanguage};
use rustywind_vite::create_vite_sorter;
use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use ahash::AHashMap as HashMap;
use ahash::AHashSet as HashSet;

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

// Wrapper to be able to use the `ValueEnum` trait without adding clap to the core crate
#[derive(Clone, Copy, Debug)]
pub struct CliClassWrapping(ClassWrapping);

impl ValueEnum for CliClassWrapping {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            CliClassWrapping(ClassWrapping::NoWrapping),
            CliClassWrapping(ClassWrapping::CommaSingleQuotes),
            CliClassWrapping(ClassWrapping::CommaDoubleQuotes),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(self.0.as_str()))
    }
}

// Wrapper to keep Clap-specific parsing out of the core crate
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliSourceLanguage {
    Html,
    Svelte,
    Astro,
    Django,
    Jinja,
    Twig,
    Liquid,
    Handlebars,
    Erb,
    Ejs,
    Php,
    Blade,
    Lit,
    Ruby,
}

impl From<CliSourceLanguage> for SourceLanguage {
    fn from(language: CliSourceLanguage) -> Self {
        match language {
            CliSourceLanguage::Html => Self::Html,
            CliSourceLanguage::Svelte => Self::Svelte,
            CliSourceLanguage::Astro => Self::Astro,
            CliSourceLanguage::Django => Self::Django,
            CliSourceLanguage::Jinja => Self::Jinja,
            CliSourceLanguage::Twig => Self::Twig,
            CliSourceLanguage::Liquid => Self::Liquid,
            CliSourceLanguage::Handlebars => Self::Handlebars,
            CliSourceLanguage::Erb => Self::Erb,
            CliSourceLanguage::Ejs => Self::Ejs,
            CliSourceLanguage::Php => Self::Php,
            CliSourceLanguage::Blade => Self::Blade,
            CliSourceLanguage::Lit => Self::Lit,
            CliSourceLanguage::Ruby => Self::Ruby,
        }
    }
}

#[derive(Debug)]
pub struct Options {
    pub stdin: Option<String>,
    pub language: Option<SourceLanguage>,
    pub stdin_filename: Option<PathBuf>,
    pub rustywind: RustyWind,
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

        let rustywind = RustyWind {
            regex: get_custom_regex_from_cli(&cli)?,
            sorter: get_sorter_from_cli(&cli)?,
            allow_duplicates: cli.allow_duplicates,
            class_wrapping: get_class_wrapping_from_cli(&cli),
            tailwind_prefix: cli.tailwind_prefix.clone(),
        };

        Ok(Options {
            stdin,
            language: cli.language.map(Into::into),
            stdin_filename: cli.stdin_filename.clone(),
            rustywind,
            starting_paths,
            search_paths,
            write_mode: get_write_mode_from_cli(&cli),
            ignored_files: get_ignored_files_from_cli(&cli),
            quiet: cli.quiet,
        })
    }

    pub fn source_language_for_path(&self, path: &Path) -> SourceLanguage {
        resolve_source_language(self.language, Some(path))
    }

    pub fn stdin_source_language(&self) -> SourceLanguage {
        resolve_source_language(self.language, self.stdin_filename.as_deref())
    }
}

fn resolve_source_language(
    language: Option<SourceLanguage>,
    path: Option<&Path>,
) -> SourceLanguage {
    language
        .or_else(|| path.map(SourceLanguage::from_path))
        .unwrap_or(SourceLanguage::Unknown)
}

fn get_sorter_from_cli(cli: &Cli) -> Result<Sorter> {
    if let Some(vite_css_url) = &cli.vite_css {
        return create_vite_sorter(vite_css_url, cli.skip_ssl_verification);
    }

    if let Some(css_file) = &cli.output_css_file {
        let css_file = std::fs::File::open(css_file)
            .wrap_err_with(|| format!("Error opening the css file {css_file}"))
            .with_suggestion(|| format!("Make sure the file {css_file} exists"))?;

        let sorter = Sorter::new_from_file(css_file)?;

        return Ok(sorter);
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

    // if no other sorter is specified, use the pattern sorter
    Ok(Sorter::PatternSorter)
}

fn get_custom_regex_from_cli(cli: &Cli) -> Result<FinderRegex> {
    match &cli.custom_regex {
        Some(regex_string) => {
            let regex = Regex::new(regex_string).wrap_err("Unable to parse custom regex")?;
            let extractor = CustomClassExtractor::new(regex)
                .wrap_err("Custom regex requires at least one capture group")?;

            Ok(FinderRegex::CustomRegex(extractor))
        }
        None => Ok(FinderRegex::DefaultRegex),
    }
}

fn get_class_wrapping_from_cli(cli: &Cli) -> ClassWrapping {
    match &cli.class_wrapping {
        Some(class_wrapping) => class_wrapping.0,
        None => ClassWrapping::NoWrapping,
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
        WriteMode::ToConsole
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rustywind_core::SourceLanguage;

    use super::resolve_source_language;

    #[test]
    fn explicit_language_overrides_filename_inference() {
        assert_eq!(
            resolve_source_language(Some(SourceLanguage::Svelte), Some(Path::new("view.html"))),
            SourceLanguage::Svelte
        );
    }

    #[test]
    fn language_falls_back_to_filename_then_unknown() {
        assert_eq!(
            resolve_source_language(None, Some(Path::new("view.blade.php"))),
            SourceLanguage::Blade
        );
        assert_eq!(resolve_source_language(None, None), SourceLanguage::Unknown);
    }
}

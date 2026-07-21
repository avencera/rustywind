use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};

use ignore::{Error as WalkError, WalkBuilder};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rustywind_core::SourceDocument;
use serde::Serialize;

use crate::options::{Options, WriteMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JsonMode {
    CheckFormatted,
    DryRun,
    Write,
    Stdin,
}

impl JsonMode {
    fn from_write_mode(write_mode: WriteMode) -> Self {
        match write_mode {
            WriteMode::CheckFormatted => Self::CheckFormatted,
            WriteMode::DryRun | WriteMode::ToConsole => Self::DryRun,
            WriteMode::ToFile => Self::Write,
            WriteMode::ToStdOut => Self::Stdin,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::CheckFormatted => "check-formatted",
            Self::DryRun => "dry-run",
            Self::Write => "write",
            Self::Stdin => "stdin",
        }
    }
}

#[derive(Clone, Debug)]
struct PathInfo {
    original: PathBuf,
    canonical: Option<PathBuf>,
}

impl PathInfo {
    fn new(path: PathBuf) -> Self {
        let canonical = path.canonicalize().ok();
        Self {
            original: path,
            canonical,
        }
    }

    fn identity(&self, current_dir: &Path) -> PathBuf {
        self.canonical
            .clone()
            .unwrap_or_else(|| absolute_path(&self.original, current_dir))
    }
}

#[derive(Clone, Debug)]
struct PendingError {
    path: PathInfo,
    message: String,
}

#[derive(Debug)]
struct Discovery {
    candidates: Vec<PathInfo>,
    errors: Vec<PendingError>,
}

#[derive(Debug)]
struct FileResult {
    path: PathInfo,
    outcome: FileOutcome,
}

#[derive(Debug)]
enum FileOutcome {
    ReadError(String),
    Changed(ChangedFile),
}

#[derive(Debug)]
struct ChangedFile {
    content: String,
    unsorted_classes: usize,
    write: WriteStatus,
}

#[derive(Debug)]
enum WriteStatus {
    NotAttempted,
    Written,
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct FormattingFile {
    path: String,
    unsorted_classes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ProposedChange {
    path: String,
    content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct WrittenFile {
    path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ErrorEntry {
    path: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct CheckResponse {
    mode: &'static str,
    ok: bool,
    files_needing_formatting: Vec<FormattingFile>,
    errors: Vec<ErrorEntry>,
}

#[derive(Debug, Serialize)]
struct DryRunResponse {
    mode: &'static str,
    ok: bool,
    files_needing_formatting: Vec<FormattingFile>,
    proposed_changes: Vec<ProposedChange>,
    errors: Vec<ErrorEntry>,
}

#[derive(Debug, Serialize)]
struct WriteResponse {
    mode: &'static str,
    ok: bool,
    files_needing_formatting: Vec<FormattingFile>,
    written_files: Vec<WrittenFile>,
    errors: Vec<ErrorEntry>,
}

#[derive(Debug, Serialize)]
struct StdinResponse {
    mode: &'static str,
    ok: bool,
    unsorted_classes: usize,
    content: String,
    errors: Vec<ErrorEntry>,
}

#[derive(Debug)]
struct AggregatedFiles {
    files_needing_formatting: Vec<FormattingFile>,
    proposed_changes: Vec<ProposedChange>,
    written_files: Vec<WrittenFile>,
    errors: Vec<ErrorEntry>,
}

pub(crate) fn run(options: &Options) -> bool {
    let mode = JsonMode::from_write_mode(options.write_mode);
    if mode == JsonMode::Stdin {
        return run_stdin(options, std::io::stdin());
    }

    run_files(options, mode)
}

fn run_stdin(options: &Options, reader: impl Read) -> bool {
    let response = match super::read_input(reader) {
        Ok(contents) => {
            let language = options.stdin_source_language();
            let report = options
                .rustywind
                .sort_document_with_report(SourceDocument::new(&contents, language));
            StdinResponse {
                mode: JsonMode::Stdin.as_str(),
                ok: report.changed_class_groups == 0,
                unsorted_classes: report.changed_class_groups,
                content: report.contents.into_owned(),
                errors: Vec::new(),
            }
        }
        Err(error) => StdinResponse {
            mode: JsonMode::Stdin.as_str(),
            ok: false,
            unsorted_classes: 0,
            content: String::new(),
            errors: vec![ErrorEntry {
                path: "<stdin>".to_string(),
                message: error.to_string(),
            }],
        },
    };
    let failed = !response.errors.is_empty();
    emit(&response);
    failed
}

fn run_files(options: &Options, mode: JsonMode) -> bool {
    let discovery = discover_files(&options.starting_paths);
    let results = discovery
        .candidates
        .into_par_iter()
        .filter_map(|path| process_file(path, options, mode))
        .collect::<Vec<_>>();
    let aggregated = aggregate_files(discovery.errors, results, &options.starting_paths);
    let clean = aggregated.files_needing_formatting.is_empty() && aggregated.errors.is_empty();
    let has_errors = !aggregated.errors.is_empty();

    match mode {
        JsonMode::CheckFormatted => {
            emit(&CheckResponse {
                mode: mode.as_str(),
                ok: clean,
                files_needing_formatting: aggregated.files_needing_formatting,
                errors: aggregated.errors,
            });
            !clean
        }
        JsonMode::DryRun => {
            emit(&DryRunResponse {
                mode: mode.as_str(),
                ok: clean,
                files_needing_formatting: aggregated.files_needing_formatting,
                proposed_changes: aggregated.proposed_changes,
                errors: aggregated.errors,
            });
            has_errors
        }
        JsonMode::Write => {
            emit(&WriteResponse {
                mode: mode.as_str(),
                ok: !has_errors,
                files_needing_formatting: aggregated.files_needing_formatting,
                written_files: aggregated.written_files,
                errors: aggregated.errors,
            });
            has_errors
        }
        JsonMode::Stdin => unreachable!("stdin handled before file discovery"),
    }
}

fn emit(response: &impl Serialize) {
    let json = serde_json::to_string(response).expect("JSON wire structs should serialize");
    println!("{json}");
}

fn discover_files(starting_paths: &[PathBuf]) -> Discovery {
    let mut candidates = Vec::new();
    let mut errors = Vec::new();

    for starting_path in starting_paths {
        for item in WalkBuilder::new(starting_path).build() {
            match item {
                Ok(entry) => {
                    if let Some(error) = entry.error() {
                        flatten_walk_error(error, entry.path(), &mut errors);
                    }

                    let is_file = if entry.path_is_symlink() {
                        match entry.metadata() {
                            Ok(metadata) => metadata.is_file(),
                            Err(error) => {
                                flatten_walk_error(&error, entry.path(), &mut errors);
                                false
                            }
                        }
                    } else {
                        entry
                            .file_type()
                            .is_some_and(|file_type| file_type.is_file())
                    };

                    if is_file {
                        candidates.push(PathInfo::new(entry.into_path()));
                    }
                }
                Err(error) => flatten_walk_error(&error, starting_path, &mut errors),
            }
        }
    }

    candidates.sort_by(|left, right| left.original.cmp(&right.original));
    let current_dir = std::env::current_dir().unwrap_or_default();
    let mut seen = HashSet::new();
    candidates.retain(|candidate| seen.insert(candidate.identity(&current_dir)));

    Discovery { candidates, errors }
}

fn flatten_walk_error(error: &WalkError, fallback: &Path, output: &mut Vec<PendingError>) {
    flatten_walk_error_with_context(error, fallback, fallback, &[], output);
}

fn flatten_walk_error_with_context(
    error: &WalkError,
    fallback: &Path,
    current_path: &Path,
    context: &[String],
    output: &mut Vec<PendingError>,
) {
    match error {
        WalkError::Partial(errors) => {
            for error in errors {
                flatten_walk_error_with_context(error, fallback, current_path, context, output);
            }
        }
        WalkError::WithLineNumber { line, err } => {
            let mut nested_context = context.to_vec();
            nested_context.push(format!("line {line}"));
            flatten_walk_error_with_context(err, fallback, current_path, &nested_context, output);
        }
        WalkError::WithPath { path, err } => {
            flatten_walk_error_with_context(err, fallback, path, context, output);
        }
        WalkError::WithDepth { depth, err } => {
            let mut nested_context = context.to_vec();
            nested_context.push(format!("depth {depth}"));
            flatten_walk_error_with_context(err, fallback, current_path, &nested_context, output);
        }
        WalkError::Loop { child, .. } => push_walk_error(child, error, context, output),
        _ => push_walk_error(
            if current_path.as_os_str().is_empty() {
                fallback
            } else {
                current_path
            },
            error,
            context,
            output,
        ),
    }
}

fn push_walk_error(
    path: &Path,
    error: &WalkError,
    context: &[String],
    output: &mut Vec<PendingError>,
) {
    let message = if context.is_empty() {
        error.to_string()
    } else {
        format!("{}: {error}", context.join(": "))
    };
    output.push(PendingError {
        path: PathInfo::new(path.to_path_buf()),
        message,
    });
}

fn process_file(path: PathInfo, options: &Options, mode: JsonMode) -> Option<FileResult> {
    process_file_with(
        path,
        options,
        mode,
        |path| fs::read_to_string(path),
        |path, contents| fs::write(path, contents),
    )
}

fn process_file_with(
    path: PathInfo,
    options: &Options,
    mode: JsonMode,
    read: impl FnOnce(&Path) -> io::Result<String>,
    write: impl FnOnce(&Path, &[u8]) -> io::Result<()>,
) -> Option<FileResult> {
    if super::should_ignore_current_file(&options.ignored_files, &path.original) {
        return None;
    }

    let contents = match read(&path.original) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::InvalidData => return None,
        Err(error) => {
            return Some(FileResult {
                path,
                outcome: FileOutcome::ReadError(error.to_string()),
            });
        }
    };
    let language = options.source_language_for_path(&path.original);
    let report = options
        .rustywind
        .sort_document_with_report(SourceDocument::new(&contents, language));
    if report.changed_class_groups == 0 {
        return None;
    }

    let content = report.contents.into_owned();
    let write_status = if mode == JsonMode::Write {
        match write(&path.original, content.as_bytes()) {
            Ok(()) => WriteStatus::Written,
            Err(error) => WriteStatus::Failed(error.to_string()),
        }
    } else {
        WriteStatus::NotAttempted
    };

    Some(FileResult {
        path,
        outcome: FileOutcome::Changed(ChangedFile {
            content,
            unsorted_classes: report.changed_class_groups,
            write: write_status,
        }),
    })
}

fn aggregate_files(
    discovery_errors: Vec<PendingError>,
    files: Vec<FileResult>,
    starting_paths: &[PathBuf],
) -> AggregatedFiles {
    let current_dir = std::env::current_dir().unwrap_or_default();
    let paths = discovery_errors
        .iter()
        .map(|error| &error.path)
        .chain(files.iter().map(|file| &file.path));
    let resolved_paths = resolve_display_paths(paths, starting_paths, &current_dir);
    let display = |path: &PathInfo| {
        resolved_paths
            .get(&path.identity(&current_dir))
            .expect("every collected path should have a resolved display path")
            .clone()
    };

    let mut files_needing_formatting = Vec::new();
    let mut proposed_changes = Vec::new();
    let mut written_files = Vec::new();
    let mut errors = discovery_errors
        .into_iter()
        .map(|error| ErrorEntry {
            path: display(&error.path),
            message: error.message,
        })
        .collect::<Vec<_>>();

    for file in files {
        let path = display(&file.path);
        match file.outcome {
            FileOutcome::ReadError(message) => errors.push(ErrorEntry { path, message }),
            FileOutcome::Changed(changed) => {
                files_needing_formatting.push(FormattingFile {
                    path: path.clone(),
                    unsorted_classes: changed.unsorted_classes,
                });
                proposed_changes.push(ProposedChange {
                    path: path.clone(),
                    content: changed.content,
                });
                match changed.write {
                    WriteStatus::NotAttempted => (),
                    WriteStatus::Written => written_files.push(WrittenFile { path }),
                    WriteStatus::Failed(message) => errors.push(ErrorEntry { path, message }),
                }
            }
        }
    }

    files_needing_formatting.sort_by(|left, right| left.path.cmp(&right.path));
    proposed_changes.sort_by(|left, right| left.path.cmp(&right.path));
    written_files.sort_by(|left, right| left.path.cmp(&right.path));
    errors.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.message.cmp(&right.message))
    });
    errors.dedup();

    AggregatedFiles {
        files_needing_formatting,
        proposed_changes,
        written_files,
        errors,
    }
}

fn resolve_display_paths<'a>(
    paths: impl Iterator<Item = &'a PathInfo>,
    starting_paths: &[PathBuf],
    current_dir: &Path,
) -> HashMap<PathBuf, String> {
    let mut representatives = BTreeMap::<PathBuf, PathInfo>::new();
    for path in paths {
        let identity = path.identity(current_dir);
        representatives
            .entry(identity)
            .and_modify(|existing| {
                if path.original < existing.original {
                    *existing = path.clone();
                }
            })
            .or_insert_with(|| path.clone());
    }

    let mut displays = representatives
        .iter()
        .map(|(identity, path)| {
            (
                identity.clone(),
                super::get_file_name(&path.original, starting_paths),
            )
        })
        .collect::<HashMap<_, _>>();
    replace_collisions(&mut displays, &representatives, |path| {
        cwd_relative(&path.original, current_dir)
            .display()
            .to_string()
    });
    replace_collisions(&mut displays, &representatives, |path| {
        path.canonical
            .clone()
            .unwrap_or_else(|| absolute_path(&path.original, current_dir))
            .display()
            .to_string()
    });

    displays
}

fn replace_collisions(
    displays: &mut HashMap<PathBuf, String>,
    representatives: &BTreeMap<PathBuf, PathInfo>,
    replacement: impl Fn(&PathInfo) -> String,
) {
    let mut groups = HashMap::<String, Vec<PathBuf>>::new();
    for (identity, display) in displays.iter() {
        groups
            .entry(display.clone())
            .or_default()
            .push(identity.clone());
    }
    for identities in groups.into_values().filter(|group| group.len() > 1) {
        for identity in identities {
            let path = representatives
                .get(&identity)
                .expect("display identity should have a representative");
            displays.insert(identity, replacement(path));
        }
    }
}

fn cwd_relative<'a>(path: &'a Path, current_dir: &Path) -> &'a Path {
    if path.is_absolute() {
        path.strip_prefix(current_dir).unwrap_or(path)
    } else {
        path
    }
}

fn absolute_path(path: &Path, current_dir: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::io::{self, ErrorKind};
    use std::path::PathBuf;

    use clap::Parser;
    use serde_json::Value;

    use super::{
        CheckResponse, DryRunResponse, ErrorEntry, FileOutcome, FormattingFile, JsonMode, PathInfo,
        ProposedChange, StdinResponse, WriteResponse, WriteStatus, WrittenFile, process_file_with,
    };
    use crate::{Cli, options::Options};

    fn keys(value: &Value) -> BTreeSet<&str> {
        value
            .as_object()
            .expect("response should be an object")
            .keys()
            .map(String::as_str)
            .collect()
    }

    #[test]
    fn check_shape_has_only_check_fields() {
        let value = serde_json::to_value(CheckResponse {
            mode: "check-formatted",
            ok: true,
            files_needing_formatting: Vec::new(),
            errors: Vec::new(),
        })
        .unwrap();

        assert_eq!(
            keys(&value),
            BTreeSet::from(["errors", "files_needing_formatting", "mode", "ok"])
        );
        assert_eq!(value["files_needing_formatting"], Value::Array(vec![]));
        assert_eq!(value["errors"], Value::Array(vec![]));
    }

    #[test]
    fn dry_run_shape_has_only_preview_fields() {
        let value = serde_json::to_value(DryRunResponse {
            mode: "dry-run",
            ok: false,
            files_needing_formatting: vec![FormattingFile {
                path: "input.html".to_string(),
                unsorted_classes: 1,
            }],
            proposed_changes: vec![ProposedChange {
                path: "input.html".to_string(),
                content: "sorted".to_string(),
            }],
            errors: Vec::new(),
        })
        .unwrap();

        assert_eq!(
            keys(&value),
            BTreeSet::from([
                "errors",
                "files_needing_formatting",
                "mode",
                "ok",
                "proposed_changes"
            ])
        );
    }

    #[test]
    fn write_shape_has_only_write_fields() {
        let value = serde_json::to_value(WriteResponse {
            mode: "write",
            ok: true,
            files_needing_formatting: Vec::new(),
            written_files: Vec::new(),
            errors: Vec::new(),
        })
        .unwrap();

        assert_eq!(
            keys(&value),
            BTreeSet::from([
                "errors",
                "files_needing_formatting",
                "mode",
                "ok",
                "written_files"
            ])
        );
        assert_eq!(value["written_files"], Value::Array(vec![]));
    }

    #[test]
    fn stdin_shape_has_only_stdin_fields() {
        let value = serde_json::to_value(StdinResponse {
            mode: "stdin",
            ok: true,
            unsorted_classes: 0,
            content: String::new(),
            errors: Vec::new(),
        })
        .unwrap();

        assert_eq!(
            keys(&value),
            BTreeSet::from(["content", "errors", "mode", "ok", "unsorted_classes"])
        );
        assert_eq!(value["content"], "");
    }

    #[test]
    fn processing_skips_invalid_utf8_and_reports_other_read_errors() {
        let options = Options::new_from_cli(
            Cli::try_parse_from(["rustywind", "--json", "input.html"]).unwrap(),
        )
        .unwrap();
        let invalid = process_file_with(
            PathInfo::new(PathBuf::from("input.html")),
            &options,
            JsonMode::DryRun,
            |_| Err(io::Error::new(ErrorKind::InvalidData, "binary")),
            |_, _| Ok(()),
        );
        assert!(invalid.is_none());

        let unreadable = process_file_with(
            PathInfo::new(PathBuf::from("input.html")),
            &options,
            JsonMode::DryRun,
            |_| Err(io::Error::new(ErrorKind::PermissionDenied, "denied")),
            |_, _| Ok(()),
        );
        let Some(unreadable) = unreadable else {
            panic!("ordinary read errors should be retained");
        };
        let FileOutcome::ReadError(message) = unreadable.outcome else {
            panic!("read failure should be a read error outcome");
        };
        assert_eq!(message, "denied");
    }

    #[test]
    fn processing_retains_change_when_write_fails() {
        let options = Options::new_from_cli(
            Cli::try_parse_from(["rustywind", "--write", "--json", "input.html"]).unwrap(),
        )
        .unwrap();
        let result = process_file_with(
            PathInfo::new(PathBuf::from("input.html")),
            &options,
            JsonMode::Write,
            |_| Ok(r#"<div class="p-4 m-4"></div>"#.to_string()),
            |_, _| Err(io::Error::new(ErrorKind::StorageFull, "disk full")),
        );
        let Some(file) = result else {
            panic!("changed file should produce a result");
        };
        let FileOutcome::Changed(changed) = file.outcome else {
            panic!("a changed file with a failed write should stay a change");
        };

        assert!(
            matches!(changed.write, WriteStatus::Failed(ref message) if message == "disk full")
        );
    }

    #[test]
    fn error_entry_wire_fields_are_required() {
        let value = serde_json::to_value(ErrorEntry {
            path: "missing.html".to_string(),
            message: "not found".to_string(),
        })
        .unwrap();

        assert_eq!(keys(&value), BTreeSet::from(["message", "path"]));
    }

    #[test]
    fn file_entry_wire_fields_are_required() {
        let formatting = serde_json::to_value(FormattingFile {
            path: "input.html".to_string(),
            unsorted_classes: 2,
        })
        .unwrap();
        let proposed = serde_json::to_value(ProposedChange {
            path: "input.html".to_string(),
            content: "content".to_string(),
        })
        .unwrap();
        let written = serde_json::to_value(WrittenFile {
            path: "input.html".to_string(),
        })
        .unwrap();

        assert_eq!(
            keys(&formatting),
            BTreeSet::from(["path", "unsorted_classes"])
        );
        assert_eq!(keys(&proposed), BTreeSet::from(["content", "path"]));
        assert_eq!(keys(&written), BTreeSet::from(["path"]));
    }
}

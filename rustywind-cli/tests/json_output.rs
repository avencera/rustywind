use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use serde_json::Value;
use tempfile::TempDir;

fn command(current_dir: &Path) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("rustywind"));
    command.current_dir(current_dir);
    command
}

fn run(current_dir: &Path, args: &[&str]) -> Output {
    command(current_dir)
        .args(args)
        .output()
        .expect("rustywind should run")
}

fn run_with_stdin(current_dir: &Path, args: &[&str], stdin: &str) -> Output {
    let mut child = command(current_dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rustywind should start");
    std::io::Write::write_all(
        child.stdin.as_mut().expect("stdin should be piped"),
        stdin.as_bytes(),
    )
    .expect("stdin should be written");
    child.wait_with_output().expect("rustywind should finish")
}

fn json_stdout(output: &Output) -> Value {
    assert!(output.stdout.ends_with(b"\n"));
    assert!(!output.stdout[..output.stdout.len() - 1].contains(&b'\n'));
    assert!(!output.stdout.ends_with(b"\n\n"));
    serde_json::from_slice(&output.stdout[..output.stdout.len() - 1])
        .expect("stdout should contain exactly one JSON document")
}

fn write_unsorted(path: impl AsRef<Path>) {
    fs::write(path, r#"<div class="p-4 m-4"></div>"#).unwrap();
}

#[test]
fn check_json_reports_changes_in_deterministic_order() {
    let directory = TempDir::new().unwrap();
    write_unsorted(directory.path().join("b.html"));
    write_unsorted(directory.path().join("a.html"));

    let output = run(directory.path(), &["--check-formatted", "--json", "."]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let json = json_stdout(&output);

    assert_eq!(json["mode"], "check-formatted");
    assert_eq!(json["ok"], false);
    assert_eq!(json["errors"], serde_json::json!([]));
    // a `.` starting path displays as `./a.html` in human mode, and JSON reuses
    // the human display algorithm
    assert_eq!(
        json["files_needing_formatting"],
        serde_json::json!([
            {"path":"./a.html","unsorted_classes":1},
            {"path":"./b.html","unsorted_classes":1}
        ])
    );
    assert!(json.get("proposed_changes").is_none());
    assert!(json.get("written_files").is_none());
}

#[test]
fn dry_run_and_bare_json_include_full_formatted_content() {
    let directory = TempDir::new().unwrap();
    write_unsorted(directory.path().join("input.html"));

    for args in [
        &["--dry-run", "--json", "input.html"][..],
        &["--json", "input.html"][..],
    ] {
        let output = run(directory.path(), args);
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        let json = json_stdout(&output);

        assert_eq!(json["mode"], "dry-run");
        assert_eq!(json["ok"], false);
        assert_eq!(
            json["proposed_changes"],
            serde_json::json!([{
                "path":"input.html",
                "content":r#"<div class="m-4 p-4"></div>"#
            }])
        );
        assert!(json.get("written_files").is_none());
    }

    assert_eq!(
        fs::read_to_string(directory.path().join("input.html")).unwrap(),
        r#"<div class="p-4 m-4"></div>"#
    );
}

#[test]
fn write_json_reports_and_applies_changes() {
    let directory = TempDir::new().unwrap();
    write_unsorted(directory.path().join("input.html"));

    let output = run(directory.path(), &["--write", "--json", "input.html"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let json = json_stdout(&output);

    assert_eq!(json["mode"], "write");
    assert_eq!(json["ok"], true);
    assert_eq!(
        json["files_needing_formatting"],
        serde_json::json!([{"path":"input.html","unsorted_classes":1}])
    );
    assert_eq!(
        json["written_files"],
        serde_json::json!([{"path":"input.html"}])
    );
    assert!(json.get("proposed_changes").is_none());
    assert_eq!(
        fs::read_to_string(directory.path().join("input.html")).unwrap(),
        r#"<div class="m-4 p-4"></div>"#
    );
}

#[test]
fn stdin_json_reports_formatted_content_without_human_warning() {
    let directory = TempDir::new().unwrap();
    let output = run_with_stdin(
        directory.path(),
        &["--stdin", "--json"],
        r#"<div class="p-4 m-4"></div>"#,
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let json = json_stdout(&output);

    assert_eq!(
        json,
        serde_json::json!({
            "mode":"stdin",
            "ok":false,
            "unsorted_classes":1,
            "content":r#"<div class="m-4 p-4"></div>"#,
            "errors":[]
        })
    );

    let no_classes = run_with_stdin(directory.path(), &["--stdin", "--json"], "plain text");
    assert_eq!(no_classes.status.code(), Some(0));
    assert!(no_classes.stderr.is_empty());
    assert_eq!(json_stdout(&no_classes)["content"], "plain text");
}

#[test]
fn json_continues_after_missing_input_and_returns_partial_results() {
    let directory = TempDir::new().unwrap();
    write_unsorted(directory.path().join("valid.html"));

    let output = run(
        directory.path(),
        &["--dry-run", "--json", "valid.html", "missing.html"],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let json = json_stdout(&output);

    assert_eq!(json["ok"], false);
    assert_eq!(json["files_needing_formatting"][0]["path"], "valid.html");
    assert_eq!(json["proposed_changes"][0]["path"], "valid.html");
    assert_eq!(json["errors"][0]["path"], "missing.html");
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .is_some_and(|message| !message.is_empty())
    );
}

#[test]
fn missing_directory_is_a_walk_error() {
    let directory = TempDir::new().unwrap();
    let output = run(
        directory.path(),
        &["--check-formatted", "--json", "missing-directory"],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let json = json_stdout(&output);

    assert_eq!(json["files_needing_formatting"], serde_json::json!([]));
    assert_eq!(json["errors"][0]["path"], "missing-directory");
}

#[test]
fn invalid_utf8_is_skipped_without_an_error() {
    let directory = TempDir::new().unwrap();
    fs::write(directory.path().join("binary.html"), [0xff, 0xfe]).unwrap();

    let output = run(
        directory.path(),
        &["--check-formatted", "--json", "binary.html"],
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let json = json_stdout(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["files_needing_formatting"], serde_json::json!([]));
    assert_eq!(json["errors"], serde_json::json!([]));
}

#[test]
fn overlapping_paths_are_processed_once() {
    let directory = TempDir::new().unwrap();
    fs::create_dir(directory.path().join("nested")).unwrap();
    write_unsorted(directory.path().join("nested/input.html"));

    let output = run(
        directory.path(),
        &["--dry-run", "--json", ".", "nested/input.html"],
    );
    assert_eq!(output.status.code(), Some(0));
    let json = json_stdout(&output);

    assert_eq!(
        json["files_needing_formatting"].as_array().unwrap().len(),
        1
    );
    assert_eq!(json["proposed_changes"].as_array().unwrap().len(), 1);
}

#[test]
fn colliding_human_paths_fall_back_to_cwd_relative_paths() {
    let directory = TempDir::new().unwrap();
    for root in ["left/src", "right/src"] {
        fs::create_dir_all(directory.path().join(root)).unwrap();
        write_unsorted(directory.path().join(root).join("input.html"));
    }

    let output = run(
        directory.path(),
        &["--dry-run", "--json", "left/src", "right/src"],
    );
    assert_eq!(output.status.code(), Some(0));
    let json = json_stdout(&output);
    let paths = json["files_needing_formatting"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(paths, ["left/src/input.html", "right/src/input.html"]);
    assert_eq!(json["proposed_changes"][0]["path"], paths[0]);
    assert_eq!(json["proposed_changes"][1]["path"], paths[1]);
}

#[test]
fn quiet_dry_run_is_allowed_only_for_json() {
    let directory = TempDir::new().unwrap();
    write_unsorted(directory.path().join("input.html"));

    let json = run(
        directory.path(),
        &["--quiet", "--dry-run", "--json", "input.html"],
    );
    assert_eq!(json.status.code(), Some(0));
    json_stdout(&json);

    let human = run(directory.path(), &["--quiet", "--dry-run", "input.html"]);
    assert_eq!(human.status.code(), Some(2));
    assert!(human.stdout.is_empty());
    assert!(!human.stderr.is_empty());
}

#[test]
fn representative_human_modes_keep_their_output() {
    let directory = TempDir::new().unwrap();
    write_unsorted(directory.path().join("input.html"));

    let dry_run = run(directory.path(), &["--dry-run", "input.html"]);
    assert_eq!(dry_run.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(dry_run.stdout).unwrap(),
        "\ndry run mode activated: here is a list of files that would be changed when you run with the --write flag\n  * input.html\n"
    );

    let check = run(directory.path(), &["--check-formatted", "input.html"]);
    assert_eq!(check.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(check.stdout).unwrap(),
        "\nonly printing changed files\n"
    );
    assert_eq!(
        String::from_utf8(check.stderr).unwrap(),
        "  * [UNFORMATTED FILE] input.html\n"
    );

    let stdin = run_with_stdin(directory.path(), &["--stdin"], "plain text");
    assert_eq!(stdin.status.code(), Some(0));
    assert_eq!(stdin.stdout, b"plain text");
    assert_eq!(stdin.stderr, b"[WARN] No classes were found in STDIN");
}

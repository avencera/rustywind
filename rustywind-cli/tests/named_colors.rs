use std::io::Write as _;
use std::process::{Command, Stdio};

fn sort_stdin(args: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("rustywind"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rustywind should start");
    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("stdin should be written");
    child.wait_with_output().expect("rustywind should finish")
}

#[test]
fn no_named_colors_keeps_ambiguous_names_unknown() {
    if std::env::var_os("CROSS_RUNNER").is_some() {
        return;
    }

    let input = r#"<div class="text-display flex"></div>"#;
    let default = sort_stdin(&["--stdin", "--stdin-filename", "input.html"], input);
    let disabled = sort_stdin(
        &[
            "--stdin",
            "--stdin-filename",
            "input.html",
            "--no-named-colors",
        ],
        input,
    );

    assert!(default.status.success());
    assert!(default.stderr.is_empty());
    assert_eq!(
        String::from_utf8(default.stdout).unwrap(),
        r#"<div class="flex text-display"></div>"#
    );
    assert!(disabled.status.success());
    assert!(disabled.stderr.is_empty());
    assert_eq!(String::from_utf8(disabled.stdout).unwrap(), input);
}

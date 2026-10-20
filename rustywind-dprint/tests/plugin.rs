use std::path::Path;

use dprint_core::configuration::{ConfigKeyMap, ConfigKeyValue, GlobalConfiguration};
use dprint_core::plugins::{
    FormatConfigId, FormatRange, FormatResult, NullCancellationToken,
    PluginResolveConfigurationResult, SyncFormatRequest, SyncPluginHandler,
};
use dprint_plugin_rustywind::{Configuration, RustyWindPluginHandler};
use pretty_assertions::assert_eq;

fn config(entries: &[(&str, ConfigKeyValue)]) -> ConfigKeyMap {
    entries
        .iter()
        .map(|(key, value)| (key.to_string(), value.clone()))
        .collect()
}

fn resolve(entries: &[(&str, ConfigKeyValue)]) -> PluginResolveConfigurationResult<Configuration> {
    RustyWindPluginHandler.resolve_config(config(entries), &GlobalConfiguration::default())
}

fn format_with(
    config: &Configuration,
    file_path: &str,
    bytes: &[u8],
    range: FormatRange,
) -> Option<String> {
    let result: FormatResult = RustyWindPluginHandler.format(
        SyncFormatRequest {
            file_path: Path::new(file_path),
            file_bytes: bytes.to_vec(),
            config_id: FormatConfigId::from_raw(1),
            config,
            range,
            token: &NullCancellationToken,
        },
        |_| unreachable!("the plugin never formats through the host"),
    );

    result
        .expect("formatting should succeed")
        .map(|bytes| String::from_utf8(bytes).expect("output should be UTF-8"))
}

fn format(entries: &[(&str, ConfigKeyValue)], file_path: &str, text: &str) -> Option<String> {
    let resolved = resolve(entries);
    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );

    format_with(&resolved.config, file_path, text.as_bytes(), None)
}

#[test]
fn sorts_language_aware_documents_and_reports_unchanged_files() {
    let tsx = r#"export const Card = () => <div className={cx("p-4")} class="z" className="p-4 flex m-2">hi</div>;"#;
    assert_eq!(
        format(&[], "src/Card.tsx", tsx).as_deref(),
        Some(
            r#"export const Card = () => <div className={cx("p-4")} class="z" className="m-2 flex p-4">hi</div>;"#
        )
    );

    // the file path selects the template profile: Blade sorts around `{{ }}`,
    // while plain HTML leaves template-looking attributes alone
    let template = r#"<div class="p-4 m-2 {{ $active }} p-2 flex"></div>"#;
    assert_eq!(
        format(&[], "views/card.blade.php", template).as_deref(),
        Some(r#"<div class="m-2 p-4 {{ $active }} flex p-2"></div>"#)
    );
    assert_eq!(format(&[], "views/card.html", template), None);

    assert_eq!(
        format(&[], "page.html", r#"<div class="m-2 flex p-4"></div>"#),
        None
    );
}

#[test]
fn applies_cli_equivalent_options() {
    assert_eq!(
        format(
            &[
                ("tailwindPrefix", ConfigKeyValue::String("tw-".into())),
                ("allowDuplicates", ConfigKeyValue::Bool(true)),
            ],
            "page.html",
            r#"<div class="tw-p-4 tw-flex tw-p-4"></div>"#,
        )
        .as_deref(),
        Some(r#"<div class="tw-flex tw-p-4 tw-p-4"></div>"#)
    );

    assert_eq!(
        format(
            &[(
                "sortOrder",
                ConfigKeyValue::Array(vec![
                    ConfigKeyValue::String("p-4".into()),
                    ConfigKeyValue::String("flex".into()),
                ]),
            )],
            "page.html",
            r#"<div class="flex p-4"></div>"#,
        )
        .as_deref(),
        Some(r#"<div class="p-4 flex"></div>"#)
    );

    assert_eq!(
        format(
            &[
                (
                    "customRegex",
                    ConfigKeyValue::String(r"classes\(([^)]*)\)".into())
                ),
                (
                    "classWrapping",
                    ConfigKeyValue::String("comma-single-quotes".into())
                ),
            ],
            "app.py",
            "classes('p-4', 'flex')",
        )
        .as_deref(),
        Some("classes('flex', 'p-4')")
    );
}

#[test]
fn reports_invalid_options_and_matches_files_additively() {
    let resolved = resolve(&[
        ("classWrapping", ConfigKeyValue::String("comma".into())),
        (
            "customRegex",
            ConfigKeyValue::String("class=\"[^\"]*\"".into()),
        ),
        (
            "sortOrder",
            ConfigKeyValue::Array(vec![
                ConfigKeyValue::String("flex".into()),
                ConfigKeyValue::Number(1),
            ]),
        ),
        ("allowDuplicates", ConfigKeyValue::String("yes".into())),
        ("lineWidth", ConfigKeyValue::Number(80)),
    ]);

    let properties: Vec<_> = resolved
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.property_name.as_str())
        .collect();
    assert_eq!(
        properties,
        [
            "allowDuplicates",
            "classWrapping",
            "customRegex",
            "sortOrder[1]",
            "lineWidth"
        ]
    );

    assert!(resolved.file_matching.additive);
    assert!(
        resolved
            .file_matching
            .file_extensions
            .contains(&"tsx".to_string())
    );
}

#[test]
fn skips_ranges_unreadable_input_and_null_options() {
    let resolved = resolve(&[
        ("sortOrder", ConfigKeyValue::Null),
        ("tailwindPrefix", ConfigKeyValue::Null),
    ]);
    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );

    let text = r#"<div class="p-4 flex"></div>"#;
    assert_eq!(
        format_with(&resolved.config, "page.html", text.as_bytes(), None).as_deref(),
        Some(r#"<div class="flex p-4"></div>"#)
    );

    // class order is per attribute, so partial ranges are left alone
    assert_eq!(
        format_with(
            &resolved.config,
            "page.html",
            text.as_bytes(),
            Some(0..text.len())
        ),
        None
    );

    // Latin-1 bytes are skipped rather than failing the whole dprint run
    let latin1 = b"<div class=\"p-4 flex\">caf\xe9</div>";
    assert_eq!(
        format_with(&resolved.config, "page.html", latin1, None),
        None
    );
}

// the wasm is uploaded to the CLI's GitHub release, so the plugin version
// dprint reports must match that release tag
#[test]
fn plugin_version_matches_cli_release() {
    let cli_manifest = include_str!("../../rustywind-cli/Cargo.toml");
    let cli_version = cli_manifest
        .lines()
        .find_map(|line| line.strip_prefix("version = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .expect("CLI manifest should declare a version");

    assert_eq!(RustyWindPluginHandler.plugin_info().version, cli_version);
}

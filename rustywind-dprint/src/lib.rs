//! A dprint Wasm plugin that sorts Tailwind CSS classes with RustyWind
//!
//! The plugin is additive: it sorts classes in files that another plugin,
//! such as dprint-plugin-typescript or dprint-plugin-markup, already formats
mod configuration;

use std::path::Path;

use dprint_core::configuration::{ConfigKeyMap, GlobalConfiguration};
use dprint_core::plugins::{
    CheckConfigUpdatesMessage, ConfigChange, FileMatchingInfo, FormatError, FormatResult,
    PluginInfo, PluginResolveConfigurationResult, SyncFormatRequest, SyncHostFormatRequest,
    SyncPluginHandler,
};
use rustywind_core::{SourceDocument, SourceLanguage};

pub use configuration::{Configuration, resolve_config};

/// Extensions with a RustyWind language profile, matching `SourceLanguage::from_path`
///
/// Compound names such as `.blade.php` and `.html.erb` are covered by their
/// final extension, and the full file name still selects the profile
pub const FILE_EXTENSIONS: &[&str] = &[
    "astro",
    "blade",
    "django",
    "djhtml",
    "ejs",
    "erb",
    "handlebars",
    "hbs",
    "htm",
    "html",
    "j2",
    "jinja",
    "jinja2",
    "jsx",
    "liquid",
    "lit",
    "php",
    "phtml",
    "rb",
    "svelte",
    "tsx",
    "twig",
];

/// The dprint plugin handler for RustyWind
#[derive(Debug, Default)]
pub struct RustyWindPluginHandler;

impl SyncPluginHandler<Configuration> for RustyWindPluginHandler {
    fn resolve_config(
        &mut self,
        config: ConfigKeyMap,
        _global_config: &GlobalConfiguration,
    ) -> PluginResolveConfigurationResult<Configuration> {
        let resolved = resolve_config(config);

        PluginResolveConfigurationResult {
            config: resolved.config,
            diagnostics: resolved.diagnostics,
            file_matching: FileMatchingInfo {
                file_extensions: FILE_EXTENSIONS.iter().map(ToString::to_string).collect(),
                file_names: Vec::new(),
                // sorting runs alongside the plugin that claims the file
                // instead of competing with it for the file
                additive: true,
            },
        }
    }

    fn plugin_info(&mut self) -> PluginInfo {
        PluginInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_key: "rustywind".to_string(),
            help_url: "https://github.com/avencera/rustywind#dprint-plugin".to_string(),
            config_schema_url: String::new(),
            update_url: None,
        }
    }

    fn license_text(&mut self) -> String {
        include_str!("../../LICENSE").to_string()
    }

    fn check_config_updates(
        &self,
        _message: CheckConfigUpdatesMessage,
    ) -> Result<Vec<ConfigChange>, FormatError> {
        Ok(Vec::new())
    }

    fn format(
        &mut self,
        request: SyncFormatRequest<Configuration>,
        _format_with_host: impl FnMut(SyncHostFormatRequest) -> FormatResult,
    ) -> FormatResult {
        // class order is a whole-attribute property, so a partial range can't
        // be sorted on its own
        if request.range.is_some() {
            return Ok(None);
        }

        // skipped like the CLI does, since an additive plugin erroring would fail
        // `dprint check` on legacy-encoded templates no other plugin touches
        let Ok(text) = String::from_utf8(request.file_bytes) else {
            return Ok(None);
        };

        Ok(format_text(request.file_path, &text, request.config).map(String::into_bytes))
    }
}

/// Sorts the classes in `text`, returning `None` when nothing changes
///
/// Files containing a dprint ignore directive are left unchanged
pub fn format_text(file_path: &Path, text: &str, config: &Configuration) -> Option<String> {
    // `dprint-ignore` exempts the next node, which only the claiming plugin's
    // parser can delimit, so the whole file is skipped rather than risk sorting
    // excluded classes
    if has_ignore_directive(text) {
        return None;
    }

    let language = SourceLanguage::from_path(file_path);
    let sorted = config
        .rustywind
        .sort_document(SourceDocument::new(text, language));

    (sorted != text).then(|| sorted.into_owned())
}

/// Comment openers across the supported languages, where `*` also covers `/*`,
/// JSX `{/*`, and block comment continuation lines, and `#` covers Ruby, ERB
/// `<%#`, and Jinja or Twig `{#`
const COMMENT_OPENERS: &[&str] = &["//", "*", "<!--", "{{!--", "{{--", "{{!", "#"];

// a directive must open a comment, so prose or a class named `dprint-ignore`
// keeps sorting. The prefix also covers `-file`, `-start`, and `-end`
fn has_ignore_directive(text: &str) -> bool {
    text.match_indices("dprint-ignore").any(|(index, _)| {
        let before = text[..index].trim_end_matches([' ', '\t']);
        COMMENT_OPENERS
            .iter()
            .any(|opener| before.ends_with(opener))
    })
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
dprint_core::generate_plugin_code!(
    RustyWindPluginHandler,
    RustyWindPluginHandler,
    Configuration
);

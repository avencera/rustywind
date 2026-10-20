use dprint_core::configuration::{
    ConfigKeyMap, ConfigKeyValue, ConfigurationDiagnostic, ResolveConfigurationResult,
    get_nullable_value, get_nullable_vec, get_unknown_property_diagnostics, get_value,
};
use regex::Regex;
use rustywind_core::RustyWind;
use rustywind_core::class_wrapping::ClassWrapping;
use rustywind_core::sorter::{CustomClassExtractor, FinderRegex, Sorter};
use serde::Serialize;

const CLASS_WRAPPINGS: [ClassWrapping; 3] = [
    ClassWrapping::NoWrapping,
    ClassWrapping::CommaSingleQuotes,
    ClassWrapping::CommaDoubleQuotes,
];

/// Resolved plugin configuration
///
/// Mirrors the `rustywind` CLI options that make sense without filesystem or
/// network access, using camelCase keys in the `rustywind` section of a dprint
/// config file
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    /// Keep repeated classes instead of removing duplicates
    pub allow_duplicates: bool,
    /// How classes captured by `customRegex` are wrapped
    pub class_wrapping: &'static str,
    /// Regex whose `classes` or first capture group contains a class list
    pub custom_regex: Option<String>,
    /// Tailwind prefix, such as `tw-`, ignored while computing sort order
    pub tailwind_prefix: Option<String>,
    /// Keep the original whitespace between classes
    pub preserve_whitespace: bool,
    /// Explicit class order that replaces the built-in Tailwind ordering
    pub sort_order: Option<Vec<String>>,
    // dprint caches the resolved configuration per config id, so building the
    // sorter here compiles a custom regex once instead of once per file
    #[serde(skip)]
    pub(crate) rustywind: RustyWind,
}

/// Resolves a dprint `rustywind` config section into a [`Configuration`]
///
/// Invalid values fall back to their defaults and are reported as diagnostics
pub fn resolve_config(mut config: ConfigKeyMap) -> ResolveConfigurationResult<Configuration> {
    let mut diagnostics = Vec::new();

    let allow_duplicates = get_value(&mut config, "allowDuplicates", false, &mut diagnostics);
    let preserve_whitespace = get_value(&mut config, "preserveWhitespace", false, &mut diagnostics);
    let tailwind_prefix: Option<String> =
        get_nullable_value(&mut config, "tailwindPrefix", &mut diagnostics);
    let class_wrapping = resolve_class_wrapping(&mut config, &mut diagnostics);
    let (custom_regex, regex) = resolve_custom_regex(&mut config, &mut diagnostics);
    // `get_nullable_vec` rejects null, while every other option treats it as unset
    if matches!(config.get("sortOrder"), Some(ConfigKeyValue::Null)) {
        config.shift_remove("sortOrder");
    }
    let sort_order = get_nullable_vec(&mut config, "sortOrder", sort_order_item, &mut diagnostics);

    diagnostics.extend(get_unknown_property_diagnostics(config));

    let sorter = match &sort_order {
        Some(classes) => Sorter::new(
            classes
                .iter()
                .enumerate()
                .map(|(index, class)| (class.clone(), index))
                .collect(),
        ),
        None => Sorter::PatternSorter,
    };

    let rustywind = RustyWind {
        regex,
        sorter,
        allow_duplicates,
        class_wrapping,
        tailwind_prefix: tailwind_prefix.clone(),
        preserve_whitespace,
    };

    ResolveConfigurationResult {
        config: Configuration {
            allow_duplicates,
            class_wrapping: class_wrapping.as_str(),
            custom_regex,
            tailwind_prefix,
            preserve_whitespace,
            sort_order,
            rustywind,
        },
        diagnostics,
    }
}

// validated here because `ClassWrapping::from` silently maps unknown values to
// `no-wrapping`, which would hide a typo in the config file
fn resolve_class_wrapping(
    config: &mut ConfigKeyMap,
    diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> ClassWrapping {
    let Some(value) = get_nullable_value::<String>(config, "classWrapping", diagnostics) else {
        return ClassWrapping::NoWrapping;
    };

    if let Some(wrapping) = CLASS_WRAPPINGS
        .into_iter()
        .find(|wrapping| wrapping.as_str() == value)
    {
        return wrapping;
    }

    let expected = CLASS_WRAPPINGS.map(|wrapping| format!("'{}'", wrapping.as_str()));
    diagnostics.push(ConfigurationDiagnostic {
        property_name: "classWrapping".to_string(),
        message: format!("Expected one of {}", expected.join(", ")),
    });

    ClassWrapping::NoWrapping
}

fn resolve_custom_regex(
    config: &mut ConfigKeyMap,
    diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> (Option<String>, FinderRegex) {
    let Some(pattern) = get_nullable_value::<String>(config, "customRegex", diagnostics) else {
        return (None, FinderRegex::DefaultRegex);
    };

    let extractor = Regex::new(&pattern)
        .map_err(|error| error.to_string())
        .and_then(|regex| CustomClassExtractor::new(regex).map_err(|error| error.to_string()));

    match extractor {
        Ok(extractor) => (Some(pattern), FinderRegex::CustomRegex(extractor)),
        Err(message) => {
            diagnostics.push(ConfigurationDiagnostic {
                property_name: "customRegex".to_string(),
                message,
            });

            (None, FinderRegex::DefaultRegex)
        }
    }
}

fn sort_order_item(
    value: ConfigKeyValue,
    index: usize,
    diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> Option<String> {
    match value {
        ConfigKeyValue::String(class) => Some(class),
        _ => {
            diagnostics.push(ConfigurationDiagnostic {
                property_name: format!("sortOrder[{index}]"),
                message: "Expected a class name string".to_string(),
            });

            None
        }
    }
}

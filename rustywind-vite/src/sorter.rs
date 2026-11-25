use std::io::BufReader;

use color_eyre::Help;
use eyre::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use rustywind_core::sorter::Sorter;
use ureq::tls::TlsConfig;

/// Captures the quoted string literal, with its quotes, that holds the CSS.
static VITE_CSS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"const __vite__css = ("(?:[^"\\]|\\.)*")"#).unwrap());

pub fn create_sorter(url: &str, skip_ssl_verification: bool) -> Result<Sorter> {
    let mut agent = ureq::Agent::config_builder();

    if skip_ssl_verification && url.starts_with("https") {
        let tls_config = TlsConfig::builder().disable_verification(true).build();
        agent = agent.tls_config(tls_config);
    }

    let config = agent.build();
    let agent = ureq::Agent::new_with_config(config);

    let mut css_string_response = agent.get(url).call()
        .wrap_err_with(|| format!("Vite url ({url}) is not valid"))
        .with_suggestion(|| format!("Make sure the URL is correct, try running curl {url}, to see if you get the css file"));

    if css_string_response.is_err() && url.starts_with("https") {
        css_string_response = css_string_response
            .with_suggestion(|| "Try running with the --skip-ssl-verification flag");
    }

    let module = css_string_response?.into_body().read_to_string()?;
    sorter_from_module(&module)
}

/// Builds the sorter from the JavaScript module that the Vite dev server sends for a CSS file.
///
/// Vite writes the module as `const __vite__css = ${JSON.stringify(css)}`, so the CSS is one
/// JSON string literal.
///
/// # Errors
///
/// Returns an error when the module has no `__vite__css` string literal, or when the literal is
/// not a valid JSON string.
fn sorter_from_module(module: &str) -> Result<Sorter> {
    let captures = VITE_CSS_RE
        .captures(module)
        .ok_or_else(|| eyre::eyre!("Could not find css string in vite css file"))?;
    let literal = &captures[1];

    let css: String = serde_json::from_str(literal)
        .wrap_err("Could not decode the css string in vite css file")?;
    Sorter::new_from_reader(BufReader::new(css.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds the module that the Vite dev server sends for a CSS file. Vite embeds the CSS with
    /// `JSON.stringify`, which gives the same text as `serde_json::to_string` for this CSS.
    fn vite_module(css: &str) -> String {
        let css_literal = serde_json::to_string(css).unwrap();
        format!(
            "import {{ updateStyle as __vite__updateStyle }} from \"/@vite/client\"\n\
             const __vite__id = \"/src/app.css\"\n\
             const __vite__css = {css_literal}\n\
             __vite__updateStyle(__vite__id, __vite__css)\n"
        )
    }

    #[test]
    fn reads_the_css_inside_a_javascript_string_literal() {
        // In the module, the escaped class becomes `\\:` and the quotes become `\"`.
        let css = [
            ".flex { display: flex; }",
            ":where(.divide-y > :not(:last-child)) { border-top-width: 1px; }",
            r#".hover\:bg-w:hover { content: "x"; }"#,
            ".é { }",
        ]
        .join("\n");

        let sorter = sorter_from_module(&vite_module(&css)).unwrap();

        assert_eq!(sorter.get("flex"), Some(&0));
        assert_eq!(sorter.get("divide-y"), Some(&1));
        assert_eq!(sorter.get("hover:bg-w"), Some(&2));
        assert_eq!(sorter.get("é"), Some(&3));
        assert_eq!(sorter.len(), 4);
    }

    #[test]
    fn decodes_a_unicode_escape_of_the_javascript_string_literal() {
        // The module line is literal text: `é` is the escape for `é`.
        let sorter = sorter_from_module(r#"const __vite__css = ".é { }""#).unwrap();

        assert_eq!(sorter.get("é"), Some(&0));
        assert_eq!(sorter.len(), 1);
    }

    #[test]
    fn reports_a_module_without_the_css_constant() {
        let error = sorter_from_module("export default {}").unwrap_err();

        assert_eq!(
            error.to_string(),
            "Could not find css string in vite css file"
        );
    }

    #[test]
    fn reports_a_css_literal_that_is_not_valid_json() {
        // `\x` is a valid escape in JavaScript and an invalid one in JSON.
        let error = sorter_from_module(r#"const __vite__css = ".a\x41 { }""#).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Could not decode the css string in vite css file"
        );
    }
}

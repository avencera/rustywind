use std::{io::BufReader, sync::Arc};

use color_eyre::Help;
use eyre::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use ureq::AgentBuilder;

use crate::{parser, tls::NoCertificateVerification};

use super::Sorter;

static VITE_CSS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"const __vite__css = "(.*)""#).unwrap());

pub fn create_sorter(url: &str, skip_ssl_verification: bool) -> Result<Sorter> {
    let mut agent = AgentBuilder::new();

    if skip_ssl_verification && url.starts_with("https") {
        let ssl = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification {}))
            .with_no_client_auth();

        agent = agent.tls_config(Arc::new(ssl));
    }

    let agent = agent.build();

    let mut css_string_response = agent.get(url).call()
        .wrap_err_with(|| format!("Vite url ({url}) is not valid"))
        .with_suggestion(|| format!("Make sure the URL is correct, try running curl {url}, to see if you get the css file"));

    if css_string_response.is_err() && url.starts_with("https") {
        css_string_response = css_string_response
            .with_suggestion(|| "Try running with the --skip-ssl-verification flag");
    }

    let css_string = css_string_response?.into_string()?;
    let css_string = VITE_CSS_RE
        .captures(&css_string)
        .ok_or_else(|| eyre::eyre!("Could not find css string in vite css file"))?
        .get(1)
        .ok_or_else(|| eyre::eyre!("First capture not found"))?
        .as_str();

    let reader = BufReader::new(css_string.as_bytes());
    let sorter = parser::parse_classes(reader)
        .wrap_err("Error parsing css classes from the vite css file")?;

    Ok(Sorter::CustomSorter(sorter))
}

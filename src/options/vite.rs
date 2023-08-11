use std::io::BufReader;

use color_eyre::Help;
use eyre::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::parser;

use super::Sorter;

static VITE_CSS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"const __vite__css = "(.*)""#).unwrap());

pub fn create_sorter(url: &str) -> Result<Sorter> {
    let css_string = ureq::get(url).call()
        .wrap_err_with(|| format!("Vite url ({url}) is not valid"))
        .with_suggestion(|| format!("Make sure the URL is correct, try running curl {url}, to see if you get the css file"))?
        .into_string()?;

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

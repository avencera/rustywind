use once_cell::sync::Lazy;
use regex::Regex;

pub(crate) static PARSER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\.[^\s]+)[ ]").unwrap());

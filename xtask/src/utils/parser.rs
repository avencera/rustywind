use regex::Regex;
use std::sync::OnceLock;

/// Get the regex pattern for parsing pass count
fn pass_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN
        .get_or_init(|| Regex::new(r"(\d+) passed").expect("Failed to compile pass pattern regex"))
}

/// Parse the number of passed tests from test output
pub fn parse_pass_count(output: &str) -> Option<usize> {
    let pattern = pass_pattern();
    pattern.captures(output).and_then(|cap| cap[1].parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pass_count() {
        let output = "Tests: 95 passed, 5 failed, 100 total";
        assert_eq!(parse_pass_count(output), Some(95));
    }

    #[test]
    fn test_parse_pass_count_no_match() {
        let output = "All tests failed";
        assert_eq!(parse_pass_count(output), None);
    }
}

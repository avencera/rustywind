use regex::Regex;
use std::sync::OnceLock;

/// Get the regex pattern for parsing pass count
fn pass_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN
        .get_or_init(|| Regex::new(r"(\d+) passed").expect("Failed to compile pass pattern regex"))
}

/// Get the regex pattern for parsing pass and failure counts
fn result_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"(\d+) passed,\s+(\d+) failed")
            .expect("Failed to compile result count pattern regex")
    })
}

/// Parse the number of passed tests from test output
pub fn parse_pass_count(output: &str) -> Option<usize> {
    let pattern = pass_pattern();
    let total: usize = pattern
        .captures_iter(output)
        .filter_map(|cap| cap[1].parse::<usize>().ok())
        .sum();

    (total > 0).then_some(total)
}

/// Parse the total number of tests from test output
pub fn parse_total_count(output: &str) -> Option<usize> {
    let pattern = result_pattern();
    let total: usize = pattern
        .captures_iter(output)
        .filter_map(|cap| {
            let passed = cap[1].parse::<usize>().ok()?;
            let failed = cap[2].parse::<usize>().ok()?;
            Some(passed + failed)
        })
        .sum();

    (total > 0).then_some(total)
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
    fn test_parse_pass_count_sums_multiple_suites() {
        let output = "Results: 100 passed, 0 failed\nResults: 98 passed, 2 failed";
        assert_eq!(parse_pass_count(output), Some(198));
    }

    #[test]
    fn test_parse_total_count_sums_multiple_suites() {
        let output = "Results: 100 passed, 0 failed\nResults: 98 passed, 2 failed";
        assert_eq!(parse_total_count(output), Some(200));
    }

    #[test]
    fn test_parse_pass_count_no_match() {
        let output = "All tests failed";
        assert_eq!(parse_pass_count(output), None);
    }
}

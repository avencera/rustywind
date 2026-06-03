use std::borrow::Cow;

/// Normalize a Tailwind-prefixed class into the class name used for sorting.
///
/// Tailwind v3 places the prefix on the utility (`md:tw-text-lg`), while v4
/// places it as the first variant-like segment (`tw:md:text-lg`). The returned
/// value is only used to compute sort keys; callers should keep the original
/// class string for output.
pub fn normalize_tailwind_prefix<'a>(class: &'a str, prefix: Option<&str>) -> Cow<'a, str> {
    let Some(prefix) = prefix.and_then(normalize_tailwind_prefix_value) else {
        return Cow::Borrowed(class);
    };

    if let Some(rest) = class
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix(':'))
        && !rest.is_empty()
    {
        return Cow::Borrowed(rest);
    }

    let utility_start = utility_start(class);
    let (variants, utility) = class.split_at(utility_start);

    match normalize_v3_utility(utility, prefix) {
        Some(Cow::Borrowed(utility)) if variants.is_empty() => Cow::Borrowed(utility),
        Some(utility) => Cow::Owned(format!("{variants}{utility}")),
        None => Cow::Borrowed(class),
    }
}

pub(crate) fn normalize_tailwind_prefix_value(prefix: &str) -> Option<&str> {
    let prefix = prefix.trim_end_matches(['-', ':']);
    (!prefix.is_empty()).then_some(prefix)
}

fn utility_start(class: &str) -> usize {
    let mut start = 0;
    let mut bracket_depth: u32 = 0;

    for (index, character) in class.char_indices() {
        match character {
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ':' if bracket_depth == 0 => start = index + 1,
            _ => {}
        }
    }

    start
}

fn normalize_v3_utility<'a>(utility: &'a str, prefix: &str) -> Option<Cow<'a, str>> {
    let (important, utility) = utility
        .strip_prefix('!')
        .map_or(("", utility), |utility| ("!", utility));
    let prefixed = format!("{prefix}-");

    if let Some(rest) = utility.strip_prefix(&prefixed) {
        return Some(with_important_prefix(important, rest));
    }

    if let Some(rest) = utility
        .strip_prefix('-')
        .and_then(|utility| utility.strip_prefix(&prefixed))
    {
        return Some(Cow::Owned(format!("{important}-{rest}")));
    }

    None
}

fn with_important_prefix<'a>(important: &str, utility: &'a str) -> Cow<'a, str> {
    if important.is_empty() {
        Cow::Borrowed(utility)
    } else {
        Cow::Owned(format!("{important}{utility}"))
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_tailwind_prefix;

    #[test]
    fn normalizes_tailwind_v3_prefixes() {
        assert_eq!(
            normalize_tailwind_prefix("tw-text-xl", Some("tw")),
            "text-xl"
        );
        assert_eq!(
            normalize_tailwind_prefix("md:tw-text-xl", Some("tw")),
            "md:text-xl"
        );
        assert_eq!(
            normalize_tailwind_prefix("hover:-tw-mr-4", Some("tw")),
            "hover:-mr-4"
        );
    }

    #[test]
    fn normalizes_tailwind_v4_prefixes() {
        assert_eq!(
            normalize_tailwind_prefix("tw:text-xl", Some("tw")),
            "text-xl"
        );
        assert_eq!(
            normalize_tailwind_prefix("tw:md:text-xl", Some("tw")),
            "md:text-xl"
        );
        assert_eq!(
            normalize_tailwind_prefix("tw:hover:-mr-4", Some("tw")),
            "hover:-mr-4"
        );
    }

    #[test]
    fn supports_dash_suffix_in_configured_prefix() {
        assert_eq!(
            normalize_tailwind_prefix("md:tw-text-xl", Some("tw-")),
            "md:text-xl"
        );
        assert_eq!(
            normalize_tailwind_prefix("tw:md:text-xl", Some("tw-")),
            "md:text-xl"
        );
    }

    #[test]
    fn preserves_important_modifiers() {
        assert_eq!(
            normalize_tailwind_prefix("hover:!tw-bg-red-500", Some("tw")),
            "hover:!bg-red-500"
        );
        assert_eq!(
            normalize_tailwind_prefix("hover:!-tw-mr-4", Some("tw")),
            "hover:!-mr-4"
        );
    }

    #[test]
    fn ignores_colons_inside_arbitrary_variants() {
        assert_eq!(
            normalize_tailwind_prefix("[&:hover]:tw-bg-red-500", Some("tw")),
            "[&:hover]:bg-red-500"
        );
    }

    #[test]
    fn leaves_unprefixed_classes_unchanged() {
        assert_eq!(
            normalize_tailwind_prefix("md:text-xl", Some("tw")),
            "md:text-xl"
        );
        assert_eq!(normalize_tailwind_prefix("tw-text-xl", None), "tw-text-xl");
    }
}

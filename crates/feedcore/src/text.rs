//! Text normalization for titles and summaries.

/// Placeholder stored when an item has no title.
pub const UNTITLED: &str = "(untitled)";

/// Normalize an optional title: trim, and fall back to [`UNTITLED`] when the
/// value is absent or empty after trimming.
///
/// Entity unescaping and CDATA handling happen in the parser as text is read;
/// this only applies the missing-title rule.
pub fn title_or_untitled(raw: Option<String>) -> String {
    match raw {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => UNTITLED.to_string(),
    }
}

/// Case-insensitive (ASCII) substring test used by the `q` entries filter.
pub fn contains_ascii_ci(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_title() {
        assert_eq!(title_or_untitled(None), "(untitled)");
        assert_eq!(title_or_untitled(Some("   ".into())), "(untitled)");
        assert_eq!(title_or_untitled(Some(" Hi ".into())), "Hi");
    }

    #[test]
    fn ci_substring() {
        assert!(contains_ascii_ci("Hello World", "world"));
        assert!(contains_ascii_ci("Hello", ""));
        assert!(!contains_ascii_ci("Hello", "xyz"));
    }
}

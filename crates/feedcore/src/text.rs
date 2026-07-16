//! Text handling for feed titles and summaries.
//!
//! quick-xml already unescapes XML entities and yields CDATA verbatim while
//! reading text nodes, so [`normalize`] just trims trailing/leading
//! whitespace-only noise conservatively. [`title_or_untitled`] applies the
//! pinned rule that a missing title is stored as `(untitled)`.

/// The placeholder stored when a feed item has no usable title.
pub const UNTITLED: &str = "(untitled)";

/// Collapse an optional, possibly-empty title into a concrete stored title.
pub fn title_or_untitled(raw: Option<String>) -> String {
    match raw {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => UNTITLED.to_string(),
    }
}

/// Normalize a text field: trim outer whitespace; `None` if empty.
pub fn clean_opt(raw: Option<String>) -> Option<String> {
    match raw {
        Some(t) if !t.trim().is_empty() => Some(t.trim().to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_title_is_untitled() {
        assert_eq!(title_or_untitled(None), "(untitled)");
        assert_eq!(title_or_untitled(Some("   ".into())), "(untitled)");
    }

    #[test]
    fn present_title_trimmed() {
        assert_eq!(title_or_untitled(Some("  Hello  ".into())), "Hello");
    }

    #[test]
    fn clean_opt_empty_is_none() {
        assert_eq!(clean_opt(Some(" ".into())), None);
        assert_eq!(clean_opt(None), None);
        assert_eq!(clean_opt(Some(" x ".into())), Some("x".to_string()));
    }
}

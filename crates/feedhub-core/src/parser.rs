//! RSS 2.0 and Atom (RFC 4287) parsing.
//!
//! Hand-rolled on top of `quick-xml` rather than delegating to a general feed
//! crate. The spec pins identity, date, and text semantics precisely — notably
//! that a missing date stores NULL and is *never* backfilled with fetch time —
//! and general-purpose feed crates make their own (reasonable, different)
//! choices there. Owning the mapping keeps those pins enforceable and
//! unit-testable.

use crate::datetime::parse_feed_date;
use crate::model::{FeedKind, ParsedEntry, ParsedFeed, UNTITLED};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("feed is not valid UTF-8")]
    NotUtf8,
    #[error("malformed XML: {0}")]
    Xml(String),
    #[error("unrecognized feed format: expected an RSS <rss> or an Atom <feed> root element")]
    UnknownFormat,
}

/// UTF-8 byte-order mark. Feeds in the wild ship it; the spec says tolerate it.
const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Which text field an active capture is filling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    FeedTitle,
    Guid,
    Id,
    Title,
    Link,
    Description,
    Summary,
    Content,
    PubDate,
    Published,
    Updated,
}

/// An in-progress text capture.
///
/// `depth` is the stack depth at which the capture opened, so nested markup
/// (Atom `content type="xhtml"`, for instance) contributes its text without
/// prematurely ending the capture.
struct Capture {
    field: Field,
    depth: usize,
    text: String,
}

#[derive(Default)]
struct ItemBuilder {
    guid: Option<String>,
    id: Option<String>,
    title: Option<String>,
    link: Option<String>,
    /// Atom `<link>` elements in document order: `(rel, href)`.
    links: Vec<(Option<String>, String)>,
    description: Option<String>,
    summary: Option<String>,
    content: Option<String>,
    pub_date: Option<String>,
    published: Option<String>,
    updated: Option<String>,
}

/// Treat empty / whitespace-only text as absent.
fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}

impl ItemBuilder {
    fn set(&mut self, field: Field, text: String) {
        let slot = match field {
            Field::Guid => &mut self.guid,
            Field::Id => &mut self.id,
            Field::Title => &mut self.title,
            Field::Link => &mut self.link,
            Field::Description => &mut self.description,
            Field::Summary => &mut self.summary,
            Field::Content => &mut self.content,
            Field::PubDate => &mut self.pub_date,
            Field::Published => &mut self.published,
            Field::Updated => &mut self.updated,
            Field::FeedTitle => return,
        };
        // First occurrence wins; feeds occasionally repeat elements.
        if slot.is_none() {
            *slot = Some(text);
        }
    }

    /// Pick the Atom entry link: `rel="alternate"` first, else the first link.
    ///
    /// RFC 4287 §4.2.7.2 makes `rel` default to `alternate` when absent, so a
    /// bare `<link href="..."/>` counts as an alternate.
    fn atom_link(&self) -> Option<String> {
        self.links
            .iter()
            .find(|(rel, _)| matches!(rel.as_deref(), None | Some("alternate")))
            .or_else(|| self.links.first())
            .map(|(_, href)| href.clone())
    }

    fn finish(self, kind: FeedKind) -> Option<ParsedEntry> {
        let (guid, link, summary, date) = match kind {
            FeedKind::Rss => {
                let link = non_empty(self.link.clone());
                // Identity is guid, falling back to link.
                let guid = non_empty(self.guid).or_else(|| link.clone())?;
                (guid, link, non_empty(self.description), self.pub_date)
            }
            FeedKind::Atom => {
                let link = non_empty(self.atom_link());
                // Identity is id, with no fallback.
                let guid = non_empty(self.id)?;
                let summary = non_empty(self.summary).or_else(|| non_empty(self.content));
                let date = self.published.or(self.updated);
                (guid, link, summary, date)
            }
        };

        Some(ParsedEntry {
            guid,
            // A missing title becomes the pinned placeholder; the entry is kept.
            title: non_empty(self.title).unwrap_or_else(|| UNTITLED.to_string()),
            link,
            summary,
            // Missing or unparseable dates are NULL. Never fetch time.
            published_at: date.as_deref().and_then(parse_feed_date),
        })
    }
}

fn local_name(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).into_owned()
}

/// Read the `rel` and `href` of an Atom `<link>` element.
fn read_link_attrs(e: &BytesStart<'_>) -> (Option<String>, Option<String>) {
    let mut rel = None;
    let mut href = None;
    for attr in e.attributes().flatten() {
        let key = local_name(attr.key.local_name().as_ref());
        let value = match attr.unescape_value() {
            Ok(v) => v.into_owned(),
            Err(_) => continue,
        };
        match key.as_str() {
            "rel" => rel = Some(value),
            "href" => href = Some(value),
            _ => {}
        }
    }
    (rel, href)
}

/// Map an element inside an `<item>`/`<entry>` to the field it fills.
fn field_for_item(kind: FeedKind, name: &str) -> Option<Field> {
    let field = match (kind, name) {
        (FeedKind::Rss, "guid") => Field::Guid,
        (FeedKind::Rss, "title") => Field::Title,
        (FeedKind::Rss, "link") => Field::Link,
        (FeedKind::Rss, "description") => Field::Description,
        (FeedKind::Rss, "pubDate") => Field::PubDate,

        (FeedKind::Atom, "id") => Field::Id,
        (FeedKind::Atom, "title") => Field::Title,
        (FeedKind::Atom, "summary") => Field::Summary,
        (FeedKind::Atom, "content") => Field::Content,
        (FeedKind::Atom, "published") => Field::Published,
        (FeedKind::Atom, "updated") => Field::Updated,
        _ => return None,
    };
    Some(field)
}

/// Recognize the feed-level `<title>`, which must not be confused with an
/// item's own `<title>`.
fn feed_title_field(kind: FeedKind, name: &str, stack: &[String]) -> Option<Field> {
    if name != "title" {
        return None;
    }
    let path: Vec<&str> = stack.iter().map(String::as_str).collect();
    match (kind, path.as_slice()) {
        (FeedKind::Rss, ["rss", "channel"]) => Some(Field::FeedTitle),
        (FeedKind::Atom, ["feed"]) => Some(Field::FeedTitle),
        _ => None,
    }
}

#[derive(Default)]
struct ParserState {
    kind: Option<FeedKind>,
    stack: Vec<String>,
    feed_title: Option<String>,
    entries: Vec<ParsedEntry>,
    item: Option<ItemBuilder>,
    capture: Option<Capture>,
    skipped: usize,
}

impl ParserState {
    fn open(&mut self, e: &BytesStart<'_>) -> Result<(), ParseError> {
        let name = local_name(e.local_name().as_ref());

        // The root element decides the grammar.
        if self.stack.is_empty() {
            self.kind = Some(match name.as_str() {
                "rss" => FeedKind::Rss,
                "feed" => FeedKind::Atom,
                _ => return Err(ParseError::UnknownFormat),
            });
            self.stack.push(name);
            return Ok(());
        }

        let kind = self.kind.ok_or(ParseError::UnknownFormat)?;
        let parent = self.stack.last().map(String::as_str).unwrap_or_default();

        let starts_item = match kind {
            FeedKind::Rss => name == "item" && parent == "channel",
            FeedKind::Atom => name == "entry" && parent == "feed",
        };
        if starts_item && self.item.is_none() {
            self.item = Some(ItemBuilder::default());
            self.stack.push(name);
            return Ok(());
        }

        // Atom entry links carry their data in attributes, not in text.
        if kind == FeedKind::Atom && name == "link" && parent == "entry" {
            let (rel, href) = read_link_attrs(e);
            if let (Some(href), Some(builder)) = (non_empty(href), self.item.as_mut()) {
                builder.links.push((rel, href));
            }
            self.stack.push(name);
            return Ok(());
        }

        // Open a text capture, unless one is already running (nested markup).
        if self.capture.is_none() {
            let field = if self.item.is_some() {
                if matches!(parent, "item" | "entry") {
                    field_for_item(kind, &name)
                } else {
                    None
                }
            } else {
                feed_title_field(kind, &name, &self.stack)
            };
            if let Some(field) = field {
                self.capture = Some(Capture {
                    field,
                    depth: self.stack.len(),
                    text: String::new(),
                });
            }
        }

        self.stack.push(name);
        Ok(())
    }

    fn close(&mut self) {
        // Close an active capture when its own element ends.
        if matches!(&self.capture, Some(c) if self.stack.len() == c.depth + 1) {
            if let Some(cap) = self.capture.take() {
                let text = cap.text.trim().to_string();
                match cap.field {
                    Field::FeedTitle => {
                        if self.feed_title.is_none() {
                            self.feed_title = non_empty(Some(text));
                        }
                    }
                    other => {
                        if let Some(builder) = self.item.as_mut() {
                            builder.set(other, text);
                        }
                    }
                }
            }
        }

        let Some(name) = self.stack.pop() else {
            return;
        };

        let ends_item = match self.kind {
            Some(FeedKind::Rss) => name == "item",
            Some(FeedKind::Atom) => name == "entry",
            None => false,
        };
        if ends_item {
            if let (Some(builder), Some(kind)) = (self.item.take(), self.kind) {
                match builder.finish(kind) {
                    Some(entry) => self.entries.push(entry),
                    // No usable identity: counted, not silently dropped.
                    None => self.skipped += 1,
                }
            }
        }
    }

    fn push_text(&mut self, text: &str) {
        if let Some(cap) = self.capture.as_mut() {
            cap.text.push_str(text);
        }
    }
}

/// Outcome of parsing, including entries dropped for lacking an identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutcome {
    pub feed: ParsedFeed,
    /// Items that carried no usable dedupe identity and were dropped.
    ///
    /// Surfaced rather than swallowed: a feed whose every item lacks an
    /// identity would otherwise be indistinguishable from an empty feed.
    pub skipped_without_identity: usize,
}

/// Parse an RSS 2.0 or Atom document.
///
/// Tolerates a leading UTF-8 BOM. Returns [`ParseError`] for non-UTF-8 input,
/// malformed XML, or a root element that is neither `<rss>` nor `<feed>`.
pub fn parse_feed(bytes: &[u8]) -> Result<ParseOutcome, ParseError> {
    let bytes = bytes.strip_prefix(BOM).unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|_| ParseError::NotUtf8)?;

    let mut reader = Reader::from_str(text);
    // Malformed documents must surface as an error, not be silently salvaged.
    reader.config_mut().check_end_names = true;

    let mut st = ParserState::default();

    loop {
        let event = reader
            .read_event()
            .map_err(|e| ParseError::Xml(e.to_string()))?;

        match event {
            Event::Eof => break,
            Event::Start(e) => st.open(&e)?,
            Event::Empty(e) => {
                // Self-closing: opens and closes in one event.
                st.open(&e)?;
                st.close();
            }
            Event::End(_) => st.close(),
            Event::Text(e) => {
                if st.capture.is_some() {
                    // Entity references are resolved here: `&amp;` -> `&`.
                    let decoded = e.unescape().map_err(|e| ParseError::Xml(e.to_string()))?;
                    st.push_text(&decoded);
                }
            }
            Event::CData(e) => {
                if st.capture.is_some() {
                    // CDATA is taken verbatim: no entity expansion.
                    let raw = e.into_inner();
                    let decoded = String::from_utf8_lossy(&raw).into_owned();
                    st.push_text(&decoded);
                }
            }
            _ => {}
        }
    }

    let kind = st.kind.ok_or(ParseError::UnknownFormat)?;
    Ok(ParseOutcome {
        feed: ParsedFeed {
            kind,
            title: st.feed_title,
            entries: st.entries,
        },
        skipped_without_identity: st.skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datetime::format_rfc3339;

    fn parse(s: &str) -> ParsedFeed {
        parse_feed(s.as_bytes()).expect("fixture should parse").feed
    }

    const RSS: &str = r#"<?xml version="1.0"?>
<rss version="2.0"><channel>
  <title>Channel Title</title>
  <link>https://example.com</link>
  <item>
    <title>First</title>
    <link>https://example.com/1</link>
    <guid isPermaLink="false">guid-1</guid>
    <description>Summary one</description>
    <pubDate>Tue, 10 Jun 2003 04:00:00 GMT</pubDate>
  </item>
  <item>
    <title>Second</title>
    <link>https://example.com/2</link>
    <description>Summary two</description>
  </item>
</channel></rss>"#;

    const ATOM: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Feed Title</title>
  <entry>
    <id>urn:uuid:1</id>
    <title>Alpha</title>
    <link rel="self" href="https://example.com/self"/>
    <link rel="alternate" href="https://example.com/alpha"/>
    <summary>Alpha summary</summary>
    <published>2003-12-13T08:29:29-04:00</published>
    <updated>2005-01-01T00:00:00Z</updated>
  </entry>
</feed>"#;

    #[test]
    fn rss_happy_path() {
        let feed = parse(RSS);
        assert_eq!(feed.kind, FeedKind::Rss);
        assert_eq!(feed.title.as_deref(), Some("Channel Title"));
        assert_eq!(feed.entries.len(), 2);

        let first = &feed.entries[0];
        assert_eq!(first.guid, "guid-1");
        assert_eq!(first.title, "First");
        assert_eq!(first.link.as_deref(), Some("https://example.com/1"));
        assert_eq!(first.summary.as_deref(), Some("Summary one"));
        assert_eq!(
            first.published_at.map(format_rfc3339).as_deref(),
            Some("2003-06-10T04:00:00Z")
        );
    }

    #[test]
    fn rss_identity_falls_back_to_link_and_missing_date_is_null() {
        let feed = parse(RSS);
        let second = &feed.entries[1];
        assert_eq!(second.guid, "https://example.com/2", "no guid, so link is identity");
        assert_eq!(
            second.published_at, None,
            "a missing pubDate must be NULL, never fetch time"
        );
    }

    #[test]
    fn rss_channel_title_is_not_taken_from_an_item() {
        // The channel <title> and the item <title> share an element name; only
        // the depth distinguishes them.
        let feed = parse(RSS);
        assert_eq!(feed.title.as_deref(), Some("Channel Title"));
        assert_eq!(feed.entries[0].title, "First");
    }

    #[test]
    fn atom_happy_path_prefers_alternate_link_and_published_over_updated() {
        let feed = parse(ATOM);
        assert_eq!(feed.kind, FeedKind::Atom);
        assert_eq!(feed.title.as_deref(), Some("Feed Title"));

        let entry = &feed.entries[0];
        assert_eq!(entry.guid, "urn:uuid:1");
        assert_eq!(
            entry.link.as_deref(),
            Some("https://example.com/alpha"),
            "rel=alternate wins over an earlier rel=self"
        );
        assert_eq!(
            entry.published_at.map(format_rfc3339).as_deref(),
            Some("2003-12-13T12:29:29Z"),
            "published wins over updated, normalized to UTC"
        );
    }

    #[test]
    fn atom_falls_back_to_updated_and_to_content() {
        let xml = r#"<feed xmlns="http://www.w3.org/2005/Atom">
          <entry>
            <id>e1</id><title>T</title>
            <content>Content body</content>
            <updated>2005-01-01T00:00:00Z</updated>
          </entry></feed>"#;
        let feed = parse(xml);
        let entry = &feed.entries[0];
        assert_eq!(entry.summary.as_deref(), Some("Content body"));
        assert_eq!(
            entry.published_at.map(format_rfc3339).as_deref(),
            Some("2005-01-01T00:00:00Z")
        );
    }

    #[test]
    fn atom_link_without_rel_counts_as_alternate() {
        let xml = r#"<feed xmlns="http://www.w3.org/2005/Atom">
          <entry><id>e1</id><link href="https://example.com/bare"/></entry></feed>"#;
        let feed = parse(xml);
        assert_eq!(feed.entries[0].link.as_deref(), Some("https://example.com/bare"));
    }

    #[test]
    fn atom_falls_back_to_first_link_when_no_alternate_exists() {
        let xml = r#"<feed xmlns="http://www.w3.org/2005/Atom">
          <entry><id>e1</id>
            <link rel="self" href="https://example.com/self"/>
            <link rel="via" href="https://example.com/via"/>
          </entry></feed>"#;
        let feed = parse(xml);
        assert_eq!(feed.entries[0].link.as_deref(), Some("https://example.com/self"));
    }

    #[test]
    fn missing_title_becomes_the_pinned_placeholder() {
        let xml = r#"<rss version="2.0"><channel><item>
            <guid>g1</guid><description>d</description>
          </item></channel></rss>"#;
        let feed = parse(xml);
        assert_eq!(feed.entries[0].title, UNTITLED);
    }

    #[test]
    fn entities_are_unescaped_and_cdata_is_verbatim() {
        let xml = r#"<rss version="2.0"><channel><item>
            <guid>g1</guid>
            <title>Tom &amp; Jerry &lt;b&gt;</title>
            <description><![CDATA[Raw &amp; <b>bold</b>]]></description>
          </item></channel></rss>"#;
        let feed = parse(xml);
        let entry = &feed.entries[0];
        assert_eq!(entry.title, "Tom & Jerry <b>", "entities unescape");
        assert_eq!(
            entry.summary.as_deref(),
            Some("Raw &amp; <b>bold</b>"),
            "CDATA is taken verbatim, so &amp; stays literal"
        );
    }

    #[test]
    fn leading_bom_is_tolerated() {
        let mut bytes = BOM.to_vec();
        bytes.extend_from_slice(RSS.as_bytes());
        let outcome = parse_feed(&bytes).expect("a BOM must not break parsing");
        assert_eq!(outcome.feed.entries.len(), 2);
    }

    #[test]
    fn unparseable_date_stores_null_but_keeps_the_entry() {
        let xml = r#"<rss version="2.0"><channel><item>
            <guid>g1</guid><title>T</title><pubDate>not a date at all</pubDate>
          </item></channel></rss>"#;
        let feed = parse(xml);
        assert_eq!(feed.entries.len(), 1, "the entry is still stored");
        assert_eq!(feed.entries[0].published_at, None);
    }

    #[test]
    fn malformed_xml_is_an_error() {
        let err = parse_feed(b"<rss><channel><item></channel></rss>").unwrap_err();
        assert!(matches!(err, ParseError::Xml(_)), "got {err:?}");
    }

    #[test]
    fn unknown_root_element_is_an_error() {
        assert_eq!(
            parse_feed(b"<html><body>nope</body></html>").unwrap_err(),
            ParseError::UnknownFormat
        );
        assert_eq!(parse_feed(b"").unwrap_err(), ParseError::UnknownFormat);
    }

    #[test]
    fn non_utf8_input_is_an_error() {
        // Latin-1 encoded 'é' is not valid UTF-8. The spec pins UTF-8.
        let bytes = b"<rss version=\"2.0\"><channel><title>caf\xe9</title></channel></rss>";
        assert_eq!(parse_feed(bytes).unwrap_err(), ParseError::NotUtf8);
    }

    #[test]
    fn entries_without_identity_are_skipped_and_counted() {
        let xml = r#"<rss version="2.0"><channel>
            <item><title>No identity at all</title></item>
            <item><guid>g1</guid><title>Fine</title></item>
          </channel></rss>"#;
        let outcome = parse_feed(xml.as_bytes()).unwrap();
        assert_eq!(outcome.feed.entries.len(), 1);
        assert_eq!(outcome.skipped_without_identity, 1);
    }

    #[test]
    fn atom_without_id_is_skipped() {
        let xml = r#"<feed xmlns="http://www.w3.org/2005/Atom">
          <entry><title>No id</title></entry></feed>"#;
        let outcome = parse_feed(xml.as_bytes()).unwrap();
        assert!(outcome.feed.entries.is_empty());
        assert_eq!(outcome.skipped_without_identity, 1);
    }

    #[test]
    fn nested_markup_inside_content_contributes_its_text() {
        let xml = r#"<feed xmlns="http://www.w3.org/2005/Atom">
          <entry><id>e1</id>
            <content type="xhtml"><div xmlns="http://www.w3.org/1999/xhtml">Hello <b>world</b></div></content>
          </entry></feed>"#;
        let feed = parse(xml);
        let summary = feed.entries[0].summary.as_deref().unwrap();
        assert!(summary.contains("Hello"), "got {summary:?}");
        assert!(summary.contains("world"), "got {summary:?}");
    }

    #[test]
    fn empty_feed_parses_to_zero_entries() {
        let feed = parse(r#"<rss version="2.0"><channel><title>Empty</title></channel></rss>"#);
        assert!(feed.entries.is_empty());
        assert_eq!(feed.title.as_deref(), Some("Empty"));
    }
}

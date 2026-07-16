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
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;

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
        // An empty element must never claim the slot. Feeds emit placeholder
        // empties (`<link/>`, `<title></title>`) ahead of the populated
        // element; letting `""` win would lock out the real value, and
        // `finish()` would then turn it back into "absent" — unreachable.
        if text.trim().is_empty() {
            return;
        }
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
    /// Namespace of the root element; `None` when the root is unnamespaced, as
    /// RSS 2.0 is.
    ///
    /// Only elements in this namespace supply feed/item fields. Matching on the
    /// local name alone conflates `<atom:link>` with RSS's own `<link>`, which
    /// is not a hypothetical: `<atom:link rel="self"/>` is near-universal in
    /// real RSS, and it silently destroyed entries.
    root_ns: Option<Vec<u8>>,
    stack: Vec<String>,
    feed_title: Option<String>,
    entries: Vec<ParsedEntry>,
    item: Option<ItemBuilder>,
    /// Stack depth at which the current item opened. The item ends when the
    /// stack returns to it — matching on the element name instead would let a
    /// nested `<item>`/`<entry>` close the real one early.
    item_depth: Option<usize>,
    capture: Option<Capture>,
    skipped: usize,
}

impl ParserState {
    /// Is this element in the feed's own namespace?
    ///
    /// Foreign-namespace elements (`atom:`, `media:`, `dc:`, …) routinely reuse
    /// the local names RSS and Atom care about, so only same-namespace elements
    /// may supply fields.
    fn is_native(&self, ns: &ResolveResult<'_>) -> bool {
        match (ns, &self.root_ns) {
            (ResolveResult::Unbound, None) => true,
            (ResolveResult::Bound(found), Some(root)) => found.as_ref() == root.as_slice(),
            _ => false,
        }
    }

    fn open(&mut self, e: &BytesStart<'_>, ns: &ResolveResult<'_>) -> Result<(), ParseError> {
        let name = local_name(e.local_name().as_ref());

        // The root element decides the grammar, and its namespace becomes the
        // one that counts. Taking it from the root rather than hard-coding the
        // Atom URI keeps prefixed Atom (`<atom:feed><atom:id>`) and
        // namespace-less Atom both working.
        if self.stack.is_empty() {
            self.kind = Some(match name.as_str() {
                "rss" => FeedKind::Rss,
                "feed" => FeedKind::Atom,
                _ => return Err(ParseError::UnknownFormat),
            });
            self.root_ns = match ns {
                ResolveResult::Bound(found) => Some(found.as_ref().to_vec()),
                _ => None,
            };
            self.stack.push(name);
            return Ok(());
        }

        let kind = self.kind.ok_or(ParseError::UnknownFormat)?;
        let parent = self.stack.last().map(String::as_str).unwrap_or_default();
        let native = self.is_native(ns);

        let starts_item = native
            && match kind {
                FeedKind::Rss => name == "item" && parent == "channel",
                FeedKind::Atom => name == "entry" && parent == "feed",
            };
        if starts_item && self.item.is_none() {
            self.item = Some(ItemBuilder::default());
            self.item_depth = Some(self.stack.len());
            self.stack.push(name);
            return Ok(());
        }

        // Atom entry links carry their data in attributes, not in text.
        if kind == FeedKind::Atom && native && name == "link" && parent == "entry" {
            let (rel, href) = read_link_attrs(e);
            if let (Some(href), Some(builder)) = (non_empty(href), self.item.as_mut()) {
                builder.links.push((rel, href));
            }
            self.stack.push(name);
            return Ok(());
        }

        // Open a text capture, unless one is already running (nested markup).
        // A foreign element opens nothing, but text inside an already-open
        // capture still counts — that is how Atom xhtml content works.
        if self.capture.is_none() && native {
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

        if self.stack.pop().is_none() {
            return;
        }

        // Finalize on the item's *depth*, not on the element name. Matching by
        // name means a nested element that happens to be called <item> closes
        // the real item early, dropping every field after it.
        if self.item_depth == Some(self.stack.len()) {
            self.item_depth = None;
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

    // NsReader, not Reader: field mapping has to know an element's namespace,
    // not just its local name.
    let mut reader = NsReader::from_str(text);
    // Malformed documents must surface as an error, not be silently salvaged.
    reader.config_mut().check_end_names = true;

    let mut st = ParserState::default();

    loop {
        let (ns, event) = reader
            .read_resolved_event()
            .map_err(|e| ParseError::Xml(e.to_string()))?;

        match event {
            Event::Eof => break,
            Event::Start(e) => st.open(&e, &ns)?,
            Event::Empty(e) => {
                // Self-closing: opens and closes in one event.
                st.open(&e, &ns)?;
                st.close();
            }
            Event::End(_) => st.close(),
            Event::Text(e) if st.capture.is_some() => {
                // Entity references are resolved here: `&amp;` -> `&`.
                let decoded = e.unescape().map_err(|e| ParseError::Xml(e.to_string()))?;
                st.push_text(&decoded);
            }
            Event::CData(e) if st.capture.is_some() => {
                // CDATA is taken verbatim: no entity expansion.
                let raw = e.into_inner();
                let decoded = String::from_utf8_lossy(&raw).into_owned();
                st.push_text(&decoded);
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
        assert_eq!(
            second.guid, "https://example.com/2",
            "no guid, so link is identity"
        );
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
        assert_eq!(
            feed.entries[0].link.as_deref(),
            Some("https://example.com/bare")
        );
    }

    #[test]
    fn atom_falls_back_to_first_link_when_no_alternate_exists() {
        let xml = r#"<feed xmlns="http://www.w3.org/2005/Atom">
          <entry><id>e1</id>
            <link rel="self" href="https://example.com/self"/>
            <link rel="via" href="https://example.com/via"/>
          </entry></feed>"#;
        let feed = parse(xml);
        assert_eq!(
            feed.entries[0].link.as_deref(),
            Some("https://example.com/self")
        );
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
    fn a_nested_element_named_item_does_not_close_the_real_item() {
        // Finalizing on the element name rather than the item's depth would end
        // the item at the inner </item>, losing every field after it.
        let xml = r#"<rss version="2.0"><channel><item>
            <guid>g1</guid>
            <description><item>a literal item element inside the body</item></description>
            <title>Title after the nested item</title>
          </item></channel></rss>"#;
        let feed = parse(xml);
        assert_eq!(feed.entries.len(), 1, "exactly one entry");
        assert_eq!(
            feed.entries[0].title, "Title after the nested item",
            "fields after the nested <item> must still be captured"
        );
    }

    #[test]
    fn the_channel_image_title_is_not_mistaken_for_the_feed_title() {
        // RSS 2.0 <channel><image><title> is common and shares an element name
        // with the channel's own title.
        let xml = r#"<rss version="2.0"><channel>
            <title>Real Channel Title</title>
            <image>
              <title>Image Alt Text</title>
              <url>https://example.com/logo.png</url>
            </image>
            <item><guid>g1</guid><title>Item</title></item>
          </channel></rss>"#;
        let feed = parse(xml);
        assert_eq!(feed.title.as_deref(), Some("Real Channel Title"));
        assert_eq!(feed.entries[0].title, "Item");
    }

    #[test]
    fn an_image_title_before_the_channel_title_is_still_not_the_feed_title() {
        // First-occurrence-wins would pick the image's title if the path check
        // were not doing the work.
        let xml = r#"<rss version="2.0"><channel>
            <image><title>Image Alt Text</title></image>
            <title>Real Channel Title</title>
          </channel></rss>"#;
        assert_eq!(parse(xml).title.as_deref(), Some("Real Channel Title"));
    }

    #[test]
    fn an_atom_entry_source_does_not_hijack_the_entry_id_or_title() {
        // RFC 4287 §4.2.11: <entry><source> carries the *originating feed's*
        // id/title. Taking them as the entry's own would corrupt identity.
        let xml = r#"<feed xmlns="http://www.w3.org/2005/Atom">
            <title>Aggregator</title>
            <entry>
              <id>real-entry-id</id>
              <title>Real Entry Title</title>
              <source>
                <id>the-source-feeds-id</id>
                <title>The Source Feed</title>
              </source>
            </entry></feed>"#;
        let feed = parse(xml);
        assert_eq!(feed.entries[0].guid, "real-entry-id");
        assert_eq!(feed.entries[0].title, "Real Entry Title");
    }

    #[test]
    fn an_rss_item_source_element_is_not_the_item_title() {
        let xml = r#"<rss version="2.0"><channel><item>
            <guid>g1</guid>
            <title>Real Title</title>
            <source url="https://other.example.com/feed">Other Feed</source>
          </item></channel></rss>"#;
        let feed = parse(xml);
        assert_eq!(feed.entries[0].title, "Real Title");
    }

    #[test]
    fn a_namespaced_child_element_does_not_supply_item_fields() {
        // media:content has local name "content"; an Atom-shaped field name
        // appearing inside an RSS item must not be picked up, and its nested
        // <title> must not become the item's.
        let xml = r#"<rss version="2.0" xmlns:media="http://search.yahoo.com/mrss/">
          <channel><item>
            <guid>g1</guid>
            <title>Real Title</title>
            <media:group><title>Media Title</title></media:group>
          </item></channel></rss>"#;
        let feed = parse(xml);
        assert_eq!(feed.entries[0].title, "Real Title");
    }

    #[test]
    fn a_self_closing_atom_link_in_an_rss_item_does_not_destroy_the_entry() {
        // <atom:link rel="self"/> is near-universal in real RSS. Matching on the
        // local name alone made it Field::Link; being self-closing it captured
        // "", first-occurrence-wins locked the slot, the real <link> was
        // ignored, non_empty turned "" back into None, identity fell through,
        // and the entry vanished with only a WARN.
        let xml = r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
          <channel><item>
            <atom:link rel="self" href="https://example.com/self"/>
            <link>https://example.com/real</link>
            <title>T</title>
          </item></channel></rss>"#;
        let outcome = parse_feed(xml.as_bytes()).unwrap();
        assert_eq!(
            outcome.skipped_without_identity, 0,
            "the entry must survive"
        );
        assert_eq!(outcome.feed.entries.len(), 1);
        assert_eq!(
            outcome.feed.entries[0].link.as_deref(),
            Some("https://example.com/real"),
            "the RSS <link> wins; the foreign atom:link is not a link field"
        );
        // No guid, so identity falls back to the real link.
        assert_eq!(outcome.feed.entries[0].guid, "https://example.com/real");
    }

    #[test]
    fn a_foreign_namespace_element_does_not_hijack_a_field() {
        // media:description before the real description would win under
        // first-occurrence-wins if the namespace were ignored.
        let xml = r#"<rss version="2.0" xmlns:media="http://search.yahoo.com/mrss/">
          <channel><item>
            <guid>g1</guid>
            <media:description>media blurb</media:description>
            <description>real description</description>
            <media:title>media title</media:title>
            <title>real title</title>
          </item></channel></rss>"#;
        let feed = parse(xml);
        assert_eq!(feed.entries[0].summary.as_deref(), Some("real description"));
        assert_eq!(feed.entries[0].title, "real title");
    }

    #[test]
    fn an_atom_feed_using_a_prefix_still_parses() {
        // The native namespace is taken from the root, so prefixed Atom works
        // exactly like default-namespaced Atom.
        let xml = r#"<atom:feed xmlns:atom="http://www.w3.org/2005/Atom">
            <atom:title>Prefixed Feed</atom:title>
            <atom:entry>
              <atom:id>e1</atom:id>
              <atom:title>Prefixed Entry</atom:title>
              <atom:link rel="alternate" href="https://example.com/1"/>
              <atom:updated>2020-01-01T00:00:00Z</atom:updated>
            </atom:entry>
          </atom:feed>"#;
        let feed = parse(xml);
        assert_eq!(feed.title.as_deref(), Some("Prefixed Feed"));
        assert_eq!(feed.entries.len(), 1);
        assert_eq!(feed.entries[0].guid, "e1");
        assert_eq!(feed.entries[0].title, "Prefixed Entry");
        assert_eq!(
            feed.entries[0].link.as_deref(),
            Some("https://example.com/1")
        );
    }

    #[test]
    fn an_empty_element_does_not_lock_out_the_real_value() {
        // Storing "" would satisfy first-occurrence-wins forever, and finish()
        // would then map it back to absent — so the real value could never win.
        let xml = r#"<rss version="2.0"><channel><item>
            <guid>g1</guid>
            <title></title>
            <title>Real Title</title>
            <link/>
            <link>https://example.com/real</link>
            <description></description>
            <description>Real summary</description>
          </item></channel></rss>"#;
        let feed = parse(xml);
        let entry = &feed.entries[0];
        assert_eq!(entry.title, "Real Title");
        assert_eq!(entry.link.as_deref(), Some("https://example.com/real"));
        assert_eq!(entry.summary.as_deref(), Some("Real summary"));
    }

    #[test]
    fn common_html_entities_do_not_destroy_the_feed() {
        // A single &nbsp; used to fail the whole document, taking every entry
        // with it — and real feeds are full of them.
        let xml = r#"<rss version="2.0"><channel><item>
            <guid>g1</guid>
            <title>Q1&nbsp;2024 results &mdash; up 5&percnt;</title>
            <description>Alice&rsquo;s take&hellip;</description>
          </item>
          <item><guid>g2</guid><title>Second survives too</title></item>
          </channel></rss>"#;
        let feed = parse(xml);
        assert_eq!(feed.entries.len(), 2, "no entry may be lost to an entity");
        assert!(
            feed.entries[0].title.contains("2024 results"),
            "got {:?}",
            feed.entries[0].title
        );
        assert!(
            feed.entries[0].title.contains('—'),
            "&mdash; should resolve"
        );
        assert!(
            feed.entries[0]
                .summary
                .as_deref()
                .unwrap()
                .contains('\u{2019}'),
            "&rsquo; should resolve"
        );
    }

    #[test]
    fn empty_feed_parses_to_zero_entries() {
        let feed = parse(r#"<rss version="2.0"><channel><title>Empty</title></channel></rss>"#);
        assert!(feed.entries.is_empty());
        assert_eq!(feed.title.as_deref(), Some("Empty"));
    }
}

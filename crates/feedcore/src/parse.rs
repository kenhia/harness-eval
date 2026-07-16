//! RSS 2.0 and Atom (RFC 4287) parsing.
//!
//! The parser is a single streaming pass over the XML events. Text is stored
//! after XML entity unescaping; CDATA is taken verbatim. Titles fall back to
//! `None` (callers substitute `(untitled)`), and dates are normalized to UTC.

use crate::date::parse_date;
use crate::model::{ParsedEntry, ParsedFeed};
use quick_xml::events::Event;
use quick_xml::Reader;

/// Error returned when a document cannot be recognized or parsed as a feed.
#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "feed parse error: {}", self.0)
    }
}

impl std::error::Error for ParseError {}

fn strip_bom(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes)
}

/// Local element name, lowercased, namespace prefix stripped.
fn local_name(raw: &[u8]) -> String {
    let s = String::from_utf8_lossy(raw);
    let local = s.rsplit(':').next().unwrap_or(&s);
    local.to_ascii_lowercase()
}

#[derive(Default)]
struct EntryBuilder {
    guid: Option<String>,
    id: Option<String>,
    title: Option<String>,
    rss_link: Option<String>,
    alternate_link: Option<String>,
    first_link: Option<String>,
    summary: Option<String>,
    content: Option<String>,
    date: Option<String>,
}

impl EntryBuilder {
    fn finish(self, is_atom: bool) -> ParsedEntry {
        let link = if is_atom {
            self.alternate_link.or(self.first_link)
        } else {
            self.rss_link.clone()
        };
        let guid = if is_atom {
            self.id.clone()
        } else {
            self.guid.clone().or_else(|| self.rss_link.clone())
        }
        .unwrap_or_default();
        let summary = self.summary.or(self.content);
        let published_at = self.date.as_deref().and_then(parse_date);
        ParsedEntry {
            guid,
            title: self.title,
            link,
            summary,
            published_at,
        }
    }
}

/// Parse a feed document (RSS 2.0 or Atom) from raw bytes.
pub fn parse_feed(bytes: &[u8]) -> Result<ParsedFeed, ParseError> {
    let bytes = strip_bom(bytes);
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(false);

    let mut feed = ParsedFeed::default();
    let mut is_atom = false;
    let mut saw_root = false;

    // Stack of lowercased element names currently open.
    let mut stack: Vec<String> = Vec::new();
    // Text accumulator for the innermost element.
    let mut text = String::new();
    let mut current: Option<EntryBuilder> = None;
    let mut in_entry = false;

    let mut buf = Vec::new();
    loop {
        let ev = reader
            .read_event_into(&mut buf)
            .map_err(|e| ParseError(e.to_string()))?;
        match ev {
            Event::Eof => break,
            Event::Start(e) => {
                let name = local_name(e.name().as_ref());
                if !saw_root {
                    saw_root = true;
                    match name.as_str() {
                        "feed" => is_atom = true,
                        "rss" | "rdf" => is_atom = false,
                        other => {
                            return Err(ParseError(format!("unrecognized root element <{other}>")))
                        }
                    }
                }
                if (name == "item" && !is_atom) || (name == "entry" && is_atom) {
                    current = Some(EntryBuilder::default());
                    in_entry = true;
                }
                if is_atom && name == "link" && in_entry {
                    handle_atom_link(&e, current.as_mut());
                }
                stack.push(name);
                text.clear();
            }
            Event::Empty(e) => {
                let name = local_name(e.name().as_ref());
                if is_atom && name == "link" && in_entry {
                    handle_atom_link(&e, current.as_mut());
                }
            }
            Event::Text(t) => {
                let unescaped = t.unescape().map_err(|e| ParseError(e.to_string()))?;
                text.push_str(&unescaped);
            }
            Event::CData(c) => {
                // CDATA content is taken verbatim (no entity processing).
                text.push_str(&String::from_utf8_lossy(c.as_ref()));
            }
            Event::End(_) => {
                let name = stack.pop().unwrap_or_default();
                let value = text.trim().to_string();
                text.clear();

                if (name == "item" && !is_atom) || (name == "entry" && is_atom) {
                    if let Some(b) = current.take() {
                        feed.entries.push(b.finish(is_atom));
                    }
                    in_entry = false;
                } else if in_entry {
                    apply_entry_field(current.as_mut(), &name, value, is_atom);
                } else if name == "title" && feed.title.is_none() {
                    feed.title = Some(value);
                }
            }
            _ => {}
        }
        buf.clear();
    }

    if !saw_root {
        return Err(ParseError("no recognizable feed root element".into()));
    }
    // A truncated document (e.g. an upstream server that cut the response off
    // mid-element) reaches EOF with elements still open. quick_xml reports EOF
    // rather than an error in that case, so guard against it explicitly.
    if let Some(open) = stack.last() {
        return Err(ParseError(format!(
            "unexpected end of document: <{open}> was never closed"
        )));
    }
    Ok(feed)
}

fn handle_atom_link(e: &quick_xml::events::BytesStart<'_>, entry: Option<&mut EntryBuilder>) {
    let Some(entry) = entry else { return };
    let mut href: Option<String> = None;
    let mut rel: Option<String> = None;
    for attr in e.attributes().flatten() {
        let key = local_name(attr.key.as_ref());
        let val = attr
            .unescape_value()
            .map(|v| v.to_string())
            .unwrap_or_default();
        match key.as_str() {
            "href" => href = Some(val),
            "rel" => rel = Some(val),
            _ => {}
        }
    }
    let Some(href) = href else { return };
    if entry.first_link.is_none() {
        entry.first_link = Some(href.clone());
    }
    let rel = rel.as_deref().unwrap_or("alternate");
    if rel == "alternate" && entry.alternate_link.is_none() {
        entry.alternate_link = Some(href);
    }
}

fn apply_entry_field(entry: Option<&mut EntryBuilder>, name: &str, value: String, is_atom: bool) {
    let Some(entry) = entry else { return };
    if is_atom {
        match name {
            "id" => entry.id = Some(value),
            "title" => entry.title = Some(value),
            "summary" => entry.summary = Some(value),
            "content" => entry.content = Some(value),
            "published" => entry.date = Some(value),
            "updated" if entry.date.is_none() => entry.date = Some(value),
            _ => {}
        }
    } else {
        match name {
            "guid" => entry.guid = Some(value),
            "title" => entry.title = Some(value),
            "link" => entry.rss_link = Some(value),
            "description" => entry.summary = Some(value),
            "pubdate" => entry.date = Some(value),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rss() {
        let xml = br#"<?xml version="1.0"?>
        <rss version="2.0"><channel>
          <title>Chan &amp; Co</title>
          <item>
            <title>First &amp; foremost</title>
            <link>http://example.com/1</link>
            <guid>id-1</guid>
            <description><![CDATA[Body & <b>bold</b>]]></description>
            <pubDate>Thu, 04 Mar 2021 05:06:07 GMT</pubDate>
          </item>
        </channel></rss>"#;
        let feed = parse_feed(xml).unwrap();
        assert_eq!(feed.title.as_deref(), Some("Chan & Co"));
        assert_eq!(feed.entries.len(), 1);
        let e = &feed.entries[0];
        assert_eq!(e.guid, "id-1");
        assert_eq!(e.title.as_deref(), Some("First & foremost"));
        assert_eq!(e.link.as_deref(), Some("http://example.com/1"));
        assert_eq!(e.summary.as_deref(), Some("Body & <b>bold</b>"));
        assert!(e.published_at.is_some());
    }

    #[test]
    fn rss_guid_falls_back_to_link() {
        let xml = br#"<rss version="2.0"><channel>
          <item><link>http://example.com/x</link></item>
        </channel></rss>"#;
        let feed = parse_feed(xml).unwrap();
        assert_eq!(feed.entries[0].guid, "http://example.com/x");
        assert_eq!(feed.entries[0].title, None);
    }

    #[test]
    fn parses_atom_with_alternate_link() {
        let xml = br#"<?xml version="1.0"?>
        <feed xmlns="http://www.w3.org/2005/Atom">
          <title>Atom Feed</title>
          <entry>
            <id>urn:uuid:1</id>
            <title>Hello</title>
            <link rel="self" href="http://example.com/self"/>
            <link rel="alternate" href="http://example.com/alt"/>
            <content>Content body</content>
            <updated>2021-03-04T05:06:07Z</updated>
          </entry>
        </feed>"#;
        let feed = parse_feed(xml).unwrap();
        assert_eq!(feed.entries.len(), 1);
        let e = &feed.entries[0];
        assert_eq!(e.guid, "urn:uuid:1");
        assert_eq!(e.link.as_deref(), Some("http://example.com/alt"));
        assert_eq!(e.summary.as_deref(), Some("Content body"));
        assert!(e.published_at.is_some());
    }

    #[test]
    fn atom_published_beats_updated() {
        let xml = br#"<feed xmlns="http://www.w3.org/2005/Atom">
          <entry>
            <id>1</id>
            <published>2020-01-01T00:00:00Z</published>
            <updated>2022-01-01T00:00:00Z</updated>
          </entry>
        </feed>"#;
        let feed = parse_feed(xml).unwrap();
        let p = feed.entries[0].published_at.unwrap();
        assert_eq!(p.to_rfc3339(), "2020-01-01T00:00:00+00:00");
    }

    #[test]
    fn bom_is_tolerated() {
        let mut xml = vec![0xEF, 0xBB, 0xBF];
        xml.extend_from_slice(
            br#"<rss version="2.0"><channel><item><guid>g</guid></item></channel></rss>"#,
        );
        let feed = parse_feed(&xml).unwrap();
        assert_eq!(feed.entries[0].guid, "g");
    }

    #[test]
    fn malformed_is_error() {
        assert!(parse_feed(b"<rss><channel><item></oops>").is_err());
        assert!(parse_feed(b"not xml at all").is_err());
    }

    #[test]
    fn truncated_mid_element_is_error() {
        // Upstream server cut the response off mid-tag: the document reaches
        // EOF with <rss>/<channel>/<item>/<title> still open. This must be a
        // parse error, not a silently-successful empty fetch.
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"><channel><title>Nightly</title>
<item><guid>n-1</guid><title>Release no"#;
        assert!(parse_feed(xml).is_err());
    }

    #[test]
    fn truncated_after_root_open_is_error() {
        assert!(parse_feed(br#"<rss version="2.0"><channel>"#).is_err());
    }
}

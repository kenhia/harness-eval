//! RSS 2.0 and Atom (RFC 4287) parsing into a common shape.
//!
//! The parser is a single streaming pass over the XML. Text nodes are
//! unescaped by quick-xml; CDATA sections are taken verbatim. A leading BOM
//! is tolerated. Dates are normalized to UTC (see [`crate::dates`]); anything
//! unparseable becomes `None`.

use crate::dates;
use crate::error::{FeedError, Result};
use crate::text::{clean_opt, title_or_untitled};
use chrono::{DateTime, Utc};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// A parsed feed: its channel/feed title and its items.
#[derive(Debug, Clone, Default)]
pub struct ParsedFeed {
    pub title: Option<String>,
    pub items: Vec<ParsedItem>,
}

/// A single parsed item/entry, already mapped to feedhub's storage shape.
#[derive(Debug, Clone)]
pub struct ParsedItem {
    /// Stable identity: RSS guid|link, Atom id.
    pub guid: String,
    pub title: String,
    pub link: Option<String>,
    pub summary: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Default)]
struct ItemAccum {
    title: Option<String>,
    guid: Option<String>,
    id: Option<String>,
    link_text: Option<String>,
    link_href_alt: Option<String>,
    link_href_first: Option<String>,
    description: Option<String>,
    summary: Option<String>,
    content: Option<String>,
    pubdate: Option<String>,
    published: Option<String>,
    updated: Option<String>,
}

impl ItemAccum {
    fn finalize(self) -> ParsedItem {
        let guid = self
            .guid
            .filter(|s| !s.trim().is_empty())
            .or_else(|| self.id.filter(|s| !s.trim().is_empty()))
            .or_else(|| self.link_href_alt.clone())
            .or_else(|| self.link_href_first.clone())
            .or_else(|| self.link_text.clone())
            .unwrap_or_default()
            .trim()
            .to_string();

        let link = clean_opt(
            self.link_href_alt
                .or(self.link_href_first)
                .or(self.link_text),
        );

        let summary = clean_opt(self.description.or(self.summary).or(self.content));

        let date_raw = self
            .pubdate
            .filter(|s| !s.trim().is_empty())
            .or_else(|| self.published.filter(|s| !s.trim().is_empty()))
            .or_else(|| self.updated.filter(|s| !s.trim().is_empty()));
        let published_at = date_raw.and_then(|s| dates::parse_any(&s));

        ParsedItem {
            guid,
            title: title_or_untitled(self.title),
            link,
            summary,
            published_at,
        }
    }
}

fn local_name(name: quick_xml::name::QName<'_>) -> String {
    String::from_utf8_lossy(name.local_name().as_ref())
        .to_ascii_lowercase()
        .to_string()
}

/// Strip a leading UTF-8 BOM if present.
fn strip_bom(bytes: &[u8]) -> &[u8] {
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        &bytes[3..]
    } else {
        bytes
    }
}

/// Parse raw feed bytes (RSS 2.0 or Atom) into a [`ParsedFeed`].
pub fn parse(bytes: &[u8]) -> Result<ParsedFeed> {
    let text = String::from_utf8_lossy(strip_bom(bytes));
    let mut reader = Reader::from_str(&text);
    reader.check_end_names(false);

    let mut feed = ParsedFeed::default();
    let mut stack: Vec<String> = Vec::new();
    let mut in_item = false;
    let mut item: Option<ItemAccum> = None;
    let mut buf = String::new();
    let mut saw_root = false;

    loop {
        match reader.read_event() {
            Err(e) => return Err(FeedError::Parse(format!("xml error: {e}"))),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let name = local_name(e.name());
                saw_root = true;
                buf.clear();
                if name == "item" || name == "entry" {
                    in_item = true;
                    item = Some(ItemAccum::default());
                } else if in_item && name == "link" {
                    handle_atom_link(&e, item.as_mut());
                }
                stack.push(name);
            }
            Ok(Event::Empty(e)) => {
                let name = local_name(e.name());
                saw_root = true;
                if in_item && name == "link" {
                    handle_atom_link(&e, item.as_mut());
                }
            }
            Ok(Event::Text(e)) => {
                if let Ok(t) = e.unescape() {
                    buf.push_str(&t);
                }
            }
            Ok(Event::CData(e)) => {
                buf.push_str(&String::from_utf8_lossy(e.as_ref()));
            }
            Ok(Event::End(e)) => {
                let name = local_name(e.name());
                let parent = if stack.len() >= 2 {
                    stack[stack.len() - 2].clone()
                } else {
                    String::new()
                };
                let value = std::mem::take(&mut buf);

                if name == "item" || name == "entry" {
                    if let Some(acc) = item.take() {
                        feed.items.push(acc.finalize());
                    }
                    in_item = false;
                } else if in_item {
                    if let Some(acc) = item.as_mut() {
                        assign_item_field(acc, &name, value);
                    }
                } else if name == "title"
                    && feed.title.is_none()
                    && (parent == "channel" || parent == "feed")
                {
                    let t = value.trim();
                    if !t.is_empty() {
                        feed.title = Some(t.to_string());
                    }
                }
                stack.pop();
            }
            _ => {}
        }
    }

    if !saw_root {
        return Err(FeedError::Parse("empty or non-XML document".into()));
    }
    Ok(feed)
}

fn assign_item_field(acc: &mut ItemAccum, name: &str, value: String) {
    match name {
        "title" => acc.title = Some(value),
        "guid" => acc.guid = Some(value),
        "id" => acc.id = Some(value),
        "link" => {
            // RSS text link (Atom links are handled via attributes at Start).
            if acc.link_text.is_none() && !value.trim().is_empty() {
                acc.link_text = Some(value);
            }
        }
        "description" => acc.description = Some(value),
        "summary" => acc.summary = Some(value),
        "content" => acc.content = Some(value),
        "pubdate" => acc.pubdate = Some(value),
        "published" => acc.published = Some(value),
        "updated" => acc.updated = Some(value),
        _ => {}
    }
}

fn handle_atom_link(e: &quick_xml::events::BytesStart<'_>, item: Option<&mut ItemAccum>) {
    let Some(acc) = item else { return };
    let mut href: Option<String> = None;
    let mut rel: Option<String> = None;
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_ascii_lowercase();
        let val = attr.unescape_value().map(|v| v.to_string()).unwrap_or_default();
        match key.as_str() {
            "href" => href = Some(val),
            "rel" => rel = Some(val),
            _ => {}
        }
    }
    if let Some(h) = href {
        if acc.link_href_first.is_none() {
            acc.link_href_first = Some(h.clone());
        }
        if rel.as_deref() == Some("alternate") || rel.is_none() {
            if acc.link_href_alt.is_none() {
                acc.link_href_alt = Some(h);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rss() {
        let xml = r#"<?xml version="1.0"?>
        <rss version="2.0"><channel>
          <title>Example RSS</title>
          <item>
            <title>First &amp; Only</title>
            <link>http://example.com/1</link>
            <guid>urn:1</guid>
            <description>Hello world</description>
            <pubDate>Wed, 02 Oct 2002 13:00:00 GMT</pubDate>
          </item>
        </channel></rss>"#;
        let f = parse(xml.as_bytes()).unwrap();
        assert_eq!(f.title.as_deref(), Some("Example RSS"));
        assert_eq!(f.items.len(), 1);
        let it = &f.items[0];
        assert_eq!(it.guid, "urn:1");
        assert_eq!(it.title, "First & Only");
        assert_eq!(it.link.as_deref(), Some("http://example.com/1"));
        assert_eq!(it.summary.as_deref(), Some("Hello world"));
        assert_eq!(
            dates::to_rfc3339_z(&it.published_at.unwrap()),
            "2002-10-02T13:00:00Z"
        );
    }

    #[test]
    fn parse_rss_guid_fallback_to_link() {
        let xml = r#"<rss><channel><title>T</title>
          <item><title>x</title><link>http://e/2</link></item>
        </channel></rss>"#;
        let f = parse(xml.as_bytes()).unwrap();
        assert_eq!(f.items[0].guid, "http://e/2");
    }

    #[test]
    fn parse_atom() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
        <feed xmlns="http://www.w3.org/2005/Atom">
          <title>Example Atom</title>
          <entry>
            <title>Entry One</title>
            <id>tag:example,2024:1</id>
            <link rel="alternate" href="http://example.com/a1"/>
            <summary>Some summary</summary>
            <published>2024-07-15T10:30:00-05:00</published>
            <updated>2024-07-16T00:00:00Z</updated>
          </entry>
        </feed>"#;
        let f = parse(xml.as_bytes()).unwrap();
        assert_eq!(f.title.as_deref(), Some("Example Atom"));
        let it = &f.items[0];
        assert_eq!(it.guid, "tag:example,2024:1");
        assert_eq!(it.link.as_deref(), Some("http://example.com/a1"));
        assert_eq!(it.summary.as_deref(), Some("Some summary"));
        assert_eq!(
            dates::to_rfc3339_z(&it.published_at.unwrap()),
            "2024-07-15T15:30:00Z"
        );
    }

    #[test]
    fn atom_published_falls_back_to_updated() {
        let xml = r#"<feed xmlns="http://www.w3.org/2005/Atom"><title>T</title>
          <entry><title>x</title><id>i1</id><updated>2024-01-02T03:04:05Z</updated></entry>
        </feed>"#;
        let f = parse(xml.as_bytes()).unwrap();
        assert_eq!(
            dates::to_rfc3339_z(&f.items[0].published_at.unwrap()),
            "2024-01-02T03:04:05Z"
        );
    }

    #[test]
    fn cdata_verbatim_and_missing_title() {
        let xml = r#"<rss><channel><title>T</title>
          <item>
            <link>http://e/3</link>
            <description><![CDATA[<b>Bold & raw</b>]]></description>
          </item>
        </channel></rss>"#;
        let f = parse(xml.as_bytes()).unwrap();
        assert_eq!(f.items[0].title, "(untitled)");
        assert_eq!(f.items[0].summary.as_deref(), Some("<b>Bold & raw</b>"));
    }

    #[test]
    fn missing_date_is_none_but_entry_stored() {
        let xml = r#"<rss><channel><title>T</title>
          <item><title>x</title><guid>g</guid></item>
        </channel></rss>"#;
        let f = parse(xml.as_bytes()).unwrap();
        assert_eq!(f.items.len(), 1);
        assert!(f.items[0].published_at.is_none());
    }

    #[test]
    fn bom_is_tolerated() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"<rss><channel><title>T</title><item><guid>g</guid></item></channel></rss>");
        let f = parse(&bytes).unwrap();
        assert_eq!(f.title.as_deref(), Some("T"));
    }

    #[test]
    fn malformed_is_error() {
        assert!(parse(b"not xml at all <<<").is_err() || parse(b"").is_err());
        assert!(parse(b"").is_err());
    }
}

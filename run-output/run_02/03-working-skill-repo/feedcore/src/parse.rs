//! RSS 2.0 and Atom (RFC 4287) parsing.
//!
//! Parsing is DOM-based (roxmltree). roxmltree resolves the predefined XML
//! entities and exposes CDATA verbatim, satisfying the pinned text rules.

use crate::dates::{parse_rfc3339, parse_rfc822};
use chrono::{DateTime, Utc};
use roxmltree::{Document, Node};

/// Placeholder title stored when an item/entry has no usable title.
pub const UNTITLED: &str = "(untitled)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedFormat {
    Rss,
    Atom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEntry {
    /// Stable identity key: RSS guid (fallback link) / Atom id.
    pub guid: String,
    /// Unescaped title; `(untitled)` when missing.
    pub title: String,
    pub link: Option<String>,
    pub summary: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFeed {
    pub format: FeedFormat,
    pub title: Option<String>,
    pub entries: Vec<ParsedEntry>,
}

#[derive(Debug)]
pub enum ParseError {
    /// The document is not well-formed XML.
    Xml(String),
    /// The root element is neither `<rss>` nor `<feed>`.
    Unrecognized,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Xml(m) => write!(f, "malformed XML: {m}"),
            ParseError::Unrecognized => write!(f, "unrecognized feed format"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Strip a leading UTF-8 BOM and decode as UTF-8 (lossy for stray bytes).
fn decode(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    String::from_utf8_lossy(bytes).into_owned()
}

/// Concatenated text of a node's descendants, trimmed; `None` if empty.
fn node_text(node: Node) -> Option<String> {
    let mut out = String::new();
    for d in node.descendants() {
        if d.is_text() {
            if let Some(t) = d.text() {
                out.push_str(t);
            }
        }
    }
    let trimmed = out.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// First direct child element with the given local name.
fn child<'a, 'input>(node: Node<'a, 'input>, name: &str) -> Option<Node<'a, 'input>> {
    node.children()
        .find(|c| c.is_element() && c.tag_name().name() == name)
}

fn child_text(node: Node, name: &str) -> Option<String> {
    child(node, name).and_then(node_text)
}

/// Parse a feed document, auto-detecting RSS 2.0 vs Atom from the root element.
pub fn parse_feed(bytes: &[u8]) -> Result<ParsedFeed, ParseError> {
    let text = decode(bytes);
    let doc = Document::parse(&text).map_err(|e| ParseError::Xml(e.to_string()))?;
    let root = doc.root_element();
    match root.tag_name().name() {
        "rss" => parse_rss(root),
        "feed" => parse_atom(root),
        _ => Err(ParseError::Unrecognized),
    }
}

fn parse_rss(root: Node) -> Result<ParsedFeed, ParseError> {
    let channel = child(root, "channel").ok_or(ParseError::Unrecognized)?;
    let title = child_text(channel, "title");
    let mut entries = Vec::new();
    for item in channel
        .children()
        .filter(|c| c.is_element() && c.tag_name().name() == "item")
    {
        let guid_text = child_text(item, "guid");
        let link = child_text(item, "link");
        let identity = guid_text
            .clone()
            .or_else(|| link.clone())
            .unwrap_or_else(|| synth_identity(child_text(item, "title"), &link));
        let title = child_text(item, "title").unwrap_or_else(|| UNTITLED.to_string());
        let summary = child_text(item, "description");
        let published_at = child_text(item, "pubDate").and_then(|d| parse_rfc822(&d));
        entries.push(ParsedEntry {
            guid: identity,
            title,
            link,
            summary,
            published_at,
        });
    }
    Ok(ParsedFeed {
        format: FeedFormat::Rss,
        title,
        entries,
    })
}

fn parse_atom(root: Node) -> Result<ParsedFeed, ParseError> {
    let title = child_text(root, "title");
    let mut entries = Vec::new();
    for entry in root
        .children()
        .filter(|c| c.is_element() && c.tag_name().name() == "entry")
    {
        let id = child_text(entry, "id");
        let link = atom_link(entry);
        let identity = id
            .clone()
            .or_else(|| link.clone())
            .unwrap_or_else(|| synth_identity(child_text(entry, "title"), &link));
        let title = child_text(entry, "title").unwrap_or_else(|| UNTITLED.to_string());
        let summary = child_text(entry, "summary").or_else(|| child_text(entry, "content"));
        let published_at = child_text(entry, "published")
            .or_else(|| child_text(entry, "updated"))
            .and_then(|d| parse_rfc3339(&d));
        entries.push(ParsedEntry {
            guid: identity,
            title,
            link,
            summary,
            published_at,
        });
    }
    Ok(ParsedFeed {
        format: FeedFormat::Atom,
        title,
        entries,
    })
}

/// Choose an Atom entry link: prefer `rel="alternate"`, else the first `<link>`.
fn atom_link(entry: Node) -> Option<String> {
    let links: Vec<Node> = entry
        .children()
        .filter(|c| c.is_element() && c.tag_name().name() == "link")
        .collect();
    let chosen = links
        .iter()
        .find(|l| l.attribute("rel") == Some("alternate"))
        .or_else(|| links.first())?;
    chosen
        .attribute("href")
        .map(|s| s.to_string())
        .or_else(|| node_text(*chosen))
}

/// Deterministic fallback identity when no guid/id/link is present.
fn synth_identity(title: Option<String>, link: &Option<String>) -> String {
    format!(
        "synth:{}|{}",
        title.unwrap_or_default(),
        link.clone().unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dates::to_rfc3339_z;

    #[test]
    fn parses_rss_basic() {
        let xml = r#"<?xml version="1.0"?>
        <rss version="2.0"><channel>
          <title>Example</title>
          <item>
            <title>Hello &amp; World</title>
            <link>http://e/1</link>
            <guid>g1</guid>
            <description>Body</description>
            <pubDate>Mon, 06 Sep 2021 16:20:00 +0000</pubDate>
          </item>
        </channel></rss>"#;
        let feed = parse_feed(xml.as_bytes()).unwrap();
        assert_eq!(feed.format, FeedFormat::Rss);
        assert_eq!(feed.title.as_deref(), Some("Example"));
        assert_eq!(feed.entries.len(), 1);
        let e = &feed.entries[0];
        assert_eq!(e.guid, "g1");
        assert_eq!(e.title, "Hello & World");
        assert_eq!(e.link.as_deref(), Some("http://e/1"));
        assert_eq!(e.summary.as_deref(), Some("Body"));
        assert_eq!(
            to_rfc3339_z(e.published_at.unwrap()),
            "2021-09-06T16:20:00Z"
        );
    }

    #[test]
    fn rss_guid_falls_back_to_link() {
        let xml = r#"<rss version="2.0"><channel><title>t</title>
          <item><link>http://e/2</link></item>
        </channel></rss>"#;
        let feed = parse_feed(xml.as_bytes()).unwrap();
        assert_eq!(feed.entries[0].guid, "http://e/2");
        assert_eq!(feed.entries[0].title, UNTITLED);
    }

    #[test]
    fn rss_cdata_is_verbatim() {
        let xml = r#"<rss version="2.0"><channel><title>t</title>
          <item><guid>g</guid><title><![CDATA[Raw <b>bold</b> &amp; stuff]]></title></item>
        </channel></rss>"#;
        let feed = parse_feed(xml.as_bytes()).unwrap();
        assert_eq!(feed.entries[0].title, "Raw <b>bold</b> &amp; stuff");
    }

    #[test]
    fn rss_missing_date_is_null() {
        let xml = r#"<rss version="2.0"><channel><title>t</title>
          <item><guid>g</guid><title>x</title></item>
        </channel></rss>"#;
        let feed = parse_feed(xml.as_bytes()).unwrap();
        assert!(feed.entries[0].published_at.is_none());
    }

    #[test]
    fn parses_atom_basic() {
        let xml = r#"<?xml version="1.0"?>
        <feed xmlns="http://www.w3.org/2005/Atom">
          <title>Atomic</title>
          <entry>
            <id>urn:1</id>
            <title>Entry One</title>
            <link rel="edit" href="http://e/edit"/>
            <link rel="alternate" href="http://e/1"/>
            <summary>Sum</summary>
            <published>2021-09-06T18:20:00+02:00</published>
          </entry>
        </feed>"#;
        let feed = parse_feed(xml.as_bytes()).unwrap();
        assert_eq!(feed.format, FeedFormat::Atom);
        assert_eq!(feed.title.as_deref(), Some("Atomic"));
        let e = &feed.entries[0];
        assert_eq!(e.guid, "urn:1");
        assert_eq!(e.link.as_deref(), Some("http://e/1"));
        assert_eq!(e.summary.as_deref(), Some("Sum"));
        assert_eq!(
            to_rfc3339_z(e.published_at.unwrap()),
            "2021-09-06T16:20:00Z"
        );
    }

    #[test]
    fn atom_published_falls_back_to_updated_and_content() {
        let xml = r#"<feed xmlns="http://www.w3.org/2005/Atom"><title>t</title>
          <entry>
            <id>urn:2</id><title>x</title>
            <content>Contented</content>
            <updated>2021-01-01T00:00:00Z</updated>
          </entry>
        </feed>"#;
        let feed = parse_feed(xml.as_bytes()).unwrap();
        let e = &feed.entries[0];
        assert_eq!(e.summary.as_deref(), Some("Contented"));
        assert_eq!(
            to_rfc3339_z(e.published_at.unwrap()),
            "2021-01-01T00:00:00Z"
        );
    }

    #[test]
    fn malformed_xml_errors() {
        let xml = b"<rss><channel><item></channel>";
        assert!(matches!(parse_feed(xml), Err(ParseError::Xml(_))));
    }

    #[test]
    fn unrecognized_root_errors() {
        let xml = b"<html><body>hi</body></html>";
        assert!(matches!(parse_feed(xml), Err(ParseError::Unrecognized)));
    }

    #[test]
    fn tolerates_leading_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(
            br#"<rss version="2.0"><channel><title>b</title><item><guid>g</guid><title>x</title></item></channel></rss>"#,
        );
        let feed = parse_feed(&bytes).unwrap();
        assert_eq!(feed.title.as_deref(), Some("b"));
    }
}

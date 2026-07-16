//! RSS 2.0 and Atom parsing built on `quick-xml`.
//!
//! The parser is tolerant: it maps the fields the spec pins and ignores the
//! rest. Text is entity-unescaped as it is read (CDATA is taken verbatim);
//! dates are normalized in [`crate::date`]; missing titles become
//! `(untitled)` at finalize time.

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::date::parse_date;
use crate::text::title_or_untitled;
use crate::types::{ParsedFeed, ParsedItem};

/// Errors that can arise while parsing feed bytes.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// The underlying XML was malformed.
    #[error("malformed XML: {0}")]
    Xml(#[from] quick_xml::Error),
    /// The XML did not look like an RSS or Atom feed.
    #[error("unrecognized feed format (expected RSS or Atom)")]
    UnknownFormat,
}

#[derive(Clone, Copy, PartialEq)]
enum Format {
    Rss,
    Atom,
}

#[derive(Default)]
struct ItemBuilder {
    identity: Option<String>,
    link: Option<String>,
    link_alternate: Option<String>,
    link_first: Option<String>,
    title: Option<String>,
    summary: Option<String>,
    content: Option<String>,
    date_primary: Option<String>,
    date_fallback: Option<String>,
}

impl ItemBuilder {
    fn finalize(self) -> ParsedItem {
        let link = self.link.or(self.link_alternate).or(self.link_first);
        let guid = self
            .identity
            .or_else(|| link.clone())
            .unwrap_or_else(|| synth_identity(&self.title, &self.summary));
        let summary = self.summary.or(self.content);
        let date_src = self.date_primary.or(self.date_fallback);
        let published_at = date_src.as_deref().and_then(parse_date);
        ParsedItem {
            guid,
            title: title_or_untitled(self.title),
            link,
            summary,
            published_at,
        }
    }
}

fn synth_identity(title: &Option<String>, summary: &Option<String>) -> String {
    format!(
        "synthetic:{}\u{1}{}",
        title.as_deref().unwrap_or(""),
        summary.as_deref().unwrap_or("")
    )
}

/// Parse RSS 2.0 or Atom feed bytes into a [`ParsedFeed`].
///
/// A leading UTF-8 BOM is tolerated. Returns [`ParseError`] on malformed XML
/// or an unrecognized root element.
pub fn parse_feed(bytes: &[u8]) -> Result<ParsedFeed, ParseError> {
    let bytes = strip_bom(bytes);
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().expand_empty_elements = false;
    reader.config_mut().check_end_names = true;
    let mut buf = Vec::new();

    let mut format: Option<Format> = None;
    let mut feed = ParsedFeed {
        title: None,
        items: Vec::new(),
    };
    let mut cur: Option<ItemBuilder> = None;

    loop {
        buf.clear();
        let event = reader.read_event_into(&mut buf)?;
        match event {
            Event::Eof => break,
            Event::Start(e) => {
                let name = local_name(e.name().as_ref());
                if format.is_none() {
                    format = detect_format(&name);
                }
                let fmt = match format {
                    Some(f) => f,
                    None => continue,
                };
                if is_container(fmt, &name) {
                    cur = Some(ItemBuilder::default());
                    continue;
                }
                // Atom links carry their target in attributes.
                if fmt == Format::Atom && name == "link" {
                    if let Some(item) = cur.as_mut() {
                        apply_atom_link(item, &e);
                    }
                    continue;
                }
                // Only read text for known leaf elements; containers such as
                // `channel`/`feed` fall through so their children are seen.
                if is_text_element(fmt, &name) {
                    let text = read_text(&mut reader, &mut buf)?;
                    assign_text(fmt, &name, &text, &mut feed, &mut cur);
                }
            }
            Event::Empty(e) => {
                let name = local_name(e.name().as_ref());
                if format.is_none() {
                    format = detect_format(&name);
                }
                if format == Some(Format::Atom) && name == "link" {
                    if let Some(item) = cur.as_mut() {
                        apply_atom_link(item, &e);
                    }
                }
            }
            Event::End(e) => {
                let name = local_name(e.name().as_ref());
                if let Some(fmt) = format {
                    if is_container(fmt, &name) {
                        if let Some(item) = cur.take() {
                            feed.items.push(item.finalize());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    match format {
        Some(_) => Ok(feed),
        None => Err(ParseError::UnknownFormat),
    }
}

fn detect_format(name: &str) -> Option<Format> {
    match name {
        "feed" => Some(Format::Atom),
        "rss" | "rdf" | "channel" => Some(Format::Rss),
        _ => None,
    }
}

fn is_container(fmt: Format, name: &str) -> bool {
    match fmt {
        Format::Rss => name == "item",
        Format::Atom => name == "entry",
    }
}

fn is_text_element(fmt: Format, name: &str) -> bool {
    if name == "title" {
        return true;
    }
    match fmt {
        Format::Rss => matches!(name, "link" | "description" | "guid" | "pubdate"),
        Format::Atom => matches!(name, "id" | "summary" | "content" | "published" | "updated"),
    }
}

fn assign_text(
    fmt: Format,
    name: &str,
    text: &str,
    feed: &mut ParsedFeed,
    cur: &mut Option<ItemBuilder>,
) {
    let text = text.trim().to_string();
    match (fmt, name) {
        (_, "title") => match cur.as_mut() {
            Some(item) => item.title = Some(text),
            None => {
                if feed.title.is_none() {
                    feed.title = Some(text);
                }
            }
        },
        (Format::Rss, "link") => {
            if let Some(item) = cur.as_mut() {
                item.link = Some(text);
            }
        }
        (Format::Rss, "description") => {
            if let Some(item) = cur.as_mut() {
                item.summary = Some(text);
            }
        }
        (Format::Rss, "guid") => {
            if let Some(item) = cur.as_mut() {
                item.identity = Some(text);
            }
        }
        (Format::Rss, "pubdate") => {
            if let Some(item) = cur.as_mut() {
                item.date_primary = Some(text);
            }
        }
        (Format::Atom, "id") => {
            if let Some(item) = cur.as_mut() {
                item.identity = Some(text);
            }
        }
        (Format::Atom, "summary") => {
            if let Some(item) = cur.as_mut() {
                item.summary = Some(text);
            }
        }
        (Format::Atom, "content") => {
            if let Some(item) = cur.as_mut() {
                item.content = Some(text);
            }
        }
        (Format::Atom, "published") => {
            if let Some(item) = cur.as_mut() {
                item.date_primary = Some(text);
            }
        }
        (Format::Atom, "updated") => {
            if let Some(item) = cur.as_mut() {
                item.date_fallback = Some(text);
            }
        }
        _ => {}
    }
}

fn apply_atom_link(item: &mut ItemBuilder, e: &quick_xml::events::BytesStart) {
    let mut rel: Option<String> = None;
    let mut href: Option<String> = None;
    for attr in e.attributes().flatten() {
        let key = local_name(attr.key.as_ref());
        let val = String::from_utf8_lossy(&attr.value).into_owned();
        match key.as_str() {
            "rel" => rel = Some(val),
            "href" => href = Some(val),
            _ => {}
        }
    }
    let Some(href) = href else { return };
    let is_alternate = rel.as_deref().map(|r| r == "alternate").unwrap_or(true);
    if is_alternate {
        item.link_alternate.get_or_insert(href);
    } else if item.link_first.is_none() {
        item.link_first = Some(href);
    }
}

/// Read the concatenated text content of the currently-open element,
/// unescaping `Text` and taking `CData` verbatim, until the matching close
/// tag (nesting-aware).
fn read_text(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<String, ParseError> {
    let mut out = String::new();
    let mut depth = 1i32;
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Text(e) => out.push_str(&e.unescape()?),
            Event::CData(e) => out.push_str(&String::from_utf8_lossy(&e.into_inner())),
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(out)
}

fn local_name(raw: &[u8]) -> String {
    let s = raw.rsplit(|&b| b == b':').next().unwrap_or(raw);
    String::from_utf8_lossy(s).to_ascii_lowercase()
}

fn strip_bom(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RSS: &str = r#"<?xml version="1.0"?>
    <rss version="2.0"><channel>
      <title>Chan</title>
      <item>
        <title>First &amp; Best</title>
        <link>http://x/1</link>
        <description><![CDATA[<b>raw & html</b>]]></description>
        <guid>id-1</guid>
        <pubDate>Mon, 02 Jan 2024 03:04:05 -0500</pubDate>
      </item>
      <item>
        <link>http://x/2</link>
        <description>plain</description>
      </item>
    </channel></rss>"#;

    const ATOM: &str = r#"<?xml version="1.0" encoding="utf-8"?>
    <feed xmlns="http://www.w3.org/2005/Atom">
      <title>AtomChan</title>
      <entry>
        <title>Entry One</title>
        <id>urn:1</id>
        <link rel="alternate" href="http://a/1"/>
        <summary>sum &lt;1&gt;</summary>
        <published>2024-01-02T03:04:05Z</published>
        <updated>2024-02-02T00:00:00Z</updated>
      </entry>
    </feed>"#;

    #[test]
    fn parses_rss() {
        let f = parse_feed(RSS.as_bytes()).unwrap();
        assert_eq!(f.title.as_deref(), Some("Chan"));
        assert_eq!(f.items.len(), 2);
        let i0 = &f.items[0];
        assert_eq!(i0.guid, "id-1");
        assert_eq!(i0.title, "First & Best");
        assert_eq!(i0.link.as_deref(), Some("http://x/1"));
        assert_eq!(i0.summary.as_deref(), Some("<b>raw & html</b>"));
        assert!(i0.published_at.is_some());
        // Second item: no guid/title -> link identity, untitled.
        let i1 = &f.items[1];
        assert_eq!(i1.guid, "http://x/2");
        assert_eq!(i1.title, "(untitled)");
        assert!(i1.published_at.is_none());
    }

    #[test]
    fn parses_atom() {
        let f = parse_feed(ATOM.as_bytes()).unwrap();
        assert_eq!(f.title.as_deref(), Some("AtomChan"));
        assert_eq!(f.items.len(), 1);
        let e = &f.items[0];
        assert_eq!(e.guid, "urn:1");
        assert_eq!(e.title, "Entry One");
        assert_eq!(e.link.as_deref(), Some("http://a/1"));
        assert_eq!(e.summary.as_deref(), Some("sum <1>"));
        // published preferred over updated.
        assert_eq!(
            crate::date::to_rfc3339_z(&e.published_at.unwrap()),
            "2024-01-02T03:04:05Z"
        );
    }

    #[test]
    fn bom_tolerated() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(ATOM.as_bytes());
        let f = parse_feed(&bytes).unwrap();
        assert_eq!(f.title.as_deref(), Some("AtomChan"));
    }

    #[test]
    fn malformed_errors() {
        let r = parse_feed(b"<rss><channel><item></wrongtag></channel></rss>");
        assert!(r.is_err());
    }

    #[test]
    fn unknown_format_errors() {
        let r = parse_feed(b"<html><body>hi</body></html>");
        assert!(matches!(r, Err(ParseError::UnknownFormat)));
    }
}

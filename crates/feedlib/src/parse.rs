//! Feed parsing for RSS 2.0 and Atom (RFC 4287).
//!
//! The parser is a single streaming pass over the document using `quick-xml`.
//! Text nodes are entity-unescaped; CDATA sections are taken verbatim.

use crate::date::parse_datetime;
use crate::model::{ParsedEntry, ParsedFeed};
use quick_xml::events::Event;
use quick_xml::Reader;

/// Errors produced while parsing a feed document.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// The XML was malformed.
    #[error("malformed XML: {0}")]
    Xml(String),
    /// The document was well-formed but not a recognized feed format.
    #[error("unrecognized feed format")]
    UnknownFormat,
}

#[derive(Clone, Copy, PartialEq)]
enum Format {
    Rss,
    Atom,
}

#[derive(Default)]
struct RawEntry {
    title: Option<String>,
    rss_link: Option<String>,
    guid: Option<String>,
    id: Option<String>,
    description: Option<String>,
    summary: Option<String>,
    content: Option<String>,
    pubdate: Option<String>,
    published: Option<String>,
    updated: Option<String>,
    alt_link: Option<String>,
    first_link: Option<String>,
}

/// What the current text buffer is being captured into.
#[derive(Clone, Copy, PartialEq)]
enum Target {
    FeedTitle,
    Title,
    RssLink,
    Guid,
    Id,
    Description,
    Summary,
    Content,
    PubDate,
    Published,
    Updated,
}

/// Strip an optional leading UTF-8 BOM.
fn strip_bom(bytes: &[u8]) -> &[u8] {
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        &bytes[3..]
    } else {
        bytes
    }
}

/// Lowercased local name (namespace prefix removed).
fn local_lower(name: &[u8]) -> String {
    let s = std::str::from_utf8(name).unwrap_or("");
    let local = s.rsplit(':').next().unwrap_or(s);
    local.to_ascii_lowercase()
}

fn nonempty(s: String) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Simple deterministic fallback identity when a feed omits guid/id/link.
fn fallback_identity(seed: &str) -> String {
    let mut hash: u64 = 1469598103934665603; // FNV-1a offset basis
    for b in seed.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("feedhub:auto:{hash:016x}")
}

/// Parse a feed document into a [`ParsedFeed`].
pub fn parse_feed(bytes: &[u8]) -> Result<ParsedFeed, ParseError> {
    let bytes = strip_bom(bytes);
    let mut reader = Reader::from_reader(bytes);

    let mut format: Option<Format> = None;
    let mut stack: Vec<String> = Vec::new();
    let mut feed_title: Option<String> = None;
    let mut entries: Vec<RawEntry> = Vec::new();
    let mut cur: Option<RawEntry> = None;

    let mut target: Option<Target> = None;
    let mut capture_depth: usize = 0;
    let mut buf = String::new();

    loop {
        match reader.read_event() {
            Err(e) => return Err(ParseError::Xml(e.to_string())),
            Ok(Event::Eof) => {
                if let Some(open) = stack.last() {
                    return Err(ParseError::Xml(format!(
                        "unexpected end of document: <{open}> was never closed"
                    )));
                }
                break;
            }
            Ok(Event::Start(e)) => {
                let name = local_lower(e.name().as_ref());
                let parent = stack.last().cloned();
                handle_start(
                    &name,
                    parent.as_deref(),
                    &mut format,
                    &mut cur,
                    &mut feed_title,
                    &mut target,
                    &mut buf,
                    &e,
                );
                stack.push(name);
                if target.is_some() && capture_depth == 0 {
                    capture_depth = stack.len();
                }
            }
            Ok(Event::Empty(e)) => {
                // Self-closing element: no stack push, no text capture.
                let name = local_lower(e.name().as_ref());
                let parent = stack.last().cloned();
                let mut ignore_target: Option<Target> = None;
                let mut ignore_buf = String::new();
                handle_start(
                    &name,
                    parent.as_deref(),
                    &mut format,
                    &mut cur,
                    &mut feed_title,
                    &mut ignore_target,
                    &mut ignore_buf,
                    &e,
                );
            }
            Ok(Event::Text(e)) => {
                if target.is_some() {
                    if let Ok(t) = e.unescape() {
                        buf.push_str(&t);
                    }
                }
            }
            Ok(Event::CData(e)) => {
                if target.is_some() {
                    if let Ok(s) = std::str::from_utf8(e.as_ref()) {
                        buf.push_str(s);
                    }
                }
            }
            Ok(Event::End(_)) => {
                if target.is_some() && stack.len() == capture_depth {
                    commit(target.take().unwrap(), &mut buf, &mut cur, &mut feed_title);
                    capture_depth = 0;
                }
                let ended = stack.pop();
                if let Some(name) = ended {
                    if name == "item" || name == "entry" {
                        if let Some(raw) = cur.take() {
                            entries.push(raw);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let format = format.ok_or(ParseError::UnknownFormat)?;
    let entries = entries
        .into_iter()
        .map(|raw| finalize(format, raw))
        .collect();
    Ok(ParsedFeed {
        title: feed_title,
        entries,
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_start(
    name: &str,
    parent: Option<&str>,
    format: &mut Option<Format>,
    cur: &mut Option<RawEntry>,
    feed_title: &mut Option<String>,
    target: &mut Option<Target>,
    buf: &mut String,
    e: &quick_xml::events::BytesStart,
) {
    match name {
        "rss" | "rdf" => {
            if format.is_none() {
                *format = Some(Format::Rss);
            }
            return;
        }
        "feed" => {
            if format.is_none() {
                *format = Some(Format::Atom);
            }
            return;
        }
        "item" | "entry" => {
            *cur = Some(RawEntry::default());
            return;
        }
        _ => {}
    }

    if cur.is_some() {
        let is_atom = *format == Some(Format::Atom);
        let t = match name {
            "title" => Some(Target::Title),
            "guid" => Some(Target::Guid),
            "id" if is_atom => Some(Target::Id),
            "description" => Some(Target::Description),
            "summary" if is_atom => Some(Target::Summary),
            "content" if is_atom => Some(Target::Content),
            "pubdate" => Some(Target::PubDate),
            "published" if is_atom => Some(Target::Published),
            "updated" if is_atom => Some(Target::Updated),
            "link" => {
                if is_atom {
                    capture_atom_link(cur.as_mut().unwrap(), e);
                    None
                } else {
                    Some(Target::RssLink)
                }
            }
            _ => None,
        };
        if t.is_some() {
            *target = t;
            buf.clear();
        }
    } else {
        // Feed/channel level.
        if name == "title"
            && feed_title.is_none()
            && matches!(parent, Some("channel") | Some("feed"))
        {
            *target = Some(Target::FeedTitle);
            buf.clear();
        }
    }
}

fn capture_atom_link(raw: &mut RawEntry, e: &quick_xml::events::BytesStart) {
    let mut rel: Option<String> = None;
    let mut href: Option<String> = None;
    for attr in e.attributes().flatten() {
        let key = local_lower(attr.key.as_ref());
        if key == "rel" {
            rel = attr.unescape_value().ok().map(|v| v.to_string());
        } else if key == "href" {
            href = attr.unescape_value().ok().map(|v| v.to_string());
        }
    }
    let href = match href {
        Some(h) => h,
        None => return,
    };
    let rel = rel.unwrap_or_else(|| "alternate".to_string());
    if rel == "alternate" && raw.alt_link.is_none() {
        raw.alt_link = Some(href.clone());
    }
    if raw.first_link.is_none() {
        raw.first_link = Some(href);
    }
}

fn commit(
    target: Target,
    buf: &mut String,
    cur: &mut Option<RawEntry>,
    feed_title: &mut Option<String>,
) {
    let value = std::mem::take(buf);
    match target {
        Target::FeedTitle => *feed_title = nonempty(value),
        _ => {
            if let Some(raw) = cur.as_mut() {
                match target {
                    Target::Title => raw.title = nonempty(value),
                    Target::RssLink => raw.rss_link = nonempty(value),
                    Target::Guid => raw.guid = nonempty(value),
                    Target::Id => raw.id = nonempty(value),
                    Target::Description => raw.description = nonempty(value),
                    Target::Summary => raw.summary = nonempty(value),
                    Target::Content => raw.content = nonempty(value),
                    Target::PubDate => raw.pubdate = nonempty(value),
                    Target::Published => raw.published = nonempty(value),
                    Target::Updated => raw.updated = nonempty(value),
                    Target::FeedTitle => {}
                }
            }
        }
    }
}

fn finalize(format: Format, raw: RawEntry) -> ParsedEntry {
    let (link, summary, date_str, identity) = match format {
        Format::Rss => {
            let identity = raw
                .guid
                .clone()
                .or_else(|| raw.rss_link.clone())
                .unwrap_or_else(|| {
                    fallback_identity(&format!(
                        "{}|{}|{}",
                        raw.title.clone().unwrap_or_default(),
                        raw.description.clone().unwrap_or_default(),
                        raw.pubdate.clone().unwrap_or_default()
                    ))
                });
            (
                raw.rss_link.clone(),
                raw.description.clone(),
                raw.pubdate.clone(),
                identity,
            )
        }
        Format::Atom => {
            let link = raw.alt_link.clone().or_else(|| raw.first_link.clone());
            let summary = raw.summary.clone().or_else(|| raw.content.clone());
            let date_str = raw.published.clone().or_else(|| raw.updated.clone());
            let identity = raw.id.clone().or_else(|| link.clone()).unwrap_or_else(|| {
                fallback_identity(&format!(
                    "{}|{}|{}",
                    raw.title.clone().unwrap_or_default(),
                    summary.clone().unwrap_or_default(),
                    date_str.clone().unwrap_or_default()
                ))
            });
            (link, summary, date_str, identity)
        }
    };

    let title = raw.title.unwrap_or_else(|| "(untitled)".to_string());
    let published_at = date_str.as_deref().and_then(parse_datetime);

    ParsedEntry {
        guid: identity,
        title,
        link,
        summary,
        published_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::to_rfc3339_z;

    const RSS: &str = r#"<?xml version="1.0"?>
    <rss version="2.0"><channel>
      <title>Example Feed</title>
      <item>
        <title>First &amp; Foremost</title>
        <link>http://example.com/1</link>
        <description>Hello &lt;world&gt;</description>
        <guid>id-1</guid>
        <pubDate>Mon, 02 Jan 2006 15:04:05 -0500</pubDate>
      </item>
      <item>
        <link>http://example.com/2</link>
        <description><![CDATA[Raw <b>bold</b> & stuff]]></description>
        <pubDate>bogus date</pubDate>
      </item>
    </channel></rss>"#;

    const ATOM: &str = r#"<?xml version="1.0" encoding="utf-8"?>
    <feed xmlns="http://www.w3.org/2005/Atom">
      <title>Atom Example</title>
      <entry>
        <title>Atom Entry</title>
        <id>urn:uuid:1</id>
        <link rel="edit" href="http://example.com/edit"/>
        <link rel="alternate" href="http://example.com/a1"/>
        <summary>A &amp; B</summary>
        <published>2006-01-02T15:04:05Z</published>
        <updated>2007-01-02T15:04:05Z</updated>
      </entry>
    </feed>"#;

    #[test]
    fn parses_rss() {
        let feed = parse_feed(RSS.as_bytes()).unwrap();
        assert_eq!(feed.title.as_deref(), Some("Example Feed"));
        assert_eq!(feed.entries.len(), 2);
        let e0 = &feed.entries[0];
        assert_eq!(e0.guid, "id-1");
        assert_eq!(e0.title, "First & Foremost");
        assert_eq!(e0.summary.as_deref(), Some("Hello <world>"));
        assert_eq!(e0.link.as_deref(), Some("http://example.com/1"));
        assert_eq!(
            to_rfc3339_z(&e0.published_at.unwrap()),
            "2006-01-02T20:04:05Z"
        );

        let e1 = &feed.entries[1];
        // no guid -> identity falls back to link
        assert_eq!(e1.guid, "http://example.com/2");
        assert_eq!(e1.title, "(untitled)");
        assert_eq!(e1.summary.as_deref(), Some("Raw <b>bold</b> & stuff"));
        assert!(e1.published_at.is_none());
    }

    #[test]
    fn parses_atom() {
        let feed = parse_feed(ATOM.as_bytes()).unwrap();
        assert_eq!(feed.title.as_deref(), Some("Atom Example"));
        assert_eq!(feed.entries.len(), 1);
        let e = &feed.entries[0];
        assert_eq!(e.guid, "urn:uuid:1");
        assert_eq!(e.title, "Atom Entry");
        assert_eq!(e.link.as_deref(), Some("http://example.com/a1"));
        assert_eq!(e.summary.as_deref(), Some("A & B"));
        // published preferred over updated
        assert_eq!(
            to_rfc3339_z(&e.published_at.unwrap()),
            "2006-01-02T15:04:05Z"
        );
    }

    #[test]
    fn bom_tolerated() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(RSS.as_bytes());
        let feed = parse_feed(&bytes).unwrap();
        assert_eq!(feed.title.as_deref(), Some("Example Feed"));
    }

    #[test]
    fn malformed_is_error() {
        let bad = b"<rss><channel><item><title>oops</notclosed>";
        assert!(parse_feed(bad).is_err());
    }

    #[test]
    fn truncated_document_is_error() {
        // Upstream server cut the response off mid-element: the document is
        // not well-formed (elements are left unclosed), so it must be a parse
        // error rather than a successful empty fetch.
        let truncated = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
            <rss version=\"2.0\"><channel><title>Nightly</title>\n\
            <item><guid>n-1</guid><title>Release no";
        assert!(parse_feed(truncated).is_err());
    }
}

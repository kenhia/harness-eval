//! RSS 2.0 and Atom parsing into one common shape.
//!
//! Both formats are read with a single event loop that tracks the element path,
//! so `<title>` under `<channel>` and `<title>` under `<item>` never get
//! confused. Anything that is not an `<rss>` or `<feed>` document, and anything
//! the XML reader rejects, comes back as a [`ParseError`] — callers record that
//! against the feed rather than letting it affect other feeds.

use std::fmt;

use chrono::{DateTime, Utc};
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesRef, BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

use crate::UNTITLED;
use crate::dates::{parse_rfc822, parse_rfc3339};

/// A feed after parsing, before it is reconciled against what is stored.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedFeed {
    /// The channel/feed title, if the document has one.
    pub title: Option<String>,
    /// Entries in document order.
    pub entries: Vec<ParsedEntry>,
}

/// One item/entry after parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEntry {
    /// Identity within the feed: RSS `guid` (falling back to `link`), Atom `id`.
    pub guid: String,
    /// Entry title, or [`UNTITLED`] if the feed did not provide one.
    pub title: String,
    pub link: Option<String>,
    pub summary: Option<String>,
    /// Normalized to UTC; `None` when the feed had no parseable date.
    pub published_at: Option<DateTime<Utc>>,
}

/// Why a document could not be read as a feed.
#[derive(Debug)]
pub enum ParseError {
    /// The bytes were not valid UTF-8.
    Encoding,
    /// The XML itself is broken.
    Xml(String),
    /// Well-formed XML, but not an RSS or Atom document.
    UnsupportedFormat,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Encoding => write!(f, "feed is not valid UTF-8"),
            ParseError::Xml(msg) => write!(f, "malformed XML: {msg}"),
            ParseError::UnsupportedFormat => {
                write!(f, "document is neither RSS 2.0 nor Atom")
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Rss,
    Atom,
}

/// Which element's text we are currently collecting.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    FeedTitle,
    Guid,
    Title,
    Link,
    Summary,
    Content,
    Published,
    Updated,
    PubDate,
}

/// Text of one element, kept as segments so that ordinary text can be trimmed
/// while CDATA and resolved entities are preserved exactly.
#[derive(Default)]
struct TextAccum {
    segments: Vec<(bool, String)>,
}

impl TextAccum {
    fn push_text(&mut self, s: &str) {
        self.segments.push((false, s.to_string()));
    }

    /// Push text that must survive trimming: CDATA content and the
    /// replacement text of an entity reference.
    fn push_verbatim(&mut self, s: &str) {
        self.segments.push((true, s.to_string()));
    }

    /// Concatenate the segments, trimming only whitespace that came from plain
    /// text — leading/trailing whitespace inside CDATA is content.
    fn finish(mut self) -> String {
        let mut i = 0;
        while i < self.segments.len() && !self.segments[i].0 {
            self.segments[i].1 = self.segments[i].1.trim_start().to_string();
            if !self.segments[i].1.is_empty() {
                break;
            }
            i += 1;
        }
        let mut j = self.segments.len();
        while j > 0 && !self.segments[j - 1].0 {
            self.segments[j - 1].1 = self.segments[j - 1].1.trim_end().to_string();
            if !self.segments[j - 1].1.is_empty() {
                break;
            }
            j -= 1;
        }
        self.segments.iter().map(|(_, s)| s.as_str()).collect()
    }
}

/// Raw strings collected for one item/entry, before format-specific mapping.
#[derive(Default)]
struct ItemBuf {
    guid: Option<String>,
    title: Option<String>,
    link: Option<String>,
    alternate_link: Option<String>,
    first_link: Option<String>,
    summary: Option<String>,
    content: Option<String>,
    published: Option<String>,
    updated: Option<String>,
    pub_date: Option<String>,
}

impl ItemBuf {
    fn into_entry(self, format: Format) -> Option<ParsedEntry> {
        let (link, summary, published_at) = match format {
            Format::Rss => (
                self.link.clone(),
                self.summary.clone(),
                self.pub_date.as_deref().and_then(parse_rfc822),
            ),
            Format::Atom => {
                let link = self
                    .alternate_link
                    .clone()
                    .or_else(|| self.first_link.clone());
                let summary = self.summary.clone().or_else(|| self.content.clone());
                // `published` is preferred, but a feed that only has `updated`
                // (or whose `published` is junk) still gets a date.
                let published_at = self
                    .published
                    .as_deref()
                    .and_then(parse_rfc3339)
                    .or_else(|| self.updated.as_deref().and_then(parse_rfc3339));
                (link, summary, published_at)
            }
        };

        // Without an identity key there is nothing to dedupe against, so the
        // entry is dropped rather than duplicated on every refresh.
        let guid = non_empty(self.guid).or_else(|| non_empty(link.clone()))?;

        Some(ParsedEntry {
            guid,
            title: non_empty(self.title).unwrap_or_else(|| UNTITLED.to_string()),
            link: non_empty(link),
            summary: non_empty(summary),
            published_at,
        })
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
}

/// Parse an RSS 2.0 or Atom document. A leading UTF-8 BOM is tolerated.
pub fn parse_feed(input: &[u8]) -> Result<ParsedFeed, ParseError> {
    let input = input.strip_prefix(b"\xef\xbb\xbf").unwrap_or(input);
    let text = std::str::from_utf8(input).map_err(|_| ParseError::Encoding)?;

    let mut reader = Reader::from_str(text);
    // Emitting `<link/>` as a Start/End pair keeps the loop below from needing a
    // separate Empty arm for every element.
    reader.config_mut().expand_empty_elements = true;

    let mut path: Vec<String> = Vec::new();
    let mut format: Option<Format> = None;
    let mut feed = ParsedFeed::default();
    let mut item: Option<ItemBuf> = None;
    let mut accum: Option<(Field, usize, TextAccum)> = None;

    loop {
        match reader.read_event() {
            Err(e) => return Err(ParseError::Xml(e.to_string())),
            Ok(Event::Eof) => break,
            Ok(Event::Start(start)) => {
                let name = local_name(&start);
                path.push(name);

                let Some(format) = format else {
                    // The root element picks the dialect for the whole document.
                    match path[0].as_str() {
                        "rss" => format = Some(Format::Rss),
                        "feed" => format = Some(Format::Atom),
                        _ => return Err(ParseError::UnsupportedFormat),
                    }
                    continue;
                };

                match format {
                    Format::Rss => on_rss_start(&path, &mut item, &mut accum),
                    Format::Atom => on_atom_start(&path, &start, &mut item, &mut accum)?,
                }
            }
            Ok(Event::End(_)) => {
                let depth = path.len();
                let name = path.pop();

                if let Some((_, field_depth, _)) = &accum
                    && *field_depth == depth
                {
                    let (field, _, text) = accum.take().expect("accum checked above");
                    store_field(field, text.finish(), &mut feed, &mut item);
                }

                if let (Some(format), Some(name)) = (format, name)
                    && is_item_element(format, &name)
                    && path.len() == item_parent_depth(format)
                    && let Some(buf) = item.take()
                    && let Some(entry) = buf.into_entry(format)
                {
                    feed.entries.push(entry);
                }
            }
            Ok(Event::Text(text)) => {
                if let Some((_, _, accum)) = accum.as_mut() {
                    let decoded = text
                        .xml10_content()
                        .map_err(|e| ParseError::Xml(e.to_string()))?;
                    accum.push_text(&decoded);
                }
            }
            Ok(Event::GeneralRef(entity)) => {
                if let Some((_, _, accum)) = accum.as_mut() {
                    accum.push_verbatim(&resolve_entity(&entity)?);
                }
            }
            Ok(Event::CData(cdata)) => {
                if let Some((_, _, accum)) = accum.as_mut() {
                    let decoded = cdata.decode().map_err(|e| ParseError::Xml(e.to_string()))?;
                    accum.push_verbatim(&decoded);
                }
            }
            Ok(_) => {}
        }
    }

    if format.is_none() {
        return Err(ParseError::UnsupportedFormat);
    }
    Ok(feed)
}

/// Resolve `&amp;`-style references. Undefined entities (`&nbsp;` and friends
/// appear in real feeds) are kept as written rather than failing the document.
fn resolve_entity(entity: &BytesRef<'_>) -> Result<String, ParseError> {
    if let Some(ch) = entity
        .resolve_char_ref()
        .map_err(|e| ParseError::Xml(e.to_string()))?
    {
        return Ok(ch.to_string());
    }
    let name = entity
        .decode()
        .map_err(|e| ParseError::Xml(e.to_string()))?;
    Ok(match resolve_predefined_entity(&name) {
        Some(text) => text.to_string(),
        None => format!("&{name};"),
    })
}

fn local_name(start: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(start.local_name().as_ref()).into_owned()
}

fn is_item_element(format: Format, name: &str) -> bool {
    match format {
        Format::Rss => name == "item",
        Format::Atom => name == "entry",
    }
}

/// Depth of the element that contains items: `rss > channel` (1) or `feed` (0),
/// measured after the item element itself has been popped.
fn item_parent_depth(format: Format) -> usize {
    match format {
        Format::Rss => 2,
        Format::Atom => 1,
    }
}

fn on_rss_start(
    path: &[String],
    item: &mut Option<ItemBuf>,
    accum: &mut Option<(Field, usize, TextAccum)>,
) {
    let depth = path.len();
    match path {
        [_, channel, name] if channel == "channel" => match name.as_str() {
            "item" => *item = Some(ItemBuf::default()),
            "title" => start_field(accum, Field::FeedTitle, depth),
            _ => {}
        },
        [_, channel, item_el, name] if channel == "channel" && item_el == "item" => {
            let field = match name.as_str() {
                "title" => Field::Title,
                "link" => Field::Link,
                "description" => Field::Summary,
                "guid" => Field::Guid,
                "pubDate" => Field::PubDate,
                _ => return,
            };
            start_field(accum, field, depth);
        }
        _ => {}
    }
}

fn on_atom_start(
    path: &[String],
    start: &BytesStart<'_>,
    item: &mut Option<ItemBuf>,
    accum: &mut Option<(Field, usize, TextAccum)>,
) -> Result<(), ParseError> {
    let depth = path.len();
    match path {
        [_, name] => match name.as_str() {
            "entry" => *item = Some(ItemBuf::default()),
            "title" => start_field(accum, Field::FeedTitle, depth),
            _ => {}
        },
        [_, entry, name] if entry == "entry" => {
            if name == "link" {
                if let Some(buf) = item.as_mut() {
                    record_atom_link(start, buf)?;
                }
                return Ok(());
            }
            let field = match name.as_str() {
                "title" => Field::Title,
                "id" => Field::Guid,
                "summary" => Field::Summary,
                "content" => Field::Content,
                "published" => Field::Published,
                "updated" => Field::Updated,
                _ => return Ok(()),
            };
            start_field(accum, field, depth);
        }
        _ => {}
    }
    Ok(())
}

/// Apply RFC 4287 link selection: remember the first `rel="alternate"` link
/// (`rel` defaults to `alternate` when absent) and, separately, the first link
/// of any relation as a fallback.
fn record_atom_link(start: &BytesStart<'_>, buf: &mut ItemBuf) -> Result<(), ParseError> {
    let mut href = None;
    let mut rel = None;
    for attr in start.attributes() {
        let attr = attr.map_err(|e| ParseError::Xml(e.to_string()))?;
        let value = attr
            .normalized_value(XmlVersion::Explicit1_0)
            .map_err(|e| ParseError::Xml(e.to_string()))?
            .into_owned();
        match attr.key.local_name().as_ref() {
            b"href" => href = Some(value),
            b"rel" => rel = Some(value),
            _ => {}
        }
    }

    let Some(href) = non_empty(href) else {
        return Ok(());
    };
    if buf.first_link.is_none() {
        buf.first_link = Some(href.clone());
    }
    if rel.as_deref().unwrap_or("alternate") == "alternate" && buf.alternate_link.is_none() {
        buf.alternate_link = Some(href);
    }
    Ok(())
}

/// Begin collecting text for `field`. A field already in progress wins, so that
/// markup nested inside e.g. Atom XHTML content cannot hijack the collector.
fn start_field(accum: &mut Option<(Field, usize, TextAccum)>, field: Field, depth: usize) {
    if accum.is_none() {
        *accum = Some((field, depth, TextAccum::default()));
    }
}

fn store_field(field: Field, value: String, feed: &mut ParsedFeed, item: &mut Option<ItemBuf>) {
    if let Field::FeedTitle = field {
        if feed.title.is_none() {
            feed.title = non_empty(Some(value));
        }
        return;
    }

    let Some(buf) = item.as_mut() else { return };
    let slot = match field {
        Field::Guid => &mut buf.guid,
        Field::Title => &mut buf.title,
        Field::Link => &mut buf.link,
        Field::Summary => &mut buf.summary,
        Field::Content => &mut buf.content,
        Field::Published => &mut buf.published,
        Field::Updated => &mut buf.updated,
        Field::PubDate => &mut buf.pub_date,
        Field::FeedTitle => unreachable!("handled above"),
    };
    // First occurrence wins; feeds occasionally repeat elements.
    if slot.is_none() {
        *slot = Some(value);
    }
}

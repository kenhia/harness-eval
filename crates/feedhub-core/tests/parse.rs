//! Parser behavior for the item-mapping and text rules pinned by the spec.

use feedhub_core::{ParseError, UNTITLED, format_utc, parse_feed};
use pretty_assertions::assert_eq;

const RSS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0">
  <channel>
    <title>Example Channel</title>
    <link>https://example.invalid/</link>
    <description>A channel</description>
    <item>
      <title>First post</title>
      <link>https://example.invalid/1</link>
      <description>The first one</description>
      <guid isPermaLink="false">tag:example,1</guid>
      <pubDate>Fri, 01 Mar 2024 12:30:00 +0000</pubDate>
    </item>
    <item>
      <title>Second post</title>
      <link>https://example.invalid/2</link>
      <description>The second one</description>
      <pubDate>Fri, 01 Mar 2024 08:00:00 -0500</pubDate>
    </item>
  </channel>
</rss>
"#;

const ATOM: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Example Atom</title>
  <updated>2024-03-01T12:00:00Z</updated>
  <entry>
    <title>Atom one</title>
    <id>urn:uuid:1</id>
    <link rel="edit" href="https://example.invalid/edit/1"/>
    <link rel="alternate" href="https://example.invalid/a1"/>
    <summary>Summary one</summary>
    <content>Content one</content>
    <published>2024-03-01T12:30:00.500+02:00</published>
    <updated>2024-03-02T09:00:00Z</updated>
  </entry>
  <entry>
    <title>Atom two</title>
    <id>urn:uuid:2</id>
    <link href="https://example.invalid/a2"/>
    <content>Content two</content>
    <updated>2024-03-03T09:00:00Z</updated>
  </entry>
</feed>
"#;

#[test]
fn rss_item_mapping() {
    let feed = parse_feed(RSS.as_bytes()).expect("valid RSS should parse");
    assert_eq!(feed.title.as_deref(), Some("Example Channel"));
    assert_eq!(feed.entries.len(), 2);

    let first = &feed.entries[0];
    assert_eq!(first.guid, "tag:example,1");
    assert_eq!(first.title, "First post");
    assert_eq!(first.link.as_deref(), Some("https://example.invalid/1"));
    assert_eq!(first.summary.as_deref(), Some("The first one"));
    assert_eq!(
        first.published_at.map(format_utc).as_deref(),
        Some("2024-03-01T12:30:00Z")
    );

    // No <guid>, so identity falls back to <link>.
    let second = &feed.entries[1];
    assert_eq!(second.guid, "https://example.invalid/2");
    assert_eq!(
        second.published_at.map(format_utc).as_deref(),
        Some("2024-03-01T13:00:00Z")
    );
}

#[test]
fn atom_entry_mapping() {
    let feed = parse_feed(ATOM.as_bytes()).expect("valid Atom should parse");
    assert_eq!(feed.title.as_deref(), Some("Example Atom"));
    assert_eq!(feed.entries.len(), 2);

    let first = &feed.entries[0];
    assert_eq!(first.guid, "urn:uuid:1");
    // rel="alternate" wins over the earlier rel="edit" link.
    assert_eq!(first.link.as_deref(), Some("https://example.invalid/a1"));
    // summary wins over content.
    assert_eq!(first.summary.as_deref(), Some("Summary one"));
    // published wins over updated, and fractional seconds/offsets are handled.
    assert_eq!(
        first.published_at.map(format_utc).as_deref(),
        Some("2024-03-01T10:30:00Z")
    );

    let second = &feed.entries[1];
    // A link with no rel is an alternate link.
    assert_eq!(second.link.as_deref(), Some("https://example.invalid/a2"));
    // Falls back to content, and to updated for the date.
    assert_eq!(second.summary.as_deref(), Some("Content two"));
    assert_eq!(
        second.published_at.map(format_utc).as_deref(),
        Some("2024-03-03T09:00:00Z")
    );
}

#[test]
fn entities_are_unescaped_and_cdata_is_verbatim() {
    let xml = r#"<rss version="2.0"><channel>
      <title>Tips &amp; Tricks</title>
      <item>
        <guid>a</guid>
        <title>Fish &amp; Chips &lt;yum&gt;</title>
        <description><![CDATA[ <b>raw &amp; markup</b> ]]></description>
      </item>
      <item>
        <guid>b</guid>
        <title><![CDATA[  spaced  ]]></title>
        <description>caf&#233; &unknown; done</description>
      </item>
    </channel></rss>"#;

    let feed = parse_feed(xml.as_bytes()).expect("should parse");
    assert_eq!(feed.title.as_deref(), Some("Tips & Tricks"));

    assert_eq!(feed.entries[0].title, "Fish & Chips <yum>");
    // CDATA is taken exactly as written, inner whitespace and markup included.
    assert_eq!(
        feed.entries[0].summary.as_deref(),
        Some(" <b>raw &amp; markup</b> ")
    );

    assert_eq!(feed.entries[1].title, "  spaced  ");
    // Numeric references resolve; an entity nothing defines is left as written.
    assert_eq!(
        feed.entries[1].summary.as_deref(),
        Some("café &unknown; done")
    );
}

#[test]
fn missing_title_becomes_untitled() {
    let xml = r#"<rss version="2.0"><channel><item>
        <guid>a</guid>
        <link>https://example.invalid/a</link>
      </item></channel></rss>"#;
    let feed = parse_feed(xml.as_bytes()).expect("should parse");
    assert_eq!(feed.entries[0].title, UNTITLED);
}

#[test]
fn missing_or_unparseable_dates_are_null() {
    let xml = r#"<rss version="2.0"><channel>
      <item><guid>a</guid><title>No date</title></item>
      <item><guid>b</guid><title>Junk date</title><pubDate>last Tuesday</pubDate></item>
      <item><guid>c</guid><title>Good date</title><pubDate>Fri, 01 Mar 2024 12:00:00 PST</pubDate></item>
    </channel></rss>"#;
    let feed = parse_feed(xml.as_bytes()).expect("should parse");
    assert_eq!(feed.entries[0].published_at, None);
    assert_eq!(feed.entries[1].published_at, None);
    assert_eq!(
        feed.entries[2].published_at.map(format_utc).as_deref(),
        Some("2024-03-01T20:00:00Z")
    );
}

#[test]
fn leading_bom_is_tolerated() {
    let with_bom = [b"\xef\xbb\xbf".as_slice(), RSS.as_bytes()].concat();
    let feed = parse_feed(&with_bom).expect("BOM-prefixed feed should parse");
    assert_eq!(feed.entries.len(), 2);
}

#[test]
fn channel_and_item_titles_do_not_collide() {
    let xml = r#"<rss version="2.0"><channel>
      <item><guid>a</guid><title>Item title</title></item>
      <title>Channel title</title>
    </channel></rss>"#;
    let feed = parse_feed(xml.as_bytes()).expect("should parse");
    assert_eq!(feed.title.as_deref(), Some("Channel title"));
    assert_eq!(feed.entries[0].title, "Item title");
}

#[test]
fn entry_without_identity_is_dropped() {
    let xml = r#"<rss version="2.0"><channel>
      <item><title>Anonymous</title></item>
      <item><guid>a</guid><title>Identified</title></item>
    </channel></rss>"#;
    let feed = parse_feed(xml.as_bytes()).expect("should parse");
    assert_eq!(feed.entries.len(), 1);
    assert_eq!(feed.entries[0].guid, "a");
}

#[test]
fn malformed_xml_is_an_error() {
    let xml = r#"<rss version="2.0"><channel><item><title>Broken</titel></item></channel></rss>"#;
    assert!(matches!(
        parse_feed(xml.as_bytes()),
        Err(ParseError::Xml(_))
    ));
}

#[test]
fn truncated_document_is_an_error() {
    // Upstream cut the response off mid-element: the feed opens <rss>, <channel>,
    // <item>, <title> and then simply ends. This must be reported as malformed,
    // not silently accepted as a successful empty fetch.
    let xml = concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
        "<rss version=\"2.0\"><channel><title>Nightly</title>\n",
        "<item><guid>n-1</guid><title>Release no",
    );
    assert!(matches!(
        parse_feed(xml.as_bytes()),
        Err(ParseError::Xml(_))
    ));
}

#[test]
fn non_feed_documents_are_rejected() {
    assert!(matches!(
        parse_feed(b"<html><body>hi</body></html>"),
        Err(ParseError::UnsupportedFormat)
    ));
    assert!(matches!(
        parse_feed(b""),
        Err(ParseError::UnsupportedFormat)
    ));
}

#[test]
fn invalid_utf8_is_an_error() {
    assert!(matches!(
        parse_feed(&[0xff, 0xfe, 0x00]),
        Err(ParseError::Encoding)
    ));
}

//! The fixture corpus.
//!
//! These files are the only feeds feedhub's tests ever fetch. Each one exists
//! to pin a specific behavior; see [`CORPUS`] for the catalog and `README.md`
//! (written alongside the corpus by [`write_corpus`]) for the prose version.

use std::path::Path;

/// One fixture file plus the reason it exists.
pub struct Fixture {
    pub name: &'static str,
    /// What this fixture is for. Rendered into the corpus README.
    pub purpose: &'static str,
    pub body: &'static str,
}

/// A well-formed RSS 2.0 feed. The happy path.
pub const RSS_BASIC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Example RSS Feed</title>
    <link>https://example.com/</link>
    <description>A well-formed RSS 2.0 feed.</description>
    <item>
      <title>First post</title>
      <link>https://example.com/posts/1</link>
      <guid isPermaLink="false">post-0001</guid>
      <description>The first post's summary.</description>
      <pubDate>Tue, 10 Jun 2003 04:00:00 GMT</pubDate>
    </item>
    <item>
      <title>Second post</title>
      <link>https://example.com/posts/2</link>
      <guid isPermaLink="false">post-0002</guid>
      <description>The second post's summary.</description>
      <pubDate>Wed, 11 Jun 2003 09:30:00 -0500</pubDate>
    </item>
    <item>
      <title>Third post, no guid</title>
      <link>https://example.com/posts/3</link>
      <description>Identity falls back to the link.</description>
      <pubDate>Thu, 12 Jun 2003 12:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>
"#;

/// A well-formed Atom feed exercising link selection and date fallback.
pub const ATOM_BASIC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Example Atom Feed</title>
  <link rel="self" href="https://example.com/atom"/>
  <updated>2005-07-31T12:29:29Z</updated>
  <id>urn:uuid:60a76c80-d399-11d9-b91C-0003939e0af6</id>
  <entry>
    <title>Atom entry with an alternate link</title>
    <id>urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a</id>
    <link rel="self" href="https://example.com/entries/1.atom"/>
    <link rel="alternate" href="https://example.com/entries/1"/>
    <summary>The alternate link wins over the earlier self link.</summary>
    <published>2003-12-13T08:29:29-04:00</published>
    <updated>2005-07-31T12:29:29Z</updated>
  </entry>
  <entry>
    <title>Atom entry falling back to updated and content</title>
    <id>urn:uuid:2225c695-cfb8-4ebb-aaaa-80da344efa6b</id>
    <link href="https://example.com/entries/2"/>
    <content>No summary element, so content becomes the summary.</content>
    <updated>2006-01-02T15:04:05Z</updated>
  </entry>
</feed>
"#;

/// Dates that exercise the RFC 822 zone-name table, plus a missing date and an
/// unparseable one. Both of the latter must store `published_at = null`.
pub const DATES_EDGE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Edge Case Dates</title>
    <link>https://example.com/dates</link>
    <description>Zone names, a missing date, and an unparseable date.</description>
    <item>
      <title>Zone name EST</title>
      <guid>date-est</guid>
      <pubDate>Wed, 01 Jan 2020 12:00:00 EST</pubDate>
    </item>
    <item>
      <title>Zone name PDT</title>
      <guid>date-pdt</guid>
      <pubDate>Wed, 01 Jan 2020 12:00:00 PDT</pubDate>
    </item>
    <item>
      <title>Zone name UT</title>
      <guid>date-ut</guid>
      <pubDate>Wed, 01 Jan 2020 12:00:00 UT</pubDate>
    </item>
    <item>
      <title>Numeric offset</title>
      <guid>date-numeric</guid>
      <pubDate>Wed, 01 Jan 2020 12:00:00 -0500</pubDate>
    </item>
    <item>
      <title>No date at all</title>
      <guid>date-missing</guid>
      <description>published_at must be null, never the fetch time.</description>
    </item>
    <item>
      <title>Unparseable date</title>
      <guid>date-garbage</guid>
      <pubDate>sometime last Tuesday</pubDate>
    </item>
  </channel>
</rss>
"#;

/// CDATA and entity handling. Entities unescape; CDATA is verbatim.
pub const CDATA_ENTITIES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>CDATA &amp; Entities</title>
    <link>https://example.com/cdata</link>
    <description>Entity unescaping and CDATA verbatim handling.</description>
    <item>
      <title>Tom &amp; Jerry &lt;the sequel&gt;</title>
      <guid>cdata-entities-1</guid>
      <link>https://example.com/cdata/1</link>
      <description><![CDATA[Verbatim CDATA: &amp; stays literal, <b>markup</b> is not parsed.]]></description>
      <pubDate>Wed, 01 Jan 2020 00:00:00 GMT</pubDate>
    </item>
    <item>
      <title><![CDATA[A CDATA title with <angle> brackets]]></title>
      <guid>cdata-entities-2</guid>
      <description>Entities in text: 5 &lt; 10 &amp;&amp; 10 &gt; 5</description>
      <pubDate>Thu, 02 Jan 2020 00:00:00 GMT</pubDate>
    </item>
    <item>
      <guid>cdata-entities-3</guid>
      <description>This item has no title, so it must store "(untitled)".</description>
      <pubDate>Fri, 03 Jan 2020 00:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>
"#;

/// Malformed XML: `<item>` is closed by `</channel>`. Must produce a parse
/// error recorded to the feed's `last_error`, and must not affect other feeds.
pub const MALFORMED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Malformed
    <link>https://example.com/malformed</link>
    <item>
      <title>This item is never closed
      <guid>broken-1</guid>
  </channel>
</rss>
"#;

/// Truncated XML: a broken upstream cut the response off mid-element, so the
/// document simply stops — no closing tags at all. Distinct from `MALFORMED`,
/// whose fault is a *mismatched* end tag: `check_end_names` catches that one at
/// the bad tag, but a document that just ends reaches EOF with its elements
/// still open. Both must produce a parse error recorded to `last_error`, never
/// a silently successful empty fetch.
pub const TRUNCATED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<rss version=\"2.0\"><channel><title>Nightly</title>\n\
<item><guid>n-1</guid><title>Release no";

/// The full corpus.
pub const CORPUS: &[Fixture] = &[
    Fixture {
        name: "rss-basic.rss",
        purpose: "A well-formed RSS 2.0 feed. Third item has no guid, so identity falls back to link.",
        body: RSS_BASIC,
    },
    Fixture {
        name: "atom-basic.atom",
        purpose: "A well-formed Atom feed. Covers rel=alternate link selection, a bare link, summary-vs-content, and published-vs-updated fallback.",
        body: ATOM_BASIC,
    },
    Fixture {
        name: "dates-edge.rss",
        purpose: "RFC 822 zone names (EST, PDT, UT), a numeric offset, a missing date, and an unparseable date. The last two must store published_at = null.",
        body: DATES_EDGE,
    },
    Fixture {
        name: "cdata-entities.rss",
        purpose: "Entity unescaping (&amp; -> &), CDATA taken verbatim, and an item with no title (stored as \"(untitled)\").",
        body: CDATA_ENTITIES,
    },
    Fixture {
        name: "malformed.xml",
        purpose: "Malformed XML: an unclosed <item>. Must set last_error on its feed and leave other feeds untouched.",
        body: MALFORMED,
    },
    Fixture {
        name: "truncated.xml",
        purpose: "Truncated XML: the body ends mid-element with nothing closed, as a broken origin would leave it. Must set last_error, not report a successful empty fetch.",
        body: TRUNCATED,
    },
];

/// Look up a fixture by file name.
pub fn get(name: &str) -> Option<&'static Fixture> {
    CORPUS.iter().find(|f| f.name == name)
}

fn readme() -> String {
    let mut out = String::from(
        "# feedgen fixture corpus\n\n\
         Generated by `feedgen make-fixtures DIR`. Serve it with\n\
         `feedgen serve --dir DIR`.\n\n\
         These files are the only feeds feedhub's tests fetch; nothing here\n\
         touches the real internet. Each file pins a specific behavior.\n\n\
         | file | purpose |\n|---|---|\n",
    );
    for f in CORPUS {
        // Escape the pipe so the markdown table survives.
        let purpose = f.purpose.replace('|', "\\|");
        out.push_str(&format!("| `{}` | {} |\n", f.name, purpose));
    }
    out.push_str(
        "\n## Content types\n\n\
         `feedgen serve` picks the `Content-Type` from the file extension:\n\n\
         | extension | content type |\n|---|---|\n\
         | `.rss` | `application/rss+xml; charset=utf-8` |\n\
         | `.atom` | `application/atom+xml; charset=utf-8` |\n\
         | `.xml` | `application/xml; charset=utf-8` |\n\
         | `.md` | `text/markdown; charset=utf-8` |\n\
         | `.txt` | `text/plain; charset=utf-8` |\n\
         | anything else | `application/octet-stream` |\n\n\
         ## Conditional GET\n\n\
         Every response carries a content-derived `ETag` (FNV-1a over the file\n\
         bytes) and a `Last-Modified` taken from the file's mtime. `feedgen`\n\
         answers `304 Not Modified` to a matching `If-None-Match`, or to an\n\
         `If-Modified-Since` at or after the mtime.\n",
    );
    out
}

/// Write the corpus and its README into `dir`, creating it if needed.
pub fn write_corpus(dir: &Path) -> std::io::Result<Vec<String>> {
    std::fs::create_dir_all(dir)?;
    let mut written = Vec::new();
    for f in CORPUS {
        std::fs::write(dir.join(f.name), f.body)?;
        written.push(f.name.to_string());
    }
    std::fs::write(dir.join("README.md"), readme())?;
    written.push("README.md".to_string());
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_covers_every_category_the_spec_requires() {
        assert!(get("rss-basic.rss").is_some(), "valid RSS 2.0");
        assert!(get("atom-basic.atom").is_some(), "valid Atom");
        assert!(get("dates-edge.rss").is_some(), "edge-case dates");
        assert!(get("cdata-entities.rss").is_some(), "CDATA / entities");
        assert!(get("malformed.xml").is_some(), "malformed XML");
    }

    #[test]
    fn every_valid_fixture_parses_and_the_broken_ones_do_not() {
        for f in CORPUS {
            let result = feedhub_core::parse_feed(f.body.as_bytes());
            if matches!(f.name, "malformed.xml" | "truncated.xml") {
                assert!(result.is_err(), "{} must not parse", f.name);
            } else {
                assert!(result.is_ok(), "{} must parse: {:?}", f.name, result.err());
            }
        }
    }

    #[test]
    fn write_corpus_emits_every_file_plus_a_readme() {
        let dir = tempfile::tempdir().unwrap();
        let written = write_corpus(dir.path()).unwrap();
        assert_eq!(written.len(), CORPUS.len() + 1);
        for f in CORPUS {
            assert!(dir.path().join(f.name).exists(), "{} not written", f.name);
        }
        let readme = std::fs::read_to_string(dir.path().join("README.md")).unwrap();
        // The README must actually document each file, not just exist.
        for f in CORPUS {
            assert!(readme.contains(f.name), "README omits {}", f.name);
        }
    }
}

//! The fixture corpus.
//!
//! Every document here is deliberately boring except in the one way it is meant
//! to be interesting, and all of its dates are constants — tests assert exact
//! `published_at` values against them, so nothing may depend on the wall clock.

use std::path::{Path, PathBuf};

use anyhow::Context;

/// One fixture file: its name, what it is for, and its contents.
pub struct Fixture {
    pub name: &'static str,
    pub description: &'static str,
    pub contents: &'static str,
}

/// A plain RSS 2.0 feed: guids, links, descriptions, RFC 822 dates.
pub const RSS_BASIC: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0">
  <channel>
    <title>Basic RSS Feed</title>
    <link>http://feedgen.invalid/rss</link>
    <description>A well-formed RSS 2.0 feed with three items.</description>
    <language>en-us</language>
    <item>
      <title>Rust release notes</title>
      <link>http://feedgen.invalid/rss/1</link>
      <description>What shipped in the latest release.</description>
      <guid isPermaLink="false">rss-1</guid>
      <pubDate>Mon, 04 Mar 2024 09:00:00 +0000</pubDate>
    </item>
    <item>
      <title>Filesystem notes</title>
      <link>http://feedgen.invalid/rss/2</link>
      <description>Notes on durability and fsync.</description>
      <guid isPermaLink="false">rss-2</guid>
      <pubDate>Sun, 03 Mar 2024 12:00:00 -0500</pubDate>
    </item>
    <item>
      <title>Networking notes</title>
      <link>http://feedgen.invalid/rss/3</link>
      <description>Notes on sockets and timeouts.</description>
      <guid isPermaLink="false">rss-3</guid>
      <pubDate>Sat, 02 Mar 2024 22:30:00 GMT</pubDate>
    </item>
  </channel>
</rss>
"#;

/// A plain Atom feed: ids, typed links, summary and content, RFC 3339 dates.
pub const ATOM_BASIC: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Basic Atom Feed</title>
  <link rel="self" href="http://feedgen.invalid/atom"/>
  <id>urn:feedgen:atom-basic</id>
  <updated>2024-03-05T08:15:00Z</updated>
  <entry>
    <title>Atom entry with a summary</title>
    <id>atom-1</id>
    <link rel="edit" href="http://feedgen.invalid/atom/1/edit"/>
    <link rel="alternate" href="http://feedgen.invalid/atom/1"/>
    <summary>The summary is preferred over the content.</summary>
    <content type="text">Content that should be ignored.</content>
    <published>2024-03-05T08:15:00Z</published>
    <updated>2024-03-06T10:00:00Z</updated>
  </entry>
  <entry>
    <title>Atom entry with content only</title>
    <id>atom-2</id>
    <link href="http://feedgen.invalid/atom/2"/>
    <content type="text">No summary here, so the content is the summary.</content>
    <published>2024-03-01T06:00:00+02:00</published>
    <updated>2024-03-01T06:00:00+02:00</updated>
  </entry>
</feed>
"#;

/// RSS whose dates exercise every zone name feedhub accepts, both numeric
/// offset directions, a missing date and an unparseable one. Each item is
/// nominally noon on 2024-03-01 in its own zone.
pub const DATES_EDGE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0">
  <channel>
    <title>Date Edge Cases</title>
    <link>http://feedgen.invalid/dates</link>
    <description>One item per date form feedhub has to handle.</description>
    <item>
      <title>Zone GMT</title><guid>date-gmt</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 GMT</pubDate>
    </item>
    <item>
      <title>Zone UT</title><guid>date-ut</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 UT</pubDate>
    </item>
    <item>
      <title>Zone Z</title><guid>date-z</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 Z</pubDate>
    </item>
    <item>
      <title>Zone EST</title><guid>date-est</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 EST</pubDate>
    </item>
    <item>
      <title>Zone EDT</title><guid>date-edt</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 EDT</pubDate>
    </item>
    <item>
      <title>Zone CST</title><guid>date-cst</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 CST</pubDate>
    </item>
    <item>
      <title>Zone CDT</title><guid>date-cdt</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 CDT</pubDate>
    </item>
    <item>
      <title>Zone MST</title><guid>date-mst</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 MST</pubDate>
    </item>
    <item>
      <title>Zone MDT</title><guid>date-mdt</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 MDT</pubDate>
    </item>
    <item>
      <title>Zone PST</title><guid>date-pst</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 PST</pubDate>
    </item>
    <item>
      <title>Zone PDT</title><guid>date-pdt</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 PDT</pubDate>
    </item>
    <item>
      <title>Numeric offset east</title><guid>date-plus</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 +0530</pubDate>
    </item>
    <item>
      <title>Numeric offset west</title><guid>date-minus</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 -0800</pubDate>
    </item>
    <item>
      <title>No date at all</title><guid>date-missing</guid>
    </item>
    <item>
      <title>Unparseable date</title><guid>date-junk</guid>
      <pubDate>sometime last Tuesday</pubDate>
    </item>
  </channel>
</rss>
"#;

/// Atom date edge cases: fractional seconds, a non-UTC offset, an entry with
/// only `updated`, and an entry whose dates are junk.
pub const ATOM_DATES_EDGE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Atom Date Edge Cases</title>
  <id>urn:feedgen:atom-dates</id>
  <updated>2024-03-05T00:00:00Z</updated>
  <entry>
    <title>Fractional seconds</title>
    <id>atom-frac</id>
    <published>2024-03-01T12:00:00.123456Z</published>
  </entry>
  <entry>
    <title>Offset east of UTC</title>
    <id>atom-offset</id>
    <published>2024-03-01T12:00:00+05:30</published>
  </entry>
  <entry>
    <title>Updated but never published</title>
    <id>atom-updated-only</id>
    <updated>2024-03-02T08:00:00-08:00</updated>
  </entry>
  <entry>
    <title>Dates are junk</title>
    <id>atom-junk</id>
    <published>March the first</published>
  </entry>
</feed>
"#;

/// CDATA and entity handling, plus an item with no title.
pub const CDATA_ENTITIES: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0">
  <channel>
    <title>CDATA &amp; Entities</title>
    <link>http://feedgen.invalid/cdata</link>
    <description>Text handling corner cases.</description>
    <item>
      <title><![CDATA[Raw <b>markup</b> & ampersands]]></title>
      <link>http://feedgen.invalid/cdata/1</link>
      <description><![CDATA[<p>CDATA is stored verbatim, including &amp; as written.</p>]]></description>
      <guid>cdata-1</guid>
      <pubDate>Fri, 01 Mar 2024 10:00:00 GMT</pubDate>
    </item>
    <item>
      <title>Fish &amp; Chips &lt;battered&gt;</title>
      <link>http://feedgen.invalid/cdata/2</link>
      <description>Caf&#233; &amp; cr&#232;me, unescaped on the way in.</description>
      <guid>entity-1</guid>
      <pubDate>Fri, 01 Mar 2024 11:00:00 GMT</pubDate>
    </item>
    <item>
      <link>http://feedgen.invalid/cdata/3</link>
      <description>This item has no title element at all.</description>
      <guid>untitled-1</guid>
      <pubDate>Fri, 01 Mar 2024 12:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>
"#;

/// Not well-formed: `<title>` is closed with a mismatched tag. Fetching this
/// must record an error on the feed and leave every other feed alone.
pub const MALFORMED: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0">
  <channel>
    <title>Malformed Feed</title>
    <item>
      <title>This title is closed by the wrong tag</titel>
      <guid>broken-1</guid>
    </item>
  </channel>
</rss>
"#;

/// The whole corpus, in the order `make-fixtures` writes it.
pub const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "rss-basic.xml",
        description: "Valid RSS 2.0: three items with guid, link, description and RFC 822 dates.",
        contents: RSS_BASIC,
    },
    Fixture {
        name: "atom-basic.xml",
        description: "Valid Atom (RFC 4287): rel=\"alternate\" link selection, summary-over-content, RFC 3339 dates.",
        contents: ATOM_BASIC,
    },
    Fixture {
        name: "dates-edge.xml",
        description: "RSS dates: every accepted zone name, both offset directions, a missing date and an unparseable one.",
        contents: DATES_EDGE,
    },
    Fixture {
        name: "atom-dates-edge.xml",
        description: "Atom dates: fractional seconds, non-UTC offsets, updated-without-published, and junk.",
        contents: ATOM_DATES_EDGE,
    },
    Fixture {
        name: "cdata-entities.xml",
        description: "Text handling: CDATA kept verbatim, entities unescaped, and an item with no title.",
        contents: CDATA_ENTITIES,
    },
    Fixture {
        name: "malformed.xml",
        description: "Not well-formed XML: a mismatched closing tag. Fetching it must record last_error.",
        contents: MALFORMED,
    },
];

/// Write the corpus and its README into `dir`, creating the directory if it
/// does not exist. Returns the paths written.
pub fn write_fixtures(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    std::fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;

    let mut written = Vec::new();
    for fixture in FIXTURES {
        let path = dir.join(fixture.name);
        std::fs::write(&path, fixture.contents)
            .with_context(|| format!("cannot write {}", path.display()))?;
        written.push(path);
    }

    let readme = dir.join("README.md");
    std::fs::write(&readme, readme_text())
        .with_context(|| format!("cannot write {}", readme.display()))?;
    written.push(readme);

    Ok(written)
}

/// Documentation for the corpus, written alongside it.
pub fn readme_text() -> String {
    let mut out = String::from(
        "# feedgen fixture corpus\n\n\
         Generated by `feedgen make-fixtures DIR`. Serve it with\n\
         `feedgen serve --dir DIR` and point `feedd` at the individual files.\n\n\
         All dates are constants, so tests can assert exact `published_at`\n\
         values without depending on the wall clock.\n\n",
    );
    for fixture in FIXTURES {
        out.push_str(&format!("- `{}` — {}\n", fixture.name, fixture.description));
    }
    out.push_str(
        "\nThe server derives each `ETag` from file contents and each\n\
         `Last-Modified` from the file's mtime, and answers `304 Not Modified`\n\
         to a matching `If-None-Match` or `If-Modified-Since`. Editing a file in\n\
         place is therefore enough to make `feedd` see new entries on refresh.\n",
    );
    out
}

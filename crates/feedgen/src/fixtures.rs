//! The fixture corpus: a small set of feeds covering the parsing edge cases
//! feedhub must handle. See [`make_fixtures`] and the generated `README.md`.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// A single named fixture file.
pub struct Fixture {
    /// File name written into the corpus directory.
    pub name: &'static str,
    /// File contents.
    pub body: &'static str,
    /// One-line description for the corpus README.
    pub note: &'static str,
}

/// The full corpus. Every entry is deterministic so ETags stay stable.
pub const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "rss.xml",
        note: "Valid RSS 2.0 feed with two items (guid identity, RFC 822 dates).",
        body: RSS,
    },
    Fixture {
        name: "atom.xml",
        note: "Valid Atom (RFC 4287) feed with alternate links and RFC 3339 dates.",
        body: ATOM,
    },
    Fixture {
        name: "dates.xml",
        note: "RSS feed exercising zone-name dates (EST/PST/GMT), a numeric offset, and a missing/unparseable date (stored as null).",
        body: DATES,
    },
    Fixture {
        name: "cdata.xml",
        note: "RSS feed exercising CDATA (verbatim) and XML entities (unescaped).",
        body: CDATA,
    },
    Fixture {
        name: "malformed.xml",
        note: "Malformed XML (mismatched tags) — must be recorded as a feed error, not crash.",
        body: MALFORMED,
    },
];

/// Write the fixture corpus (and a `README.md` describing it) into `dir`,
/// creating the directory if needed.
pub fn make_fixtures(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    for f in FIXTURES {
        let path = dir.join(f.name);
        fs::write(&path, f.body).with_context(|| format!("writing {}", path.display()))?;
    }
    fs::write(dir.join("README.md"), readme()).context("writing corpus README")?;
    Ok(())
}

fn readme() -> String {
    let mut s = String::from(
        "# feedgen fixture corpus\n\nDeterministic test feeds served by `feedgen serve`. \
Files:\n\n",
    );
    for f in FIXTURES {
        s.push_str(&format!("- `{}` — {}\n", f.name, f.note));
    }
    s.push_str("- `README.md` — this file.\n");
    s
}

const RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Example RSS Feed</title>
    <link>http://example.invalid/</link>
    <description>A valid RSS 2.0 fixture.</description>
    <item>
      <title>Hello RSS</title>
      <link>http://example.invalid/hello</link>
      <description>First post.</description>
      <guid>urn:example:hello</guid>
      <pubDate>Mon, 02 Jan 2024 15:04:05 -0500</pubDate>
    </item>
    <item>
      <title>Second Post</title>
      <link>http://example.invalid/second</link>
      <description>Another one.</description>
      <guid>urn:example:second</guid>
      <pubDate>Tue, 03 Jan 2024 09:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>
"#;

const ATOM: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Example Atom Feed</title>
  <link rel="self" href="http://example.invalid/atom.xml"/>
  <link rel="alternate" href="http://example.invalid/"/>
  <id>urn:example:atom</id>
  <updated>2024-02-01T00:00:00Z</updated>
  <entry>
    <title>Atom Entry One</title>
    <id>urn:example:atom:1</id>
    <link rel="alternate" href="http://example.invalid/atom/1"/>
    <summary>Entry one summary.</summary>
    <published>2024-01-10T12:30:00Z</published>
    <updated>2024-01-11T00:00:00Z</updated>
  </entry>
  <entry>
    <title>Atom Entry Two</title>
    <id>urn:example:atom:2</id>
    <link rel="alternate" href="http://example.invalid/atom/2"/>
    <content>Entry two content, used because summary is absent.</content>
    <updated>2024-01-12T08:15:00+02:00</updated>
  </entry>
</feed>
"#;

const DATES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Edge-Case Dates</title>
    <link>http://example.invalid/dates</link>
    <description>Feeds with tricky pubDate values.</description>
    <item>
      <title>Eastern zone</title>
      <guid>urn:dates:est</guid>
      <pubDate>Mon, 02 Jan 2024 12:00:00 EST</pubDate>
    </item>
    <item>
      <title>Pacific zone</title>
      <guid>urn:dates:pst</guid>
      <pubDate>Mon, 02 Jan 2024 12:00:00 PST</pubDate>
    </item>
    <item>
      <title>GMT zone</title>
      <guid>urn:dates:gmt</guid>
      <pubDate>Mon, 02 Jan 2024 12:00:00 GMT</pubDate>
    </item>
    <item>
      <title>Numeric offset</title>
      <guid>urn:dates:offset</guid>
      <pubDate>02 Jan 2024 12:00:00 +0530</pubDate>
    </item>
    <item>
      <title>Missing date</title>
      <guid>urn:dates:none</guid>
      <description>No pubDate at all — stored as null.</description>
    </item>
    <item>
      <title>Unparseable date</title>
      <guid>urn:dates:bad</guid>
      <pubDate>sometime last week</pubDate>
    </item>
  </channel>
</rss>
"#;

const CDATA: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>CDATA &amp; Entities</title>
    <link>http://example.invalid/cdata</link>
    <description>Text-handling fixture.</description>
    <item>
      <title>Fish &amp; Chips</title>
      <link>http://example.invalid/entities</link>
      <description>Tom &amp; Jerry &lt;cartoon&gt;</description>
      <guid>urn:cdata:entities</guid>
      <pubDate>Mon, 02 Jan 2024 00:00:00 GMT</pubDate>
    </item>
    <item>
      <title><![CDATA[Raw <b>markup</b> & ampersand]]></title>
      <link>http://example.invalid/cdata</link>
      <description><![CDATA[<p>Verbatim & <i>not</i> unescaped</p>]]></description>
      <guid>urn:cdata:raw</guid>
      <pubDate>Tue, 03 Jan 2024 00:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>
"#;

const MALFORMED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Broken</title>
    <item>
      <title>Unterminated
      <guid>urn:broken:1</guid>
    </wrongtag>
  </channel>
</rss>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_corpus() {
        let dir = std::env::temp_dir().join(format!("feedgen-fix-{}", std::process::id()));
        make_fixtures(&dir).unwrap();
        for f in FIXTURES {
            assert!(dir.join(f.name).exists(), "missing {}", f.name);
        }
        assert!(dir.join("README.md").exists());
        let _ = fs::remove_dir_all(&dir);
    }
}

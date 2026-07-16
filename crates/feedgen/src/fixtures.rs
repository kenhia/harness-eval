//! Fixture corpus generation for `feedgen make-fixtures`.

use std::fs;
use std::path::Path;

const RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Example RSS Feed</title>
    <link>https://example.com/</link>
    <description>A valid RSS 2.0 feed</description>
    <item>
      <title>First post</title>
      <link>https://example.com/posts/1</link>
      <guid>https://example.com/posts/1</guid>
      <description>Hello from the first post.</description>
      <pubDate>Mon, 04 Jan 2021 08:00:00 GMT</pubDate>
    </item>
    <item>
      <title>Second post</title>
      <link>https://example.com/posts/2</link>
      <guid isPermaLink="false">post-0002</guid>
      <description>The second post has more to say.</description>
      <pubDate>Tue, 05 Jan 2021 09:30:00 -0500</pubDate>
    </item>
  </channel>
</rss>
"#;

const ATOM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Example Atom Feed</title>
  <link rel="self" href="https://example.com/atom.xml"/>
  <link rel="alternate" href="https://example.com/"/>
  <id>urn:uuid:feed-0001</id>
  <updated>2021-02-10T12:00:00Z</updated>
  <entry>
    <title>Atom entry one</title>
    <id>urn:uuid:entry-0001</id>
    <link rel="alternate" href="https://example.com/atom/1"/>
    <link rel="edit" href="https://example.com/atom/1/edit"/>
    <summary>Summary of the first Atom entry.</summary>
    <published>2021-02-09T10:15:30+01:00</published>
    <updated>2021-02-10T11:00:00Z</updated>
  </entry>
  <entry>
    <title>Atom entry two</title>
    <id>urn:uuid:entry-0002</id>
    <link href="https://example.com/atom/2"/>
    <content>Content is used when summary is absent.</content>
    <updated>2021-02-11T08:45:00-05:00</updated>
  </entry>
</feed>
"#;

const DATES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Edge-case Dates Feed</title>
    <link>https://example.com/dates</link>
    <description>Exercises time-zone names and a missing date</description>
    <item>
      <title>Zone name EST</title>
      <guid>dates-est</guid>
      <link>https://example.com/dates/est</link>
      <pubDate>Wed, 06 Jan 2021 23:00:00 EST</pubDate>
    </item>
    <item>
      <title>Zone name PDT</title>
      <guid>dates-pdt</guid>
      <link>https://example.com/dates/pdt</link>
      <pubDate>Thu, 07 Jan 2021 16:30:00 PDT</pubDate>
    </item>
    <item>
      <title>No date at all</title>
      <guid>dates-none</guid>
      <link>https://example.com/dates/none</link>
      <description>This item has no pubDate and stores published_at = null.</description>
    </item>
    <item>
      <title>Unparseable date</title>
      <guid>dates-bad</guid>
      <link>https://example.com/dates/bad</link>
      <pubDate>sometime last week</pubDate>
    </item>
  </channel>
</rss>
"#;

const CDATA: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>CDATA &amp; Entities Feed</title>
    <link>https://example.com/cdata</link>
    <description>Exercises CDATA and XML entities</description>
    <item>
      <title>Fish &amp; Chips &lt;tasty&gt;</title>
      <guid>cdata-1</guid>
      <link>https://example.com/cdata/1</link>
      <description><![CDATA[Raw <b>HTML</b> & ampersands are kept verbatim.]]></description>
      <pubDate>Fri, 08 Jan 2021 12:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>
"#;

const MALFORMED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Broken Feed</title>
    <item>
      <title>Unclosed item
      <guid>malformed-1</guid>
    </channel>
</rss>
"#;

const README: &str = r#"# feedgen fixture corpus

Test feeds served by `feedgen serve`. No real network access is required.

| file            | purpose |
|-----------------|---------|
| `rss.xml`       | Valid RSS 2.0 feed with two items (GUID + link identity, RFC 822 dates incl. a numeric offset). |
| `atom.xml`      | Valid Atom (RFC 4287) feed: `alternate` link selection, `summary` vs `content`, `published`/`updated` fallback. |
| `dates.xml`     | Edge-case dates: zone names (`EST`, `PDT`), a missing date, and an unparseable date (both store `published_at = null`). |
| `cdata.xml`     | CDATA taken verbatim and XML entities unescaped (`&amp;` -> `&`). |
| `malformed.xml` | Malformed XML: fetching this must record `last_error` and never crash the server. |

Serve the corpus with:

```
feedgen serve --dir <DIR> --listen 127.0.0.1:8700
```

then register e.g. `http://127.0.0.1:8700/rss.xml` with `feedctl add`.
"#;

/// Files that make up the corpus, as `(filename, contents)` pairs.
pub const CORPUS: &[(&str, &str)] = &[
    ("rss.xml", RSS),
    ("atom.xml", ATOM),
    ("dates.xml", DATES),
    ("cdata.xml", CDATA),
    ("malformed.xml", MALFORMED),
    ("README.md", README),
];

pub fn write(dir: &str) -> Result<(), String> {
    let root = Path::new(dir);
    fs::create_dir_all(root).map_err(|e| format!("cannot create {dir}: {e}"))?;
    for (name, body) in CORPUS {
        let path = root.join(name);
        fs::write(&path, body).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

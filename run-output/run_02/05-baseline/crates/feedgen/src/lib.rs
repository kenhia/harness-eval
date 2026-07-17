//! `feedgen` — a fixture HTTP server and fixture-corpus generator.
//!
//! The server serves a directory of files over HTTP with correct
//! `Content-Type`, a content-derived `ETag`, and a `Last-Modified` header, and
//! honors `If-None-Match` / `If-Modified-Since` conditional requests with a
//! `304 Not Modified` response. It is used to develop and test the rest of
//! feedhub without touching the real internet.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeZone, Utc};
use tiny_http::{Header, Response, Server};

/// Bind an HTTP server to `addr` (e.g. `127.0.0.1:0` for an ephemeral port).
pub fn bind(addr: &str) -> Result<Server, Box<dyn std::error::Error + Send + Sync>> {
    Server::http(addr)
}

/// Serve `dir` over the given server, looping forever.
pub fn serve(server: &Server, dir: &Path) {
    for request in server.incoming_requests() {
        let dir = dir.to_path_buf();
        if let Err(e) = handle_request(request, &dir) {
            eprintln!("feedgen: request error: {e}");
        }
    }
}

/// Map a file extension to a Content-Type.
fn mime_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rss") => "application/rss+xml; charset=utf-8",
        Some("atom") => "application/atom+xml; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        _ => "text/plain; charset=utf-8",
    }
}

/// Content-derived, strong-looking ETag (quoted FNV-1a of the bytes).
pub fn etag_for(bytes: &[u8]) -> String {
    let mut hash: u64 = 1469598103934665603;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("\"{hash:016x}\"")
}

/// Format an instant as an HTTP-date (IMF-fixdate, GMT).
pub fn http_date(dt: DateTime<Utc>) -> String {
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

/// Parse an HTTP-date back into an instant (IMF-fixdate only).
fn parse_http_date(s: &str) -> Option<DateTime<Utc>> {
    chrono::NaiveDateTime::parse_from_str(s.trim(), "%a, %d %b %Y %H:%M:%S GMT")
        .ok()
        .map(|naive| Utc.from_utc_datetime(&naive))
}

/// Resolve a request URL path to a file inside `dir`, rejecting traversal.
fn resolve_path(dir: &Path, url: &str) -> Option<PathBuf> {
    let raw = url.split('?').next().unwrap_or(url);
    let trimmed = raw.trim_start_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    // Reject any path component that is not a plain file name.
    let mut path = dir.to_path_buf();
    for comp in trimmed.split('/') {
        if comp.is_empty() || comp == "." || comp == ".." {
            return None;
        }
        path.push(comp);
    }
    Some(path)
}

fn header(field: &str, value: &str) -> Header {
    Header::from_bytes(field.as_bytes(), value.as_bytes()).expect("valid header")
}

fn find_header<'a>(req: &'a tiny_http::Request, name: &str) -> Option<&'a str> {
    req.headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str())
}

/// Handle a single request against `dir`.
pub fn handle_request(request: tiny_http::Request, dir: &Path) -> io::Result<()> {
    let url = request.url().to_string();

    let path = match resolve_path(dir, &url) {
        Some(p) => p,
        None => {
            let resp = Response::from_string("400 bad request").with_status_code(400);
            return request.respond(resp);
        }
    };

    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(_) => {
            let resp = Response::from_string("404 not found").with_status_code(404);
            return request.respond(resp);
        }
    };

    let etag = etag_for(&bytes);
    let mtime = fs::metadata(&path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|d| Utc.timestamp_opt(d.as_secs() as i64, 0).single())
        .unwrap_or_else(Utc::now);
    let last_modified = http_date(mtime);

    // Conditional GET: If-None-Match wins over If-Modified-Since.
    let inm = find_header(&request, "If-None-Match").map(|s| s.to_string());
    let ims = find_header(&request, "If-Modified-Since").map(|s| s.to_string());

    let not_modified = if let Some(inm) = inm {
        inm.split(',').any(|t| t.trim() == etag || t.trim() == "*")
    } else if let Some(ims) = ims {
        match parse_http_date(&ims) {
            // Truncate to whole seconds for comparison.
            Some(since) => mtime.timestamp() <= since.timestamp(),
            None => false,
        }
    } else {
        false
    };

    if not_modified {
        let resp = Response::from_data(Vec::new())
            .with_status_code(304)
            .with_header(header("ETag", &etag))
            .with_header(header("Last-Modified", &last_modified));
        return request.respond(resp);
    }

    let resp = Response::from_data(bytes)
        .with_header(header("Content-Type", mime_for(&path)))
        .with_header(header("ETag", &etag))
        .with_header(header("Last-Modified", &last_modified));
    request.respond(resp)
}

/// Write the fixture corpus into `dir`, creating it if necessary.
pub fn make_fixtures(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    for (name, content) in FIXTURES {
        fs::write(dir.join(name), content)?;
    }
    Ok(())
}

/// The fixture corpus: (filename, contents).
pub const FIXTURES: &[(&str, &str)] = &[
    ("rss.xml", RSS_FIXTURE),
    ("atom.xml", ATOM_FIXTURE),
    ("dates.xml", DATES_FIXTURE),
    ("cdata.xml", CDATA_FIXTURE),
    ("malformed.xml", MALFORMED_FIXTURE),
    ("README.md", CORPUS_README),
];

const RSS_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Example RSS Feed</title>
    <link>http://example.com/</link>
    <description>A valid RSS 2.0 feed.</description>
    <item>
      <title>Hello World</title>
      <link>http://example.com/posts/1</link>
      <description>The first post.</description>
      <guid>http://example.com/posts/1</guid>
      <pubDate>Mon, 02 Jan 2006 15:04:05 -0500</pubDate>
    </item>
    <item>
      <title>Second Post</title>
      <link>http://example.com/posts/2</link>
      <description>Another post &amp; more.</description>
      <guid isPermaLink="false">post-2</guid>
      <pubDate>Tue, 03 Jan 2006 09:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>
"#;

const ATOM_FIXTURE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Example Atom Feed</title>
  <link rel="alternate" href="http://example.com/"/>
  <id>urn:uuid:feed-0001</id>
  <updated>2006-01-02T15:04:05Z</updated>
  <entry>
    <title>Atom Entry One</title>
    <id>urn:uuid:entry-0001</id>
    <link rel="edit" href="http://example.com/edit/1"/>
    <link rel="alternate" href="http://example.com/atom/1"/>
    <summary>Summary of entry one.</summary>
    <published>2006-01-02T15:04:05Z</published>
    <updated>2006-01-02T16:00:00Z</updated>
  </entry>
  <entry>
    <title>Atom Entry Two</title>
    <id>urn:uuid:entry-0002</id>
    <link href="http://example.com/atom/2"/>
    <content>Content stands in for a missing summary.</content>
    <updated>2006-01-03T12:00:00+02:00</updated>
  </entry>
</feed>
"#;

const DATES_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Edge-Case Dates</title>
    <link>http://example.com/dates</link>
    <description>Exercises RFC 822 zone names and a missing date.</description>
    <item>
      <title>Pacific Standard</title>
      <link>http://example.com/dates/pst</link>
      <guid>dates-pst</guid>
      <pubDate>Sat, 01 Jan 2005 10:30:00 PST</pubDate>
    </item>
    <item>
      <title>Eastern Daylight</title>
      <link>http://example.com/dates/edt</link>
      <guid>dates-edt</guid>
      <pubDate>Wed, 15 Jun 2005 12:00:00 EDT</pubDate>
    </item>
    <item>
      <title>Universal Time</title>
      <link>http://example.com/dates/ut</link>
      <guid>dates-ut</guid>
      <pubDate>Fri, 31 Dec 2004 23:59:00 UT</pubDate>
    </item>
    <item>
      <title>No Date Here</title>
      <link>http://example.com/dates/none</link>
      <guid>dates-none</guid>
    </item>
    <item>
      <title>Unparseable Date</title>
      <link>http://example.com/dates/bad</link>
      <guid>dates-bad</guid>
      <pubDate>sometime last week</pubDate>
    </item>
  </channel>
</rss>
"#;

const CDATA_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>CDATA &amp; Entities</title>
    <link>http://example.com/cdata</link>
    <description>Exercises CDATA sections and entity unescaping.</description>
    <item>
      <title>Fish &amp; Chips</title>
      <link>http://example.com/cdata/1</link>
      <guid>cdata-1</guid>
      <description><![CDATA[Raw <b>markup</b> & ampersands stay verbatim.]]></description>
      <pubDate>Mon, 02 Jan 2006 15:04:05 +0000</pubDate>
    </item>
    <item>
      <title>Less &lt; Greater &gt;</title>
      <link>http://example.com/cdata/2</link>
      <guid>cdata-2</guid>
      <description>Entities like &amp;amp; become &amp; after unescaping.</description>
      <pubDate>Mon, 02 Jan 2006 16:04:05 +0000</pubDate>
    </item>
  </channel>
</rss>
"#;

const MALFORMED_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Broken Feed</title>
    <item>
      <title>Unterminated element
      <link>http://example.com/broken</link>
    </item>
  </channel>
"#;

const CORPUS_README: &str = r#"# feedgen fixture corpus

Test feeds served by `feedgen serve`. All content is synthetic; no real-world
feeds are included.

| file            | purpose |
|-----------------|---------|
| `rss.xml`       | A valid RSS 2.0 feed (two items, numeric offset + `GMT`). |
| `atom.xml`      | A valid Atom (RFC 4287) feed; second entry uses `content` and `updated` only. |
| `dates.xml`     | Edge-case dates: `PST`/`EDT`/`UT` zone names, a missing date, and an unparseable date. |
| `cdata.xml`     | CDATA sections (verbatim) and XML entity unescaping. |
| `malformed.xml` | Deliberately malformed XML for failure-isolation tests. |

## Serving

```
feedgen serve --dir DIR [--listen ADDR:PORT]
```

Each file is served with a content-derived `ETag` and a `Last-Modified`
header; `If-None-Match` / `If-Modified-Since` requests receive `304`.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn etag_is_stable_and_content_derived() {
        assert_eq!(etag_for(b"hello"), etag_for(b"hello"));
        assert_ne!(etag_for(b"hello"), etag_for(b"world"));
    }

    #[test]
    fn resolve_rejects_traversal() {
        let dir = Path::new("/srv/feeds");
        assert!(resolve_path(dir, "/../etc/passwd").is_none());
        assert!(resolve_path(dir, "/a/../b").is_none());
        assert!(resolve_path(dir, "/").is_none());
        assert_eq!(resolve_path(dir, "/rss.xml?x=1"), Some(dir.join("rss.xml")));
    }

    #[test]
    fn http_date_roundtrips() {
        let dt = Utc.with_ymd_and_hms(2006, 1, 2, 15, 4, 5).unwrap();
        let s = http_date(dt);
        assert_eq!(parse_http_date(&s).unwrap().timestamp(), dt.timestamp());
    }
}

//! feedgen — a fixture tool that serves and generates test feeds over local
//! HTTP so feedhub can be developed and tested without touching the internet.

use clap::{Parser, Subcommand};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tiny_http::{Header, Request, Response, Server};

#[derive(Parser)]
#[command(
    name = "feedgen",
    version,
    about = "Serve and generate test feeds over local HTTP"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the files in DIR over HTTP with ETag / Last-Modified support.
    Serve {
        #[arg(long)]
        dir: PathBuf,
        #[arg(long, default_value = "127.0.0.1:8700")]
        listen: String,
    },
    /// Write a documented fixture corpus into DIR.
    MakeFixtures {
        /// Target directory (created if missing).
        dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Command::Serve { dir, listen } => serve(&dir, &listen),
        Command::MakeFixtures { dir } => make_fixtures(&dir),
    };
    std::process::exit(code);
}

/// FNV-1a 64-bit hash, used for content-derived ETags.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("xml") | Some("rss") => "application/rss+xml; charset=utf-8",
        Some("atom") => "application/atom+xml; charset=utf-8",
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("json") => "application/json",
        Some("txt") | Some("md") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn http_date(t: SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Utc> = t.into();
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid header")
}

fn serve(dir: &Path, listen: &str) -> i32 {
    let dir = match dir.canonicalize() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("feedgen: cannot open --dir {}: {e}", dir.display());
            return 2;
        }
    };
    let server = match Server::http(listen) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("feedgen: cannot listen on {listen}: {e}");
            return 2;
        }
    };
    println!("feedgen serving {} on http://{listen}", dir.display());
    for request in server.incoming_requests() {
        handle(&dir, request);
    }
    0
}

fn handle(dir: &Path, request: Request) {
    let url = request.url().to_string();
    let rel = url.split('?').next().unwrap_or("").trim_start_matches('/');

    if rel.contains("..") {
        let _ = request.respond(text_response(403, "forbidden"));
        return;
    }
    let path = dir.join(rel);
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(_) => {
            let _ = request.respond(text_response(404, "not found"));
            return;
        }
    };

    let etag = format!("\"{:016x}\"", fnv1a(&bytes));
    let last_modified = fs::metadata(&path)
        .and_then(|m| m.modified())
        .map(http_date)
        .ok();

    // Conditional GET.
    let mut if_none_match: Option<String> = None;
    let mut if_modified_since: Option<String> = None;
    for h in request.headers() {
        let field = h.field.as_str().as_str().to_ascii_lowercase();
        if field == "if-none-match" {
            if_none_match = Some(h.value.as_str().to_string());
        } else if field == "if-modified-since" {
            if_modified_since = Some(h.value.as_str().to_string());
        }
    }

    let etag_match = if_none_match
        .as_deref()
        .map(|v| v.contains(&etag))
        .unwrap_or(false);
    let date_match = match (&if_modified_since, &last_modified) {
        (Some(ims), Some(lm)) => ims == lm,
        _ => false,
    };

    if etag_match || date_match {
        let mut resp = Response::empty(304);
        resp.add_header(header("ETag", &etag));
        if let Some(lm) = &last_modified {
            resp.add_header(header("Last-Modified", lm));
        }
        let _ = request.respond(resp);
        return;
    }

    let mut resp = Response::from_data(bytes);
    resp.add_header(header("Content-Type", content_type(&path)));
    resp.add_header(header("ETag", &etag));
    if let Some(lm) = &last_modified {
        resp.add_header(header("Last-Modified", lm));
    }
    let _ = request.respond(resp);
}

fn text_response(status: u16, body: &str) -> Response<Cursor<Vec<u8>>> {
    let mut resp = Response::from_string(body).with_status_code(status);
    resp.add_header(header("Content-Type", "text/plain; charset=utf-8"));
    resp
}

fn make_fixtures(dir: &Path) -> i32 {
    if let Err(e) = fs::create_dir_all(dir) {
        eprintln!("feedgen: cannot create {}: {e}", dir.display());
        return 2;
    }
    let files: &[(&str, &str)] = &[
        ("rss.xml", RSS),
        ("atom.xml", ATOM),
        ("dates.xml", DATES),
        ("cdata.xml", CDATA),
        ("malformed.xml", MALFORMED),
        ("truncated.xml", TRUNCATED),
        ("README.md", CORPUS_README),
    ];
    for (name, body) in files {
        if let Err(e) = fs::write(dir.join(name), body) {
            eprintln!("feedgen: cannot write {name}: {e}");
            return 2;
        }
    }
    println!("wrote {} fixtures into {}", files.len(), dir.display());
    0
}

const RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Example RSS Feed</title>
    <link>http://example.com/</link>
    <description>A valid RSS 2.0 feed for testing.</description>
    <item>
      <title>Hello RSS &amp; World</title>
      <link>http://example.com/posts/1</link>
      <guid>urn:example:post:1</guid>
      <description>First post in the RSS fixture.</description>
      <pubDate>Wed, 02 Oct 2002 13:00:00 GMT</pubDate>
    </item>
    <item>
      <title>Second Post</title>
      <link>http://example.com/posts/2</link>
      <guid>urn:example:post:2</guid>
      <description>Second post, with a numeric offset date.</description>
      <pubDate>Mon, 15 Jul 2024 10:30:00 -0500</pubDate>
    </item>
  </channel>
</rss>
"#;

const ATOM: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Example Atom Feed</title>
  <link rel="alternate" href="http://example.com/atom"/>
  <id>urn:example:atom</id>
  <updated>2024-07-16T00:00:00Z</updated>
  <entry>
    <title>Atom Entry One</title>
    <id>tag:example.com,2024:entry:1</id>
    <link rel="alternate" href="http://example.com/atom/1"/>
    <summary>Summary of the first Atom entry.</summary>
    <published>2024-07-15T10:30:00-05:00</published>
    <updated>2024-07-15T11:00:00-05:00</updated>
  </entry>
  <entry>
    <title>Atom Entry Two</title>
    <id>tag:example.com,2024:entry:2</id>
    <link href="http://example.com/atom/2"/>
    <content>Content used as the summary fallback.</content>
    <updated>2024-07-16T00:00:00Z</updated>
  </entry>
</feed>
"#;

const DATES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Edge-case Dates Feed</title>
    <link>http://example.com/dates</link>
    <description>Exercises zone names and a missing date.</description>
    <item>
      <title>Eastern Standard Time</title>
      <guid>urn:dates:est</guid>
      <pubDate>Mon, 15 Jul 2024 10:30:00 EST</pubDate>
    </item>
    <item>
      <title>Pacific Daylight Time</title>
      <guid>urn:dates:pdt</guid>
      <pubDate>Mon, 15 Jul 2024 10:30:00 PDT</pubDate>
    </item>
    <item>
      <title>Missing Date</title>
      <guid>urn:dates:none</guid>
      <description>This item has no pubDate; published_at must be null.</description>
    </item>
  </channel>
</rss>
"#;

const CDATA: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>CDATA &amp; Entities Feed</title>
    <link>http://example.com/cdata</link>
    <description>Exercises CDATA and XML entities.</description>
    <item>
      <title>Tom &amp; Jerry &lt;3</title>
      <guid>urn:cdata:1</guid>
      <description><![CDATA[<p>Raw <b>HTML</b> &amp; markup preserved verbatim.</p>]]></description>
      <pubDate>Wed, 02 Oct 2002 08:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>
"#;

const MALFORMED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Broken Feed
    <item>
      <title>Unclosed tags everywhere
      <guid>urn:broken:1
  </channel>
"#;

const TRUNCATED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<rss version=\"2.0\"><channel><title>Nightly</title>\n\
<item><guid>n-1</guid><title>Release no";

const CORPUS_README: &str = r#"# feedgen fixture corpus

These files are served by `feedgen serve --dir <this dir>` for tests.

| file | purpose |
|------|---------|
| `rss.xml` | Valid RSS 2.0 feed (two items, GMT and numeric-offset dates). |
| `atom.xml` | Valid Atom (RFC 4287) feed (alternate link, summary and content fallback). |
| `dates.xml` | Edge-case dates: zone names (EST, PDT) and one item with a missing date (stored as null). |
| `cdata.xml` | CDATA content taken verbatim plus XML entities that are unescaped in titles. |
| `malformed.xml` | Intentionally malformed XML (mismatched tags); a fetch of this file records `last_error` and never crashes the server. |
| `truncated.xml` | A well-started document cut off mid-element (a broken upstream response); a fetch records `last_error` rather than a silent empty success. |

All timestamps are normalized to UTC (RFC 3339, `Z`) when stored by `feedd`.
"#;

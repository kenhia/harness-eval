"""Fixture feed corpus for the run_02 acceptance suite.

Every date's UTC equivalent is stated next to it — tests compare instants,
not strings. SEALED: never shown to working agents.
"""


def rss(channel_title: str, items: str) -> bytes:
    return (
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        '<rss version="2.0"><channel>'
        f"<title>{channel_title}</title><link>http://example.test/</link>"
        f"<description>fixture</description>{items}"
        "</channel></rss>"
    ).encode()


def rss_item(guid: str, title: str, pubdate: str | None, link: str = "http://example.test/x") -> str:
    d = f"<pubDate>{pubdate}</pubDate>" if pubdate is not None else ""
    return f"<item><guid>{guid}</guid><title>{title}</title><link>{link}</link><description>s-{guid}</description>{d}</item>"


def atom(feed_title: str, entries: str) -> bytes:
    return (
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        '<feed xmlns="http://www.w3.org/2005/Atom">'
        f"<title>{feed_title}</title><id>urn:fixture</id><updated>2026-07-11T00:00:00Z</updated>{entries}"
        "</feed>"
    ).encode()


def atom_entry(eid: str, title: str, published: str | None, updated: str = "2026-07-11T00:00:00Z") -> str:
    p = f"<published>{published}</published>" if published else ""
    return (
        f'<entry><id>urn:{eid}</id><title>{title}</title>'
        f'<link rel="alternate" href="http://example.test/{eid}"/>'
        f"<summary>s-{eid}</summary>{p}<updated>{updated}</updated></entry>"
    )


CORPUS: dict[str, bytes] = {}

# --- core ---
CORPUS["rss_basic.xml"] = rss(
    "RSS Basic",
    rss_item("g1", "First post", "Fri, 10 Jul 2026 10:00:00 +0000")      # 2026-07-10T10:00:00Z
    + rss_item("g2", "Second post", "Fri, 10 Jul 2026 09:00:00 +0000")   # 2026-07-10T09:00:00Z
    + rss_item("g3", "Third post", "Fri, 10 Jul 2026 08:00:00 +0000"),   # 2026-07-10T08:00:00Z
)
CORPUS["atom_basic.xml"] = atom(
    "Atom Basic",
    atom_entry("a1", "Alpha", "2026-07-11T10:00:00Z")
    + atom_entry("a2", "Beta", "2026-07-11T09:00:00Z")
    + atom_entry("a3", "Gamma", "2026-07-11T08:00:00Z"),
)
CORPUS["malformed.xml"] = b"<rss><channel><item>truncated garbage"
CORPUS["rss_update_v1.xml"] = rss(
    "Updating feed",
    rss_item("u1", "Original title", "Fri, 10 Jul 2026 10:00:00 +0000"),
)
CORPUS["rss_update_v2.xml"] = rss(  # served at the SAME path as v1 after swap
    "Updating feed",
    rss_item("u1", "Revised title", "Fri, 10 Jul 2026 10:00:00 +0000")
    + rss_item("u2", "Brand new", "Fri, 10 Jul 2026 11:00:00 +0000"),    # 2026-07-10T11:00:00Z
)
CORPUS["rss_search.xml"] = rss(
    "Search feed",
    rss_item("s1", "Rust Weekly", "Fri, 10 Jul 2026 10:00:00 +0000")
    + rss_item("s2", "python daily", "Fri, 10 Jul 2026 09:00:00 +0000")
    + rss_item("s3", "RUSTACEAN news", "Fri, 10 Jul 2026 08:00:00 +0000")
    + rss_item("s4", "Crustacean recipes", "Fri, 10 Jul 2026 07:00:00 +0000"),
)

# --- hard ---
CORPUS["rss_zones.xml"] = rss(
    "Zone feed",
    rss_item("z1", "From EST", "Fri, 10 Jul 2026 07:00:00 EST")          # EST=UTC-5 -> 2026-07-10T12:00:00Z
    + rss_item("z2", "From GMT", "Fri, 10 Jul 2026 06:00:00 GMT")        # 2026-07-10T06:00:00Z
    + rss_item("z3", "No date", None)                                    # published_at null
    + rss_item("z4", "Bad date", "sometime last week"),                  # published_at null
)
CORPUS["atom_offsets.xml"] = atom(
    "Offset feed",
    atom_entry("o1", "Plus two", "2026-07-11T12:30:00+02:00")            # 2026-07-11T10:30:00Z
    + atom_entry("o2", "Fractional", "2026-07-11T05:00:00.500Z")         # 2026-07-11T05:00:00.5Z
    + atom_entry("o3", "Updated only", None, "2026-07-11T04:00:00Z"),    # falls back to updated: 04:00Z
)
CORPUS["rss_boundary.xml"] = rss(
    "Boundary feed",
    rss_item("b1", "At until", "Sun, 12 Jul 2026 00:00:00 +0000")        # exactly 2026-07-12T00:00:00Z
    + rss_item("b2", "Middle", "Sat, 11 Jul 2026 12:00:00 +0000")        # 2026-07-11T12:00:00Z
    + rss_item("b3", "At since", "Sat, 11 Jul 2026 00:00:00 +0000"),     # exactly 2026-07-11T00:00:00Z
)
CORPUS["rss_bom_cdata.xml"] = b"\xef\xbb\xbf" + rss(
    "BOM feed",
    rss_item("c1", "<![CDATA[Ben & Jerry <3]]>", "Fri, 10 Jul 2026 10:00:00 +0000")
    + rss_item("c2", "Fish &amp; Chips", "Fri, 10 Jul 2026 09:00:00 +0000"),
)
CORPUS["rss_paging.xml"] = rss(
    "Paging feed",
    "".join(
        rss_item(f"p{i:02d}", f"Item {i:02d}", f"Fri, 10 Jul 2026 {i:02d}:30:00 +0000")
        for i in range(24)  # 00:30Z .. 23:30Z, 24 items
    ),
)
CORPUS["rss_broken_sibling.xml"] = rss(
    "Healthy sibling",
    rss_item("h1", "Still fine", "Fri, 10 Jul 2026 10:00:00 +0000"),
)

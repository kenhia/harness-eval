//! Date parsing and normalization.
//!
//! Every instant feedhub stores is normalized to UTC at millisecond precision.
//! Storage keeps two representations of the same instant:
//!
//! - `published_at` TEXT — RFC 3339 with `Z`, what the API returns.
//! - `published_at_ms` INTEGER — epoch milliseconds, the ordering and range key.
//!
//! The integer column exists so that ordering and the half-open `since`/`until`
//! window compare *actual instants*, per spec, rather than leaning on the text
//! form sorting correctly. Text sorting is a trap here: `'.'` (0x2E) sorts
//! before `'Z'` (0x5A), so `2020-01-01T00:00:00.5Z` would compare *less than*
//! `2020-01-01T00:00:00Z` despite being later. Sorting on the integer sidesteps
//! that whole class of bug and lets the text form stay faithful to the source.

use chrono::{DateTime, NaiveDate, SecondsFormat, SubsecRound, TimeZone, Utc};

/// Normalize an instant to feedhub's stored precision.
///
/// Millisecond granularity keeps the TEXT and INTEGER columns exactly in
/// agreement (a sub-millisecond difference could otherwise order two entries
/// one way by instant and another by text). Feed timestamps do not carry
/// meaningful sub-millisecond precision.
pub fn normalize(dt: DateTime<Utc>) -> DateTime<Utc> {
    dt.trunc_subsecs(3)
}

/// Serialize an instant as RFC 3339 with a `Z` offset.
///
/// Precision is automatic: whole seconds when there is no fractional part,
/// milliseconds when there is. Both are valid RFC 3339.
pub fn format_rfc3339(dt: DateTime<Utc>) -> String {
    let dt = normalize(dt);
    if dt.timestamp_subsec_millis() == 0 {
        dt.to_rfc3339_opts(SecondsFormat::Secs, true)
    } else {
        dt.to_rfc3339_opts(SecondsFormat::Millis, true)
    }
}

/// The ordering / range key for an instant: epoch milliseconds.
pub fn to_millis(dt: DateTime<Utc>) -> i64 {
    normalize(dt).timestamp_millis()
}

/// The current instant, normalized.
pub fn now() -> DateTime<Utc> {
    normalize(Utc::now())
}

/// Parse an RFC 3339 instant with any numeric offset and optional fractional
/// seconds, normalized to UTC.
pub fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s.trim())
        .ok()
        .map(|dt| normalize(dt.with_timezone(&Utc)))
}

/// Map the zone names pinned by the spec to their UTC offsets in seconds.
///
/// This deliberately does not defer to chrono's RFC 2822 parser: RFC 2822 §4.3
/// declares the obsolete US zone abbreviations to carry no reliable meaning and
/// says to treat them as `-0000` (unknown offset). Real-world RSS uses them with
/// their conventional offsets and the spec pins those, so we resolve them here.
fn zone_offset_seconds(zone: &str) -> Option<i32> {
    let hours = match zone {
        "GMT" | "UT" | "UTC" | "Z" => 0,
        "EST" => -5,
        "EDT" => -4,
        "CST" => -6,
        "CDT" => -5,
        "MST" => -7,
        "MDT" => -6,
        "PST" => -8,
        "PDT" => -7,
        _ => return None,
    };
    Some(hours * 3600)
}

/// Parse a numeric RFC 822 offset such as `+0000` or `-0500`.
fn numeric_offset_seconds(zone: &str) -> Option<i32> {
    let bytes = zone.as_bytes();
    if bytes.len() != 5 {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    if !bytes[1..].iter().all(u8::is_ascii_digit) {
        return None;
    }
    let hours: i32 = zone[1..3].parse().ok()?;
    let minutes: i32 = zone[3..5].parse().ok()?;
    if minutes > 59 {
        return None;
    }
    Some(sign * (hours * 3600 + minutes * 60))
}

fn month_number(name: &str) -> Option<u32> {
    let m = match name {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    };
    Some(m)
}

/// Expand an RFC 822 year. The spec pins 4-digit years; 2- and 3-digit forms are
/// tolerated per RFC 2822 §4.3 because older feeds still emit them.
fn expand_year(raw: &str) -> Option<i32> {
    if !raw.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: i32 = raw.parse().ok()?;
    match raw.len() {
        4 => Some(n),
        3 => Some(1900 + n),
        2 if n >= 50 => Some(1900 + n),
        2 => Some(2000 + n),
        _ => None,
    }
}

/// Parse an RFC 822 / RFC 1123 date as used by RSS `<pubDate>`, normalized to
/// UTC.
///
/// Accepts an optional leading day-of-week, a 1- or 2-digit day, a 3-letter
/// month, a 4-digit year (2- and 3-digit tolerated), `HH:MM` with optional
/// `:SS`, and a zone that is either a numeric offset or one of the pinned names.
pub fn parse_rfc822(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    // Drop the optional "Tue," day-of-week prefix; it carries nothing we need
    // and feeds are not consistent about including it.
    let rest = match s.find(',') {
        Some(i) if i <= 3 => s[i + 1..].trim(),
        _ => s,
    };

    let mut parts = rest.split_whitespace();
    let day: u32 = parts.next()?.parse().ok()?;
    let month = month_number(parts.next()?)?;
    let year = expand_year(parts.next()?)?;
    let time = parts.next()?;

    // An absent zone is technically malformed; RFC 2822 §4.3 says to read an
    // unknown zone as -0000, i.e. UTC. That is the tolerant interpretation.
    let zone = parts.next().unwrap_or("GMT");
    if parts.next().is_some() {
        return None;
    }

    let mut hms = time.split(':');
    let hour: u32 = hms.next()?.parse().ok()?;
    let minute: u32 = hms.next()?.parse().ok()?;
    let second: u32 = match hms.next() {
        Some(sec) => sec.parse().ok()?,
        None => 0,
    };
    if hms.next().is_some() {
        return None;
    }

    // Clamp a leap second down to :59. chrono represents leap seconds as a
    // nanosecond overflow, which must not leak into storage.
    let second = if second == 60 { 59 } else { second };

    let offset = numeric_offset_seconds(zone).or_else(|| zone_offset_seconds(zone))?;
    let naive = NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, minute, second)?;
    let fixed = chrono::FixedOffset::east_opt(offset)?;
    let local = fixed.from_local_datetime(&naive).single()?;
    Some(normalize(local.with_timezone(&Utc)))
}

/// Parse a feed date, trying RFC 3339 first and then RFC 822.
///
/// Returns `None` for anything unparseable. Callers store `None` as a NULL
/// `published_at`; the spec is explicit that fetch time must never be
/// substituted for a missing or broken date.
pub fn parse_feed_date(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    parse_rfc3339(s).or_else(|| parse_rfc822(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(s: &str) -> DateTime<Utc> {
        parse_rfc3339(s).expect("test literal should parse")
    }

    #[test]
    fn rfc822_numeric_offsets() {
        assert_eq!(
            parse_rfc822("Tue, 10 Jun 2003 04:00:00 +0000"),
            Some(utc("2003-06-10T04:00:00Z"))
        );
        assert_eq!(
            parse_rfc822("Tue, 10 Jun 2003 04:00:00 -0500"),
            Some(utc("2003-06-10T09:00:00Z"))
        );
        assert_eq!(
            parse_rfc822("10 Jun 2003 04:00:00 +0530"),
            Some(utc("2003-06-09T22:30:00Z"))
        );
    }

    #[test]
    fn rfc822_all_pinned_zone_names() {
        let cases = [
            ("GMT", "2020-01-01T12:00:00Z"),
            ("UT", "2020-01-01T12:00:00Z"),
            ("Z", "2020-01-01T12:00:00Z"),
            ("EST", "2020-01-01T17:00:00Z"),
            ("EDT", "2020-01-01T16:00:00Z"),
            ("CST", "2020-01-01T18:00:00Z"),
            ("CDT", "2020-01-01T17:00:00Z"),
            ("MST", "2020-01-01T19:00:00Z"),
            ("MDT", "2020-01-01T18:00:00Z"),
            ("PST", "2020-01-01T20:00:00Z"),
            ("PDT", "2020-01-01T19:00:00Z"),
        ];
        for (zone, expected) in cases {
            let input = format!("Wed, 01 Jan 2020 12:00:00 {zone}");
            assert_eq!(
                parse_rfc822(&input),
                Some(utc(expected)),
                "zone {zone} should resolve to {expected}"
            );
        }
    }

    #[test]
    fn rfc822_four_digit_year_required_two_digit_tolerated() {
        assert_eq!(
            parse_rfc822("01 Jan 2020 00:00:00 GMT"),
            Some(utc("2020-01-01T00:00:00Z"))
        );
        assert_eq!(
            parse_rfc822("01 Jan 99 00:00:00 GMT"),
            Some(utc("1999-01-01T00:00:00Z"))
        );
        assert_eq!(
            parse_rfc822("01 Jan 20 00:00:00 GMT"),
            Some(utc("2020-01-01T00:00:00Z"))
        );
    }

    #[test]
    fn rfc822_optional_seconds_and_dayname() {
        assert_eq!(
            parse_rfc822("Wed, 01 Jan 2020 12:30 GMT"),
            Some(utc("2020-01-01T12:30:00Z"))
        );
        assert_eq!(
            parse_rfc822("1 Jan 2020 12:30:45 GMT"),
            Some(utc("2020-01-01T12:30:45Z"))
        );
    }

    #[test]
    fn rfc822_rejects_garbage() {
        for bad in [
            "",
            "not a date",
            "32 Jan 2020 00:00:00 GMT",   // impossible day
            "01 Xxx 2020 00:00:00 GMT",   // bad month
            "01 Jan 2020 25:00:00 GMT",   // bad hour
            "01 Jan 2020 00:00:00 XYZ",   // unknown zone
            "01 Jan 2020 00:00:00 +9999", // minutes out of range
            "01 Jan 2o2o 00:00:00 GMT",   // non-numeric year
        ] {
            assert_eq!(parse_rfc822(bad), None, "{bad:?} must not parse");
        }
    }

    #[test]
    fn rfc3339_offsets_and_fractional_seconds() {
        assert_eq!(
            parse_rfc3339("2020-01-01T00:00:00Z"),
            Some(utc("2020-01-01T00:00:00Z"))
        );
        assert_eq!(
            parse_rfc3339("2020-01-01T05:30:00+05:30"),
            Some(utc("2020-01-01T00:00:00Z"))
        );
        assert_eq!(
            parse_rfc3339("2019-12-31T19:00:00-05:00"),
            Some(utc("2020-01-01T00:00:00Z"))
        );
        // Fractional seconds are accepted and preserved to millisecond precision.
        let frac = parse_rfc3339("2020-01-01T00:00:00.123Z").unwrap();
        assert_eq!(frac.timestamp_subsec_millis(), 123);
    }

    #[test]
    fn feed_date_tries_both_grammars() {
        assert!(parse_feed_date("2020-01-01T00:00:00Z").is_some());
        assert!(parse_feed_date("Wed, 01 Jan 2020 00:00:00 GMT").is_some());
        assert_eq!(parse_feed_date("last tuesday"), None);
        assert_eq!(parse_feed_date("   "), None);
    }

    #[test]
    fn rfc3339_serialization_uses_z_and_keeps_millis() {
        assert_eq!(
            format_rfc3339(utc("2020-01-01T00:00:00Z")),
            "2020-01-01T00:00:00Z"
        );
        assert_eq!(
            format_rfc3339(utc("2020-01-01T00:00:00.123Z")),
            "2020-01-01T00:00:00.123Z"
        );
        // +05:30 input must come back out as UTC with Z, never as an offset.
        assert_eq!(
            format_rfc3339(parse_rfc3339("2020-01-01T05:30:00+05:30").unwrap()),
            "2020-01-01T00:00:00Z"
        );
    }

    #[test]
    fn millis_key_orders_sub_second_instants_that_text_would_misorder() {
        // The exact case that made lexicographic text ordering wrong: '.' sorts
        // before 'Z', so the text form would rank .5 *before* the whole second.
        let whole = utc("2020-01-01T00:00:00Z");
        let frac = utc("2020-01-01T00:00:00.500Z");
        assert!(to_millis(frac) > to_millis(whole));
        assert!(
            format_rfc3339(frac) < format_rfc3339(whole),
            "text form misorders these, which is why the integer key exists"
        );
    }

    #[test]
    fn normalize_truncates_to_millisecond_precision() {
        let dt = Utc.timestamp_nanos(1_577_836_800_123_456_789);
        assert_eq!(normalize(dt).timestamp_subsec_nanos(), 123_000_000);
        assert_eq!(to_millis(dt), 1_577_836_800_123);
    }
}

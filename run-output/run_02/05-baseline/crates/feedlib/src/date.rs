//! Date parsing and normalization.
//!
//! All parsed dates are normalized to UTC. Two input grammars are supported:
//!
//! * RFC 3339 (Atom `published`/`updated`): any numeric offset and optional
//!   fractional seconds.
//! * RFC 822/1123 (RSS `pubDate`): 4-digit years, numeric offsets
//!   (`+0000`, `-0500`) and the zone names
//!   `GMT UT Z EST EDT CST CDT MST MDT PST PDT`.
//!
//! A value that cannot be parsed yields `None`; callers must store `null`
//! rather than substituting the fetch time.

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, SecondsFormat, TimeZone, Utc};

/// Parse a date string that may be RFC 3339 or RFC 822, returning a UTC
/// instant. Returns `None` when neither grammar matches.
pub fn parse_datetime(input: &str) -> Option<DateTime<Utc>> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    parse_rfc3339(s).or_else(|| parse_rfc822(s))
}

/// Parse an RFC 3339 timestamp, normalizing to UTC.
pub fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s.trim())
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Serialize a UTC instant as RFC 3339 with a trailing `Z`, seconds precision.
pub fn to_rfc3339_z(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn month_from_abbrev(m: &str) -> Option<u32> {
    let m = m.to_ascii_lowercase();
    Some(match m.as_str() {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    })
}

/// Resolve an RFC 822 timezone token to an offset in seconds east of UTC.
fn zone_offset_seconds(tok: &str) -> Option<i32> {
    // Numeric offset, e.g. +0000, -0500, possibly +00:00.
    let bytes = tok.as_bytes();
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        let sign = if bytes[0] == b'-' { -1 } else { 1 };
        let digits: String = tok[1..].chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() == 4 {
            let hh: i32 = digits[0..2].parse().ok()?;
            let mm: i32 = digits[2..4].parse().ok()?;
            if mm >= 60 {
                return None;
            }
            return Some(sign * (hh * 3600 + mm * 60));
        }
        return None;
    }
    // Named zones (uppercase-insensitive).
    let named = match tok.to_ascii_uppercase().as_str() {
        "UT" | "GMT" | "Z" => 0,
        "EST" => -5 * 3600,
        "EDT" => -4 * 3600,
        "CST" => -6 * 3600,
        "CDT" => -5 * 3600,
        "MST" => -7 * 3600,
        "MDT" => -6 * 3600,
        "PST" => -8 * 3600,
        "PDT" => -7 * 3600,
        _ => return None,
    };
    Some(named)
}

/// Parse an RFC 822 / 1123 timestamp, normalizing to UTC.
pub fn parse_rfc822(s: &str) -> Option<DateTime<Utc>> {
    // Drop an optional leading day-of-week token ("Mon, ").
    let s = s.trim();
    let rest = match s.find(',') {
        Some(i) if i <= 3 => s[i + 1..].trim(),
        _ => s,
    };
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    // Expect: DD Mon YYYY HH:MM[:SS] ZONE
    if tokens.len() < 5 {
        return None;
    }
    let day: u32 = tokens[0].parse().ok()?;
    let month = month_from_abbrev(tokens[1])?;
    let year = parse_year(tokens[2])?;
    let (hour, minute, second) = parse_time(tokens[3])?;
    let offset_secs = zone_offset_seconds(tokens[4])?;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let time = NaiveTime::from_hms_opt(hour, minute, second)?;
    let naive = date.and_time(time);
    let offset = FixedOffset::east_opt(offset_secs)?;
    let dt = offset.from_local_datetime(&naive).single()?;
    Some(dt.with_timezone(&Utc))
}

fn parse_year(tok: &str) -> Option<i32> {
    let y: i32 = tok.parse().ok()?;
    if tok.len() == 4 {
        Some(y)
    } else if tok.len() == 2 {
        // RFC 822 two-digit years: 00-49 => 2000s, 50-99 => 1900s.
        Some(if y < 50 { 2000 + y } else { 1900 + y })
    } else {
        None
    }
}

fn parse_time(tok: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = tok.split(':').collect();
    match parts.len() {
        2 => Some((parts[0].parse().ok()?, parts[1].parse().ok()?, 0)),
        3 => Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_utc() {
        let dt = parse_datetime("2006-01-02T15:04:05Z").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2006-01-02T15:04:05Z");
    }

    #[test]
    fn rfc3339_offset_normalized() {
        let dt = parse_datetime("2006-01-02T15:04:05-05:00").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2006-01-02T20:04:05Z");
    }

    #[test]
    fn rfc3339_fractional() {
        let dt = parse_datetime("2006-01-02T15:04:05.250Z").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2006-01-02T15:04:05Z");
    }

    #[test]
    fn rfc822_numeric_offset() {
        let dt = parse_datetime("Mon, 02 Jan 2006 15:04:05 -0500").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2006-01-02T20:04:05Z");
    }

    #[test]
    fn rfc822_gmt() {
        let dt = parse_datetime("Mon, 02 Jan 2006 15:04:05 GMT").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2006-01-02T15:04:05Z");
    }

    #[test]
    fn rfc822_zone_names() {
        for (zone, expect) in [
            ("EST", "2006-01-02T20:04:05Z"),
            ("EDT", "2006-01-02T19:04:05Z"),
            ("PST", "2006-01-02T23:04:05Z"),
            ("PDT", "2006-01-02T22:04:05Z"),
            ("UT", "2006-01-02T15:04:05Z"),
            ("Z", "2006-01-02T15:04:05Z"),
        ] {
            let s = format!("Mon, 02 Jan 2006 15:04:05 {zone}");
            let dt = parse_datetime(&s).unwrap();
            assert_eq!(to_rfc3339_z(&dt), expect, "zone {zone}");
        }
    }

    #[test]
    fn rfc822_no_weekday_no_seconds() {
        let dt = parse_datetime("02 Jan 2006 15:04 +0000").unwrap();
        assert_eq!(to_rfc3339_z(&dt), "2006-01-02T15:04:00Z");
    }

    #[test]
    fn unparseable() {
        assert!(parse_datetime("not a date").is_none());
        assert!(parse_datetime("").is_none());
        assert!(parse_datetime("Mon, 32 Jan 2006 15:04:05 GMT").is_none());
    }
}

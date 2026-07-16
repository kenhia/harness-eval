//! Date parsing and normalization (pinned rules).
//!
//! All successfully parsed instants are normalized to UTC. A missing or
//! unparseable date yields `None` — callers must **never** substitute the
//! fetch time.

use chrono::{DateTime, NaiveDate, TimeZone, Utc};

/// Parse a feed date string into a UTC instant.
///
/// Accepts RFC 3339 (any numeric offset, optional fractional seconds) and
/// RFC 822 / 1123 (4-digit years — 2-digit tolerated, numeric offsets like
/// `+0000`/`-0500`, and the zone names `GMT UT Z EST EDT CST CDT MST MDT PST
/// PDT`). Returns `None` when the input is empty or cannot be parsed.
pub fn parse_date(input: &str) -> Option<DateTime<Utc>> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    // RFC 3339 first (Atom, and the format we re-serialize to).
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // RFC 822 / 1123 (RSS `pubDate`).
    parse_rfc822(s)
}

/// Serialize a UTC instant as RFC 3339 with a `Z` suffix (no fractional
/// seconds), matching the stored representation.
pub fn to_rfc3339_z(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn month_from_abbr(m: &str) -> Option<u32> {
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

/// Named time-zone offsets in minutes, per the pinned RFC 822 zone set.
fn named_zone_offset(z: &str) -> Option<i32> {
    Some(match z {
        "GMT" | "UT" | "UTC" | "Z" => 0,
        "EST" => -5 * 60,
        "EDT" => -4 * 60,
        "CST" => -6 * 60,
        "CDT" => -5 * 60,
        "MST" => -7 * 60,
        "MDT" => -6 * 60,
        "PST" => -8 * 60,
        "PDT" => -7 * 60,
        _ => return None,
    })
}

fn parse_offset(z: &str) -> Option<i32> {
    if let Some(m) = named_zone_offset(z) {
        return Some(m);
    }
    // Numeric offset: +HHMM / -HHMM (optionally +HH:MM).
    let bytes = z.as_bytes();
    if bytes.len() >= 5 && (bytes[0] == b'+' || bytes[0] == b'-') {
        let sign = if bytes[0] == b'-' { -1 } else { 1 };
        let rest = &z[1..];
        let rest = rest.replace(':', "");
        if rest.len() == 4 && rest.chars().all(|c| c.is_ascii_digit()) {
            let hh: i32 = rest[0..2].parse().ok()?;
            let mm: i32 = rest[2..4].parse().ok()?;
            return Some(sign * (hh * 60 + mm));
        }
    }
    None
}

fn parse_rfc822(s: &str) -> Option<DateTime<Utc>> {
    // Drop an optional leading weekday ("Mon, ").
    let s = match s.split_once(',') {
        Some((_, rest)) => rest.trim(),
        None => s,
    };
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.len() < 4 {
        return None;
    }
    let day: u32 = tokens[0].parse().ok()?;
    let month = month_from_abbr(tokens[1])?;
    let year = parse_year(tokens[2])?;
    let (hour, minute, second) = parse_time(tokens[3])?;
    let offset_min = match tokens.get(4) {
        Some(z) => parse_offset(z)?,
        None => 0,
    };

    let naive = NaiveDate::from_ymd_opt(year, month, day)?
        .and_hms_opt(hour, minute, second)?;
    // Convert the wall-clock time at `offset_min` into UTC.
    let utc = naive - chrono::Duration::minutes(offset_min as i64);
    Some(Utc.from_utc_datetime(&utc))
}

fn parse_year(t: &str) -> Option<i32> {
    let y: i32 = t.parse().ok()?;
    if t.len() <= 2 {
        // RFC 2822 pivot: 00-49 => 2000s, 50-99 => 1900s.
        Some(if y < 50 { 2000 + y } else { 1900 + y })
    } else {
        Some(y)
    }
}

fn parse_time(t: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = t.split(':').collect();
    let hour: u32 = parts.first()?.parse().ok()?;
    let minute: u32 = parts.get(1)?.parse().ok()?;
    let second: u32 = match parts.get(2) {
        Some(s) => s.parse().ok()?,
        None => 0,
    };
    Some((hour, minute, second))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn rfc3339_z() {
        assert_eq!(parse_date("2024-01-02T03:04:05Z"), Some(utc("2024-01-02T03:04:05Z")));
    }

    #[test]
    fn rfc3339_offset_and_fractional() {
        assert_eq!(
            parse_date("2024-01-02T03:04:05.123-05:00").map(|d| to_rfc3339_z(&d)),
            Some("2024-01-02T08:04:05Z".to_string())
        );
    }

    #[test]
    fn rfc822_numeric_offset() {
        assert_eq!(
            parse_date("Mon, 02 Jan 2024 03:04:05 -0500"),
            Some(utc("2024-01-02T08:04:05Z"))
        );
    }

    #[test]
    fn rfc822_gmt_and_zero() {
        assert_eq!(
            parse_date("Tue, 10 Jun 2003 04:00:00 GMT"),
            Some(utc("2003-06-10T04:00:00Z"))
        );
        assert_eq!(
            parse_date("10 Jun 2003 09:00:00 +0000"),
            Some(utc("2003-06-10T09:00:00Z"))
        );
    }

    #[test]
    fn rfc822_named_zones() {
        assert_eq!(
            parse_date("Mon, 02 Jan 2024 12:00:00 EST"),
            Some(utc("2024-01-02T17:00:00Z"))
        );
        assert_eq!(
            parse_date("Mon, 02 Jan 2024 12:00:00 PDT"),
            Some(utc("2024-01-02T19:00:00Z"))
        );
    }

    #[test]
    fn rfc822_no_seconds() {
        assert_eq!(
            parse_date("02 Jan 2024 03:04 UT"),
            Some(utc("2024-01-02T03:04:00Z"))
        );
    }

    #[test]
    fn two_digit_year() {
        assert_eq!(
            parse_date("02 Jan 99 00:00:00 GMT"),
            Some(utc("1999-01-02T00:00:00Z"))
        );
    }

    #[test]
    fn unparseable_is_none() {
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("not a date"), None);
        assert_eq!(parse_date("Someday in June"), None);
    }
}

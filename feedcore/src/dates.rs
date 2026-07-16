//! Date parsing and normalization.
//!
//! All parsed instants are normalized to UTC. `to_rfc3339_z` serializes with a
//! trailing `Z`. Parsing returns `None` on failure so callers can store
//! `published_at = null` per spec (never substituting fetch time).

use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};

/// Serialize a UTC instant as RFC 3339 with a trailing `Z`, seconds precision.
pub fn to_rfc3339_z(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Parse an RFC 3339 timestamp with any numeric offset and optional fractional
/// seconds, returning it normalized to UTC.
pub fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn month_from_name(m: &str) -> Option<u32> {
    let m = &m[..m.len().min(3)];
    Some(match m.to_ascii_lowercase().as_str() {
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

/// Offset in seconds for a named RFC 822 zone, or `None` if unknown.
fn named_zone_offset(z: &str) -> Option<i32> {
    let h = 3600;
    Some(match z {
        "GMT" | "UT" | "Z" => 0,
        "EST" => -5 * h,
        "EDT" => -4 * h,
        "CST" => -6 * h,
        "CDT" => -5 * h,
        "MST" => -7 * h,
        "MDT" => -6 * h,
        "PST" => -8 * h,
        "PDT" => -7 * h,
        _ => return None,
    })
}

/// Parse a numeric zone like `+0000`, `-0500`, or `+05:30`.
fn numeric_zone_offset(z: &str) -> Option<i32> {
    let bytes = z.as_bytes();
    if bytes.len() < 3 {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let digits: String = z[1..].chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 4 {
        return None;
    }
    let hh: i32 = digits[0..2].parse().ok()?;
    let mm: i32 = digits[2..4].parse().ok()?;
    if hh > 23 || mm > 59 {
        return None;
    }
    Some(sign * (hh * 3600 + mm * 60))
}

fn zone_offset(z: &str) -> Option<i32> {
    named_zone_offset(z).or_else(|| numeric_zone_offset(z))
}

/// Parse an RFC 822 / 1123 date as used by RSS `pubDate`, normalized to UTC.
///
/// Accepts 4-digit years, numeric offsets, and the zone names
/// `GMT UT Z EST EDT CST CDT MST MDT PST PDT`. 2-digit years are also
/// tolerated (RFC 822 legacy). Returns `None` on any failure.
pub fn parse_rfc822(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    // Drop an optional leading weekday token ("Mon, ").
    let rest = match s.split_once(',') {
        Some((_, r)) => r.trim(),
        None => s,
    };
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    if tokens.len() < 5 {
        return None;
    }
    let day: u32 = tokens[0].parse().ok()?;
    let month = month_from_name(tokens[1])?;
    let mut year: i32 = tokens[2].parse().ok()?;
    if tokens[2].len() <= 2 {
        // Legacy 2-digit year windowing.
        year += if year < 70 { 2000 } else { 1900 };
    }

    let time_parts: Vec<&str> = tokens[3].split(':').collect();
    if time_parts.len() < 2 {
        return None;
    }
    let hour: u32 = time_parts[0].parse().ok()?;
    let minute: u32 = time_parts[1].parse().ok()?;
    let second: u32 = match time_parts.get(2) {
        Some(s) => s.parse().ok()?,
        None => 0,
    };

    let offset_secs = zone_offset(tokens[4])?;
    let offset = FixedOffset::east_opt(offset_secs)?;
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let naive = date.and_hms_opt(hour, minute, second)?;
    let dt = offset.from_local_datetime(&naive).single()?;
    Some(dt.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc822_numeric_offset() {
        let dt = parse_rfc822("Mon, 06 Sep 2021 16:20:00 +0000").unwrap();
        assert_eq!(to_rfc3339_z(dt), "2021-09-06T16:20:00Z");
    }

    #[test]
    fn rfc822_negative_offset_shifts_to_utc() {
        let dt = parse_rfc822("Mon, 06 Sep 2021 11:20:00 -0500").unwrap();
        assert_eq!(to_rfc3339_z(dt), "2021-09-06T16:20:00Z");
    }

    #[test]
    fn rfc822_named_zones() {
        assert_eq!(
            to_rfc3339_z(parse_rfc822("Wed, 02 Oct 2002 13:00:00 GMT").unwrap()),
            "2002-10-02T13:00:00Z"
        );
        // EST is -0500.
        assert_eq!(
            to_rfc3339_z(parse_rfc822("Wed, 02 Oct 2002 08:00:00 EST").unwrap()),
            "2002-10-02T13:00:00Z"
        );
        // PDT is -0700.
        assert_eq!(
            to_rfc3339_z(parse_rfc822("Wed, 02 Oct 2002 06:00:00 PDT").unwrap()),
            "2002-10-02T13:00:00Z"
        );
    }

    #[test]
    fn rfc822_no_weekday_and_no_seconds() {
        let dt = parse_rfc822("06 Sep 2021 16:20 Z").unwrap();
        assert_eq!(to_rfc3339_z(dt), "2021-09-06T16:20:00Z");
    }

    #[test]
    fn rfc822_unparseable_returns_none() {
        assert!(parse_rfc822("not a date").is_none());
        assert!(parse_rfc822("Mon, 06 Foo 2021 16:20:00 +0000").is_none());
        assert!(parse_rfc822("Mon, 06 Sep 2021 16:20:00 XYZ").is_none());
    }

    #[test]
    fn rfc3339_with_offset_and_fraction() {
        let dt = parse_rfc3339("2021-09-06T18:20:00.500+02:00").unwrap();
        assert_eq!(to_rfc3339_z(dt), "2021-09-06T16:20:00Z");
    }

    #[test]
    fn rfc3339_zulu() {
        let dt = parse_rfc3339("2021-09-06T16:20:00Z").unwrap();
        assert_eq!(to_rfc3339_z(dt), "2021-09-06T16:20:00Z");
    }

    #[test]
    fn rfc3339_bad_returns_none() {
        assert!(parse_rfc3339("2021-13-40T99:99:99Z").is_none());
        assert!(parse_rfc3339("nope").is_none());
    }
}
